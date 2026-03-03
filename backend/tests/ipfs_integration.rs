/// IPFS 服务集成测试
///
/// 这些测试验证 IPFS 服务的完整功能，包括：
/// - 上传数据到 IPFS（Pinata 和本地 IPFS）
/// - 从 IPFS 获取数据
///
/// 运行测试：
/// ```bash
/// cd backend
/// cargo test ipfs_integration -- --nocapture
/// ```
///
/// 环境变量要求：
/// - PINATA_API_KEY: Pinata API 密钥
/// - PINATA_SECRET_KEY: Pinata 密钥
/// - DATABASE_URL: 数据库连接字符串（可选，默认 postgres://localhost/test）
use dotenvy::dotenv;
use serde_json::json;
use std::sync::Arc;
use reqwest::Client;
use sqlx::postgres::PgPoolOptions;
use dashmap::DashMap;
use tokio::sync::RwLock;
use std::collections::HashMap;

use backend::{AppState, AppConfig, AppError};
use backend::services::ipfs_service;
use backend::models::grid::Snapshot;

/// 创建测试用的 AppState
async fn create_test_app_state() -> AppState {
    // 创建测试配置
    let config = AppConfig {
        port: 3000,
        database_url: std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://localhost/test".to_string()),
        ipfs_api_url: std::env::var("IPFS_API_URL")
            .unwrap_or_else(|_| "https://api.pinata.cloud".to_string()),
        ipfs_gateway_url: std::env::var("IPFS_GATEWAY_URL")
            .unwrap_or_else(|_| "https://gateway.pinata.cloud".to_string()),
        use_pinata: true,
        pinata_api_key: std::env::var("PINATA_API_KEY").ok(),
        pinata_secret_key: std::env::var("PINATA_SECRET_KEY").ok(),
        pinata_jwt: None,
    };

    // 创建数据库连接池
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(30))
        .connect(&config.database_url)
        .await
        .expect("无法连接到数据库");

    // 创建 HTTP 客户端
    let http_client = Client::new();

    // 创建缓存
    let cache = Arc::new(DashMap::new());

    // 创建 paint_metadata_cache
    let paint_metadata_cache = Arc::new(RwLock::new(HashMap::new()));

    AppState {
        config: Arc::new(config),
        http_client,
        db_pool,
        cache,
        paint_metadata_cache,
    }
}
fn init() {
    dotenv().ok(); // 忽略错误（比如 .env 不存在）
}

/// 测试 1: 上传简单 JSON 到 Pinata
#[tokio::test]
async fn test_add_json_to_pinata() {
    init();
    println!("\n=== 测试上传 JSON 到 Pinata ===\n");

    // 检查环境变量
    if std::env::var("PINATA_API_KEY").is_err() {
        println!("⚠️  跳过测试：PINATA_API_KEY 未设置");
        return;
    }

    let state = create_test_app_state().await;

    // 创建测试数据
    let test_data = json!({
        "test": "add_json_to_ipfs",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "message": "测试 IPFS 上传功能"
    });

    println!("📤 测试数据：{}", test_data);

    // 上传到 IPFS
    let cid: String = ipfs_service::add_json_to_ipfs(&state, &test_data)
        .await
        .expect("上传到 IPFS 失败");

    println!("✅ 上传成功！");
    println!("📦 CID: {}", cid);
    println!("🔗 Gateway URL: {}/ipfs/{}", state.config.ipfs_gateway_url, cid);

    // 验证 CID
    assert!(!cid.is_empty(), "CID 不应为空");
    assert!(cid.len() > 10, "CID 长度异常");
}

/// 测试 2: 上传后获取 - 完整链路测试
#[tokio::test]
async fn test_add_and_fetch_snapshot() {
    init();
    println!("\n=== 测试上传并获取快照 - 完整链路 ===\n");

    // 检查环境变量
    if std::env::var("PINATA_API_KEY").is_err() {
        println!("⚠️  跳过测试：PINATA_API_KEY 未设置");
        return;
    }

    let state = create_test_app_state().await;

    // 创建测试快照数据
    let test_snapshot = json!({
        "version": "1.0",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "owner": "0xTestOwner123456789",
        "metadata": {
            "name": "测试快照",
            "description": "用于测试 IPFS 上传和获取"
        },
        "pixels": [
            {
                "index": 1001,
                "x": 1,
                "y": 1,
                "color": "#FF0000",
                "link": "https://example.com",
                "message": "测试像素 1",
                "timestamp": chrono::Utc::now().timestamp() as u64,
                "extra_data": null
            },
            {
                "index": 2002,
                "x": 2,
                "y": 2,
                "color": "#00FF00",
                "link": "https://example.org",
                "message": "测试像素 2",
                "timestamp": chrono::Utc::now().timestamp() as u64,
                "extra_data": null
            }
        ]
    });

    println!("📤 上传测试快照...");

    // 上传到 IPFS
    let cid: String = ipfs_service::add_json_to_ipfs(&state, &test_snapshot)
        .await
        .expect("上传到 IPFS 失败");

    println!("✅ 上传成功，CID: {}", cid);

    // 等待 IPFS 传播
    println!("⏳ 等待 IPFS 传播...");
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    // 从 IPFS 获取
    println!("📥 从 IPFS 获取快照...");
    let fetched_snapshot: Snapshot = ipfs_service::fetch_snapshot_from_ipfs(&state, &cid)
        .await
        .expect("从 IPFS 获取失败");

    println!("✅ 获取成功！");
    println!("📦 快照版本：{}", fetched_snapshot.version);
    println!("📦 所有者：{}", fetched_snapshot.owner);
    println!("📦 像素数量：{}", fetched_snapshot.pixels.len());

    // 验证数据
    assert_eq!(fetched_snapshot.version, "1.0", "版本号不匹配");
    assert_eq!(fetched_snapshot.owner, "0xTestOwner123456789", "所有者不匹配");
    assert_eq!(fetched_snapshot.pixels.len(), 2, "像素数量不匹配");

    // 验证第一个像素
    let pixel1 = &fetched_snapshot.pixels[0];
    assert_eq!(pixel1.x, 1, "像素 1 X 坐标不匹配");
    assert_eq!(pixel1.y, 1, "像素 1 Y 坐标不匹配");
    assert_eq!(pixel1.color, "#FF0000", "像素 1 颜色不匹配");

    // 验证第二个像素
    let pixel2 = &fetched_snapshot.pixels[1];
    assert_eq!(pixel2.x, 2, "像素 2 X 坐标不匹配");
    assert_eq!(pixel2.y, 2, "像素 2 Y 坐标不匹配");
    assert_eq!(pixel2.color, "#00FF00", "像素 2 颜色不匹配");

    println!("✅ 完整链路测试通过！");
}

/// 测试 3: 获取不存在的 CID
#[tokio::test]
async fn test_fetch_nonexistent_cid() {
    init();
    println!("\n=== 测试获取不存在的 CID ===\n");

    let state = create_test_app_state().await;

    // 使用无效的 CID
    let invalid_cid = "QmInvalidCidForTesting123456789";

    println!("📥 尝试获取不存在的 CID: {}", invalid_cid);

    let result: Result<Snapshot, AppError> = 
        ipfs_service::fetch_snapshot_from_ipfs(&state, invalid_cid).await;

    // 应该返回错误
    assert!(result.is_err(), "获取不存在的 CID 应该返回错误");
    println!("✅ 正确返回错误：{:?}", result.unwrap_err());
}

/// 测试 4: 上传大数据
#[tokio::test]
async fn test_add_large_json() {
    init();
    println!("\n=== 测试上传大数据 ===\n");

    // 检查环境变量
    if std::env::var("PINATA_API_KEY").is_err() {
        println!("⚠️  跳过测试：PINATA_API_KEY 未设置");
        return;
    }

    let state = create_test_app_state().await;

    // 创建包含大量像素的测试数据
    let mut pixels = Vec::new();
    for i in 0..100 {
        pixels.push(json!({
            "index": i,
            "x": i % 10,
            "y": i / 10,
            "color": format!("#{:06X}", i * 0x111111),
            "link": format!("https://example{}.com", i),
            "message": format!("像素 {}", i),
            "timestamp": chrono::Utc::now().timestamp() as u64,
            "extra_data": null
        }));
    }

    let large_data = json!({
        "version": "1.0",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "owner": "0xLargeDataTest",
        "metadata": {
            "name": "大数据测试",
            "description": "测试上传包含 100 个像素的大数据"
        },
        "pixels": pixels
    });

    println!("📤 上传大数据（100 个像素）...");

    let cid: String = ipfs_service::add_json_to_ipfs(&state, &large_data)
        .await
        .expect("上传大数据失败");

    println!("✅ 上传成功，CID: {}", cid);

    // 验证 CID
    assert!(!cid.is_empty(), "CID 不应为空");

    // 等待传播
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // 获取并验证
    let fetched: Snapshot = ipfs_service::fetch_snapshot_from_ipfs(&state, &cid)
        .await
        .expect("获取大数据失败");

    assert_eq!(fetched.pixels.len(), 100, "像素数量不匹配");
    println!("✅ 大数据测试通过！");
}
