pub mod cache;
pub mod file_storage;

// Nous supprimons cache_manager s'il existe encore, car obsolète
// pub mod cache_manager;

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

/// Moteur de stockage principal.
/// Il combine la configuration et le cache en mémoire.
/// Il est `Clone` pour être partagé entre les threads (cache thread-safe).
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

    // --- Méthodes de délégation vers file_storage ---
    // Ces méthodes sont appelées par le CollectionsManager.

    pub fn write_document(
        &self,
        space: &str,
        db: &str,
        collection: &str,
        id: &str,
        doc: &Value,
    ) -> Result<()> {
        // 1. Écriture physique
        file_storage::write_document(&self.config, space, db, collection, id, doc)?;

        // 2. Mise à jour du cache (Clé formatée: space/db/col/id)
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

        // 1. Essai Cache
        if let Some(doc) = self.cache.get(&cache_key) {
            return Ok(Some(doc));
        }

        // 2. Lecture Disque
        let doc_opt = file_storage::read_document(&self.config, space, db, collection, id)?;

        // 3. Mise en Cache
        if let Some(doc) = &doc_opt {
            self.cache.put(cache_key, doc.clone());
        }

        Ok(doc_opt)
    }

    pub fn delete_document(&self, space: &str, db: &str, collection: &str, id: &str) -> Result<()> {
        // 1. Suppression Physique
        file_storage::delete_document(&self.config, space, db, collection, id)?;

        // 2. Invalidation Cache
        let cache_key = format!("{}/{}/{}/{}", space, db, collection, id);
        self.cache.remove(&cache_key);

        Ok(())
    }
}
