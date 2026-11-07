//! 사용자 목록 조회(관리자)와 역할 부여/해제(최고 관리자), 최고 관리자 계정 부트스트랩.

use rand::Rng;
use sqlx::{PgPool, Postgres, QueryBuilder};

use super::like_pattern;
use crate::{
    error::AppError,
    models::{
        admin::{AdminUserPage, AdminUserRow},
        user::{UserProfile, UserRow},
    },
};

const SUPER_ADMIN_NICKNAME: &str = "admin";

/// 서버 기동 시 1회 호출. 최고 관리자가 한 명도 없으면 닉네임 `admin` 으로 하나 만든다.
/// 비밀번호는 무작위로 생성해 로그에 한 번만 출력하므로, 로그인 후 바로 바꿔야 한다.
pub async fn bootstrap_super_admin(pool: &PgPool) -> Result<(), AppError> {
    let (count,): (i64,) = sqlx::query_as("SELECT count(*) FROM users WHERE role = 'super_admin'")
        .fetch_one(pool)
        .await?;
    if count > 0 {
        return Ok(());
    }

    let password = generate_password();
    let hash = super::auth::hash_password(password.clone()).await?;

    let inserted: Option<(i64,)> = sqlx::query_as(
        "INSERT INTO users (nickname, password_hash, role) VALUES ($1, $2, 'super_admin') \
         ON CONFLICT (nickname) DO NOTHING RETURNING id",
    )
    .bind(SUPER_ADMIN_NICKNAME)
    .bind(hash)
    .fetch_optional(pool)
    .await?;

    match inserted {
        Some(_) => tracing::warn!(
            "최고 관리자 계정을 생성했습니다. 닉네임: {SUPER_ADMIN_NICKNAME} / 비밀번호: {password} \
             (이 비밀번호는 다시 표시되지 않으니 로그인 후 반드시 변경하세요)"
        ),
        None => tracing::error!(
            "닉네임 '{SUPER_ADMIN_NICKNAME}' 이 이미 사용 중이라 최고 관리자 계정을 자동 생성하지 못했습니다. \
             DB 에서 해당 사용자의 role 을 super_admin 으로 직접 바꿔주세요."
        ),
    }
    Ok(())
}

fn generate_password() -> String {
    const CHARS: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnpqrstuvwxyz23456789!@#$%";
    let mut rng = rand::thread_rng();
    (0..20)
        .map(|_| CHARS[rng.gen_range(0..CHARS.len())] as char)
        .collect()
}

/// GET /api/admin/users — 관리자/최고 관리자만 호출 (핸들러에서 검사)
pub async fn list_users(
    pool: &PgPool,
    q: Option<&str>,
    page: u32,
    limit: u32,
) -> Result<AdminUserPage, AppError> {
    let mut qb = QueryBuilder::<Postgres>::new(
        "SELECT u.id, u.nickname, u.avatar_url, u.role, u.created_at, \
                (SELECT count(*) FROM posts p WHERE p.user_id = u.id) AS post_count, \
                count(*) OVER () AS total \
         FROM users u WHERE TRUE",
    );
    if let Some(q) = q.map(str::trim).filter(|q| !q.is_empty()) {
        qb.push(" AND u.nickname ILIKE ").push_bind(like_pattern(q));
    }

    let offset = i64::from(page - 1) * i64::from(limit);
    qb.push(" ORDER BY u.created_at DESC, u.id DESC LIMIT ")
        .push_bind(i64::from(limit))
        .push(" OFFSET ")
        .push_bind(offset);

    let rows: Vec<AdminUserRow> = qb.build_query_as().fetch_all(pool).await?;
    let total = rows.first().map(|r| r.total).unwrap_or(0);

    Ok(AdminUserPage {
        items: rows.into_iter().map(Into::into).collect(),
        page,
        limit,
        total,
    })
}

/// 최고 관리자가 다른 계정에 관리자 권한을 부여/해제한다.
/// 자기 자신과 다른 최고 관리자의 역할은 이 API 로 바꿀 수 없다.
pub async fn set_role(
    pool: &PgPool,
    actor: &UserRow,
    target_nickname: &str,
    role: &str,
) -> Result<UserProfile, AppError> {
    if !actor.is_super_admin() {
        return Err(AppError::Forbidden);
    }
    if role != "user" && role != "admin" {
        return Err(AppError::bad_request(
            "role 은 user 또는 admin 이어야 합니다",
        ));
    }
    if target_nickname == actor.nickname {
        return Err(AppError::bad_request("자신의 역할은 변경할 수 없습니다"));
    }

    let target: Option<UserRow> = sqlx::query_as("SELECT * FROM users WHERE nickname = $1")
        .bind(target_nickname)
        .fetch_optional(pool)
        .await?;
    let target = target.ok_or_else(|| AppError::not_found("사용자를 찾을 수 없습니다"))?;
    if target.is_super_admin() {
        return Err(AppError::bad_request(
            "최고 관리자의 역할은 변경할 수 없습니다",
        ));
    }

    sqlx::query("UPDATE users SET role = $2, updated_at = now() WHERE id = $1")
        .bind(target.id)
        .bind(role)
        .execute(pool)
        .await?;

    super::users::profile_by_id(pool, target.id).await
}
