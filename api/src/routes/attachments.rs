use axum::{Router, extract::DefaultBodyLimit, routing::get};

use crate::{handlers::attachments, state::AppState};

/// 업로드 경로에만 본문 크기 제한(MAX_UPLOAD_MB)을 적용한다
pub fn router(max_upload_bytes: usize) -> Router<AppState> {
    Router::new()
        .route(
            "/api/posts/{id}/attachments",
            get(attachments::list).post(attachments::upload),
        )
        .route(
            "/api/attachments/{hashed_name}",
            get(attachments::download).delete(attachments::delete),
        )
        .layer(DefaultBodyLimit::max(max_upload_bytes))
}
