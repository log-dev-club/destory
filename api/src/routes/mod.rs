//! 도메인별 라우터를 합쳐 최종 Router 를 만든다. 모든 경로는 `/api` prefix.

use axum::{Router, routing::get};

use crate::state::AppState;

pub mod attachments;
pub mod auth;
pub mod comments;
pub mod posts;
pub mod tags;
pub mod users;

pub fn router(state: AppState) -> Router {
    let max_upload_bytes = state.config.max_upload_bytes;

    Router::new()
        .route("/api/ping", get(|| async { "pong!" }))
        .merge(auth::router())
        .merge(users::router())
        .merge(posts::router())
        .merge(comments::router())
        .merge(attachments::router(max_upload_bytes))
        .merge(tags::router())
        .with_state(state)
}
