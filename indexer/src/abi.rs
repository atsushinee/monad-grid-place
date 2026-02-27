use ethers::prelude::abigen;
use serde_json::Value as JsonValue;

abigen!(
    MonadAdWall,
    "./src/abi/MonadAdWall.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

/// IPFS 快照的完整结构（支持扩展）
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct Snapshot {
    pub version: String,
    pub timestamp: String,
    pub owner: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<SnapshotMetadata>,
    pub pixels: Vec<SnapshotPixelData>,
}

/// Owner 级别的元数据（可选）
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone, Default)]
pub struct SnapshotMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub social: Option<JsonValue>,
    #[serde(flatten)]
    pub extra: JsonValue,
}

/// 快照中单个像素的数据结构（支持扩展）
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
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
    #[serde(default, rename = "extraData")]
    pub extra_data: JsonValue,
}
