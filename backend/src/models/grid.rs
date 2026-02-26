use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};

#[derive(Debug, FromRow, Serialize, Deserialize, Clone)]
pub struct GridCell {
    pub id: i32,
    pub x: i32,
    pub y: i32,
    pub color: String,
    pub owner: String,
    pub ipfs_cid: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    // This field will be populated from IPFS after fetching from DB
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,
}
