mod chunker;
mod downloads;
mod multiple_uploads;
mod uploads;

use anyhow;
use axum::{
    Json, Router,
    extract::DefaultBodyLimit,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use chunker::*;
use dotenvy;
use downloads::*;
use multiple_uploads::*;
use serde_json::json;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool};
use std::{net::SocketAddr, str::FromStr, time::Duration};
use tokio::net::TcpListener;
use uploads::*;

#[derive(Clone)]
struct AppState {
    db: SqlitePool,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenvy::dotenv().ok();

    let conn = SqliteConnectOptions::from_str("sqlite:apps.db")?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .busy_timeout(Duration::from_secs(5));
    let pool = SqlitePool::connect_with(conn).await?;
    let state = AppState { db: pool.clone() };

    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS apps (
            id TEXT PRIMARY KEY NOT NULL,
            name VARCHAR(255) NOT NULL,
            file_name TEXT NOT NULL,
            version VARCHAR(255) NOT NULL,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            update_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        );"#,
    )
    .execute(&pool)
    .await?;

    // use later
    // let vars: Vec<String> = std::env::args().collect();

    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "8443".to_string())
        .parse()?;

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/convert", post(chunk_func))
        .route("/upload", post(upload_file))
        .route("/chunk/upload", post(chunk_uploads))
        .route("/multiple/chunk/upload", post(multiple_uploads))
        .route("/manifest/{app}", post(get_manifest))
        .route("/manifest/{app}/{chunk}", post(get_chunk))
        .route("/download-original", get(download_original))
        .layer(DefaultBodyLimit::disable())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await?;

    println!("🚀 Server is running at: http://127.0.0.1:{}", addr.port());
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({ "message": "System is live!" })),
    )
}
