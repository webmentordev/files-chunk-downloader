mod apps;
mod chunker;
mod downloads;
mod multiple_uploads;
mod uploads;

use anyhow;
use apps::*;
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

    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let argument = args[1].as_str();
        match argument {
            "client" => {
                println!("Client connected!");
                return Ok(());
            }
            "serve" => {
                let port: u16 = std::env::var("PORT")
                    .unwrap_or_else(|_| "8443".to_string())
                    .parse()?;

                let app = Router::new()
                    .route("/health", get(health_check))
                    .route("/apps", get(get_apps))
                    .route("/convert", post(chunk_func))
                    .route("/upload", post(upload_file))
                    .route("/chunk/upload", post(chunk_uploads))
                    .route("/multiple/chunk/upload", post(multiple_uploads))
                    .route("/manifest/{app}", post(get_manifest))
                    .route("/save/manifest/{app}", get(save_manifest))
                    .route("/chunk/{app}/{chunk}", get(get_chunk))
                    .route("/download-original", get(download_original))
                    .layer(DefaultBodyLimit::disable())
                    .with_state(state);

                let addr = SocketAddr::from(([0, 0, 0, 0], port));
                let listener = TcpListener::bind(addr).await?;

                println!("🚀 Server is running at: http://127.0.0.1:{}", addr.port());
                axum::serve(listener, app).await?;
            }
            _ => {}
        }
    } else {
        print_commands();
    }

    Ok(())
}

fn print_commands() {
    const RESET: &str = "\x1b[0m";
    const BOLD: &str = "\x1b[1m";
    const DIM: &str = "\x1b[2m";
    const CYAN: &str = "\x1b[36m";
    const GREEN: &str = "\x1b[32m";
    const YELLOW: &str = "\x1b[33m";

    println!();
    println!("{CYAN}{BOLD}╭────────────────────────────────────────────╮{RESET}");
    println!(
        "{CYAN}{BOLD}│{RESET}  📦 {BOLD}File Chunker{RESET} {DIM}v{}{RESET}                    {CYAN}{BOLD}│{RESET}",
        env!("CARGO_PKG_VERSION")
    );
    println!(
        "{CYAN}{BOLD}│{RESET}  {DIM}Split, hash and serve files in chunks{RESET}       {CYAN}{BOLD}│{RESET}"
    );
    println!("{CYAN}{BOLD}╰────────────────────────────────────────────╯{RESET}");
    println!();

    println!("{BOLD}{YELLOW}USAGE{RESET}");
    println!("  {DIM}${RESET} file-chunker {GREEN}<command>{RESET}");
    println!();

    println!("{BOLD}{YELLOW}COMMANDS{RESET}");
    println!("  {GREEN}{BOLD}serve{RESET}    Start the HTTP server");
    println!("  {GREEN}{BOLD}client{RESET}   Run in client mode");
    println!();

    println!("{BOLD}{YELLOW}ENVIRONMENT{RESET}");
    println!("  {CYAN}PORT{RESET}     Port for the server {DIM}(default: 8443){RESET}");
    println!();

    println!("{BOLD}{YELLOW}ENDPOINTS{RESET}");
    let endpoints = [
        ("GET ", "/health", "Health check"),
        ("GET ", "/apps", "List apps"),
        ("POST", "/convert", "Chunk a file"),
        ("POST", "/upload", "Upload a file"),
        ("POST", "/chunk/upload", "Upload a chunk"),
        ("POST", "/multiple/chunk/upload", "Upload multiple chunks"),
        ("POST", "/manifest/{app}", "Get manifest"),
        ("GET ", "/save/manifest/{app}", "Download manifest.json"),
        ("GET ", "/chunk/{app}/{chunk}", "Download a chunk"),
        ("GET ", "/download-original", "Download original file"),
    ];
    for (method, path, desc) in endpoints {
        let color = if method.trim() == "GET" {
            GREEN
        } else {
            YELLOW
        };
        println!("  {color}{BOLD}{method}{RESET}  {path:<26} {DIM}{desc}{RESET}");
    }
    println!();

    println!("{BOLD}{YELLOW}EXAMPLES{RESET}");
    println!("  {DIM}${RESET} file-chunker serve");
    println!("  {DIM}${RESET} PORT=3000 file-chunker serve");
    println!();
}

async fn health_check() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({ "message": "System is live!" })),
    )
}
