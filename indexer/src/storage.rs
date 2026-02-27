use sqlx::PgPool;
use anyhow::{Result, anyhow};
use reqwest::Client;

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
    ipfs_gateway_url: &str,
    event: &crate::abi::AreaPaintedFilter,
) -> Result<()> {
    let owner_hex = format!("0x{:x}", event.owner);
    let cid_hash_hex = format!("0x{}", hex::encode(event.cid_hash));

    println!("📍 Processing AreaPainted event for owner: {}", owner_hex);
    println!("   - CID Hash: {}", cid_hash_hex);
    println!("   - Pixel count: {}", event.pixel_count);
    println!("   - Total price: {} wei", event.total_price);

    // 1. 从后端缓存获取原始 CID
    let original_cid = fetch_cid_from_cache(http_client, backend_url, &cid_hash_hex).await?;
    println!("   ✅ Fetched Original CID: {}", original_cid);

    // 2. 从 IPFS 获取完整快照
    let snapshot = fetch_snapshot_from_ipfs(http_client, ipfs_gateway_url, &original_cid).await?;
    println!("   ✅ Fetched snapshot with {} pixels from IPFS", snapshot.pixels.len());

    let mut tx = db_pool.begin().await?;

    // 3. 删除该 owner 的所有旧记录
    let delete_result = sqlx::query("DELETE FROM grid_cells WHERE owner = $1")
        .bind(&owner_hex)
        .execute(&mut *tx)
        .await?;
    println!("   ✅ Deleted {} old records for owner {}", delete_result.rows_affected(), owner_hex);

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
    println!("   ✅ Inserted {} new records from snapshot", inserted_count);

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
    println!("   ✅ Recorded snapshot history");

    tx.commit().await?;
    println!("   ✅ Transaction committed. Snapshot processed successfully.");

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
        return Err(anyhow!("Backend returned error status for CID hash: {}", cid_hash));
    }

    let cid = response.text().await?;
    if cid.is_empty() || cid.starts_with("CID not found") {
        return Err(anyhow!("CID not found in backend cache for hash: {}", cid_hash));
    }
    Ok(cid)
}

/// 从 IPFS 获取完整快照
async fn fetch_snapshot_from_ipfs(
    http_client: &Client,
    ipfs_gateway_url: &str,
    cid: &str,
) -> Result<crate::abi::Snapshot> {
    let url = format!("{}/ipfs/{}", ipfs_gateway_url, cid);
    let response = http_client.get(&url).send().await?;

    if !response.status().is_success() {
        return Err(anyhow!("IPFS gateway returned error status for CID: {}", cid));
    }

    let snapshot = response.json::<crate::abi::Snapshot>().await?;
    Ok(snapshot)
}
