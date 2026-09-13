use std::{env, path::PathBuf};

/// GitHub OAuth App 설정. CLIENT_ID/SECRET 둘 다 있을 때만 활성화된다.
#[derive(Debug, Clone)]
pub struct GithubOAuth {
    pub client_id: String,
    pub client_secret: String,
    /// GitHub OAuth App 에 등록한 Authorization callback URL 과 동일해야 한다
    pub redirect_url: String,
}

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
    /// 미설정이면 None → GitHub 로그인 비활성화
    pub github: Option<GithubOAuth>,
    /// 로그인·회원가입·GitHub 시작 엔드포인트의 IP 당 분당 허용 요청 수
    pub auth_rate_limit_per_minute: u32,
    /// true 면 X-Forwarded-For / X-Real-IP 헤더의 IP 를 클라이언트 IP 로 신뢰한다.
    /// 리버스 프록시(nginx, Caddy) 뒤에 있을 때만 켠다. 직접 노출된 서버에서 켜면 헤더 위조로 제한을 우회할 수 있다.
    pub trust_proxy_headers: bool,
}

fn non_empty(key: &str) -> Option<String> {
    env::var(key)
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
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

        let github = match (
            non_empty("GITHUB_CLIENT_ID"),
            non_empty("GITHUB_CLIENT_SECRET"),
        ) {
            (Some(client_id), Some(client_secret)) => Some(GithubOAuth {
                client_id,
                client_secret,
                redirect_url: non_empty("GITHUB_REDIRECT_URL")
                    .unwrap_or_else(|| format!("{app_base_url}/api/auth/github/callback")),
            }),
            _ => None,
        };

        let auth_rate_limit_per_minute = env::var("AUTH_RATE_LIMIT_PER_MINUTE")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .filter(|v| *v > 0)
            .unwrap_or(10);

        let trust_proxy_headers = env::var("TRUST_PROXY_HEADERS")
            .map(|v| matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes"))
            .unwrap_or(false);

        Self {
            database_url,
            server_port,
            discord_webhook_url,
            upload_dir,
            max_upload_bytes: max_upload_mb * 1024 * 1024,
            session_ttl: time::Duration::days(session_ttl_days),
            app_base_url,
            github,
            auth_rate_limit_per_minute,
            trust_proxy_headers,
        }
    }
}
