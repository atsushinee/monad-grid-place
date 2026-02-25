use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct GridCell {
    pub id: i32,
    pub x: i32,
    pub y: i32,
    pub color: String,
    pub owner: String,
    pub ipfs_cid: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
