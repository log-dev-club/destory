//! IP 당 요청 횟수 제한 (tower_governor).
//! 로그인/회원가입처럼 무차별 대입 대상이 되는 라우터에만 붙인다.
//! 초과 시 다른 에러와 같은 `{ "error": msg }` 형식으로 429 를 돌려준다.

use std::time::Duration;

use axum::{
    Router,
    body::Body,
    http::{Response, StatusCode, header},
};
use tower_governor::{
    GovernorError, GovernorLayer,
    governor::GovernorConfigBuilder,
    key_extractor::{KeyExtractor, PeerIpKeyExtractor, SmartIpKeyExtractor},
};
use tracing::warn;

use crate::{config::Config, state::AppState};

/// 오래된 IP 항목을 메모리에서 정리하는 주기
const CLEANUP_INTERVAL: Duration = Duration::from_secs(60);

/// 설정에 따라 클라이언트 IP 판별 방식을 골라 라우터에 제한을 건다
pub fn limit_per_ip(router: Router<AppState>, config: &Config) -> Router<AppState> {
    let per_minute = config.auth_rate_limit_per_minute;
    if config.trust_proxy_headers {
        apply(router, SmartIpKeyExtractor, per_minute)
    } else {
        apply(router, PeerIpKeyExtractor, per_minute)
    }
}

fn apply<K>(router: Router<AppState>, key_extractor: K, per_minute: u32) -> Router<AppState>
where
    K: KeyExtractor + Send + Sync + 'static,
    K::Key: Send + Sync + 'static,
{
    // 토큰 버킷: 최대 per_minute 개를 한 번에 허용하고, 60초/per_minute 마다 1개씩 회복
    let config = GovernorConfigBuilder::default()
        .key_extractor(key_extractor)
        .period(Duration::from_secs(60) / per_minute)
        .burst_size(per_minute)
        .finish()
        .expect("rate limit config must be valid");

    // 한 번 접속한 IP 는 한동안 메모리에 남으므로 주기적으로 오래된 항목을 비운다
    let limiter = config.limiter().clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(CLEANUP_INTERVAL);
        loop {
            interval.tick().await;
            limiter.retain_recent();
        }
    });

    router.layer(GovernorLayer::new(config).error_handler(error_response))
}

fn error_response(error: GovernorError) -> Response<Body> {
    let (status, message, retry_after) = match error {
        GovernorError::TooManyRequests { wait_time, .. } => (
            StatusCode::TOO_MANY_REQUESTS,
            format!("요청이 너무 많습니다. {wait_time}초 후 다시 시도해주세요"),
            Some(wait_time),
        ),
        GovernorError::UnableToExtractKey => {
            warn!("rate limit: 클라이언트 IP 를 알 수 없음 (ConnectInfo 누락?)");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "요청 처리 중 오류가 발생했습니다".to_string(),
                None,
            )
        },
        GovernorError::Other { code, msg, .. } => (
            code,
            msg.unwrap_or_else(|| "요청이 거부되었습니다".to_string()),
            None,
        ),
    };

    let body = serde_json::json!({ "error": message }).to_string();
    let mut builder = Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "application/json");
    if let Some(seconds) = retry_after {
        builder = builder.header(header::RETRY_AFTER, seconds.to_string());
    }
    builder
        .body(Body::from(body))
        .expect("static response is valid")
}
