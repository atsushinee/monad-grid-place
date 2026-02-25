use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::{
    AppError,
    AppState,
    services::upload_service,
};

#[derive(Deserialize)]
pub struct UploadRequest {
    pub x: u32,
    pub y: u32,
    pub color: String,
    pub owner: String,
}

#[derive(Serialize)]
pub struct UploadResponse {
    pub message: String,
    pub cid: String,
}

pub async fn upload_handler(
    State(state): State<AppState>,
    Json(payload): Json<UploadRequest>,
) -> Result<Json<UploadResponse>, AppError> {
    println!(
        "Received upload request for coordinates ({}, {})",
        payload.x, payload.y
    );

    let cid = upload_service::handle_upload(&state, &payload).await?;

    Ok(Json(UploadResponse {
        message: "Upload successful".into(),
        cid,
    }))
}
