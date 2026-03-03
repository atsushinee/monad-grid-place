use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use reqwest::Client;
use log::info;

mod config;
mod listener;
mod storage;
mod abi;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let config = config::Config::from_env()?;

    info!("Connecting to database...");
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;
    info!("Database connected.");

    let http_client = Client::new();

    listener::start_event_listener(
        &config,
        db_pool,
        http_client,
    ).await?;

    Ok(())
}
