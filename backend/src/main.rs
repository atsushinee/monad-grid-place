use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use reqwest::Client;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

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
};
pub use error::AppError;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub http_client: Client,
    pub db_pool: PgPool,
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

    // Optional: Run migrations at startup
    // sqlx::migrate!("./migrations").run(&db_pool).await?;
    // println!("Migrations ran successfully.");

    let http_client = Client::new();

    let app_state = AppState {
        config: app_config.clone(),
        http_client,
        db_pool,
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app: Router = Router::new()
        .route("/health", get(health_handler))
        .route("/upload", post(upload_handler))
        .route("/grid", get(get_grid_cells_paginated_handler))
        .route("/grid/:x/:y", get(get_grid_cell_handler))
        .with_state(app_state)
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], app_config.port));

    println!("🚀 Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, app.into_make_service())
        .await?;

    Ok(())
}
