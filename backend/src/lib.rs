//! Backend library for monad-grid-place
//! 
//! 这个库提供了后端服务的所有功能，包括：
//! - IPFS 服务
//! - Grid 服务
//! - Cache 服务
//! - Snapshot 服务

pub mod config;
pub mod services;
pub mod error;
pub mod models;
pub mod routes;

pub use config::AppConfig;
pub use error::AppError;

use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use reqwest::Client;
use sqlx::PgPool;
use dashmap::DashMap;
use tokio::sync::RwLock;
use std::collections::HashMap;

use routes::{
    health::health_handler,
    grid::{get_grid_cell_handler, get_grid_cells_paginated_handler},
    cache::{get_cached_metadata_handler, cache_metadata_handler},
    snapshot::generate_snapshot_handler,
    paint_metadata::{submit_paint_metadata_handler, get_paint_metadata_handler},
    paint_area::{submit_paint_area_handler, get_snapshot_history_handler},
};

use routes::paint_metadata::PixelData;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub http_client: Client,
    pub db_pool: PgPool,
    pub cache: Arc<DashMap<String, String>>,
    pub paint_metadata_cache: Arc<RwLock<HashMap<String, Vec<PixelData>>>>,
}

pub async fn create_app(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/health", get(health_handler))
        .route("/grid", get(get_grid_cells_paginated_handler))
        .route("/grid/:x/:y", get(get_grid_cell_handler))
        .route("/snapshot", post(generate_snapshot_handler))
        .route("/cache", post(cache_metadata_handler))
        .route("/cache/:cid_hash", get(get_cached_metadata_handler))
        .route("/paint-metadata", post(submit_paint_metadata_handler))
        .route("/paint-metadata", get(get_paint_metadata_handler))
        .route("/paint-area", post(submit_paint_area_handler))
        .route("/snapshot-history", get(get_snapshot_history_handler))
        .with_state(state)
        .layer(cors)
}
