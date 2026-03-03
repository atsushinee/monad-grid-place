use reqwest::multipart;
use serde::Deserialize;
use serde_json::Value;
use crate::{AppError, AppState};
use crate::models::grid::Snapshot;
use log::{info, debug};

#[derive(Deserialize)]
struct IpfsAddResponse {
    #[serde(rename = "Hash")]
    hash: String,
}

#[derive(Deserialize)]
struct PinataPinResponse {
    #[serde(rename = "IpfsHash")]
    ipfs_hash: String,
}

/// 将 JSON 数据添加到 IPFS（支持 Pinata 和本地 IPFS）
pub async fn add_json_to_ipfs(
    state: &AppState,
    json_data: &Value,
) -> Result<String, AppError> {
    let json_string = serde_json::to_string(json_data)
        .map_err(|e| AppError::InternalServerError(format!("Failed to serialize JSON: {}", e)))?;

    info!("📤 [IPFS] Uploading snapshot to IPFS...");
    info!("   - Mode: {}", if state.config.use_pinata { "Pinata" } else { "Local IPFS" });
    debug!("   - JSON size: {} bytes", json_string.len());

    let cid = if state.config.use_pinata {
        // 使用 Pinata
        add_to_pinata(state, &json_string).await?
    } else {
        // 使用本地 IPFS
        add_to_local_ipfs(state, &json_string).await?
    };

    info!("✅ [IPFS] Upload successful! CID: {}", cid);
    Ok(cid)
}

/// 添加到 Pinata
async fn add_to_pinata(state: &AppState, json_string: &str) -> Result<String, AppError> {
    let pinata_api_key = state.config.pinata_api_key.as_ref()
        .ok_or_else(|| AppError::InternalServerError("Pinata API key not configured".to_string()))?;
    let pinata_secret_key = state.config.pinata_secret_key.as_ref()
        .ok_or_else(|| AppError::InternalServerError("Pinata secret key not configured".to_string()))?;

    let part = multipart::Part::bytes(json_string.as_bytes().to_vec())
        .file_name("metadata.json");
    let form = multipart::Form::new().part("file", part);

    let url = format!("{}/pinning/pinFileToIPFS", state.config.ipfs_api_url);

    info!("📡 [Pinata] Sending request to: {}", url);
    debug!("   - API Key: {}...", &pinata_api_key[..8]);

    let response = state.http_client
        .post(&url)
        .header("pinata_api_key", pinata_api_key)
        .header("pinata_secret_api_key", pinata_secret_key)
        .multipart(form)
        .send()
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to send request to Pinata: {}", e)))?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown Pinata error".to_string());
        return Err(AppError::InternalServerError(format!(
            "Pinata API returned an error: {}",
            error_text
        )));
    }

    let ipfs_response = response
        .json::<PinataPinResponse>()
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to parse Pinata response: {}", e)))?;

    info!("📥 [Pinata] Response received: IpfsHash = {}", ipfs_response.ipfs_hash);
    Ok(ipfs_response.ipfs_hash)
}

/// 添加到本地 IPFS
async fn add_to_local_ipfs(
    state: &AppState,
    json_string: &str,
) -> Result<String, AppError> {
    let part = multipart::Part::bytes(json_string.as_bytes().to_vec()).file_name("data.json");
    let form = multipart::Form::new().part("file", part);

    let ipfs_api_url = &state.config.ipfs_api_url;
    let url = format!("{}/api/v0/add", ipfs_api_url);

    info!("📡 [Local IPFS] Sending request to: {}", url);

    let response = state.http_client
        .post(&url)
        .multipart(form)
        .send()
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to send request to IPFS: {}", e)))?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown IPFS error".to_string());
        return Err(AppError::InternalServerError(format!(
            "IPFS API returned an error: {}",
            error_text
        )));
    }

    let ipfs_response = response
        .json::<IpfsAddResponse>()
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to parse IPFS response: {}", e)))?;

    info!("📥 [Local IPFS] Response received: Hash = {}", ipfs_response.hash);
    Ok(ipfs_response.hash)
}

/// 从 IPFS 获取完整的快照数据（支持 Pinata 和本地 IPFS）
pub async fn fetch_snapshot_from_ipfs(
    state: &AppState,
    cid: &str,
) -> Result<Snapshot, AppError> {
    info!("📥 [IPFS] Fetching snapshot from IPFS...");
    info!("   - Mode: {}", if state.config.use_pinata { "Pinata" } else { "Local IPFS" });
    info!("   - CID: {}", cid);

    let snapshot = if state.config.use_pinata {
        // 使用 Pinata Gateway
        fetch_from_pinata_gateway(state, cid).await?
    } else {
        // 使用本地 IPFS Gateway
        fetch_from_local_ipfs_gateway(state, cid).await?
    };

    info!("✅ [IPFS] Snapshot fetched successfully, {} pixels", snapshot.pixels.len());
    Ok(snapshot)
}

/// 从 Pinata Gateway 获取
async fn fetch_from_pinata_gateway(
    state: &AppState,
    cid: &str,
) -> Result<Snapshot, AppError> {
    let ipfs_gateway_url = &state.config.ipfs_gateway_url;
    let url = format!("{}/ipfs/{}", ipfs_gateway_url, cid);

    info!("📡 [Pinata Gateway] Requesting: {}", url);

    let mut request = state.http_client.get(&url);

    // 如果配置了 Pinata JWT，添加认证头
    if let Some(pinata_jwt) = &state.config.pinata_jwt {
        if !pinata_jwt.is_empty() {
            debug!("   - Using JWT authentication");
            request = request.header("Authorization", format!("Bearer {}", pinata_jwt));
        }
    }

    let response = request
        .send()
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to send request to Pinata Gateway: {}", e)))?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown Pinata Gateway error".to_string());
        return Err(AppError::InternalServerError(format!(
            "Pinata Gateway returned an error: {}",
            error_text
        )));
    }

    let snapshot = response
        .json::<Snapshot>()
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to parse snapshot from Pinata: {}", e)))?;

    info!("📥 [Pinata Gateway] Response received, {} pixels", snapshot.pixels.len());
    Ok(snapshot)
}

/// 从本地 IPFS Gateway 获取
async fn fetch_from_local_ipfs_gateway(
    state: &AppState,
    cid: &str,
) -> Result<Snapshot, AppError> {
    let ipfs_gateway_url = &state.config.ipfs_gateway_url;
    let url = format!("{}/ipfs/{}", ipfs_gateway_url, cid);

    info!("📡 [Local IPFS Gateway] Requesting: {}", url);

    let response = state.http_client
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to send request to IPFS: {}", e)))?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown IPFS error".to_string());
        return Err(AppError::InternalServerError(format!(
            "IPFS Gateway returned an error: {}",
            error_text
        )));
    }

    let snapshot = response
        .json::<Snapshot>()
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to parse IPFS snapshot: {}", e)))?;

    info!("📥 [Local IPFS Gateway] Response received, {} pixels", snapshot.pixels.len());
    Ok(snapshot)
}
