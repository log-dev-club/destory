use axum::{Json, extract::State};

use crate::{error::AppError, models::tag::TagCount, services, state::AppState};

/// GET /api/tags — 사용 중인 태그 목록 (게시물 수 내림차순)
pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<TagCount>>, AppError> {
    Ok(Json(services::tags::list(&state.pool).await?))
}
