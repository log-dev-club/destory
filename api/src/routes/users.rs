use axum::{
    Router,
    routing::{get, put},
};

use crate::{
    handlers::{drafts, users},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        // 정적 경로(/me)가 파라미터 경로(/{nickname})보다 우선 매칭된다
        .route("/api/users/me", get(users::me).patch(users::update_me))
        .route("/api/users/me/password", put(users::change_password))
        .route("/api/users/me/posts", get(users::my_posts))
        .route(
            "/api/users/me/draft",
            get(drafts::get).put(drafts::save).delete(drafts::delete),
        )
        .route("/api/users/{nickname}", get(users::profile))
        .route("/api/users/{nickname}/posts", get(users::user_posts))
}
