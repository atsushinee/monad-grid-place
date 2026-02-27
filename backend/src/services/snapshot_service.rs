use std::collections::HashMap;
use ethers::utils::keccak256;
use hex;
use serde_json::json;
use crate::{
    models::grid::{
        Snapshot, SnapshotPixelData, SnapshotRequest, SnapshotResponse,
        PriceBreakdown, ContractParams,
    },
    services::{ipfs_service, cache_service, grid_service},
    AppError,
    AppState,
};

/// 基础价格：0.01 ETH per pixel
const BASE_PRICE_WEI: u64 = 10_000_000_000_000_000;

/// V7 链上竞价模式：计算像素价格（从链上读取）
/// 注意：V7 模式下，前端直接从链上读取价格，此函数仅作为备用
#[allow(dead_code)]
pub async fn calculate_onchain_price(
    _state: &AppState,
    indices: &[u64],
) -> Result<u64, AppError> {
    // V7 模式下，价格从链上读取
    // 这里只是模拟计算，实际价格应该从链上获取
    let total_price = indices.len() as u64 * BASE_PRICE_WEI;
    Ok(total_price)
}

pub async fn generate_snapshot(
    state: &AppState,
    payload: &SnapshotRequest,
) -> Result<SnapshotResponse, AppError> {
    // 1. 从数据库获取 owner 已有的像素
    let old_pixels = grid_service::get_owner_pixels(&state.db_pool, &payload.owner)
        .await?;
    println!("📦 generate_snapshot: old_pixels count = {}", old_pixels.len());
    println!("📦 generate_snapshot: new_pixels count = {}", payload.new_pixels.len());

    // 2. 合并旧像素和新像素（新像素覆盖同坐标的旧像素）
    let mut pixel_map: HashMap<(i32, i32), SnapshotPixelData> = HashMap::new();
    let current_timestamp = chrono::Utc::now().timestamp() as u64;

    // 添加旧像素
    for pixel in old_pixels {
        let index = (pixel.y as u64) * 1000 + (pixel.x as u64);
        pixel_map.insert((pixel.x, pixel.y), SnapshotPixelData {
            index,
            x: pixel.x,
            y: pixel.y,
            color: pixel.color,
            link: pixel.link.unwrap_or_default(),
            message: pixel.message.unwrap_or_default(),
            timestamp: current_timestamp, // 旧像素使用当前时间
            extra_data: serde_json::Value::Null,
        });
    }
    println!("📦 pixel_map after adding old_pixels: {} pixels", pixel_map.len());

    // 添加新像素
    let mut new_pixel_count = 0;
    for new_pixel in &payload.new_pixels {
        let key = (new_pixel.x, new_pixel.y);
        let is_new = !pixel_map.contains_key(&key);
        let index = (new_pixel.y as u64) * 1000 + (new_pixel.x as u64);

        pixel_map.insert(
            key,
            SnapshotPixelData {
                index,
                x: new_pixel.x,
                y: new_pixel.y,
                color: new_pixel.color.clone(),
                link: new_pixel.link.clone(),
                message: new_pixel.message.clone(),
                timestamp: current_timestamp,
                extra_data: new_pixel.extra_data.clone().unwrap_or(serde_json::Value::Null),
            }
        );

        if is_new {
            new_pixel_count += 1;
        }
    }
    println!("📦 pixel_map after adding new_pixels: {} pixels, new_pixel_count = {}", pixel_map.len(), new_pixel_count);

    let all_pixels: Vec<SnapshotPixelData> = pixel_map.values().cloned().collect();
    let update_pixel_count = all_pixels.len() as u64 - new_pixel_count;
    println!("📦 all_pixels count = {}, update_pixel_count = {}", all_pixels.len(), update_pixel_count);

    // 3. 创建快照 JSON
    let snapshot = Snapshot {
        version: "1.0".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        owner: payload.owner.clone(),
        metadata: payload.metadata.clone(),
        pixels: all_pixels.clone(),
    };
    let snapshot_json = json!(snapshot);

    // 4. 上传到 IPFS
    let cid = ipfs_service::add_json_to_ipfs(state, &snapshot_json).await?;
    let cid_hash = keccak256(cid.as_bytes());
    let cid_hash_hex = format!("0x{}", hex::encode(cid_hash));

    // 5. 缓存映射（供 Indexer 使用）
    cache_service::set_cache(&state.cache, &cid_hash_hex, &cid);

    // 6. 计算价格
    let base_price = new_pixel_count * BASE_PRICE_WEI;
    // TODO: 未来可以在这里添加溢价逻辑（热门位置、需求定价等）
    let premium_price: Option<u64> = None;
    let discount: Option<u64> = None;
    let total_price = base_price;

    // 7. 准备响应
    Ok(SnapshotResponse {
        cid: cid.clone(),
        cid_hash: cid_hash_hex.clone(),
        pixel_count: all_pixels.len() as u64,
        new_pixel_count,
        update_pixel_count,
        total_price: total_price.to_string(),
        price_breakdown: PriceBreakdown {
            base_price: base_price.to_string(),
            premium_price: premium_price.map(|p| p.to_string()),
            discount: discount.map(|d| d.to_string()),
            total: total_price.to_string(),
        },
        contract_params: ContractParams {
            function_name: "paintArea".to_string(),
            cid_hash: cid_hash_hex,
            pixel_count: all_pixels.len().to_string(),
            value: total_price.to_string(),
        },
    })
}
