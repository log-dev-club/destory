//! 인증 관련 axum 추출기.
//! - `CurrentUser`: 로그인 필수. 세션 쿠키가 없거나 만료되면 401.
//! - `MaybeUser`: 로그인 선택. 목록/상세 조회에서 `starred` 계산용.

use axum::{extract::FromRequestParts, http::request::Parts};
use axum_extra::extract::CookieJar;

use crate::{error::AppError, models::user::UserRow, services, state::AppState};

pub const SESSION_COOKIE: &str = "session";

pub struct CurrentUser(pub UserRow);

pub struct MaybeUser(pub Option<UserRow>);

impl FromRequestParts<AppState> for MaybeUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_headers(&parts.headers);
        let Some(cookie) = jar.get(SESSION_COOKIE) else {
            return Ok(MaybeUser(None));
        };
        let user = services::auth::user_by_session(&state.pool, cookie.value()).await?;
        Ok(MaybeUser(user))
    }
}

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let MaybeUser(user) = MaybeUser::from_request_parts(parts, state).await?;
        user.map(CurrentUser).ok_or(AppError::Unauthorized)
    }
}
