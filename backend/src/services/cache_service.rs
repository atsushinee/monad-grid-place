use std::sync::Arc;
use dashmap::DashMap;

pub fn set_cache(cache: &Arc<DashMap<String, String>>, key: &str, value: &str) {
    cache.insert(key.to_string(), value.to_string());
    println!("Cached CID for hash {}: {}", key, value);
}

pub fn get_cache(cache: &Arc<DashMap<String, String>>, key: &str) -> Option<String> {
    cache.get(key).map(|v| v.value().clone())
}
