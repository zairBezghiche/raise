//! Module de gestion de cache générique en mémoire.

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct CacheEntry<V> {
    value: V,
    #[allow(dead_code)]
    created_at: Instant,
    #[allow(dead_code)]
    last_accessed: Instant,
    expires_at: Option<Instant>,
}

#[derive(Debug, Clone)]
pub struct Cache<K, V> {
    // Arc<RwLock> permet le clonage léger et l'accès concurrent
    store: Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
    capacity: usize,
    default_ttl: Option<Duration>,
}

impl<K, V> Cache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    pub fn new(capacity: usize, default_ttl: Option<Duration>) -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            capacity,
            default_ttl,
        }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        let store = self.store.read().ok()?;
        if let Some(entry) = store.get(key) {
            if let Some(expires_at) = entry.expires_at {
                if Instant::now() > expires_at {
                    return None;
                }
            }
            return Some(entry.value.clone());
        }
        None
    }

    pub fn put(&self, key: K, value: V) {
        let now = Instant::now();
        let expires_at = self.default_ttl.map(|ttl| now + ttl);

        let entry = CacheEntry {
            value,
            created_at: now,
            last_accessed: now,
            expires_at,
        };

        if let Ok(mut guard) = self.store.write() {
            if guard.len() >= self.capacity && !guard.contains_key(&key) {
                // Nettoyage expiré
                guard.retain(|_, v| v.expires_at.map(|exp| exp > now).unwrap_or(true));

                // Eviction simple si toujours plein
                if guard.len() >= self.capacity {
                    if let Some(k) = guard.keys().next().cloned() {
                        guard.remove(&k);
                    }
                }
            }
            guard.insert(key, entry);
        }
    }

    pub fn remove(&self, key: &K) {
        if let Ok(mut guard) = self.store.write() {
            guard.remove(key);
        }
    }

    pub fn clear(&self) {
        if let Ok(mut guard) = self.store.write() {
            guard.clear();
        }
    }

    pub fn len(&self) -> usize {
        self.store.read().map(|g| g.len()).unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
