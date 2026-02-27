use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tower_http::cors::{Any, CorsLayer};
use reqwest::Client;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use dashmap::DashMap;

mod routes;
mod config;
mod services;
mod error;
mod models;

use config::AppConfig;
use routes::{
    health::health_handler,
    grid::{get_grid_cell_handler, get_grid_cells_paginated_handler},
    cache::{get_cached_metadata_handler, cache_metadata_handler},
    snapshot::generate_snapshot_handler,
    paint_metadata::{submit_paint_metadata_handler, get_paint_metadata_handler},
    paint_area::{submit_paint_area_handler, get_snapshot_history_handler},
};
pub use error::AppError;

use tokio::sync::RwLock;
use std::collections::HashMap;
use crate::routes::paint_metadata::PixelData;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub http_client: Client,
    pub db_pool: PgPool,
    pub cache: Arc<DashMap<String, String>>,
    pub paint_metadata_cache: Arc<RwLock<HashMap<String, Vec<PixelData>>>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let app_config = Arc::new(AppConfig::from_env());

    println!("Connecting to database...");
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&app_config.database_url)
        .await?;
    println!("Database connected.");

    let http_client = Client::new();
    let cache = Arc::new(DashMap::new());
    let paint_metadata_cache = Arc::new(RwLock::new(HashMap::new()));

    let app_state = AppState {
        config: app_config.clone(),
        http_client,
        db_pool,
        cache: cache.clone(),
        paint_metadata_cache: paint_metadata_cache.clone(),
    };

    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(60 * 10)).await;
            println!("Clearing temporary CID cache.");
            cache.clear();
        }
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app: Router = Router::new()
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
        .with_state(app_state)
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], app_config.port));

    println!("🚀 Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, app.into_make_service())
        .await?;

    Ok(())
}
