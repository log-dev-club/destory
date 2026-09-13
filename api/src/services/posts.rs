//! 게시물 CRUD, 검색/필터, 별(star)

use std::collections::HashSet;

use sqlx::{PgPool, Postgres, QueryBuilder, Transaction};

use super::{attachments, like_pattern, truncate_chars};
use crate::{
    error::AppError,
    models::post::{
        CreatePostRequest, PostDetail, PostFilter, PostPage, PostRow, StarResponse,
        UpdatePostRequest,
    },
};

const TITLE_MAX: usize = 200;
const SUMMARY_MAX: usize = 300;
const TAG_MAX_LEN: usize = 50;
const TAG_MAX_COUNT: usize = 10;

/// 목록/상세 공용 SELECT. `PostRow` 의 필드와 컬럼명이 일치해야 한다.
/// viewer 는 `starred` 계산용 (비로그인은 None → false).
fn push_base_select(qb: &mut QueryBuilder<'_, Postgres>, viewer: Option<i64>) {
    qb.push(
        "SELECT p.id, p.title, p.summary, p.content, p.created_at, p.updated_at, \
                u.nickname, u.avatar_url, \
                COALESCE(array_agg(DISTINCT t.name::text) FILTER (WHERE t.name IS NOT NULL), ARRAY[]::text[]) AS tags, \
                (SELECT count(*) FROM comments c WHERE c.post_id = p.id) AS comment_count, \
                (SELECT count(*) FROM post_stars s WHERE s.post_id = p.id) AS star_count, \
                EXISTS (SELECT 1 FROM post_stars s WHERE s.post_id = p.id AND s.user_id = ",
    );
    qb.push_bind(viewer);
    qb.push(
        ") AS starred, \
                count(*) OVER () AS total \
         FROM posts p \
         JOIN users u ON u.id = p.user_id \
         LEFT JOIN post_tags pt ON pt.post_id = p.id \
         LEFT JOIN tags t ON t.id = pt.tag_id \
         WHERE TRUE",
    );
}

pub async fn list_posts(
    pool: &PgPool,
    filter: &PostFilter,
    viewer: Option<i64>,
) -> Result<PostPage, AppError> {
    let mut qb = QueryBuilder::new("");
    push_base_select(&mut qb, viewer);

    if let Some(author_id) = filter.author_id {
        qb.push(" AND p.user_id = ").push_bind(author_id);
    }
    if let Some(user) = &filter.user {
        qb.push(" AND u.nickname ILIKE ")
            .push_bind(like_pattern(user));
    }
    for tag in &filter.tags {
        qb.push(
            " AND EXISTS (SELECT 1 FROM post_tags pt2 JOIN tags t2 ON t2.id = pt2.tag_id \
                          WHERE pt2.post_id = p.id AND t2.name ILIKE ",
        )
        .push_bind(like_pattern(tag))
        .push(")");
    }
    for word in &filter.text {
        let pattern = like_pattern(word);
        qb.push(" AND (p.title ILIKE ")
            .push_bind(pattern.clone())
            .push(" OR p.summary ILIKE ")
            .push_bind(pattern.clone())
            .push(" OR u.nickname ILIKE ")
            .push_bind(pattern.clone())
            .push(
                " OR EXISTS (SELECT 1 FROM post_tags pt3 JOIN tags t3 ON t3.id = pt3.tag_id \
                             WHERE pt3.post_id = p.id AND t3.name ILIKE ",
            )
            .push_bind(pattern)
            .push("))");
    }

    let offset = i64::from(filter.page - 1) * i64::from(filter.limit);
    qb.push(" GROUP BY p.id, u.id ORDER BY p.created_at DESC, p.id DESC LIMIT ")
        .push_bind(i64::from(filter.limit))
        .push(" OFFSET ")
        .push_bind(offset);

    let rows: Vec<PostRow> = qb.build_query_as().fetch_all(pool).await?;
    let total = rows.first().map(|r| r.total).unwrap_or(0);

    Ok(PostPage {
        items: rows.into_iter().map(PostRow::into_summary).collect(),
        page: filter.page,
        limit: filter.limit,
        total,
    })
}

pub async fn get_post(
    pool: &PgPool,
    post_id: i64,
    viewer: Option<i64>,
) -> Result<PostDetail, AppError> {
    let mut qb = QueryBuilder::new("");
    push_base_select(&mut qb, viewer);
    qb.push(" AND p.id = ")
        .push_bind(post_id)
        .push(" GROUP BY p.id, u.id");

    let row: Option<PostRow> = qb.build_query_as().fetch_optional(pool).await?;
    let row = row.ok_or_else(|| AppError::not_found("게시물을 찾을 수 없습니다"))?;

    let attachments = attachments::list_for_post(pool, post_id).await?;
    Ok(row.into_detail(attachments))
}

/// 게시물 작성자 id. 없으면 404.
pub async fn owner_id(pool: &PgPool, post_id: i64) -> Result<i64, AppError> {
    let row: Option<(i64,)> = sqlx::query_as("SELECT user_id FROM posts WHERE id = $1")
        .bind(post_id)
        .fetch_optional(pool)
        .await?;
    row.map(|r| r.0)
        .ok_or_else(|| AppError::not_found("게시물을 찾을 수 없습니다"))
}

/// 게시물이 존재하고 요청자가 작성자인지 확인
pub async fn ensure_owner(pool: &PgPool, post_id: i64, user_id: i64) -> Result<(), AppError> {
    if owner_id(pool, post_id).await? != user_id {
        return Err(AppError::Forbidden);
    }
    Ok(())
}

/// 태그 정규화: 공백 제거, 소문자, 선행 `#` 제거, 중복 제거, 개수/길이 제한
fn normalize_tags(tags: Vec<String>) -> Result<Vec<String>, AppError> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for raw in tags {
        let tag = raw.trim().trim_start_matches('#').to_lowercase();
        if tag.is_empty() {
            continue;
        }
        if tag.chars().count() > TAG_MAX_LEN {
            return Err(AppError::bad_request(format!(
                "태그는 {TAG_MAX_LEN}자 이하여야 합니다"
            )));
        }
        if seen.insert(tag.clone()) {
            out.push(tag);
        }
    }
    if out.len() > TAG_MAX_COUNT {
        return Err(AppError::bad_request(format!(
            "태그는 최대 {TAG_MAX_COUNT}개까지 가능합니다"
        )));
    }
    Ok(out)
}

fn validate_title(title: &str) -> Result<String, AppError> {
    let title = title.trim();
    if title.is_empty() {
        return Err(AppError::bad_request("제목을 입력해주세요"));
    }
    if title.chars().count() > TITLE_MAX {
        return Err(AppError::bad_request(format!(
            "제목은 {TITLE_MAX}자 이하여야 합니다"
        )));
    }
    Ok(title.to_string())
}

fn validate_content(content: &str) -> Result<String, AppError> {
    if content.trim().is_empty() {
        return Err(AppError::bad_request("내용을 입력해주세요"));
    }
    Ok(content.to_string())
}

/// excerpt 가 없으면 Markdown 본문에서 기호를 걷어내고 앞부분을 요약으로 사용
fn resolve_summary(excerpt: Option<String>, content: &str) -> String {
    let explicit = excerpt
        .map(|e| e.trim().to_string())
        .filter(|e| !e.is_empty());
    let summary = explicit.unwrap_or_else(|| {
        let mut in_code = false;
        let mut words: Vec<String> = Vec::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("```") {
                in_code = !in_code;
                continue;
            }
            if in_code || trimmed.is_empty() {
                continue;
            }
            let cleaned: String = trimmed
                .chars()
                .filter(|c| !matches!(c, '#' | '*' | '`' | '>' | '_' | '~' | '[' | ']'))
                .collect();
            words.push(cleaned.split_whitespace().collect::<Vec<_>>().join(" "));
            if words.join(" ").chars().count() >= 150 {
                break;
            }
        }
        truncate_chars(&words.join(" "), 150)
    });
    truncate_chars(&summary, SUMMARY_MAX)
}

/// 게시물의 태그를 통째로 교체
async fn set_tags(
    tx: &mut Transaction<'_, Postgres>,
    post_id: i64,
    tags: &[String],
) -> Result<(), AppError> {
    sqlx::query("DELETE FROM post_tags WHERE post_id = $1")
        .bind(post_id)
        .execute(&mut **tx)
        .await?;

    for name in tags {
        // DO UPDATE 로 항상 id 를 돌려받는다 (DO NOTHING 은 기존 행일 때 RETURNING 이 비어 있음)
        let (tag_id,): (i64,) = sqlx::query_as(
            "INSERT INTO tags (name) VALUES ($1) \
             ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name RETURNING id",
        )
        .bind(name)
        .fetch_one(&mut **tx)
        .await?;

        sqlx::query(
            "INSERT INTO post_tags (post_id, tag_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )
        .bind(post_id)
        .bind(tag_id)
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

pub async fn create_post(
    pool: &PgPool,
    author_id: i64,
    req: CreatePostRequest,
) -> Result<PostDetail, AppError> {
    let title = validate_title(&req.title)?;
    let content = validate_content(&req.content)?;
    let summary = resolve_summary(req.excerpt, &content);
    let tags = normalize_tags(req.tags.unwrap_or_default())?;

    let mut tx = pool.begin().await?;
    let (post_id,): (i64,) = sqlx::query_as(
        "INSERT INTO posts (user_id, title, summary, content) VALUES ($1, $2, $3, $4) RETURNING id",
    )
    .bind(author_id)
    .bind(&title)
    .bind(&summary)
    .bind(&content)
    .fetch_one(&mut *tx)
    .await?;
    set_tags(&mut tx, post_id, &tags).await?;
    tx.commit().await?;

    get_post(pool, post_id, Some(author_id)).await
}

pub async fn update_post(
    pool: &PgPool,
    post_id: i64,
    user_id: i64,
    req: UpdatePostRequest,
) -> Result<PostDetail, AppError> {
    ensure_owner(pool, post_id, user_id).await?;

    let title = req.title.as_deref().map(validate_title).transpose()?;
    let content = req.content.as_deref().map(validate_content).transpose()?;
    let summary = req
        .excerpt
        .map(|e| truncate_chars(e.trim(), SUMMARY_MAX))
        .filter(|e| !e.is_empty());
    let tags = req.tags.map(normalize_tags).transpose()?;

    let mut tx = pool.begin().await?;
    sqlx::query(
        "UPDATE posts SET \
            title      = COALESCE($2, title), \
            summary    = COALESCE($3, summary), \
            content    = COALESCE($4, content), \
            updated_at = now() \
         WHERE id = $1",
    )
    .bind(post_id)
    .bind(title)
    .bind(summary)
    .bind(content)
    .execute(&mut *tx)
    .await?;
    if let Some(tags) = &tags {
        set_tags(&mut tx, post_id, tags).await?;
    }
    tx.commit().await?;

    get_post(pool, post_id, Some(user_id)).await
}

/// 게시물 삭제. DB 행은 CASCADE 로 정리되고 첨부파일은 디스크에서 별도로 지운다.
pub async fn delete_post(
    pool: &PgPool,
    post_id: i64,
    user_id: i64,
    upload_dir: &std::path::Path,
) -> Result<(), AppError> {
    ensure_owner(pool, post_id, user_id).await?;

    let hashed_names = attachments::hashed_names_for_post(pool, post_id).await?;

    sqlx::query("DELETE FROM posts WHERE id = $1")
        .bind(post_id)
        .execute(pool)
        .await?;

    for name in hashed_names {
        attachments::remove_file(upload_dir, &name).await;
    }
    Ok(())
}

async fn star_count(pool: &PgPool, post_id: i64) -> Result<i64, AppError> {
    let (count,): (i64,) = sqlx::query_as("SELECT count(*) FROM post_stars WHERE post_id = $1")
        .bind(post_id)
        .fetch_one(pool)
        .await?;
    Ok(count)
}

pub async fn star(pool: &PgPool, post_id: i64, user_id: i64) -> Result<StarResponse, AppError> {
    owner_id(pool, post_id).await?; // 존재 확인 (404)
    sqlx::query("INSERT INTO post_stars (post_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
        .bind(post_id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(StarResponse {
        starred: true,
        star_count: star_count(pool, post_id).await?,
    })
}

pub async fn unstar(pool: &PgPool, post_id: i64, user_id: i64) -> Result<StarResponse, AppError> {
    owner_id(pool, post_id).await?;
    sqlx::query("DELETE FROM post_stars WHERE post_id = $1 AND user_id = $2")
        .bind(post_id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(StarResponse {
        starred: false,
        star_count: star_count(pool, post_id).await?,
    })
}
