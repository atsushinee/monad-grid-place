use serde_json::json;
use crate::{
    routes::upload::UploadRequest,
    services::{ipfs_service, grid_service},
    AppError,
    AppState,
};

pub async fn handle_upload(
    state: &AppState,
    payload: &UploadRequest,
) -> Result<String, AppError> {
    println!("Processing upload in service...");

    let metadata = json!({
        "x": payload.x,
        "y": payload.y,
        "color": payload.color,
        "owner": payload.owner,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    let cid = ipfs_service::add_json_to_ipfs(state, &metadata).await?;
    println!("Successfully uploaded to IPFS. CID: {}", cid);

    grid_service::create_or_update_grid_cell(&state.db_pool, payload, &cid).await?;
    println!("Successfully saved metadata to database.");

    Ok(cid)
}
