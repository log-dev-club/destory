//! GitHub OAuth 로그인.
//! 흐름: /api/auth/github → GitHub 인가 화면 → /api/auth/github/callback?code&state
//!       → code 를 access token 으로 교환 → GitHub 사용자 조회 → 우리 users 행 찾기/생성/연동 → 세션 발급

use reqwest::{Url, header};
use serde::Deserialize;
use sqlx::PgPool;

use super::auth::validate_nickname;
use crate::{config::GithubOAuth, error::AppError, models::user::UserRow};

const AUTHORIZE_URL: &str = "https://github.com/login/oauth/authorize";
const TOKEN_URL: &str = "https://github.com/login/oauth/access_token";
const USER_URL: &str = "https://api.github.com/user";
const USER_AGENT: &str = "destory";

/// GitHub 인가 화면 URL. state 는 CSRF 방지용으로 쿠키에도 저장해 콜백에서 비교한다.
pub fn authorize_url(cfg: &GithubOAuth, state: &str) -> String {
    Url::parse_with_params(
        AUTHORIZE_URL,
        &[
            ("client_id", cfg.client_id.as_str()),
            ("redirect_uri", cfg.redirect_url.as_str()),
            ("scope", "read:user"),
            ("state", state),
        ],
    )
    .expect("static authorize url is valid")
    .to_string()
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

/// 인가 코드를 access token 으로 교환
pub async fn exchange_code(
    http: &reqwest::Client,
    cfg: &GithubOAuth,
    code: &str,
) -> Result<String, AppError> {
    let res = http
        .post(TOKEN_URL)
        .header(header::ACCEPT, "application/json")
        .header(header::USER_AGENT, USER_AGENT)
        .json(&serde_json::json!({
            "client_id": cfg.client_id,
            "client_secret": cfg.client_secret,
            "code": code,
            "redirect_uri": cfg.redirect_url,
        }))
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("github token request failed: {e}")))?;

    let body: TokenResponse = res
        .json()
        .await
        .map_err(|e| AppError::Internal(format!("github token response invalid: {e}")))?;

    match body.access_token {
        Some(token) if !token.is_empty() => Ok(token),
        _ => Err(AppError::bad_request(format!(
            "GitHub 인증에 실패했습니다: {}",
            body.error_description
                .or(body.error)
                .unwrap_or_else(|| "알 수 없는 오류".into())
        ))),
    }
}

#[derive(Debug, Deserialize)]
pub struct GithubUser {
    pub id: i64,
    pub login: String,
    pub avatar_url: Option<String>,
}

pub async fn fetch_user(http: &reqwest::Client, token: &str) -> Result<GithubUser, AppError> {
    let res = http
        .get(USER_URL)
        .bearer_auth(token)
        .header(header::ACCEPT, "application/vnd.github+json")
        .header(header::USER_AGENT, USER_AGENT)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("github user request failed: {e}")))?;

    if !res.status().is_success() {
        return Err(AppError::Internal(format!(
            "github user request returned {}",
            res.status()
        )));
    }

    res.json()
        .await
        .map_err(|e| AppError::Internal(format!("github user response invalid: {e}")))
}

pub async fn find_by_github_id(pool: &PgPool, github_id: i64) -> Result<Option<UserRow>, AppError> {
    let user = sqlx::query_as::<_, UserRow>("SELECT * FROM users WHERE github_id = $1")
        .bind(github_id)
        .fetch_optional(pool)
        .await?;
    Ok(user)
}

/// 이미 로그인한 사용자의 계정에 GitHub 를 연동한다. 다른 계정에 연동된 GitHub 면 409.
pub async fn link(pool: &PgPool, user_id: i64, gh: &GithubUser) -> Result<UserRow, AppError> {
    if let Some(existing) = find_by_github_id(pool, gh.id).await?
        && existing.id != user_id
    {
        return Err(AppError::conflict(
            "이 GitHub 계정은 이미 다른 사용자와 연동되어 있습니다",
        ));
    }
    let user = sqlx::query_as::<_, UserRow>(
        "UPDATE users SET github_id = $2, github_login = $3, \
            avatar_url = COALESCE(avatar_url, $4), updated_at = now() \
         WHERE id = $1 RETURNING *",
    )
    .bind(user_id)
    .bind(gh.id)
    .bind(&gh.login)
    .bind(&gh.avatar_url)
    .fetch_one(pool)
    .await?;
    Ok(user)
}

/// GitHub login 을 우리 닉네임 규칙(2~20자, 문자·숫자·_·-)에 맞추고 중복이면 -2, -3 … 을 붙인다
async fn available_nickname(pool: &PgPool, login: &str) -> Result<String, AppError> {
    let mut base: String = login
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .take(20)
        .collect();
    if base.chars().count() < 2 {
        base = format!("gh-{base}");
    }

    for attempt in 0..50u32 {
        let candidate = if attempt == 0 {
            base.clone()
        } else {
            let suffix = format!("-{}", attempt + 1);
            let keep = 20usize.saturating_sub(suffix.chars().count());
            format!("{}{suffix}", base.chars().take(keep).collect::<String>())
        };
        if validate_nickname(&candidate).is_err() {
            continue;
        }
        let taken: Option<(i64,)> = sqlx::query_as("SELECT id FROM users WHERE nickname = $1")
            .bind(&candidate)
            .fetch_optional(pool)
            .await?;
        if taken.is_none() {
            return Ok(candidate);
        }
    }
    Err(AppError::Internal(format!(
        "could not derive a free nickname from github login {login}"
    )))
}

/// GitHub 계정으로 처음 로그인하면 사용자를 만든다 (비밀번호 없음 → password_hash 빈 문자열)
pub async fn find_or_create(pool: &PgPool, gh: &GithubUser) -> Result<UserRow, AppError> {
    if let Some(user) = find_by_github_id(pool, gh.id).await? {
        return Ok(user);
    }
    let nickname = available_nickname(pool, &gh.login).await?;
    let user = sqlx::query_as::<_, UserRow>(
        "INSERT INTO users (nickname, password_hash, github_id, github_login, avatar_url) \
         VALUES ($1, '', $2, $3, $4) RETURNING *",
    )
    .bind(nickname)
    .bind(gh.id)
    .bind(&gh.login)
    .bind(&gh.avatar_url)
    .fetch_one(pool)
    .await?;
    Ok(user)
}
