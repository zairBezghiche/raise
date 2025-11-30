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

    /// Constructeur utilitaire pour les chaînes (CLI/Tests)
    pub fn from(path_str: String) -> Result<Self, String> {
        Ok(Self {
            data_root: PathBuf::from(path_str),
        })
    }

    pub fn db_root(&self, space: &str, db: &str) -> PathBuf {
        self.data_root.join(space).join(db)
    }

    pub fn db_collection_path(&self, space: &str, db: &str, collection: &str) -> PathBuf {
        self.db_root(space, db).join(collection)
    }

    pub fn db_schemas_root(&self, space: &str, db: &str) -> PathBuf {
        self.db_root(space, db).join("_system").join("schemas")
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

    // --- Méthodes déléguées vers file_storage ---
    // Ces méthodes permettent au CollectionsManager d'appeler self.storage.write_document(...)

    pub fn write_document(
        &self,
        space: &str,
        db: &str,
        collection: &str,
        id: &str,
        doc: &Value,
    ) -> Result<()> {
        file_storage::write_document(&self.config, space, db, collection, id, doc)
    }

    pub fn read_document(
        &self,
        space: &str,
        db: &str,
        collection: &str,
        id: &str,
    ) -> Result<Option<Value>> {
        // TODO: Ajouter la logique de cache ici (Get from Cache -> If None -> Read FS -> Put Cache)
        file_storage::read_document(&self.config, space, db, collection, id)
    }

    pub fn delete_document(&self, space: &str, db: &str, collection: &str, id: &str) -> Result<()> {
        // TODO: Invalider le cache ici
        file_storage::delete_document(&self.config, space, db, collection, id)
    }
}
