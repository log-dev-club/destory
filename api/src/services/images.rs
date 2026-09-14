//! 게시물 본문에 삽입하는 인라인 이미지. post_id 없이 업로드되고, 무작위 파일명으로 저장된다.

use std::path::{Path, PathBuf};

use rand::RngCore;
use sqlx::PgPool;

use super::to_hex;
use crate::error::AppError;

const ALLOWED_EXTENSIONS: &[(&str, &str)] = &[
    ("png", "image/png"),
    ("jpg", "image/jpeg"),
    ("jpeg", "image/jpeg"),
    ("gif", "image/gif"),
    ("webp", "image/webp"),
];

/// 원본 파일 이름의 확장자로 이미지 종류를 판별한다. 허용 목록에 없으면 에러.
pub fn content_type_for(original_name: &str) -> Result<&'static str, AppError> {
    let ext = original_name
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    ALLOWED_EXTENSIONS
        .iter()
        .find(|(e, _)| *e == ext)
        .map(|(_, ct)| *ct)
        .ok_or_else(|| AppError::bad_request("지원하지 않는 이미지 형식입니다 (png/jpg/gif/webp)"))
}

/// 확장자만 원본 이름에서 가져오고 나머지는 무작위로 만든 저장용 파일명
pub fn random_stored_name(original_name: &str) -> String {
    let ext = original_name
        .rfind('.')
        .map(|i| &original_name[i..])
        .unwrap_or("");
    let mut random = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut random);
    format!("{}{}", to_hex(&random), ext.to_ascii_lowercase())
}

pub fn file_path(upload_dir: &Path, stored_name: &str) -> PathBuf {
    upload_dir.join("images").join(stored_name)
}

pub async fn create(
    pool: &PgPool,
    uploader_id: i64,
    stored_name: &str,
    size_bytes: i64,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO content_images (uploader_id, stored_name, size_bytes) VALUES ($1, $2, $3)",
    )
    .bind(uploader_id)
    .bind(stored_name)
    .bind(size_bytes)
    .execute(pool)
    .await?;
    Ok(())
}

/// DB 에 등록된 stored_name 인지 확인 (경로 조작 방지)
pub async fn exists(pool: &PgPool, stored_name: &str) -> Result<bool, AppError> {
    let row: Option<(i32,)> = sqlx::query_as("SELECT 1 FROM content_images WHERE stored_name = $1")
        .bind(stored_name)
        .fetch_optional(pool)
        .await?;
    Ok(row.is_some())
}
