use sqlx::PgPool;
use crate::{
    models::grid::GridCell,
    routes::upload::UploadRequest,
    AppError,
};

pub async fn create_or_update_grid_cell(
    db_pool: &PgPool,
    payload: &UploadRequest,
    ipfs_cid: &str,
) -> Result<GridCell, AppError> {
    let cell = sqlx::query_as!(
        GridCell,
        r#"
        INSERT INTO grid_cells (x, y, color, owner, ipfs_cid)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (x, y) DO UPDATE
        SET color = $3, owner = $4, ipfs_cid = $5, updated_at = NOW()
        RETURNING *
        "#,
        payload.x as i32,
        payload.y as i32,
        &payload.color,
        &payload.owner,
        ipfs_cid
    )
    .fetch_one(db_pool)
    .await
    .map_err(|e| AppError::InternalServerError(format!("Failed to save to database: {}", e)))?;

    Ok(cell)
}

pub async fn get_grid_cell(db_pool: &PgPool, x: i32, y: i32) -> Result<Option<GridCell>, AppError> {
    let cell = sqlx::query_as!(
        GridCell,
        "SELECT * FROM grid_cells WHERE x = $1 AND y = $2",
        x,
        y
    )
    .fetch_optional(db_pool)
    .await
    .map_err(|e| AppError::InternalServerError(format!("Failed to query database: {}", e)))?;

    Ok(cell)
}

pub async fn get_grid_cells_paginated(db_pool: &PgPool, page: i64, page_size: i64) -> Result<Vec<GridCell>, AppError> {
    let offset = (page - 1) * page_size;
    let cells = sqlx::query_as!(
        GridCell,
        "SELECT * FROM grid_cells ORDER BY updated_at DESC LIMIT $1 OFFSET $2",
        page_size,
        offset
    )
    .fetch_all(db_pool)
    .await
    .map_err(|e| AppError::InternalServerError(format!("Failed to query database: {}", e)))?;

    Ok(cells)
}
