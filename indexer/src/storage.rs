use sqlx::PgPool;
use anyhow::{Result, anyhow};
use reqwest::Client;
use crate::abi::PaintedFilter;

async fn fetch_cid_from_cache(
    http_client: &Client,
    backend_url: &str,
    cid_hash: &str,
) -> Result<String> {
    let url = format!("{}/cache/{}", backend_url, cid_hash);
    let response = http_client.get(&url).send().await?;

    if !response.status().is_success() {
        return Err(anyhow!("Backend cache returned non-success status: {}", response.status()));
    }

    let cid = response.text().await?;
    Ok(cid)
}

pub async fn save_painted_event(
    db_pool: &PgPool,
    http_client: &Client,
    backend_url: &str,
    event: &PaintedFilter,
) -> Result<()> {
    let index_u32 = event.index.as_u32();
    let x = (index_u32 % 1000) as i32;
    let y = (index_u32 / 1000) as i32;
    let cid_hash_hex = format!("0x{}", hex::encode(event.cid_hash));

    // Fetch the original CID from the backend cache
    let original_cid = fetch_cid_from_cache(http_client, backend_url, &cid_hash_hex).await?;
    println!("Fetched original CID {} from cache for hash {}", original_cid, cid_hash_hex);

    sqlx::query!(
        r#"
        INSERT INTO grid_cells (x, y, color, owner, ipfs_cid, updated_at)
        VALUES ($1, $2, $3, $4, $5, NOW())
        ON CONFLICT (x, y) DO UPDATE
        SET color = $3, owner = $4, ipfs_cid = $5, updated_at = NOW()
        "#,
        x,
        y,
        format!("#{:06x}", event.color),
        format!("0x{:x}", event.owner),
        original_cid, // Store the original CID
    )
    .execute(db_pool)
    .await?;

    println!("Saved event for pixel ({}, {}) to DB with CID {}.", x, y, original_cid);
    Ok(())
}
