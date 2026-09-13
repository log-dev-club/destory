use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

use crate::{
    error::AppError,
    extract::CurrentUser,
    models::comment::{CommentDto, CommentRequest},
    services,
    state::AppState,
};

/// GET /api/posts/{id}/comments — 오래된 순
pub async fn list(
    State(state): State<AppState>,
    Path(post_id): Path<i64>,
) -> Result<Json<Vec<CommentDto>>, AppError> {
    Ok(Json(
        services::comments::list_for_post(&state.pool, post_id).await?,
    ))
}

/// POST /api/posts/{id}/comments
pub async fn create(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(post_id): Path<i64>,
    Json(req): Json<CommentRequest>,
) -> Result<(StatusCode, Json<CommentDto>), AppError> {
    let comment = services::comments::create(&state.pool, post_id, user.id, &req.content).await?;
    Ok((StatusCode::CREATED, Json(comment)))
}

/// PATCH /api/comments/{id} — 댓글 작성자만
pub async fn update(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(comment_id): Path<i64>,
    Json(req): Json<CommentRequest>,
) -> Result<Json<CommentDto>, AppError> {
    Ok(Json(
        services::comments::update(&state.pool, comment_id, user.id, &req.content).await?,
    ))
}

/// DELETE /api/comments/{id} — 댓글 작성자 또는 게시물 작성자
pub async fn delete(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(comment_id): Path<i64>,
) -> Result<StatusCode, AppError> {
    services::comments::delete(&state.pool, comment_id, user.id).await?;
    Ok(StatusCode::NO_CONTENT)
}
