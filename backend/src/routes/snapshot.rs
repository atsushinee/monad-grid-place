use axum::{extract::State, Json};
use crate::{
    models::grid::{SnapshotRequest, SnapshotResponse},
    services::snapshot_service,
    AppError,
    AppState,
};

pub async fn generate_snapshot_handler(
    State(state): State<AppState>,
    Json(payload): Json<SnapshotRequest>,
) -> Result<Json<SnapshotResponse>, AppError> {
    let response = snapshot_service::generate_snapshot(&state, &payload).await?;
    Ok(Json(response))
}
