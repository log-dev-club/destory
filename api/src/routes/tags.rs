use axum::{Router, routing::get};

use crate::{handlers::tags, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new().route("/api/tags", get(tags::list))
}
