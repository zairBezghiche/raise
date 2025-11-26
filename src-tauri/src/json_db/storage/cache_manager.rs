use super::cache::Cache;
use serde_json::Value;
use std::time::Duration;

pub struct GlobalCache {
    // Cache pour les schémas compilés ou bruts
    pub schemas: Cache<String, Value>,
    // Cache pour les documents (optionnel, attention à la mémoire)
    pub documents: Cache<String, Value>,
}

impl GlobalCache {
    pub fn new() -> Self {
        Self {
            // Garder 100 schémas max, durée de vie 10 minutes
            schemas: Cache::new(100, Some(Duration::from_secs(600))),
            // Garder 1000 documents max, durée de vie 30 secondes (cohérence faible)
            documents: Cache::new(1000, Some(Duration::from_secs(30))),
        }
    }
}
