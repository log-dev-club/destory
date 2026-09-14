use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, post},
};

use crate::{handlers::images, state::AppState};

/// 업로드 경로에만 본문 크기 제한(MAX_UPLOAD_MB)을 적용한다
pub fn router(max_upload_bytes: usize) -> Router<AppState> {
    Router::new()
        .route("/api/images", post(images::upload))
        .route("/api/images/{stored_name}", get(images::serve))
        .layer(DefaultBodyLimit::max(max_upload_bytes))
}
