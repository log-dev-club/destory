use axum::{
    Router,
    routing::{get, patch},
};

use crate::{handlers::comments, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/posts/{id}/comments",
            get(comments::list).post(comments::create),
        )
        .route(
            "/api/comments/{id}",
            patch(comments::update).delete(comments::delete),
        )
}
