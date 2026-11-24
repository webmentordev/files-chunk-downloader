use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::Serialize;

use crate::AppState;

type ApiError = (StatusCode, String);

fn internal(e: impl ToString) -> ApiError {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AppsInfo {
    id: String,
    name: String,
}

pub async fn get_apps(pool: State<AppState>) -> Result<impl IntoResponse, ApiError> {
    let rows = sqlx::query_as::<_, AppsInfo>("SELECT id, name FROM apps")
        .fetch_all(&pool.db)
        .await
        .map_err(|e| {
            eprintln!("App fetch failed: {e}");
            internal(e)
        })?;

    Ok((StatusCode::OK, Json(rows)))
}
