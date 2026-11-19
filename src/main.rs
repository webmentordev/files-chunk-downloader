mod chunker;
mod downloads;
mod uploads;

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
use serde_json::json;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use uploads::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    // use later
    // let vars: Vec<String> = std::env::args().collect();

    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "8443".to_string())
        .parse()?;

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/convert", post(chunk_func))
        .route("/upload", post(upload_file))
        .route("/manifest/{app}", post(get_manifest))
        .route("/manifest/{app}/{chunk}", post(get_chunk))
        .layer(DefaultBodyLimit::disable());

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
