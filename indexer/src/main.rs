use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use reqwest::Client;

mod config;
mod listener;
mod storage;
mod abi;

#[tokio::main]
async fn main() -> Result<()> {
    let config = config::Config::from_env()?;

    println!("Connecting to database...");
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;
    println!("Database connected.");

    let http_client = Client::new();

    listener::start_event_listener(
        &config,
        db_pool,
        http_client,
    ).await?;

    Ok(())
}
