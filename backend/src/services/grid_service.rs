use futures::future;
use crate::{
    models::grid::GridCell,
    services::ipfs_service,
    AppError,
    AppState,
};

pub async fn get_grid_cell(state: &AppState, x: i32, y: i32) -> Result<Option<GridCell>, AppError> {
    let mut cell = sqlx::query_as!(
        GridCell,
        "SELECT id, x, y, color, owner, ipfs_cid, created_at, updated_at, CAST(null as TEXT) as link FROM grid_cells WHERE x = $1 AND y = $2",
        x,
        y
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| AppError::InternalServerError(format!("Failed to query database: {}", e)))?;

    if let Some(ref mut c) = cell {
        match ipfs_service::fetch_metadata_from_ipfs(state, &c.ipfs_cid).await {
            Ok(metadata) => c.link = Some(metadata.link),
            Err(e) => eprintln!("Failed to fetch metadata for CID {}: {:?}", c.ipfs_cid, e),
        }
    }

    Ok(cell)
}

pub async fn get_grid_cells_paginated(state: &AppState, page: i64, page_size: i64) -> Result<Vec<GridCell>, AppError> {
    let offset = (page - 1) * page_size;
    let cells = sqlx::query_as!(
        GridCell,
        "SELECT id, x, y, color, owner, ipfs_cid, created_at, updated_at, CAST(null as TEXT) as link FROM grid_cells ORDER BY updated_at DESC LIMIT $1 OFFSET $2",
        page_size,
        offset
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| AppError::InternalServerError(format!("Failed to query database: {}", e)))?;

    let enriched_cells: Vec<GridCell> = future::join_all(cells.into_iter().map(|mut cell| {
        let state = state.clone();
        async move {
            match ipfs_service::fetch_metadata_from_ipfs(&state, &cell.ipfs_cid).await {
                Ok(metadata) => cell.link = Some(metadata.link),
                Err(e) => eprintln!("Failed to fetch metadata for CID {}: {:?}", cell.ipfs_cid, e),
            }
            cell
        }
    })).await;

    Ok(enriched_cells)
}
