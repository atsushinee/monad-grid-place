use std::env;

#[derive(Clone)]
pub struct AppConfig {
    pub port: u16,
    pub ipfs_api_url: String,
    pub database_url: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let port = env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .expect("Invalid PORT");

        let ipfs_api_url = env::var("IPFS_API_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:5001".to_string());

        let database_url = env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");

        Self { port, ipfs_api_url, database_url }
    }
}
