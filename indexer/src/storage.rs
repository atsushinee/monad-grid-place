use sqlx::PgPool;
use anyhow::{Result, anyhow};
use reqwest::Client;
use log::{info, error};

/// 处理 AreaPainted 事件（V6/V7 IPFS 快照模式）
///
/// 此事件包含：
/// - owner: 像素所有者
/// - cidHash: IPFS CID 的哈希
/// - pixelCount: 像素数量
/// - totalPrice: 总价格
pub async fn process_area_painted_event(
    db_pool: &PgPool,
    http_client: &Client,
    backend_url: &str,
    config: &crate::config::Config,
    event: &crate::abi::AreaPaintedFilter,
) -> Result<()> {
    let owner_hex = format!("0x{:x}", event.owner);
    let cid_hash_hex = format!("0x{}", hex::encode(event.cid_hash));

    info!("📦 [Indexer] Processing AreaPainted event:");
    info!("   - Owner: {}", owner_hex);
    info!("   - CID Hash: {}", cid_hash_hex);
    info!("   - Pixel count: {}", event.pixel_count);
    info!("   - Total price: {} wei", event.total_price);

    // 1. 从后端缓存获取原始 CID
    info!("🔍 [Indexer] Step 1: Fetching CID from backend cache...");
    info!("   - Request: GET {}/cache/{}", backend_url, cid_hash_hex);
    let original_cid = fetch_cid_from_cache(http_client, backend_url, &cid_hash_hex).await?;
    info!("✅ [Indexer] Step 1 completed - Original CID: {}", original_cid);

    // 2. 从 IPFS 获取完整快照（支持 Pinata 和本地 IPFS）
    info!("🔍 [Indexer] Step 2: Fetching snapshot from IPFS...");
    info!("   - IPFS Mode: {}", if config.use_pinata { "Pinata" } else { "Local IPFS" });
    info!("   - Gateway: {}", config.ipfs_gateway_url);
    info!("   - CID: {}", original_cid);
    let snapshot = fetch_snapshot_from_ipfs(
        http_client,
        config,
        &original_cid,
    ).await?;
    info!("✅ [Indexer] Step 2 completed - Snapshot has {} pixels", snapshot.pixels.len());

    info!("🔍 [Indexer] Step 3: Updating database...");
    let mut tx = db_pool.begin().await?;

    // 3. 删除该 owner 的所有旧记录
    let delete_result = sqlx::query("DELETE FROM grid_cells WHERE owner = $1")
        .bind(&owner_hex)
        .execute(&mut *tx)
        .await?;
    info!("   - Deleted {} old records for owner {}", delete_result.rows_affected(), owner_hex);

    // 4. 插入所有新记录（包含 color, link, message, timestamp 等字段）
    let mut inserted_count = 0;
    for pixel in &snapshot.pixels {
        let timestamp = pixel.timestamp as i64;

        sqlx::query(
            r#"
            INSERT INTO grid_cells (x, y, owner, ipfs_cid, color, link, message, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, TO_TIMESTAMP($8))
            ON CONFLICT (x, y) DO UPDATE
            SET owner = $3,
                ipfs_cid = $4,
                color = $5,
                link = $6,
                message = $7,
                updated_at = TO_TIMESTAMP($8)
            "#
        )
        .bind(pixel.x)
        .bind(pixel.y)
        .bind(&owner_hex)
        .bind(&original_cid)
        .bind(&pixel.color)
        .bind(&pixel.link)
        .bind(&pixel.message)
        .bind(timestamp)
        .execute(&mut *tx)
        .await?;
        inserted_count += 1;
    }
    info!("   - Inserted {} new records from snapshot", inserted_count);

    // 5. 记录快照历史
    let _snapshot_result = sqlx::query!(
        r#"
        INSERT INTO snapshot_history (owner, cid, cid_hash, pixel_count, total_price)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (cid_hash) DO NOTHING
        "#,
        &owner_hex,
        &original_cid,
        &cid_hash_hex,
        snapshot.pixels.len() as i32,
        event.total_price.to_string(),
    )
    .execute(&mut *tx)
    .await?;
    info!("   - Recorded snapshot history");

    tx.commit().await?;
    info!("✅ [Indexer] Step 3 completed - Transaction committed");
    info!("✅ [Indexer] Event processed successfully!");
    info!("   - Owner: {}", owner_hex);
    info!("   - CID: {}", original_cid);
    info!("   - Pixels updated: {}", inserted_count);

    Ok(())
}

/// 从后端缓存获取原始 CID
async fn fetch_cid_from_cache(
    http_client: &Client,
    backend_url: &str,
    cid_hash: &str,
) -> Result<String> {
    let url = format!("{}/cache/{}", backend_url, cid_hash);

    let response = http_client.get(&url).send().await?;

    if !response.status().is_success() {
        let status = response.status();
        error!("❌ [Indexer] Backend cache returned error status: {}", status);
        return Err(anyhow!("Backend returned error status for CID hash: {}", cid_hash));
    }

    let cid = response.text().await?;
    if cid.is_empty() || cid.starts_with("CID not found") {
        error!("❌ [Indexer] CID not found in backend cache");
        return Err(anyhow!("CID not found in backend cache for hash: {}", cid_hash));
    }

    Ok(cid)
}

/// 从 IPFS 获取完整快照（支持 Pinata 和本地 IPFS）
async fn fetch_snapshot_from_ipfs(
    http_client: &Client,
    config: &crate::config::Config,
    cid: &str,
) -> Result<crate::abi::Snapshot> {
    info!("   - Fetching from: {}", config.ipfs_gateway_url);

    let snapshot = if config.use_pinata {
        // 使用 Pinata Gateway
        fetch_from_pinata_gateway(http_client, config, cid).await?
    } else {
        // 使用本地 IPFS Gateway
        fetch_from_local_ipfs_gateway(http_client, config, cid).await?
    };

    Ok(snapshot)
}

/// 从 Pinata Gateway 获取快照
async fn fetch_from_pinata_gateway(
    http_client: &Client,
    config: &crate::config::Config,
    cid: &str,
) -> Result<crate::abi::Snapshot> {
    let ipfs_gateway_url = &config.ipfs_gateway_url;
    let url = format!("{}/ipfs/{}", ipfs_gateway_url, cid);

    info!("   - Pinata Gateway URL: {}", url);

    let mut request = http_client.get(&url);

    // 如果配置了 Pinata JWT，添加认证头
    if let Some(pinata_jwt) = &config.pinata_jwt {
        if !pinata_jwt.is_empty() {
            info!("   - Using JWT authentication");
            request = request.header("Authorization", format!("Bearer {}", pinata_jwt));
        }
    }

    let response = request.send().await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown Pinata Gateway error".to_string());
        error!("❌ [Indexer] Pinata Gateway returned error: {} - {}", status, error_text);
        return Err(anyhow!("Pinata Gateway returned an error: {}", error_text));
    }

    let snapshot = response.json::<crate::abi::Snapshot>().await?;
    info!("   - Received {} pixels from Pinata Gateway", snapshot.pixels.len());
    Ok(snapshot)
}

/// 从本地 IPFS Gateway 获取快照
async fn fetch_from_local_ipfs_gateway(
    http_client: &Client,
    config: &crate::config::Config,
    cid: &str,
) -> Result<crate::abi::Snapshot> {
    let ipfs_gateway_url = &config.ipfs_gateway_url;
    let url = format!("{}/ipfs/{}", ipfs_gateway_url, cid);

    info!("   - Local IPFS Gateway URL: {}", url);

    let response = http_client.get(&url).send().await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown IPFS error".to_string());
        error!("❌ [Indexer] Local IPFS Gateway returned error: {} - {}", status, error_text);
        return Err(anyhow!("IPFS Gateway returned an error: {}", error_text));
    }

    let snapshot = response.json::<crate::abi::Snapshot>().await?;
    info!("   - Received {} pixels from Local IPFS Gateway", snapshot.pixels.len());
    Ok(snapshot)
}
