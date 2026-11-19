use axum::{Json, extract::Multipart, http::StatusCode, response::IntoResponse};
use serde_json::json;
use tokio::fs;

pub async fn upload_file(
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?
    {
        let name = field.name().unwrap_or("file").to_string();
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
