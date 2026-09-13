use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;

use super::user::Author;

#[derive(Debug, FromRow)]
pub struct CommentRow {
    pub id: i64,
    pub post_id: i64,
    pub content: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub nickname: String,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentDto {
    pub id: i64,
    pub post_id: i64,
    pub content: String,
    pub author: Author,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

impl From<CommentRow> for CommentDto {
    fn from(r: CommentRow) -> Self {
        Self {
            id: r.id,
            post_id: r.post_id,
            content: r.content,
            author: Author {
                nickname: r.nickname,
                avatar_url: r.avatar_url,
            },
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

/// POST /api/posts/{id}/comments, PATCH /api/comments/{id}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentRequest {
    pub content: String,
}
