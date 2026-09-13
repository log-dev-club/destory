use axum::{
    Router,
    routing::{get, put},
};

use crate::{handlers::posts, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/posts", get(posts::list).post(posts::create))
        .route(
            "/api/posts/{id}",
            get(posts::get).patch(posts::update).delete(posts::delete),
        )
        .route(
            "/api/posts/{id}/star",
            put(posts::star).delete(posts::unstar),
        )
}
