//! 부트스트랩만 담당한다. 비즈니스 로직은 services/, 라우팅은 routes/ 에 둔다.

mod config;
mod error;
mod extract;
mod handlers;
mod models;
mod rate_limit;
mod routes;
mod services;
mod state;

use std::{net::SocketAddr, sync::Arc};

use dotenvy::dotenv;
use sqlx::{Postgres, migrate::MigrateDatabase, postgres::PgPoolOptions};
use tracing::info;
use tracing_subscriber::EnvFilter;

use crate::{config::Config, state::AppState};

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

    let config = Arc::new(Config::from_env());

    // 3. 데이터베이스가 없으면 생성
    // DATABASE_URL 의 계정으로 postgres 관리 DB 에 접속해 대상 DB 존재 여부를 확인하고 없으면 만든다.
    // 마이그레이션은 이미 접속한 DB 안에서 실행되므로 CREATE DATABASE 는 여기서만 가능하다.
    // (계정에 CREATEDB 권한이 필요. 로컬의 postgres 슈퍼유저는 기본으로 갖고 있음)
    let db_exists = Postgres::database_exists(&config.database_url)
        .await
        .expect(
            "Failed to check database existence. Is PostgreSQL running and DATABASE_URL correct?",
        );
    if !db_exists {
        info!("Database not found, creating it...");
        Postgres::create_database(&config.database_url)
            .await
            .expect(
                "Failed to create database. Does the DATABASE_URL user have CREATEDB privilege?",
            );
        info!("Database created");
    }

    info!("Connecting to database...");

    // 4. 데이터베이스 커넥션 풀
    // 팀원 참고: 임시로 로컬 개발 환경에서 max_connections를 5로 설정했습니다.
    // 실제 운영 환경에서는 값을 조정할 필요가 있음
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to the database");

    info!("Successfully connected to the database!");

    // 5. 마이그레이션 적용 (api/migrations/ 디렉토리의 SQL 파일을 순서대로 실행)
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run database migrations");

    info!("Database migrations applied");

    // 6. 첨부파일 디렉토리 준비
    tokio::fs::create_dir_all(&config.upload_dir)
        .await
        .expect("Failed to create upload directory");

    if config.discord_webhook_url.is_none() {
        info!("DISCORD_WEBHOOK_URL 미설정: 게시물 알림을 보내지 않습니다");
    }

    // 7. 라우터 설정
    let state = AppState {
        pool,
        config: config.clone(),
        http: reqwest::Client::new(),
    };
    let app = routes::router(state);

    // 8. 서버 바인딩
    // 팀원 참고: 로컬 개발을 위해 127.0.0.1에 바인딩했습니다.
    // Docker를 통해 배포할 때는 0.0.0.0 으로 변경 필요
    let addr = format!("127.0.0.1:{}", config.server_port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|_| panic!("Failed to bind to {addr}. Is the port already in use?"));

    info!("Server is running on http://{addr}");

    // 요청 횟수 제한이 클라이언트 IP 를 알 수 있도록 ConnectInfo 를 붙인다
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .expect("Failed to start the web server");
}
