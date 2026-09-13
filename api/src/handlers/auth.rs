use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::Redirect,
};
use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, SameSite},
};
use rand::RngCore;
use serde::Deserialize;
use serde_json::{Value, json};
use tracing::warn;

use crate::{
    config::Config,
    error::AppError,
    extract::{CurrentUser, MaybeUser, SESSION_COOKIE},
    models::{
        auth::{LoginRequest, RegisterRequest},
        user::UserProfile,
    },
    services::{self, percent_encode, to_hex},
    state::AppState,
};

/// GitHub OAuth state 를 담는 임시 쿠키 (CSRF 방지). 콜백 경로에서만 전송된다.
const GITHUB_STATE_COOKIE: &str = "gh_oauth_state";
const GITHUB_STATE_TTL: time::Duration = time::Duration::minutes(10);

/// 세션 쿠키 생성. HttpOnly + SameSite=Lax, COOKIE_SECURE=true 면 Secure (HTTPS 전용)
pub fn session_cookie(token: String, config: &Config) -> Cookie<'static> {
    Cookie::build((SESSION_COOKIE, token))
        .path("/")
        .http_only(true)
        .secure(config.cookie_secure)
        .same_site(SameSite::Lax)
        .max_age(config.session_ttl)
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
    let jar = jar.add(session_cookie(token, &state.config));
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

/// GET /api/auth/providers — 프론트가 로그인 버튼을 표시할지 판단하는 데 사용
pub async fn providers(State(state): State<AppState>) -> Json<Value> {
    Json(json!({ "github": state.config.github.is_some() }))
}

fn github_state_cookie(value: String, secure: bool) -> Cookie<'static> {
    Cookie::build((GITHUB_STATE_COOKIE, value))
        .path("/api/auth/github")
        .http_only(true)
        .secure(secure)
        .same_site(SameSite::Lax)
        .max_age(GITHUB_STATE_TTL)
        .build()
}

fn github_state_removal() -> Cookie<'static> {
    Cookie::build((GITHUB_STATE_COOKIE, ""))
        .path("/api/auth/github")
        .build()
}

/// GET /api/auth/github — GitHub 인가 화면으로 이동
pub async fn github_start(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<(CookieJar, Redirect), AppError> {
    let cfg = state
        .config
        .github
        .as_ref()
        .ok_or_else(|| AppError::not_found("GitHub 로그인이 설정되지 않았습니다"))?;

    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    let oauth_state = to_hex(&bytes);

    let url = services::github::authorize_url(cfg, &oauth_state);
    Ok((
        jar.add(github_state_cookie(oauth_state, state.config.cookie_secure)),
        Redirect::to(&url),
    ))
}

#[derive(Debug, Deserialize)]
pub struct GithubCallbackQuery {
    code: Option<String>,
    state: Option<String>,
    /// 사용자가 GitHub 화면에서 취소하면 error=access_denied 로 돌아온다
    error: Option<String>,
    error_description: Option<String>,
}

/// 로그인 화면으로 오류 메시지를 들고 돌아간다
fn login_redirect_with_error(message: &str) -> Redirect {
    Redirect::to(&format!("/login?error={}", percent_encode(message)))
}

/// GET /api/auth/github/callback — GitHub 에서 돌아온 뒤 세션 발급.
/// 이미 로그인한 상태면 현재 계정에 GitHub 를 연동하고, 아니면 GitHub 계정으로 로그인/가입한다.
/// 브라우저 리다이렉트 흐름이라 오류도 JSON 이 아니라 /login?error= 로 돌려보낸다.
pub async fn github_callback(
    State(state): State<AppState>,
    MaybeUser(current): MaybeUser,
    jar: CookieJar,
    Query(query): Query<GithubCallbackQuery>,
) -> (CookieJar, Redirect) {
    // state 쿠키는 한 번만 쓰고 제거한다 (읽은 뒤에 제거해야 함)
    let jar_state = jar.get(GITHUB_STATE_COOKIE).map(|c| c.value().to_string());
    let jar = jar.remove(github_state_removal());

    let result = github_callback_inner(&state, current, jar_state, query).await;
    match result {
        Ok((user_id, redirect_to)) => {
            match services::auth::create_session(&state.pool, user_id, state.config.session_ttl)
                .await
            {
                Ok(token) => (
                    jar.add(session_cookie(token, &state.config)),
                    Redirect::to(&redirect_to),
                ),
                Err(e) => {
                    warn!(error = %e, "github login: session create failed");
                    (jar, login_redirect_with_error("세션 생성에 실패했습니다"))
                },
            }
        },
        Err(e) => {
            let message = match &e {
                AppError::Internal(detail) => {
                    warn!(detail, "github login failed");
                    "GitHub 로그인 처리 중 오류가 발생했습니다".to_string()
                },
                other => other.to_string(),
            };
            (jar, login_redirect_with_error(&message))
        },
    }
}

/// 성공 시 (세션을 발급할 user id, 이동할 경로)
async fn github_callback_inner(
    state: &AppState,
    current: Option<crate::models::user::UserRow>,
    cookie_state: Option<String>,
    query: GithubCallbackQuery,
) -> Result<(i64, String), AppError> {
    let cfg = state
        .config
        .github
        .as_ref()
        .ok_or_else(|| AppError::not_found("GitHub 로그인이 설정되지 않았습니다"))?;

    if let Some(error) = query.error {
        let detail = query.error_description.unwrap_or(error);
        return Err(AppError::bad_request(format!(
            "GitHub 인증이 취소되었습니다: {detail}"
        )));
    }

    let code = query
        .code
        .filter(|c| !c.is_empty())
        .ok_or_else(|| AppError::bad_request("GitHub 인가 코드가 없습니다"))?;

    match (cookie_state, query.state) {
        (Some(expected), Some(actual)) if expected == actual => {},
        _ => {
            return Err(AppError::bad_request(
                "로그인 요청이 만료되었거나 유효하지 않습니다. 다시 시도해주세요",
            ));
        },
    }

    let token = services::github::exchange_code(&state.http, cfg, &code).await?;
    let gh_user = services::github::fetch_user(&state.http, &token).await?;

    // 로그인 상태에서 왔으면 "연동", 아니면 "로그인/가입"
    let (user, redirect_to) = match current {
        Some(me) => (
            services::github::link(&state.pool, me.id, &gh_user).await?,
            "/me",
        ),
        None => (
            services::github::find_or_create(&state.pool, &gh_user).await?,
            "/",
        ),
    };
    Ok((user.id, redirect_to.to_string()))
}
