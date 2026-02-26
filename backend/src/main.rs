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
    upload::upload_handler,
    grid::{get_grid_cell_handler, get_grid_cells_paginated_handler},
    cache::{cache_metadata_handler, get_cached_metadata_handler},
};
pub use error::AppError;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub http_client: Client,
    pub db_pool: PgPool,
    pub cache: Arc<DashMap<String, String>>,
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

    let app_state = AppState {
        config: app_config.clone(),
        http_client,
        db_pool,
        cache: cache.clone(),
    };

    // Spawn a task to clean up the cache periodically
    tokio::spawn(async move {
        // This is a simple cache cleanup, a proper solution might use TTL caches
        loop {
            tokio::time::sleep(Duration::from_secs(60 * 10)).await; // Clean every 10 minutes
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
        .route("/upload", post(upload_handler))
        .route("/grid", get(get_grid_cells_paginated_handler))
        .route("/grid/:x/:y", get(get_grid_cell_handler))
        .route("/cache", post(cache_metadata_handler))
        .route("/cache/:cid_hash", get(get_cached_metadata_handler))
        .with_state(app_state)
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], app_config.port));

    println!("🚀 Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, app.into_make_service())
        .await?;

    Ok(())
}
