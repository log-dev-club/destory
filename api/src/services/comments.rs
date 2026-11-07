//! 댓글 CRUD. 수정은 작성자만, 삭제는 댓글 작성자·게시물 작성자 또는 관리자.

use sqlx::PgPool;

use super::posts;
use crate::{
    error::AppError,
    models::{
        comment::{CommentDto, CommentRow},
        user::UserRow,
    },
};

const CONTENT_MAX: usize = 2000;

const COMMENT_SELECT: &str = "SELECT c.id, c.post_id, c.content, c.created_at, c.updated_at, \
            u.nickname, u.avatar_url \
     FROM comments c JOIN users u ON u.id = c.user_id";

fn validate_content(content: &str) -> Result<String, AppError> {
    let content = content.trim();
    if content.is_empty() {
        return Err(AppError::bad_request("댓글 내용을 입력해주세요"));
    }
    if content.chars().count() > CONTENT_MAX {
        return Err(AppError::bad_request(format!(
            "댓글은 {CONTENT_MAX}자 이하여야 합니다"
        )));
    }
    Ok(content.to_string())
}

pub async fn list_for_post(pool: &PgPool, post_id: i64) -> Result<Vec<CommentDto>, AppError> {
    posts::owner_id(pool, post_id).await?; // 존재 확인 (404)
    let rows = sqlx::query_as::<_, CommentRow>(&format!(
        "{COMMENT_SELECT} WHERE c.post_id = $1 ORDER BY c.created_at ASC, c.id ASC"
    ))
    .bind(post_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(CommentDto::from).collect())
}

async fn get_comment(pool: &PgPool, comment_id: i64) -> Result<CommentDto, AppError> {
    sqlx::query_as::<_, CommentRow>(&format!("{COMMENT_SELECT} WHERE c.id = $1"))
        .bind(comment_id)
        .fetch_optional(pool)
        .await?
        .map(CommentDto::from)
        .ok_or_else(|| AppError::not_found("댓글을 찾을 수 없습니다"))
}

pub async fn create(
    pool: &PgPool,
    post_id: i64,
    user_id: i64,
    content: &str,
) -> Result<CommentDto, AppError> {
    posts::owner_id(pool, post_id).await?;
    let content = validate_content(content)?;

    let (id,): (i64,) = sqlx::query_as(
        "INSERT INTO comments (post_id, user_id, content) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(post_id)
    .bind(user_id)
    .bind(content)
    .fetch_one(pool)
    .await?;

    get_comment(pool, id).await
}

/// (댓글 작성자 id, 게시물 작성자 id)
async fn owners(pool: &PgPool, comment_id: i64) -> Result<(i64, i64), AppError> {
    sqlx::query_as::<_, (i64, i64)>(
        "SELECT c.user_id, p.user_id FROM comments c JOIN posts p ON p.id = c.post_id WHERE c.id = $1",
    )
    .bind(comment_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::not_found("댓글을 찾을 수 없습니다"))
}

pub async fn update(
    pool: &PgPool,
    comment_id: i64,
    user_id: i64,
    content: &str,
) -> Result<CommentDto, AppError> {
    let (author_id, _) = owners(pool, comment_id).await?;
    if author_id != user_id {
        return Err(AppError::Forbidden);
    }
    let content = validate_content(content)?;

    sqlx::query("UPDATE comments SET content = $2, updated_at = now() WHERE id = $1")
        .bind(comment_id)
        .bind(content)
        .execute(pool)
        .await?;

    get_comment(pool, comment_id).await
}

pub async fn delete(pool: &PgPool, comment_id: i64, actor: &UserRow) -> Result<(), AppError> {
    let (author_id, post_owner_id) = owners(pool, comment_id).await?;
    if author_id != actor.id && post_owner_id != actor.id && !actor.is_admin() {
        return Err(AppError::Forbidden);
    }
    sqlx::query("DELETE FROM comments WHERE id = $1")
        .bind(comment_id)
        .execute(pool)
        .await?;
    Ok(())
}
