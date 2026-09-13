use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;

/// users 테이블 한 행. 핸들러 밖으로 그대로 내보내지 않는다 (password_hash 포함).
#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)] // sqlx 가 채우는 컬럼이라 코드에서 직접 읽지 않는 필드가 있음
pub struct UserRow {
    pub id: i64,
    pub nickname: String,
    pub password_hash: String,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

/// 프론트엔드 `Author` 타입과 1:1
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Author {
    pub nickname: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
}

/// 마이페이지/공개 프로필 응답
#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct UserProfile {
    pub id: i64,
    pub nickname: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    /// 작성한 게시물 수
    pub post_count: i64,
    /// 작성한 게시물이 받은 별 합계
    pub star_count: i64,
}

/// PATCH /api/users/me
/// 생략한 필드는 그대로 두고, `avatarUrl`/`bio` 에 빈 문자열을 보내면 값을 지운다.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProfileRequest {
    pub nickname: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
}

/// PUT /api/users/me/password
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}
