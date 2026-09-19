//! 프로필 조회/수정, 비밀번호 변경

use sqlx::PgPool;

use super::auth::{hash_password, validate_nickname, validate_password, verify_password};
use crate::{
    error::AppError,
    models::user::{ChangePasswordRequest, UpdateProfileRequest, UserProfile, UserRow},
};

const PROFILE_SELECT: &str = "SELECT u.id, u.nickname, u.avatar_url, u.bio, u.github_login, u.role, u.created_at, \
     (SELECT count(*) FROM posts p WHERE p.user_id = u.id) AS post_count, \
     (SELECT count(*) FROM post_stars s JOIN posts p ON p.id = s.post_id WHERE p.user_id = u.id) AS star_count \
     FROM users u";

pub async fn profile_by_id(pool: &PgPool, user_id: i64) -> Result<UserProfile, AppError> {
    sqlx::query_as::<_, UserProfile>(&format!("{PROFILE_SELECT} WHERE u.id = $1"))
        .bind(user_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::not_found("사용자를 찾을 수 없습니다"))
}

pub async fn profile_by_nickname(pool: &PgPool, nickname: &str) -> Result<UserProfile, AppError> {
    sqlx::query_as::<_, UserProfile>(&format!("{PROFILE_SELECT} WHERE u.nickname = $1"))
        .bind(nickname)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::not_found("사용자를 찾을 수 없습니다"))
}

/// 생략한 필드는 유지. avatar_url/bio 는 빈 문자열이면 NULL 로 지운다.
pub async fn update_profile(
    pool: &PgPool,
    user_id: i64,
    req: UpdateProfileRequest,
) -> Result<UserProfile, AppError> {
    let nickname = req.nickname.map(|n| n.trim().to_string());
    if let Some(n) = &nickname {
        validate_nickname(n)?;
    }

    let avatar_url = req.avatar_url.map(|v| v.trim().to_string());
    if let Some(v) = &avatar_url {
        if v.chars().count() > 500 {
            return Err(AppError::bad_request("avatarUrl 은 500자 이하여야 합니다"));
        }
        if !v.is_empty() && !(v.starts_with("http://") || v.starts_with("https://")) {
            return Err(AppError::bad_request(
                "avatarUrl 은 http(s) URL 이어야 합니다",
            ));
        }
    }

    let bio = req.bio.map(|v| v.trim().to_string());
    if let Some(v) = &bio
        && v.chars().count() > 300
    {
        return Err(AppError::bad_request("bio 는 300자 이하여야 합니다"));
    }

    sqlx::query(
        "UPDATE users SET \
            nickname   = COALESCE($2, nickname), \
            avatar_url = CASE WHEN $3::text IS NULL THEN avatar_url WHEN $3 = '' THEN NULL ELSE $3 END, \
            bio        = CASE WHEN $4::text IS NULL THEN bio        WHEN $4 = '' THEN NULL ELSE $4 END, \
            updated_at = now() \
         WHERE id = $1",
    )
    .bind(user_id)
    .bind(nickname)
    .bind(avatar_url)
    .bind(bio)
    .execute(pool)
    .await
    .map_err(|e| match AppError::from(e) {
        AppError::Conflict(_) => AppError::conflict("이미 사용 중인 닉네임입니다"),
        other => other,
    })?;

    profile_by_id(pool, user_id).await
}

pub async fn change_password(
    pool: &PgPool,
    user: &UserRow,
    req: ChangePasswordRequest,
) -> Result<(), AppError> {
    validate_password(&req.new_password)?;

    if !verify_password(req.current_password, user.password_hash.clone()).await? {
        return Err(AppError::bad_request("현재 비밀번호가 올바르지 않습니다"));
    }

    let hash = hash_password(req.new_password).await?;

    let mut tx = pool.begin().await?;
    sqlx::query("UPDATE users SET password_hash = $2, updated_at = now() WHERE id = $1")
        .bind(user.id)
        .bind(hash)
        .execute(&mut *tx)
        .await?;
    // 비밀번호 변경 시 다른 기기의 세션은 모두 만료시킨다 (현재 세션은 핸들러에서 새로 발급)
    sqlx::query("DELETE FROM sessions WHERE user_id = $1")
        .bind(user.id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;

    Ok(())
}
