use axum::{
    extract::{State, Path, Query},
    Json,
};
use serde::Deserialize;
use crate::{
    models::grid::GridCell,
    services::grid_service,
    AppError,
    AppState,
};

pub async fn get_grid_cell_handler(
    State(state): State<AppState>,
    Path((x, y)): Path<(i32, i32)>,
) -> Result<Json<GridCell>, AppError> {
    let cell = grid_service::get_grid_cell(&state, x, y)
        .await?
        .ok_or_else(|| AppError::NotFound("Grid cell not found".to_string()))?;
    Ok(Json(cell))
}

#[derive(Deserialize)]
pub struct Pagination {
    #[serde(default = "default_page")]
    page: i64,
    #[serde(default = "default_page_size")]
    page_size: i64,
}

fn default_page() -> i64 { 1 }
fn default_page_size() -> i64 { 20 }

pub async fn get_grid_cells_paginated_handler(
    State(state): State<AppState>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<Vec<GridCell>>, AppError> {
    let cells = grid_service::get_grid_cells_paginated(&state, pagination.page, pagination.page_size).await?;
    Ok(Json(cells))
}
