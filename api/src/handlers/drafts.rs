//! 게시글 임시저장 (사용자당 1개, 로그인 필요)

use axum::{Json, extract::State, http::StatusCode};

use crate::{
    error::AppError,
    extract::CurrentUser,
    models::draft::{DraftDto, SaveDraftRequest},
    services,
    state::AppState,
};

/// GET /api/users/me/draft — 저장된 임시저장이 없으면 `null`
pub async fn get(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> Result<Json<Option<DraftDto>>, AppError> {
    Ok(Json(services::drafts::get(&state.pool, user.id).await?))
}

/// PUT /api/users/me/draft — upsert
pub async fn save(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(req): Json<SaveDraftRequest>,
) -> Result<Json<DraftDto>, AppError> {
    Ok(Json(
        services::drafts::save(&state.pool, user.id, req).await?,
    ))
}

/// DELETE /api/users/me/draft
pub async fn delete(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> Result<StatusCode, AppError> {
    services::drafts::delete(&state.pool, user.id).await?;
    Ok(StatusCode::NO_CONTENT)
}
