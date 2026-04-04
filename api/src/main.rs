use axum::{routing::get, Router};
use dotenvy::dotenv;
use sqlx::mysql::MySqlPoolOptions;
use std::env;
use tracing::info;

#[tokio::main]
async fn main() {
    // 1. 로깅 설정
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // 2. 환경 변수 로드
    // 팀원 참고: .env.example 파일을 복사하여 .env 파일을 만들고 DATABASE_URL을 채워주세요.
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");

    info!("Connecting to database...");
    
    // 3. 데이터베이스 커넥션 풀
    // 팀원 참고: 임시로 로컬 개발 환경에서 max_connections를 5로 설정했습니다. 
    // 실제 운영 환경에서는 값을 조정할 필요가 있음
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Failed to connect to the database");
    
    info!("Successfully connected to the database!");

    // 4. 라우터 설정
    let app = Router::new()
        .route("/api/ping", get(|| async { "pong!" }))
        .with_state(pool);

    // 5. 서버 바인딩
    // 팀원 참고: 로컬 개발을 위해 127.0.0.1에 바인딩했습니다. 
    // Docker를 통해 배포할 때는 변경 필요
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
    .await
    .expect("Failed to bind to port 8080. Is the port already in use?");
    
    info!("Server is running on http://127.0.0.1:8080");
    
    axum::serve(listener, app)
    .await
    .expect("Failed to start the web server");
}