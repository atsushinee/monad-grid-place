use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;

/// 数据库中的网格单元记录
/// color 字段直接存储在数据库中，用于快速渲染
#[derive(Debug, FromRow, Serialize, Deserialize, Clone)]
pub struct GridCell {
    pub id: i32,
    pub x: i32,
    pub y: i32,
    pub owner: String,
    pub ipfs_cid: String,
    pub color: String,
    pub link: Option<String>,
    pub message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// IPFS 快照的完整结构（支持扩展）
/// 
/// 设计原则：
/// 1. version 用于未来结构升级
/// 2. metadata 存储 Owner 级别的元数据
/// 3. pixels 中的 extraData 支持任意扩展字段
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Snapshot {
    pub version: String,
    pub timestamp: String,
    pub owner: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<SnapshotMetadata>,
    pub pixels: Vec<SnapshotPixelData>,
}

/// Owner 级别的元数据（可选）
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SnapshotMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub social: Option<JsonValue>,  // 社交信息（Twitter, Discord 等）
    #[serde(flatten)]
    pub extra: JsonValue,  // 其他任意字段
}

/// 快照中单个像素的数据结构（支持扩展）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SnapshotPixelData {
    /// 像素在网格中的位置索引 (index = y * 1000 + x)
    pub index: u64,
    pub x: i32,
    pub y: i32,
    pub color: String,
    #[serde(default)]
    pub link: String,
    #[serde(default)]
    pub message: String,
    /// 涂色时间戳（Unix 秒）
    pub timestamp: u64,
    /// 扩展字段：支持动画、NFT 绑定、时间限制等
    #[serde(default, rename = "extraData", skip_serializing_if = "JsonValue::is_null")]
    pub extra_data: JsonValue,
}

/// 前端提交的新像素数据（用于生成快照）
#[derive(Debug, Deserialize, Clone)]
pub struct NewPixelData {
    pub x: i32,
    pub y: i32,
    pub color: String,
    #[serde(default)]
    pub link: String,
    #[serde(default)]
    pub message: String,
    /// 扩展字段
    #[serde(default, rename = "extraData")]
    pub extra_data: Option<JsonValue>,
}

/// 数据库中的像素数据（用于合并快照）
#[derive(Debug, FromRow, Serialize, Deserialize, Clone)]
pub struct DbPixelData {
    pub x: i32,
    pub y: i32,
    pub owner: String,
    pub color: String,
    pub link: Option<String>,
    pub message: Option<String>,
}

/// 快照生成请求
#[derive(Debug, Deserialize)]
pub struct SnapshotRequest {
    pub owner: String,
    pub new_pixels: Vec<NewPixelData>,
    /// Owner 级别的元数据（可选）
    #[serde(default)]
    pub metadata: Option<SnapshotMetadata>,
}

/// 快照生成响应
#[derive(Debug, Serialize)]
pub struct SnapshotResponse {
    // IPFS 信息
    pub cid: String,
    pub cid_hash: String,
    
    // 价格信息
    pub pixel_count: u64,
    pub new_pixel_count: u64,
    pub update_pixel_count: u64,
    pub total_price: String,
    pub price_breakdown: PriceBreakdown,
    
    // 合约调用参数
    pub contract_params: ContractParams,
}

/// 价格明细
#[derive(Debug, Serialize)]
pub struct PriceBreakdown {
    pub base_price: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub premium_price: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discount: Option<String>,
    pub total: String,
}

/// 合约调用参数
#[derive(Debug, Serialize)]
pub struct ContractParams {
    pub function_name: String,
    pub cid_hash: String,
    pub pixel_count: String,
    pub value: String,
}
