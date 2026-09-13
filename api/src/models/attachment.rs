use serde::Serialize;
use sqlx::FromRow;
use time::OffsetDateTime;

#[derive(Debug, FromRow)]
pub struct AttachmentRow {
    pub id: i64,
    pub post_id: i64,
    pub original_name: String,
    pub hashed_name: String,
    pub size_bytes: i64,
    pub sent_at: OffsetDateTime,
    pub created_at: OffsetDateTime,
}

/// 프론트엔드 `HashedRelease` 와 대응. `sentAt` 은 밀리초 epoch (JS `Date.now()` 형식)
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentDto {
    pub id: i64,
    pub post_id: i64,
    pub original_name: String,
    pub hashed_name: String,
    pub size: i64,
    pub sent_at: i64,
    /// 다운로드 경로 (`/api/attachments/{hashedName}`)
    pub download_url: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

impl From<AttachmentRow> for AttachmentDto {
    fn from(r: AttachmentRow) -> Self {
        Self {
            id: r.id,
            post_id: r.post_id,
            download_url: format!("/api/attachments/{}", r.hashed_name),
            original_name: r.original_name,
            hashed_name: r.hashed_name,
            size: r.size_bytes,
            sent_at: (r.sent_at.unix_timestamp_nanos() / 1_000_000) as i64,
            created_at: r.created_at,
        }
    }
}
