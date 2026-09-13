//! 릴리즈 첨부파일. 디스크에는 `hashed_name` 으로만 저장하고 원본 이름은 DB 에만 둔다.
//! `hashed_name` 은 프론트엔드 `hashFile.ts` 와 같은 방식(SHA-256("{sentAt}:{originalName}") + 확장자)으로
//! 서버에서 재계산해 검증한다.

use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use sqlx::PgPool;
use time::OffsetDateTime;
use tracing::warn;

use super::to_hex;
use crate::{
    error::AppError,
    models::attachment::{AttachmentDto, AttachmentRow},
};

const HASHED_NAME_MAX: usize = 80; // post_attachments.hashed_name VARCHAR(80)
const ORIGINAL_NAME_MAX: usize = 255;

/// 프론트엔드 `hashReleaseFileName` 과 동일한 계산
pub fn expected_hashed_name(original_name: &str, sent_at_ms: i64) -> String {
    let digest = Sha256::digest(format!("{sent_at_ms}:{original_name}").as_bytes());
    let ext = original_name
        .rfind('.')
        .map(|i| &original_name[i..])
        .unwrap_or("");
    format!("{}{}", to_hex(&digest), ext)
}

/// 업로드 메타데이터 검증. 통과하면 sent_at 을 OffsetDateTime 으로 돌려준다.
pub fn validate_upload(
    original_name: &str,
    hashed_name: &str,
    sent_at_ms: i64,
) -> Result<OffsetDateTime, AppError> {
    if original_name.is_empty() || original_name.chars().count() > ORIGINAL_NAME_MAX {
        return Err(AppError::bad_request("파일 이름이 비어 있거나 너무 깁니다"));
    }
    if hashed_name.len() > HASHED_NAME_MAX
        || hashed_name.contains(['/', '\\'])
        || hashed_name.contains("..")
    {
        return Err(AppError::bad_request("hashedName 형식이 올바르지 않습니다"));
    }
    if expected_hashed_name(original_name, sent_at_ms) != hashed_name {
        return Err(AppError::bad_request(
            "hashedName 이 sentAt 과 파일 이름으로 계산한 값과 일치하지 않습니다",
        ));
    }
    OffsetDateTime::from_unix_timestamp_nanos(i128::from(sent_at_ms) * 1_000_000)
        .map_err(|_| AppError::bad_request("sentAt 값이 유효한 시각이 아닙니다"))
}

pub fn file_path(upload_dir: &Path, hashed_name: &str) -> PathBuf {
    upload_dir.join(hashed_name)
}

/// 디스크에서 파일 제거. 실패해도 요청은 실패시키지 않고 로그만 남긴다.
pub async fn remove_file(upload_dir: &Path, hashed_name: &str) {
    let path = file_path(upload_dir, hashed_name);
    if let Err(e) = tokio::fs::remove_file(&path).await
        && e.kind() != std::io::ErrorKind::NotFound
    {
        warn!(path = %path.display(), error = %e, "첨부파일 삭제 실패");
    }
}

pub async fn create(
    pool: &PgPool,
    post_id: i64,
    original_name: &str,
    hashed_name: &str,
    size_bytes: i64,
    sent_at: OffsetDateTime,
) -> Result<AttachmentDto, AppError> {
    let row = sqlx::query_as::<_, AttachmentRow>(
        "INSERT INTO post_attachments (post_id, original_name, hashed_name, size_bytes, sent_at) \
         VALUES ($1, $2, $3, $4, $5) RETURNING *",
    )
    .bind(post_id)
    .bind(original_name)
    .bind(hashed_name)
    .bind(size_bytes)
    .bind(sent_at)
    .fetch_one(pool)
    .await
    .map_err(|e| match AppError::from(e) {
        AppError::Conflict(_) => AppError::conflict("같은 hashedName 의 첨부파일이 이미 있습니다"),
        other => other,
    })?;
    Ok(row.into())
}

pub async fn list_for_post(pool: &PgPool, post_id: i64) -> Result<Vec<AttachmentDto>, AppError> {
    let rows = sqlx::query_as::<_, AttachmentRow>(
        "SELECT * FROM post_attachments WHERE post_id = $1 ORDER BY created_at ASC, id ASC",
    )
    .bind(post_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(AttachmentDto::from).collect())
}

pub async fn hashed_names_for_post(pool: &PgPool, post_id: i64) -> Result<Vec<String>, AppError> {
    let rows: Vec<(String,)> =
        sqlx::query_as("SELECT hashed_name FROM post_attachments WHERE post_id = $1")
            .bind(post_id)
            .fetch_all(pool)
            .await?;
    Ok(rows.into_iter().map(|r| r.0).collect())
}

pub async fn by_hashed_name(pool: &PgPool, hashed_name: &str) -> Result<AttachmentRow, AppError> {
    sqlx::query_as::<_, AttachmentRow>("SELECT * FROM post_attachments WHERE hashed_name = $1")
        .bind(hashed_name)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::not_found("첨부파일을 찾을 수 없습니다"))
}

/// 게시물 작성자만 삭제 가능
pub async fn delete(
    pool: &PgPool,
    hashed_name: &str,
    user_id: i64,
    upload_dir: &Path,
) -> Result<(), AppError> {
    let row = by_hashed_name(pool, hashed_name).await?;
    super::posts::ensure_owner(pool, row.post_id, user_id).await?;

    sqlx::query("DELETE FROM post_attachments WHERE id = $1")
        .bind(row.id)
        .execute(pool)
        .await?;
    remove_file(upload_dir, &row.hashed_name).await;
    Ok(())
}
