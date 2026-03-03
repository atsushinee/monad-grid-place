use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use crate::{AppState, AppError};
use log::info;

/// 提交像素元数据请求
#[derive(Debug, Deserialize)]
pub struct PaintMetadataRequest {
    pub player: String,
    pub indices: Vec<String>,
    pub pixels: Vec<PixelData>,
}

/// 像素数据
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PixelData {
    pub x: i32,
    pub y: i32,
    pub color: String,
    pub link: String,
    pub message: String,
}

/// 临时存储像素元数据（内存缓存）
/// 在实际生产中，应该使用 Redis 或数据库
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

/// 玩家最近的像素元数据
pub type PaintMetadataCache = Arc<RwLock<HashMap<String, Vec<PixelData>>>>;

/// 处理提交像素元数据
pub async fn submit_paint_metadata_handler(
    State(state): State<AppState>,
    Json(payload): Json<PaintMetadataRequest>,
) -> Result<StatusCode, AppError> {
    // 将元数据存入缓存
    let mut cache = state.paint_metadata_cache.write().await;
    cache.insert(payload.player.clone(), payload.pixels);

    info!("📦 Stored paint metadata for player: {}", payload.player);

    Ok(StatusCode::OK)
}

/// 获取玩家最近的像素元数据
pub async fn get_paint_metadata_handler(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<GetMetadataParams>,
) -> Result<Json<Vec<PixelData>>, AppError> {
    let cache = state.paint_metadata_cache.read().await;

    let pixels = cache.get(&params.player)
        .cloned()
        .unwrap_or_default();

    Ok(Json(pixels))
}

#[derive(Debug, Deserialize)]
pub struct GetMetadataParams {
    pub player: String,
    #[serde(default)]
    pub count: usize,
}
