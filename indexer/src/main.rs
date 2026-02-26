use anyhow::Result;
use sqlx::postgres::PgPoolOptions;

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

    listener::start_event_listener(
        &config.rpc_wss_url,
        &config.contract_address,
        db_pool,
    ).await?;

    Ok(())
}
