use axum::{Json, extract::State, http::StatusCode};
use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, SameSite},
};

use crate::{
    error::AppError,
    extract::{CurrentUser, SESSION_COOKIE},
    models::{
        auth::{LoginRequest, RegisterRequest},
        user::UserProfile,
    },
    services,
    state::AppState,
};

/// 세션 쿠키 생성. HttpOnly + SameSite=Lax. (HTTPS 배포 시 `.secure(true)` 추가 필요)
pub fn session_cookie(token: String, ttl: time::Duration) -> Cookie<'static> {
    Cookie::build((SESSION_COOKIE, token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(ttl)
        .build()
}

fn removal_cookie() -> Cookie<'static> {
    Cookie::build((SESSION_COOKIE, "")).path("/").build()
}

async fn issue_session(
    state: &AppState,
    jar: CookieJar,
    user_id: i64,
) -> Result<(CookieJar, Json<UserProfile>), AppError> {
    let token =
        services::auth::create_session(&state.pool, user_id, state.config.session_ttl).await?;
    let profile = services::users::profile_by_id(&state.pool, user_id).await?;
    let jar = jar.add(session_cookie(token, state.config.session_ttl));
    Ok((jar, Json(profile)))
}

/// POST /api/auth/register
pub async fn register(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(req): Json<RegisterRequest>,
) -> Result<(StatusCode, CookieJar, Json<UserProfile>), AppError> {
    let user = services::auth::register(&state.pool, &req.nickname, &req.password).await?;
    let (jar, profile) = issue_session(&state, jar, user.id).await?;
    Ok((StatusCode::CREATED, jar, profile))
}

/// POST /api/auth/login
pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(req): Json<LoginRequest>,
) -> Result<(CookieJar, Json<UserProfile>), AppError> {
    let user = services::auth::login(&state.pool, &req.nickname, &req.password).await?;
    issue_session(&state, jar, user.id).await
}

/// POST /api/auth/logout
pub async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<(StatusCode, CookieJar), AppError> {
    if let Some(cookie) = jar.get(SESSION_COOKIE) {
        services::auth::delete_session(&state.pool, cookie.value()).await?;
    }
    Ok((StatusCode::NO_CONTENT, jar.remove(removal_cookie())))
}

/// GET /api/auth/me — 현재 로그인 사용자 (비로그인 401)
pub async fn me(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> Result<Json<UserProfile>, AppError> {
    Ok(Json(
        services::users::profile_by_id(&state.pool, user.id).await?,
    ))
}
