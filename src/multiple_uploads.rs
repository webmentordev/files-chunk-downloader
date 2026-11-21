use axum::{Json, extract::Multipart, http::StatusCode, response::IntoResponse};
use futures_util::TryStreamExt;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::io;
use std::path::Path;
use std::time::Instant;
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt},
};
use tokio_util::io::StreamReader;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Chunk {
    id: String,
    size: u64,
    hash: String,
}

#[derive(Serialize, Deserialize)]
pub struct Manifest {
    file: String,
    total_size: u64,
    chunk_size: u64,
    chunks: Vec<Chunk>,
}

#[derive(Serialize)]
pub struct UploadedFile {
    app_id: String,
    file: String,
    total_size: u64,
    chunks: usize,
    manifest: String,
    elapsed_ms: u128,
}

type ApiError = (StatusCode, String);

const CHUNK_SIZE: usize = 4 * 1024 * 1024;

fn internal(e: impl ToString) -> ApiError {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}

fn bad_request(e: impl ToString) -> ApiError {
    (StatusCode::BAD_REQUEST, e.to_string())
}

pub async fn multiple_uploads(mut multipart: Multipart) -> Result<impl IntoResponse, ApiError> {
    let start = Instant::now();
    let mut uploaded: Vec<UploadedFile> = Vec::new();

    while let Some(field) = multipart.next_field().await.map_err(bad_request)? {
        match field.name() {
            Some("file") | Some("files") | Some("files[]") => {}
            _ => continue,
        }

        uploaded.push(process_file(field).await?);
    }

    if uploaded.is_empty() {
        return Err(bad_request("no files provided"));
    }

    let message = format!(
        "{} file(s) uploaded successfully! - {}ms",
        uploaded.len(),
        start.elapsed().as_millis()
    );

    Ok(Json(json!({ "message": message, "files": uploaded })))
}

async fn process_file(
    field: axum::extract::multipart::Field<'_>,
) -> Result<UploadedFile, ApiError> {
    let file_start = Instant::now();

    let file_name = field
        .file_name()
        .map(|s| s.to_owned())
        .ok_or_else(|| bad_request("missing file name"))?;

    let safe_name = Path::new(&file_name)
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| bad_request("invalid file name"))?
        .to_owned();

    let app_id = Uuid::new_v4();
    let dir = format!("./apps/{}", app_id);
    let uploads = "./uploads";
    let upload = format!("{}/{}_{}", uploads, app_id, safe_name);

    fs::create_dir_all(&dir).await.map_err(internal)?;
    fs::create_dir_all(uploads).await.map_err(internal)?;

    let stream = field.map_err(io::Error::other);
    let mut reader = StreamReader::new(stream);
    let mut buf = vec![0u8; CHUNK_SIZE];

    let mut original = fs::File::create(&upload).await.map_err(internal)?;
    let mut chunks: Vec<Chunk> = Vec::new();
    let mut total_size: u64 = 0;

    loop {
        let mut filled = 0;
        while filled < buf.len() {
            let n = reader.read(&mut buf[filled..]).await.map_err(bad_request)?;
            if n == 0 {
                break;
            }
            filled += n;
        }
        if filled == 0 {
            break;
        }

        let chunk = &buf[..filled];
        original.write_all(chunk).await.map_err(internal)?;

        let id = Uuid::new_v4().to_string();
        fs::write(format!("{}/{}.chunk", dir, id), chunk)
            .await
            .map_err(internal)?;

        chunks.push(Chunk {
            id,
            size: filled as u64,
            hash: blake3::hash(chunk).to_hex().to_string(),
        });
        total_size += filled as u64;
    }

    original.flush().await.map_err(internal)?;

    let chunk_count = chunks.len();
    let manifest = Manifest {
        file: safe_name.clone(),
        total_size,
        chunk_size: CHUNK_SIZE as u64,
        chunks,
    };
    let manifest_path = format!("{}/manifest.json", dir);
    let json = serde_json::to_string_pretty(&manifest).map_err(internal)?;
    fs::write(&manifest_path, json).await.map_err(internal)?;

    Ok(UploadedFile {
        app_id: app_id.to_string(),
        file: safe_name,
        total_size,
        chunks: chunk_count,
        manifest: manifest_path,
        elapsed_ms: file_start.elapsed().as_millis(),
    })
}
