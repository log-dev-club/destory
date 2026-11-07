use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;

/// GET /api/admin/users 목록 항목의 원본 행
#[derive(Debug, FromRow)]
pub struct AdminUserRow {
    pub id: i64,
    pub nickname: String,
    pub avatar_url: Option<String>,
    pub role: String,
    pub created_at: OffsetDateTime,
    pub post_count: i64,
    /// 필터 적용 후 전체 건수 (count(*) OVER ())
    pub total: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminUserSummary {
    pub id: i64,
    pub nickname: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    pub role: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    pub post_count: i64,
}

impl From<AdminUserRow> for AdminUserSummary {
    fn from(r: AdminUserRow) -> Self {
        Self {
            id: r.id,
            nickname: r.nickname,
            avatar_url: r.avatar_url,
            role: r.role,
            created_at: r.created_at,
            post_count: r.post_count,
        }
    }
}

/// GET /api/admin/users 응답
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminUserPage {
    pub items: Vec<AdminUserSummary>,
    pub page: u32,
    pub limit: u32,
    pub total: i64,
}

/// GET /api/admin/users 쿼리 파라미터
#[derive(Debug, Default, Deserialize)]
pub struct AdminUserListQuery {
    /// 닉네임 부분 일치
    pub q: Option<String>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

/// PUT /api/admin/users/{nickname}/role
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRoleRequest {
    /// `user` 또는 `admin` 만 허용 (super_admin 은 이 API 로 만들 수 없음)
    pub role: String,
}
