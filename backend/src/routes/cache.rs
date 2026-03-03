use axum::{
    extract::{State, Path},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use crate::{services::cache_service, AppState, AppError};
use log::{info, error};

#[derive(Deserialize)]
pub struct CacheRequest {
    pub cid_hash: String,
    pub cid: String,
}

#[allow(dead_code)]
pub async fn cache_metadata_handler(
    State(state): State<AppState>,
    Json(payload): Json<CacheRequest>,
) -> Result<StatusCode, AppError> {
    info!("💾 [Cache API] Caching CID mapping:");
    info!("   - CID Hash: {}", payload.cid_hash);
    info!("   - CID: {}", payload.cid);
    cache_service::set_cache(&state.cache, &payload.cid_hash, &payload.cid);
    Ok(StatusCode::OK)
}

pub async fn get_cached_metadata_handler(
    State(state): State<AppState>,
    Path(cid_hash): Path<String>,
) -> Result<String, AppError> {
    info!("💾 [Cache API] GET /cache/{} - Indexer requesting CID", cid_hash);

    let cid = cache_service::get_cache(&state.cache, &cid_hash)
        .ok_or_else(|| {
            error!("❌ [Cache API] CID not found in cache for hash: {}", cid_hash);
            AppError::NotFound("CID not found in cache".to_string())
        })?;

    info!("✅ [Cache API] CID found: {}", cid);
    Ok(cid)
}
