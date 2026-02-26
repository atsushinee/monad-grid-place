use serde_json::json;
use ethers::utils::keccak256;
use hex;
use crate::{
    routes::upload::UploadRequest,
    services::ipfs_service,
    AppError,
    AppState,
};

pub async fn handle_upload(
    state: &AppState,
    payload: &UploadRequest,
) -> Result<(String, String), AppError> {
    println!("Processing upload in service...");

    let metadata = json!({
        "link": payload.link,
        "message": payload.message,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    let cid = ipfs_service::add_json_to_ipfs(state, &metadata).await?;
    println!("Successfully uploaded to IPFS. CID: {}", cid);

    let cid_hash = keccak256(cid.as_bytes());
    let cid_hash_hex = format!("0x{}", hex::encode(cid_hash));
    println!("Calculated CID Hash: {}", cid_hash_hex);

    // This service's only job is to upload to IPFS and return the hash.
    // The database is updated by the Indexer after the on-chain event.
    Ok((cid, cid_hash_hex))
}
