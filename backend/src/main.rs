use backend::{create_app, AppState, AppConfig};
use std::sync::Arc;
use std::time::Duration;
use reqwest::Client;
use sqlx::postgres::PgPoolOptions;
use dashmap::DashMap;
use tokio::sync::RwLock;
use std::collections::HashMap;
use log::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let app_config = Arc::new(AppConfig::from_env());

    info!("Connecting to database...");
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&app_config.database_url)
        .await?;
    info!("Database connected.");

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
            info!("Clearing temporary CID cache.");
            cache.clear();
        }
    });

    let addr = format!("0.0.0.0:{}", app_config.port);

    info!("🚀 Server running on http://{}", addr);

    let app = create_app(app_state).await;

    let listener = tokio::net::TcpListener::bind(&addr).await?;

    axum::serve(listener, app.into_make_service())
        .await?;

    Ok(())
}
