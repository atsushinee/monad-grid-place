use std::env;
use anyhow::{Result, anyhow};

#[derive(Clone, Debug)]
pub struct Config {
    pub rpc_wss_url: String,
    pub contract_address: String,
    pub database_url: String,
    pub backend_api_url: String,
    
    // IPFS 配置：使用 Pinata 或本地 IPFS
    pub use_pinata: bool,
    pub ipfs_gateway_url: String,
    pub pinata_api_key: Option<String>,
    pub pinata_secret_key: Option<String>,
    pub pinata_jwt: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        let rpc_wss_url = env::var("RPC_WSS_URL")
            .map_err(|_| anyhow!("RPC_WSS_URL must be set"))?;
        let contract_address = env::var("CONTRACT_ADDRESS")
            .map_err(|_| anyhow!("CONTRACT_ADDRESS must be set"))?;
        let database_url = env::var("DATABASE_URL")
            .map_err(|_| anyhow!("DATABASE_URL must be set"))?;
        let backend_api_url = env::var("BACKEND_API_URL")
            .map_err(|_| anyhow!("BACKEND_API_URL must be set"))?;

        // IPFS 配置
        let use_pinata = env::var("USE_PINATA")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);

        let ipfs_gateway_url = if use_pinata {
            env::var("PINATA_GATEWAY_URL")
                .unwrap_or_else(|_| "https://gateway.pinata.cloud".to_string())
        } else {
            env::var("IPFS_GATEWAY_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:8080".to_string())
        };

        // Pinata API 凭证（可选，某些认证方式需要）
        let pinata_api_key = if use_pinata {
            Some(env::var("PINATA_API_KEY")
                .map_err(|_| anyhow!("PINATA_API_KEY must be set when USE_PINATA=true"))?)
        } else {
            None
        };

        let pinata_secret_key = if use_pinata {
            Some(env::var("PINATA_SECRET_KEY")
                .map_err(|_| anyhow!("PINATA_SECRET_KEY must be set when USE_PINATA=true"))?)
        } else {
            None
        };

        let pinata_jwt = env::var("PINATA_JWT").ok();

        Ok(Self {
            rpc_wss_url,
            contract_address,
            database_url,
            backend_api_url,
            use_pinata,
            ipfs_gateway_url,
            pinata_api_key,
            pinata_secret_key,
            pinata_jwt,
        })
    }
}
