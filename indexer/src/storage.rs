use sqlx::PgPool;
use anyhow::Result;
use crate::abi::PaintedFilter;

pub async fn save_painted_event(db_pool: &PgPool, event: &PaintedFilter) -> Result<()> {
    let index_u32 = event.index.as_u32();
    let x = (index_u32 % 1000) as i32;
    let y = (index_u32 / 1000) as i32;

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
        format!("0x{}", hex::encode(event.cid_hash)),
    )
    .execute(db_pool)
    .await?;

    println!("Saved event for pixel ({}, {}) to DB.", x, y);
    Ok(())
}
