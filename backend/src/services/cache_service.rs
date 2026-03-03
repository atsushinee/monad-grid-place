use std::sync::Arc;
use dashmap::DashMap;
use log::info;

pub fn set_cache(cache: &Arc<DashMap<String, String>>, key: &str, value: &str) {
    cache.insert(key.to_string(), value.to_string());
    info!("💾 [Cache] CID mapping stored:");
    info!("   - CID Hash: {}", key);
    info!("   - CID: {}", value);
}

pub fn get_cache(cache: &Arc<DashMap<String, String>>, key: &str) -> Option<String> {
    let result = cache.get(key).map(|v| v.value().clone());
    match &result {
        Some(cid) => {
            info!("💾 [Cache] Cache hit for hash {}: {}", key, cid);
        }
        None => {
            info!("💾 [Cache] Cache miss for hash {}", key);
        }
    }
    result
}
