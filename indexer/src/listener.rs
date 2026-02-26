use ethers::{
    prelude::*,
    providers::{Provider, Ws},
    types::Address,
};
use std::sync::Arc;
use anyhow::Result;
use sqlx::PgPool;
use futures_util::stream::StreamExt;

use crate::abi::MonadAdWall;
use crate::storage;

pub async fn start_event_listener(
    rpc_url: &str,
    contract_address: &str,
    db_pool: PgPool,
) -> Result<()> {
    let provider = Provider::<Ws>::connect(rpc_url).await?;
    let client = Arc::new(provider);

    let address: Address = contract_address.parse()?;
    let contract = MonadAdWall::new(address, client);

    println!("Successfully connected to WebSocket RPC.");
    println!("Listening for Painted events on contract: {}", contract_address);

    let events = contract.painted_filter();
    let mut stream = events.subscribe().await?.with_meta();

    while let Some(Ok((log, meta))) = stream.next().await {
        println!("---------------------------------");
        println!("Received Painted event in block: {}", meta.block_number);
        println!("  - Index: {}", log.index);
        println!("  - Owner: 0x{:x}", log.owner);
        println!("  - Color: #{:06x}", log.color);
        println!("  - CID Hash: 0x{}", hex::encode(log.cid_hash));

        if let Err(e) = storage::save_painted_event(&db_pool, &log).await {
            eprintln!("Error saving event to database: {:?}", e);
        }
        println!("---------------------------------");
    }

    eprintln!("Event stream ended unexpectedly.");
    Ok(())
}
