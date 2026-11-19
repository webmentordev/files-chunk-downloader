use axum::{Json, extract::Path, http::StatusCode, response::IntoResponse};
use serde_json::json;
use tokio::fs;
use uuid::Uuid;

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

pub async fn get_chunk(Path((app_uuid, chunk_uuid)): Path<(Uuid, Uuid)>) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(
            json!({ "message": "Downloaded chunk!", "app_id": app_uuid.to_string(), "chunk_id": chunk_uuid.to_string() }),
        ),
    )
}
