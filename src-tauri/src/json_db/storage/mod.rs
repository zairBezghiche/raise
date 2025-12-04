// FICHIER : src-tauri/src/json_db/storage/mod.rs

pub mod cache;
pub mod file_storage;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;

// --- CONFIGURATION ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonDbConfig {
    pub data_root: PathBuf,
}

impl JsonDbConfig {
    pub fn new(data_root: PathBuf) -> Self {
        Self { data_root }
    }

    pub fn from(path_str: String) -> Result<Self, String> {
        Ok(Self {
            data_root: PathBuf::from(path_str),
        })
    }

    pub fn db_root(&self, space: &str, db: &str) -> PathBuf {
        self.data_root.join(space).join(db)
    }

    pub fn db_collection_path(&self, space: &str, db: &str, collection: &str) -> PathBuf {
        // CORRECTION PRÉCÉDENTE MAINTENUE : Ajout de "collections"
        self.db_root(space, db).join("collections").join(collection)
    }

    pub fn db_schemas_root(&self, space: &str, _db: &str) -> PathBuf {
        // CORRECTION : Centralisation absolue dans _system
        // On ignore l'argument `_db` pour forcer le chemin vers la base système
        self.db_root(space, "_system").join("schemas")
    }
}

// --- MOTEUR DE STOCKAGE ---

#[derive(Debug, Clone)]
pub struct StorageEngine {
    pub config: JsonDbConfig,
    pub cache: cache::Cache<String, Value>,
}

impl StorageEngine {
    pub fn new(config: JsonDbConfig) -> Self {
        Self {
            config,
            cache: cache::Cache::new(1000, None),
        }
    }

    pub fn write_document(
        &self,
        space: &str,
        db: &str,
        collection: &str,
        id: &str,
        doc: &Value,
    ) -> Result<()> {
        file_storage::write_document(&self.config, space, db, collection, id, doc)?;
        let cache_key = format!("{}/{}/{}/{}", space, db, collection, id);
        self.cache.put(cache_key, doc.clone());
        Ok(())
    }

    pub fn read_document(
        &self,
        space: &str,
        db: &str,
        collection: &str,
        id: &str,
    ) -> Result<Option<Value>> {
        let cache_key = format!("{}/{}/{}/{}", space, db, collection, id);
        if let Some(doc) = self.cache.get(&cache_key) {
            return Ok(Some(doc));
        }
        let doc_opt = file_storage::read_document(&self.config, space, db, collection, id)?;
        if let Some(doc) = &doc_opt {
            self.cache.put(cache_key, doc.clone());
        }
        Ok(doc_opt)
    }

    pub fn delete_document(&self, space: &str, db: &str, collection: &str, id: &str) -> Result<()> {
        file_storage::delete_document(&self.config, space, db, collection, id)?;
        let cache_key = format!("{}/{}/{}/{}", space, db, collection, id);
        self.cache.remove(&cache_key);
        Ok(())
    }
}
