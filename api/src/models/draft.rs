use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;

/// GET/PUT /api/users/me/draft 응답
#[derive(Debug, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DraftDto {
    pub title: String,
    pub content: String,
    pub tags_input: String,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

/// PUT /api/users/me/draft
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveDraftRequest {
    pub title: String,
    pub content: String,
    pub tags_input: String,
}
