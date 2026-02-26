use ethers::{
    providers::{Provider, Ws},
    types::{Address, Log},
    contract::Event,
};
use std::sync::Arc;
use anyhow::Result;
use sqlx::PgPool;
use futures_util::stream::StreamExt;
use reqwest::Client;

use crate::abi::MonadAdWall;
use crate::storage;
use crate::config::Config;

pub async fn start_event_listener(
    config: &Config,
    db_pool: PgPool,
    http_client: Client,
) -> Result<()> {
    let provider = Provider::<Ws>::connect(&config.rpc_wss_url).await?;
    let client = Arc::new(provider);

    let address: Address = config.contract_address.parse()?;
    let contract = MonadAdWall::new(address, client);

    println!("Successfully connected to WebSocket RPC.");
    println!("Listening for Painted events on contract: {}", config.contract_address);

    let events = contract.painted_filter();
    let mut stream = events.subscribe().await?.with_meta();

    while let Some(Ok((log, meta))) = stream.next().await {
        println!("---------------------------------");
        println!("Received Painted event in block: {}", meta.block_number);

        if let Err(e) = storage::save_painted_event(&db_pool, &http_client, &config.backend_api_url, &log).await {
            eprintln!("Error processing event: {:?}", e);
        }
        println!("---------------------------------");
    }

    eprintln!("Event stream ended unexpectedly.");
    Ok(())
}
