use std::env;
use anyhow::Result;

#[derive(Clone, Debug)]
pub struct Config {
    pub rpc_wss_url: String,
    pub contract_address: String,
    pub database_url: String,
    pub backend_api_url: String,
    pub ipfs_gateway_url: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        let rpc_wss_url = env::var("RPC_WSS_URL")?;
        let contract_address = env::var("CONTRACT_ADDRESS")?;
        let database_url = env::var("DATABASE_URL")?;
        let backend_api_url = env::var("BACKEND_API_URL")?;
        let ipfs_gateway_url = env::var("IPFS_GATEWAY_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:8080".to_string());

        Ok(Self {
            rpc_wss_url,
            contract_address,
            database_url,
            backend_api_url,
            ipfs_gateway_url,
        })
    }
}
