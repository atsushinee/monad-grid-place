use reqwest::multipart;
use serde::Deserialize;
use serde_json::Value;
use crate::{AppError, AppState};

#[derive(Deserialize)]
struct IpfsAddResponse {
    #[serde(rename = "Hash")]
    hash: String,
}

#[derive(Deserialize)]
pub struct IpfsMetadata {
    pub link: String,
    pub message: String,
}

pub async fn add_json_to_ipfs(
    state: &AppState,
    json_data: &Value,
) -> Result<String, AppError> {
    let json_string = serde_json::to_string(json_data)
        .map_err(|e| AppError::InternalServerError(format!("Failed to serialize JSON: {}", e)))?;

    let part = multipart::Part::bytes(json_string.into_bytes()).file_name("data.json");
    let form = multipart::Form::new().part("file", part);

    let ipfs_api_url = &state.config.ipfs_api_url;
    let url = format!("{}/api/v0/add", ipfs_api_url);

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

    Ok(ipfs_response.hash)
}

pub async fn fetch_metadata_from_ipfs(
    state: &AppState,
    cid: &str,
) -> Result<IpfsMetadata, AppError> {
    let ipfs_api_url = &state.config.ipfs_api_url;
    let url = format!("{}/api/v0/cat?arg={}", ipfs_api_url, cid);

    let response = state.http_client
        .post(&url)
        .send()
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to send request to IPFS: {}", e)))?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown IPFS error".to_string());
        return Err(AppError::InternalServerError(format!(
            "IPFS API returned an error while fetching: {}",
            error_text
        )));
    }

    let metadata = response
        .json::<IpfsMetadata>()
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to parse IPFS metadata: {}", e)))?;

    Ok(metadata)
}
