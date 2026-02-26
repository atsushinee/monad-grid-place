use std::env;
use anyhow::Result;

#[derive(Clone, Debug)]
pub struct Config {
    pub rpc_wss_url: String,
    pub contract_address: String,
    pub database_url: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        let rpc_wss_url = env::var("RPC_WSS_URL")?;
        let contract_address = env::var("CONTRACT_ADDRESS")?;
        let database_url = env::var("DATABASE_URL")?;

        Ok(Self {
            rpc_wss_url,
            contract_address,
            database_url,
        })
    }
}
