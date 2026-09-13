//! 게시글 임시저장 (사용자당 1개, upsert)

use sqlx::PgPool;

use crate::{
    error::AppError,
    models::draft::{DraftDto, SaveDraftRequest},
};

pub async fn get(pool: &PgPool, user_id: i64) -> Result<Option<DraftDto>, AppError> {
    let draft = sqlx::query_as::<_, DraftDto>(
        "SELECT title, content, tags_input, updated_at FROM post_drafts WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    Ok(draft)
}

pub async fn save(
    pool: &PgPool,
    user_id: i64,
    req: SaveDraftRequest,
) -> Result<DraftDto, AppError> {
    let draft = sqlx::query_as::<_, DraftDto>(
        "INSERT INTO post_drafts (user_id, title, content, tags_input, updated_at) \
         VALUES ($1, $2, $3, $4, now()) \
         ON CONFLICT (user_id) DO UPDATE SET \
            title      = EXCLUDED.title, \
            content    = EXCLUDED.content, \
            tags_input = EXCLUDED.tags_input, \
            updated_at = now() \
         RETURNING title, content, tags_input, updated_at",
    )
    .bind(user_id)
    .bind(&req.title)
    .bind(&req.content)
    .bind(&req.tags_input)
    .fetch_one(pool)
    .await?;
    Ok(draft)
}

pub async fn delete(pool: &PgPool, user_id: i64) -> Result<(), AppError> {
    sqlx::query("DELETE FROM post_drafts WHERE user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}
