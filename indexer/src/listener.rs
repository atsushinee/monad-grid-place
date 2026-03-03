use ethers::{
    providers::{Provider, Ws},
    types::Address,
};
use std::sync::Arc;
use anyhow::Result;
use sqlx::PgPool;
use futures_util::stream::StreamExt;
use reqwest::Client;
use log::{info, error};

use crate::abi::MonadAdWall;
use crate::storage;
use crate::config::Config;

pub async fn start_event_listener(
    config: &Config,
    db_pool: PgPool,
    http_client: Client,
) -> Result<()> {
    info!("\n═══════════════════════════════════════════════════════════");
    info!("🚀 [Indexer] Starting Event Listener");
    info!("═══════════════════════════════════════════════════════════");
    info!("   - RPC URL: {}", config.rpc_wss_url);
    info!("   - Contract: {}", config.contract_address);
    info!("   - Backend API: {}", config.backend_api_url);
    info!("   - IPFS Mode: {}", if config.use_pinata { "Pinata" } else { "Local IPFS" });
    info!("   - IPFS Gateway: {}", config.ipfs_gateway_url);
    info!("═══════════════════════════════════════════════════════════\n");

    let provider = Provider::<Ws>::connect(&config.rpc_wss_url).await?;
    let client = Arc::new(provider);

    let address: Address = config.contract_address.parse()?;
    let contract = MonadAdWall::new(address, client);

    info!("✅ [Indexer] Successfully connected to WebSocket RPC.");
    info!("📡 [Indexer] Listening for AreaPainted events on contract: {}", config.contract_address);
    info!("═══════════════════════════════════════════════════════════\n");

    // 监听 AreaPainted 事件（IPFS 快照模式）
    let events = contract.area_painted_filter();
    let mut stream = events.subscribe().await?;

    while let Some(Ok(log)) = stream.next().await {
        info!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        info!("📍 [Indexer] Received AreaPainted event!");
        info!("   - Owner: 0x{:x}", log.owner);
        info!("   - CID Hash: 0x{}", hex::encode(log.cid_hash));
        info!("   - Pixel count: {}", log.pixel_count);
        info!("   - Total price: {} wei", log.total_price);
        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        if let Err(e) = storage::process_area_painted_event(
            &db_pool,
            &http_client,
            &config.backend_api_url,
            config,
            &log
        ).await {
            error!("❌ [Indexer] Error processing event: {:?}", e);
        }
        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    }

    error!("❌ [Indexer] Event stream ended unexpectedly.");
    Ok(())
}
