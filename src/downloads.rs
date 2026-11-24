use axum::{
    Json,
    body::Body,
    extract::Path,
    http::{StatusCode, header},
    response::IntoResponse,
};
use serde_json::json;
use tokio::fs::{self, File};
use tokio_util::io::ReaderStream;
use uuid::Uuid;

pub async fn download_original() -> Result<impl IntoResponse, (StatusCode, String)> {
    let file = File::open("./uploads/electronics.pdf")
        .await
        .map_err(|err| (StatusCode::NOT_FOUND, err.to_string()))?;

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    let headers = [
        (header::CONTENT_TYPE, "application/pdf"),
        (
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"Electronics-book.pdf\"",
        ),
    ];
    Ok((headers, body))
}

pub async fn get_manifest(Path(app_uuid): Path<Uuid>) -> impl IntoResponse {
    let dir = format!("./apps/{app_uuid}");

    match fs::try_exists(&dir).await {
        Ok(true) => (
            StatusCode::OK,
            Json(json!({ "message": "AppId found!", "app_id": app_uuid })),
        ),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "message": "AppId not found!", "app_id": app_uuid })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "message": "Internal error" })),
        ),
    }
}

pub async fn save_manifest(
    Path(app_uuid): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let file_path = format!("./apps/{}/manifest.json", app_uuid);
    let file = File::open(file_path)
        .await
        .map_err(|err| (StatusCode::NOT_FOUND, err.to_string()))?;
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);
    let headers = [
        (header::CONTENT_TYPE, "application/json"),
        (
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"manifest.json\"",
        ),
    ];

    Ok((headers, body))
}

pub async fn get_chunk(Path((app_uuid, chunk_uuid)): Path<(Uuid, Uuid)>) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(
            json!({ "message": "Downloaded chunk!", "app_id": app_uuid.to_string(), "chunk_id": chunk_uuid.to_string() }),
        ),
    )
}
