use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

#[derive(Clone)]
pub struct CacheEntry<T> {
    data: T,
    expiry: SystemTime,
}

#[allow(dead_code)]
pub struct Cache<T: Clone> {
    store: Arc<RwLock<HashMap<String, CacheEntry<T>>>>,
    default_ttl: Duration,
}

#[allow(dead_code)]
impl<T: Clone> Cache<T> {
    pub fn new(default_ttl: Duration) -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            default_ttl,
        }
    }

    pub fn get(&self, key: &str) -> Option<T> {
        let store = self.store.read().ok()?;
        
        if let Some(entry) = store.get(key) {
            if SystemTime::now() < entry.expiry {
                return Some(entry.data.clone());
            }
        }
        
        None
    }

    pub fn set(&self, key: String, value: T) -> Result<()> {
        self.set_with_ttl(key, value, self.default_ttl)
    }

    pub fn set_with_ttl(&self, key: String, value: T, ttl: Duration) -> Result<()> {
        let expiry = SystemTime::now() + ttl;
        let entry = CacheEntry { data: value, expiry };
        
        let mut store = self.store.write()
            .map_err(|e| anyhow::anyhow!("Failed to acquire write lock: {}", e))?;
        
        store.insert(key, entry);
        Ok(())
    }

    pub fn invalidate(&self, key: &str) -> Result<()> {
        let mut store = self.store.write()
            .map_err(|e| anyhow::anyhow!("Failed to acquire write lock: {}", e))?;
        
        store.remove(key);
        Ok(())
    }

    pub fn clear(&self) -> Result<()> {
        let mut store = self.store.write()
            .map_err(|e| anyhow::anyhow!("Failed to acquire write lock: {}", e))?;
        
        store.clear();
        Ok(())
    }

    pub fn cleanup_expired(&self) -> Result<usize> {
        let mut store = self.store.write()
            .map_err(|e| anyhow::anyhow!("Failed to acquire write lock: {}", e))?;
        
        let now = SystemTime::now();
        let initial_len = store.len();
        
        store.retain(|_, entry| now < entry.expiry);
        
        Ok(initial_len - store.len())
    }

    pub fn size(&self) -> usize {
        self.store.read().map(|s| s.len()).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_set_and_get() {
        let cache = Cache::new(Duration::from_secs(60));
        
        cache.set("key1".to_string(), "value1".to_string()).unwrap();
        
        let result = cache.get("key1");
        assert_eq!(result, Some("value1".to_string()));
    }

    #[test]
    fn test_cache_expiry() {
        let cache = Cache::new(Duration::from_millis(100));
        
        cache.set("key1".to_string(), "value1".to_string()).unwrap();
        
        // Should be available immediately
        assert!(cache.get("key1").is_some());
        
        // Wait for expiry
        std::thread::sleep(Duration::from_millis(150));
        
        // Should be expired now
        assert!(cache.get("key1").is_none());
    }

    #[test]
    fn test_cache_invalidate() {
        let cache = Cache::new(Duration::from_secs(60));
        
        cache.set("key1".to_string(), "value1".to_string()).unwrap();
        assert!(cache.get("key1").is_some());
        
        cache.invalidate("key1").unwrap();
        assert!(cache.get("key1").is_none());
    }

    #[test]
    fn test_cache_clear() {
        let cache = Cache::new(Duration::from_secs(60));
        
        cache.set("key1".to_string(), "value1".to_string()).unwrap();
        cache.set("key2".to_string(), "value2".to_string()).unwrap();
        
        assert_eq!(cache.size(), 2);
        
        cache.clear().unwrap();
        assert_eq!(cache.size(), 0);
    }

    #[test]
    fn test_cache_cleanup_expired() {
        let cache = Cache::new(Duration::from_millis(100));
        
        cache.set("key1".to_string(), "value1".to_string()).unwrap();
        cache.set_with_ttl("key2".to_string(), "value2".to_string(), Duration::from_secs(60)).unwrap();
        
        // Wait for key1 to expire
        std::thread::sleep(Duration::from_millis(150));
        
        let removed = cache.cleanup_expired().unwrap();
        assert_eq!(removed, 1);
        assert_eq!(cache.size(), 1);
        assert!(cache.get("key2").is_some());
    }
}
