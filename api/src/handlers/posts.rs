use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};

use crate::{
    error::AppError,
    extract::{CurrentUser, MaybeUser},
    models::post::{
        CreatePostRequest, PostDetail, PostListQuery, PostPage, StarResponse, UpdatePostRequest,
    },
    services,
    state::AppState,
};

/// GET /api/posts?tag=&user=&q=&page=&limit=
pub async fn list(
    State(state): State<AppState>,
    MaybeUser(viewer): MaybeUser,
    Query(query): Query<PostListQuery>,
) -> Result<Json<PostPage>, AppError> {
    let filter = query.into_filter(None);
    let viewer_id = viewer.map(|u| u.id);
    Ok(Json(
        services::posts::list_posts(&state.pool, &filter, viewer_id).await?,
    ))
}

/// POST /api/posts — 작성 후 Discord 알림
pub async fn create(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(req): Json<CreatePostRequest>,
) -> Result<(StatusCode, Json<PostDetail>), AppError> {
    let post = services::posts::create_post(&state.pool, user.id, req).await?;
    services::discord::notify_new_post(&state, &post);
    Ok((StatusCode::CREATED, Json(post)))
}

/// GET /api/posts/{id}
pub async fn get(
    State(state): State<AppState>,
    MaybeUser(viewer): MaybeUser,
    Path(post_id): Path<i64>,
) -> Result<Json<PostDetail>, AppError> {
    let viewer_id = viewer.map(|u| u.id);
    Ok(Json(
        services::posts::get_post(&state.pool, post_id, viewer_id).await?,
    ))
}

/// PATCH /api/posts/{id} — 작성자만
pub async fn update(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(post_id): Path<i64>,
    Json(req): Json<UpdatePostRequest>,
) -> Result<Json<PostDetail>, AppError> {
    Ok(Json(
        services::posts::update_post(&state.pool, post_id, user.id, req).await?,
    ))
}

/// DELETE /api/posts/{id} — 작성자 또는 관리자
pub async fn delete(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(post_id): Path<i64>,
) -> Result<StatusCode, AppError> {
    services::posts::delete_post(&state.pool, post_id, &user, &state.config.upload_dir).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// PUT /api/posts/{id}/star
pub async fn star(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(post_id): Path<i64>,
) -> Result<Json<StarResponse>, AppError> {
    Ok(Json(
        services::posts::star(&state.pool, post_id, user.id).await?,
    ))
}

/// DELETE /api/posts/{id}/star
pub async fn unstar(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(post_id): Path<i64>,
) -> Result<Json<StarResponse>, AppError> {
    Ok(Json(
        services::posts::unstar(&state.pool, post_id, user.id).await?,
    ))
}
