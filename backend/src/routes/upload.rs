use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::{
    AppError,
    AppState,
    services::upload_service,
};

// The request now only contains data destined for IPFS
#[derive(Deserialize)]
pub struct UploadRequest {
    pub link: String,
    pub message: String,
}

// The response includes both the raw CID and its keccak256 hash
#[derive(Serialize)]
pub struct UploadResponse {
    pub cid: String,
    pub cid_hash: String,
}

pub async fn upload_handler(
    State(state): State<AppState>,
    Json(payload): Json<UploadRequest>,
) -> Result<Json<UploadResponse>, AppError> {
    println!("Received upload request.");

    let (cid, cid_hash) = upload_service::handle_upload(&state, &payload).await?;

    Ok(Json(UploadResponse {
        cid,
        cid_hash,
    }))
}
