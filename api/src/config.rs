use std::{env, path::PathBuf};

/// .env 에서 읽어오는 서버 설정. 새 환경 변수를 추가하면 여기와 .env.example 에 함께 추가한다.
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server_port: u16,
    /// 미설정이거나 예시값(xxxx)이면 None → Discord 알림을 건너뛴다
    pub discord_webhook_url: Option<String>,
    pub upload_dir: PathBuf,
    pub max_upload_bytes: usize,
    pub session_ttl: time::Duration,
    /// Discord 알림에 넣을 게시물 링크의 기준 URL (프론트엔드 주소)
    pub app_base_url: String,
}

impl Config {
    pub fn from_env() -> Self {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");

        let server_port = env::var("SERVER_PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(8080);

        let discord_webhook_url = env::var("DISCORD_WEBHOOK_URL")
            .ok()
            .map(|v| v.trim().to_string())
            .filter(|v| v.starts_with("https://discord.com/api/webhooks/") && !v.contains("xxxx"));

        let upload_dir =
            PathBuf::from(env::var("UPLOAD_DIR").unwrap_or_else(|_| "./uploads".into()));

        let max_upload_mb: usize = env::var("MAX_UPLOAD_MB")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(100);

        let session_ttl_days: i64 = env::var("SESSION_TTL_DAYS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(30);

        let app_base_url = env::var("APP_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:5173".into())
            .trim_end_matches('/')
            .to_string();

        Self {
            database_url,
            server_port,
            discord_webhook_url,
            upload_dir,
            max_upload_bytes: max_upload_mb * 1024 * 1024,
            session_ttl: time::Duration::days(session_ttl_days),
            app_base_url,
        }
    }
}
