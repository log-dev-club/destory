use axum::{
    Router,
    routing::{get, put},
};

use crate::{handlers::admin, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/admin/users", get(admin::list_users))
        .route("/api/admin/users/{nickname}/role", put(admin::set_role))
}
