use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use axum_extra::extract::CookieJar;

use super::auth::session_cookie;
use crate::{
    error::AppError,
    extract::{CurrentUser, MaybeUser},
    models::{
        post::{PostListQuery, PostPage},
        user::{ChangePasswordRequest, UpdateProfileRequest, UserProfile},
    },
    services,
    state::AppState,
};

/// GET /api/users/me — 마이페이지 프로필
pub async fn me(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> Result<Json<UserProfile>, AppError> {
    Ok(Json(
        services::users::profile_by_id(&state.pool, user.id).await?,
    ))
}

/// PATCH /api/users/me — 프로필 수정
pub async fn update_me(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(req): Json<UpdateProfileRequest>,
) -> Result<Json<UserProfile>, AppError> {
    Ok(Json(
        services::users::update_profile(&state.pool, user.id, req).await?,
    ))
}

/// PUT /api/users/me/password — 비밀번호 변경. 다른 세션은 모두 만료되고 현재 세션은 재발급된다.
pub async fn change_password(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    jar: CookieJar,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<(StatusCode, CookieJar), AppError> {
    services::users::change_password(&state.pool, &user, req).await?;
    let token =
        services::auth::create_session(&state.pool, user.id, state.config.session_ttl).await?;
    Ok((
        StatusCode::NO_CONTENT,
        jar.add(session_cookie(token, &state.config)),
    ))
}

/// GET /api/users/me/posts — 내가 작성한 게시물 (검색 파라미터 동일하게 지원)
pub async fn my_posts(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Query(query): Query<PostListQuery>,
) -> Result<Json<PostPage>, AppError> {
    let filter = query.into_filter(Some(user.id));
    Ok(Json(
        services::posts::list_posts(&state.pool, &filter, Some(user.id)).await?,
    ))
}

/// GET /api/users/{nickname} — 공개 프로필
pub async fn profile(
    State(state): State<AppState>,
    Path(nickname): Path<String>,
) -> Result<Json<UserProfile>, AppError> {
    Ok(Json(
        services::users::profile_by_nickname(&state.pool, &nickname).await?,
    ))
}

/// GET /api/users/{nickname}/posts — 특정 사용자의 게시물
pub async fn user_posts(
    State(state): State<AppState>,
    MaybeUser(viewer): MaybeUser,
    Path(nickname): Path<String>,
    Query(query): Query<PostListQuery>,
) -> Result<Json<PostPage>, AppError> {
    let target = services::users::profile_by_nickname(&state.pool, &nickname).await?;
    let filter = query.into_filter(Some(target.id));
    let viewer_id = viewer.map(|u| u.id);
    Ok(Json(
        services::posts::list_posts(&state.pool, &filter, viewer_id).await?,
    ))
}
