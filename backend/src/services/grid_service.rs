use crate::{
    models::grid::{GridCell, DbPixelData},
    AppError,
    AppState,
};

/// 获取单个像素的信息
pub async fn get_grid_cell(state: &AppState, x: i32, y: i32) -> Result<Option<GridCell>, AppError> {
    let cell = sqlx::query_as!(
        GridCell,
        r#"SELECT
            id, x, y, owner, ipfs_cid, color,
            created_at, updated_at,
            link, message
        FROM grid_cells WHERE x = $1 AND y = $2"#,
        x,
        y
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| AppError::InternalServerError(format!("Failed to query database: {}", e)))?;

    Ok(cell)
}

/// 分页获取网格像素
pub async fn get_grid_cells_paginated(state: &AppState, page: i64, page_size: i64) -> Result<Vec<GridCell>, AppError> {
    let offset = (page - 1) * page_size;
    let cells = sqlx::query_as!(
        GridCell,
        r#"SELECT
            id, x, y, owner, ipfs_cid, color,
            created_at, updated_at,
            link, message
        FROM grid_cells ORDER BY updated_at DESC LIMIT $1 OFFSET $2"#,
        page_size,
        offset
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| AppError::InternalServerError(format!("Failed to query database: {}", e)))?;

    Ok(cells)
}

/// 获取某个 owner 的所有像素（用于生成快照）
pub async fn get_owner_pixels(db_pool: &sqlx::PgPool, owner: &str) -> Result<Vec<DbPixelData>, AppError> {
    let pixels = sqlx::query_as!(
        DbPixelData,
        r#"SELECT
            x, y,
            owner,
            color,
            link,
            message
        FROM grid_cells WHERE owner = $1"#,
        owner
    )
    .fetch_all(db_pool)
    .await
    .map_err(|e| AppError::InternalServerError(format!("Failed to query owner pixels: {}", e)))?;

    Ok(pixels)
}
