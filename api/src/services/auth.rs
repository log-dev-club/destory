//! 회원가입/로그인/세션. 비밀번호는 argon2 로 해시하고 세션 토큰은 HttpOnly 쿠키로 전달한다.

use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use rand::RngCore;
use sqlx::PgPool;
use time::OffsetDateTime;

use super::to_hex;
use crate::{error::AppError, models::user::UserRow};

const NICKNAME_MIN: usize = 2;
const NICKNAME_MAX: usize = 20;
const PASSWORD_MIN: usize = 8;

/// 닉네임 규칙: 2~20자, 유니코드 문자/숫자와 `_`, `-` 만 허용
pub fn validate_nickname(nickname: &str) -> Result<(), AppError> {
    let len = nickname.chars().count();
    if !(NICKNAME_MIN..=NICKNAME_MAX).contains(&len) {
        return Err(AppError::bad_request(format!(
            "닉네임은 {NICKNAME_MIN}~{NICKNAME_MAX}자여야 합니다"
        )));
    }
    if !nickname
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return Err(AppError::bad_request(
            "닉네임은 문자, 숫자, '_', '-' 만 사용할 수 있습니다",
        ));
    }
    Ok(())
}

/// 특수문자: 문자·숫자·공백이 아닌 모든 문자 (!@#$%^&* 등). 프론트엔드 LoginPage 의 규칙과 동일해야 한다.
fn is_special_char(c: char) -> bool {
    !c.is_alphanumeric() && !c.is_whitespace()
}

/// 비밀번호 규칙: 8자 이상, 특수문자 1개 이상, 공백 불가
pub fn validate_password(password: &str) -> Result<(), AppError> {
    if password.chars().count() < PASSWORD_MIN {
        return Err(AppError::bad_request(format!(
            "비밀번호는 {PASSWORD_MIN}자 이상이어야 합니다"
        )));
    }
    if password.chars().any(char::is_whitespace) {
        return Err(AppError::bad_request(
            "비밀번호에 공백을 포함할 수 없습니다",
        ));
    }
    if !password.chars().any(is_special_char) {
        return Err(AppError::bad_request(
            "비밀번호에 특수문자(!@#$%^&* 등)를 1개 이상 포함해야 합니다",
        ));
    }
    Ok(())
}

/// argon2 해시 (CPU 작업이라 blocking 스레드에서 실행)
pub async fn hash_password(password: String) -> Result<String, AppError> {
    tokio::task::spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map(|h| h.to_string())
            .map_err(|e| AppError::Internal(format!("password hash failed: {e}")))
    })
    .await?
}

pub async fn verify_password(password: String, hash: String) -> Result<bool, AppError> {
    tokio::task::spawn_blocking(move || {
        let Ok(parsed) = PasswordHash::new(&hash) else {
            return Ok(false);
        };
        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok())
    })
    .await?
}

pub async fn register(pool: &PgPool, nickname: &str, password: &str) -> Result<UserRow, AppError> {
    let nickname = nickname.trim();
    validate_nickname(nickname)?;
    validate_password(password)?;

    let hash = hash_password(password.to_string()).await?;

    let user = sqlx::query_as::<_, UserRow>(
        "INSERT INTO users (nickname, password_hash) VALUES ($1, $2) RETURNING *",
    )
    .bind(nickname)
    .bind(hash)
    .fetch_one(pool)
    .await
    .map_err(|e| match AppError::from(e) {
        AppError::Conflict(_) => AppError::conflict("이미 사용 중인 닉네임입니다"),
        other => other,
    })?;

    Ok(user)
}

pub async fn login(pool: &PgPool, nickname: &str, password: &str) -> Result<UserRow, AppError> {
    let user = sqlx::query_as::<_, UserRow>("SELECT * FROM users WHERE nickname = $1")
        .bind(nickname.trim())
        .fetch_optional(pool)
        .await?;

    // 존재하지 않는 닉네임과 틀린 비밀번호를 구분하지 않는다 (계정 존재 여부 노출 방지)
    let invalid = || AppError::bad_request("닉네임 또는 비밀번호가 올바르지 않습니다");
    let user = user.ok_or_else(invalid)?;

    if user.password_hash.is_empty()
        || !verify_password(password.to_string(), user.password_hash.clone()).await?
    {
        return Err(invalid());
    }

    Ok(user)
}

/// 세션 생성 후 토큰 반환. 만료된 세션도 이 시점에 정리한다.
pub async fn create_session(
    pool: &PgPool,
    user_id: i64,
    ttl: time::Duration,
) -> Result<String, AppError> {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    let token = to_hex(&bytes);

    let expires_at = OffsetDateTime::now_utc() + ttl;

    sqlx::query("DELETE FROM sessions WHERE expires_at < now()")
        .execute(pool)
        .await?;

    sqlx::query("INSERT INTO sessions (token, user_id, expires_at) VALUES ($1, $2, $3)")
        .bind(&token)
        .bind(user_id)
        .bind(expires_at)
        .execute(pool)
        .await?;

    Ok(token)
}

pub async fn delete_session(pool: &PgPool, token: &str) -> Result<(), AppError> {
    sqlx::query("DELETE FROM sessions WHERE token = $1")
        .bind(token)
        .execute(pool)
        .await?;
    Ok(())
}

/// 유효한 세션 토큰이면 사용자 반환
pub async fn user_by_session(pool: &PgPool, token: &str) -> Result<Option<UserRow>, AppError> {
    if token.len() != 64 || !token.chars().all(|c| c.is_ascii_hexdigit()) {
        return Ok(None);
    }
    let user = sqlx::query_as::<_, UserRow>(
        "SELECT u.* FROM sessions s JOIN users u ON u.id = s.user_id \
         WHERE s.token = $1 AND s.expires_at > now()",
    )
    .bind(token)
    .fetch_optional(pool)
    .await?;
    Ok(user)
}
