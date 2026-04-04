use axum::{routing::get, Router};
use dotenvy::dotenv;
use sqlx::mysql::MySqlPoolOptions;
use std::env;
use tracing::info;

#[tokio::main]
async fn main() {
    // 1. Logging setup
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // 2. Load environment variables
    // Note for team: Ensure you have copied .env.example to .env and filled in DATABASE_URL
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");

    info!("Connecting to database...");
    
    // 3. Database connection pool
    // Note for team: max_connections is set to 5 for local development. 
    // This should be increased via environment variables for production environments.
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Failed to connect to the database");
    
    info!("Successfully connected to the database!");

    // 4. Router setup
    // Note for team: Register new API controllers/handlers here.
    let app = Router::new()
        .route("/api/ping", get(|| async { "pong!" }))
        .with_state(pool);

    // 5. Server binding
    // Note for team: Bound to 127.0.0.1 for local dev. 
    // If deploying via Docker, change this to 0.0.0.0:8080.
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
    .await
    .expect("Failed to bind to port 8080. Is the port already in use?");
    
    info!("Server is running on http://127.0.0.1:8080");
    
    axum::serve(listener, app)
    .await
    .expect("Failed to start the web server");
}