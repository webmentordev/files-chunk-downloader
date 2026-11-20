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

const CHUNK_SIZE: usize = 4 * 1024 * 1024;

// Worked great with 4.2GB Lord of the rings upload, No RAM spikes.
pub async fn chunk_uploads(
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let start = Instant::now();
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?
    {
        if field.name() != Some("file") {
            continue;
        }

        let file_name = field
            .file_name()
            .map(|s| s.to_owned())
            .ok_or((StatusCode::BAD_REQUEST, "missing file name".to_string()))?;

        let safe_name = Path::new(&file_name)
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or((StatusCode::BAD_REQUEST, "invalid file name".to_string()))?
            .to_owned();

        let stream = field.map_err(|err| io::Error::other(err));
        let mut reader = StreamReader::new(stream);
        let mut buf = vec![0u8; CHUNK_SIZE];

        let app_id = Uuid::new_v4();
        let dir = format!("./apps/{}", app_id);
        let uploads = "./uploads";
        let upload = format!("{}/{}", uploads, safe_name);

        fs::create_dir_all(&dir)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        fs::create_dir_all(uploads)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        let mut original = fs::File::create(&upload)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        let mut chunks: Vec<Chunk> = Vec::new();
        let mut total_size: u64 = 0;

        loop {
            let mut filled = 0;
            while filled < buf.len() {
                let n = reader
                    .read(&mut buf[filled..])
                    .await
                    .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
                if n == 0 {
                    break;
                }
                filled += n;
            }
            if filled == 0 {
                break;
            }

            let chunk = &buf[..filled];
            original
                .write_all(chunk)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            // Must not print to keep everything stable.
            // println!("Saved `{}` ({} bytes)", file_name, chunk.len());

            let id = Uuid::new_v4().to_string();
            fs::write(format!("{}/{}.chunk", dir, id), chunk)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            chunks.push(Chunk {
                id,
                size: filled as u64,
                hash: blake3::hash(chunk).to_hex().to_string(),
            });
            total_size += filled as u64;
        }

        original
            .flush()
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        let manifest = Manifest {
            file: safe_name,
            total_size,
            chunk_size: CHUNK_SIZE as u64,
            chunks,
        };
        let json = serde_json::to_string_pretty(&manifest)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        fs::write(format!("{}/manifest.json", dir), json)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        println!("Manifest at {}/manifest.json", dir);
    }

    let now = start.elapsed().as_millis();
    let message = format!("File uploaded successfully! - {}ms", now);

    Ok(Json(json!({ "message": message })))
}

// Almost crashed my PC with 4.2GB Lord of the rings upload (locally) 😹
pub async fn upload_file(
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?
    {
        let _name = field.name().unwrap_or("file").to_string();
        let file_name = field.file_name().unwrap_or("upload.bin").to_string();

        let path = format!("./example/{}", file_name);
        let data = field
            .bytes()
            .await
            .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

        fs::write(&path, &data)
            .await
            .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

        println!(
            "Saved `{}` ({} bytes) from field `{}`",
            file_name,
            data.len(),
            name
        );
    }
    Ok(Json(json!({"message": "File uploaded successfully!"})))
}
