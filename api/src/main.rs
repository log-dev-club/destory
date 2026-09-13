use axum::{Router, routing::get};
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    // 1. 환경 변수 로드
    // 팀원 참고: .env.example 파일을 복사하여 .env 파일을 만들고 DATABASE_URL을 채워주세요.
    dotenv().ok();

    // 2. 로깅 설정 (RUST_LOG 환경 변수로 레벨 제어, 기본값 info)
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    let port = env::var("SERVER_PORT").unwrap_or_else(|_| "8080".to_string());

    info!("Connecting to database...");

    // 3. 데이터베이스 커넥션 풀
    // 팀원 참고: 임시로 로컬 개발 환경에서 max_connections를 5로 설정했습니다.
    // 실제 운영 환경에서는 값을 조정할 필요가 있음
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Failed to connect to the database");

    info!("Successfully connected to the database!");

    // 4. 마이그레이션 적용 (api/migrations/ 디렉토리의 SQL 파일을 순서대로 실행)
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run database migrations");

    info!("Database migrations applied");

    // 5. 라우터 설정
    let app = Router::new()
        .route("/api/ping", get(|| async { "pong!" }))
        .with_state(pool);

    // 6. 서버 바인딩
    // 팀원 참고: 로컬 개발을 위해 127.0.0.1에 바인딩했습니다.
    // Docker를 통해 배포할 때는 0.0.0.0 으로 변경 필요
    let addr = format!("127.0.0.1:{port}");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|_| panic!("Failed to bind to {addr}. Is the port already in use?"));

    info!("Server is running on http://{addr}");

    axum::serve(listener, app)
        .await
        .expect("Failed to start the web server");
}
