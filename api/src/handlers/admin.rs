use axum::{
    Json,
    extract::{Path, Query, State},
};

use crate::{
    error::AppError,
    extract::CurrentUser,
    models::{
        admin::{AdminUserListQuery, AdminUserPage, UpdateRoleRequest},
        user::UserProfile,
    },
    services,
    state::AppState,
};

/// GET /api/admin/users?q=&page=&limit= — 관리자/최고 관리자만
pub async fn list_users(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Query(query): Query<AdminUserListQuery>,
) -> Result<Json<AdminUserPage>, AppError> {
    if !user.is_admin() {
        return Err(AppError::Forbidden);
    }
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    Ok(Json(
        services::admin::list_users(&state.pool, query.q.as_deref(), page, limit).await?,
    ))
}

/// PUT /api/admin/users/{nickname}/role — 최고 관리자만. 관리자 권한 부여/해제
pub async fn set_role(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(nickname): Path<String>,
    Json(req): Json<UpdateRoleRequest>,
) -> Result<Json<UserProfile>, AppError> {
    Ok(Json(
        services::admin::set_role(&state.pool, &user, &nickname, &req.role).await?,
    ))
}
