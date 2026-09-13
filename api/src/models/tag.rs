use serde::Serialize;
use sqlx::FromRow;

/// GET /api/tags 항목
#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct TagCount {
    pub name: String,
    pub post_count: i64,
}
