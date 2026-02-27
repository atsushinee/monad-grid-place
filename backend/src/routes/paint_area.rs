use axum::{extract::State, Json};
use serde::Deserialize;
use crate::{
    AppError,
    AppState,
};

/// 涂色请求
#[derive(Debug, Deserialize)]
pub struct PaintAreaRequest {
    /// 用户地址
    pub owner: String,
    /// IPFS CID
    pub cid: String,
    /// CID Hash (bytes32)
    pub cid_hash: String,
    /// 像素总数
    pub pixel_count: u64,
    /// 总价格 (wei)
    pub total_price: String,
    /// 交易哈希（可选，链上确认后填充）
    #[serde(default)]
    pub tx_hash: Option<String>,
    /// 区块号（可选，链上确认后填充）
    #[serde(default)]
    pub block_number: Option<u64>,
}

/// 涂色响应
#[derive(Debug, serde::Serialize)]
pub struct PaintAreaResponse {
    pub success: bool,
    pub message: String,
    pub snapshot_id: Option<i32>,
}

/// 提交涂色快照记录
/// 
/// 此接口用于：
/// 1. 在用户调用合约前，预先记录快照信息
/// 2. 或者在链上确认后，更新交易信息
pub async fn submit_paint_area_handler(
    State(state): State<AppState>,
    Json(payload): Json<PaintAreaRequest>,
) -> Result<Json<PaintAreaResponse>, AppError> {
    // 插入快照历史记录
    let result = sqlx::query!(
        r#"
        INSERT INTO snapshot_history (owner, cid, cid_hash, pixel_count, total_price, tx_hash, block_number)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (cid_hash) DO UPDATE
        SET tx_hash = COALESCE($6, snapshot_history.tx_hash),
            block_number = COALESCE($7, snapshot_history.block_number)
        RETURNING id
        "#,
        &payload.owner,
        &payload.cid,
        &payload.cid_hash,
        payload.pixel_count as i32,
        &payload.total_price,
        payload.tx_hash,
        payload.block_number.map(|n| n as i64),
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| AppError::InternalServerError(format!("Failed to insert snapshot history: {}", e)))?;

    Ok(Json(PaintAreaResponse {
        success: true,
        message: "Snapshot recorded successfully".to_string(),
        snapshot_id: Some(result.id),
    }))
}

/// 获取用户的快照历史
#[derive(Debug, serde::Serialize)]
pub struct SnapshotHistoryRecord {
    pub id: i32,
    pub owner: String,
    pub cid: String,
    pub cid_hash: String,
    pub pixel_count: i32,
    pub total_price: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub tx_hash: Option<String>,
    pub block_number: Option<i64>,
}

#[derive(Debug, serde::Serialize)]
pub struct SnapshotHistoryList {
    pub snapshots: Vec<SnapshotHistoryRecord>,
    pub total: i64,
}

pub async fn get_snapshot_history_handler(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HistoryQuery>,
) -> Result<Json<SnapshotHistoryList>, AppError> {
    let owner = params.owner.to_lowercase();
    let limit = params.limit.unwrap_or(20);
    let offset = params.offset.unwrap_or(0);

    // 查询总数
    let count = sqlx::query_scalar!(
        r#"SELECT COUNT(*) FROM snapshot_history WHERE LOWER(owner) = $1"#,
        &owner
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| AppError::InternalServerError(format!("Failed to count snapshots: {}", e)))?
    .unwrap_or(0);

    // 查询列表
    let snapshots = sqlx::query_as!(
        SnapshotHistoryRecord,
        r#"
        SELECT id, owner, cid, cid_hash, pixel_count, total_price, created_at, tx_hash, block_number
        FROM snapshot_history
        WHERE LOWER(owner) = $1
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
        &owner,
        limit as i64,
        offset as i64,
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| AppError::InternalServerError(format!("Failed to query snapshot history: {}", e)))?;

    Ok(Json(SnapshotHistoryList {
        snapshots,
        total: count,
    }))
}

#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    pub owner: String,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}
