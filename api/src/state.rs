use std::sync::Arc;

use sqlx::PgPool;

use crate::config::Config;

/// 모든 핸들러가 `State<AppState>` 로 받는 공유 상태
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<Config>,
    /// Discord Webhook 등 외부 HTTP 호출용 클라이언트 (커넥션 재사용)
    pub http: reqwest::Client,
}
