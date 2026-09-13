use axum::{
    Router,
    routing::{get, post},
};

use crate::{config::Config, handlers::auth, rate_limit, state::AppState};

pub fn router(config: &Config) -> Router<AppState> {
    // 무차별 대입 대상이 되는 엔드포인트만 IP 당 요청 횟수를 제한한다
    let limited = Router::new()
        .route("/api/auth/register", post(auth::register))
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/github", get(auth::github_start));
    let limited = rate_limit::limit_per_ip(limited, config);

    Router::new()
        .route("/api/auth/logout", post(auth::logout))
        .route("/api/auth/me", get(auth::me))
        .route("/api/auth/providers", get(auth::providers))
        .route("/api/auth/github/callback", get(auth::github_callback))
        .merge(limited)
}
