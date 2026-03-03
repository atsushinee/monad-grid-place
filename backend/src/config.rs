use std::env;

#[derive(Clone)]
pub struct AppConfig {
    pub port: u16,
    pub ipfs_api_url: String,
    pub ipfs_gateway_url: String,
    pub database_url: String,
    // IPFS 配置：使用 Pinata 或本地 IPFS
    pub use_pinata: bool,
    pub pinata_api_key: Option<String>,
    pub pinata_secret_key: Option<String>,
    pub pinata_jwt: Option<String>,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let port = env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .expect("Invalid PORT");

        // IPFS 配置
        let use_pinata = env::var("USE_PINATA")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);

        let ipfs_api_url = if use_pinata {
            env::var("PINATA_API_URL")
                .unwrap_or_else(|_| "https://api.pinata.cloud".to_string())
        } else {
            env::var("IPFS_API_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:5001".to_string())
        };

        let ipfs_gateway_url = if use_pinata {
            env::var("PINATA_GATEWAY_URL")
                .unwrap_or_else(|_| "https://gateway.pinata.cloud".to_string())
        } else {
            env::var("IPFS_GATEWAY_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:8080".to_string())
        };

        let pinata_api_key = if use_pinata {
            Some(env::var("PINATA_API_KEY").expect("PINATA_API_KEY must be set when using Pinata"))
        } else {
            None
        };

        let pinata_secret_key = if use_pinata {
            Some(env::var("PINATA_SECRET_KEY").expect("PINATA_SECRET_KEY must be set when using Pinata"))
        } else {
            None
        };

        // Pinata JWT（可选，如果使用 JWT 认证）
        let pinata_jwt = env::var("PINATA_JWT").ok();

        let database_url = env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");

        Self {
            port,
            ipfs_api_url,
            ipfs_gateway_url,
            database_url,
            use_pinata,
            pinata_api_key,
            pinata_secret_key,
            pinata_jwt,
        }
    }
}
