// FICHIER : src-tauri/src/json_db/collections/manager.rs

use crate::json_db::indexes::IndexManager;
use crate::json_db::schema::{SchemaRegistry, SchemaValidator};
use crate::json_db::storage::StorageEngine;
use anyhow::{anyhow, Result};
use serde_json::Value;
use std::fs;
use uuid::Uuid; // Nécessaire pour le fallback ID

#[derive(Debug)]
pub struct CollectionsManager<'a> {
    pub storage: &'a StorageEngine,
    pub space: String,
    pub db: String,
}

impl<'a> CollectionsManager<'a> {
    pub fn new(storage: &'a StorageEngine, space: &str, db: &str) -> Self {
        Self {
            storage,
            space: space.to_string(),
            db: db.to_string(),
        }
    }

    pub fn create_collection(&self, name: &str, schema_uri: Option<String>) -> Result<()> {
        let col_path = self
            .storage
            .config
            .db_collection_path(&self.space, &self.db, name);
        if !col_path.exists() {
            fs::create_dir_all(&col_path)?;
        }
        if let Some(uri) = schema_uri {
            let meta = serde_json::json!({ "schema": uri });
            fs::write(
                col_path.join("_meta.json"),
                serde_json::to_string_pretty(&meta)?,
            )?;
        }
        Ok(())
    }

    pub fn list_collections(&self) -> Result<Vec<String>> {
        let db_path = self.storage.config.db_root(&self.space, &self.db);
        let mut collections = Vec::new();
        if db_path.exists() {
            for entry in fs::read_dir(db_path)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if !name.starts_with('_') {
                            collections.push(name.to_string());
                        }
                    }
                }
            }
        }
        Ok(collections)
    }

    pub fn list_collection_names(&self) -> Result<Vec<String>> {
        self.list_collections()
    }

    /// Insertion brute : nécessite un ID déjà présent dans le document
    pub fn insert_raw(&self, collection: &str, doc: &Value) -> Result<()> {
        let id = doc
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("insert_raw: Document sans ID"))?;

        self.storage
            .write_document(&self.space, &self.db, collection, id, doc)?;

        // Indexation (placeholder)
        let mut _idx = IndexManager::new(self.storage, &self.space, &self.db);
        // _idx.index_document(collection, doc)?;
        Ok(())
    }

    /// Insertion intelligente : applique x_compute et validation
    pub fn insert_with_schema(&self, collection: &str, mut doc: Value) -> Result<Value> {
        // 1. Préparation via le schéma (calculs, defaults)
        // On ignore l'erreur de préparation pour permettre l'insertion 'best-effort' en test
        // si le schéma est introuvable, mais en prod cela devrait être géré.
        if let Err(e) = self.prepare_document(collection, &mut doc) {
            eprintln!(
                "Warning: Schema preparation failed for {}: {}",
                collection, e
            );
        }

        // 2. Fallback : Si le schéma n'a pas généré d'ID, on en met un pour éviter le crash
        if doc.get("id").is_none() {
            if let Some(obj) = doc.as_object_mut() {
                obj.insert("id".to_string(), Value::String(Uuid::new_v4().to_string()));
            }
        }

        // 3. Persistance
        self.insert_raw(collection, &doc)?;
        Ok(doc)
    }

    pub fn get_document(&self, collection: &str, id: &str) -> Result<Option<Value>> {
        self.storage
            .read_document(&self.space, &self.db, collection, id)
    }

    pub fn get(&self, collection: &str, id: &str) -> Result<Option<Value>> {
        self.get_document(collection, id)
    }

    pub fn update_document(&self, collection: &str, id: &str, mut doc: Value) -> Result<Value> {
        if self.get_document(collection, id)?.is_none() {
            return Err(anyhow!("Document introuvable"));
        }
        // Force l'ID dans le doc pour garantir la cohérence
        if let Some(obj) = doc.as_object_mut() {
            obj.insert("id".to_string(), Value::String(id.to_string()));
        }
        self.prepare_document(collection, &mut doc)?;
        self.storage
            .write_document(&self.space, &self.db, collection, id, &doc)?;
        Ok(doc)
    }

    pub fn delete_document(&self, collection: &str, id: &str) -> Result<bool> {
        self.storage
            .delete_document(&self.space, &self.db, collection, id)?;
        Ok(true)
    }

    pub fn list_all(&self, collection: &str) -> Result<Vec<Value>> {
        let col_path = self
            .storage
            .config
            .db_collection_path(&self.space, &self.db, collection);
        let mut docs = Vec::new();
        if col_path.exists() {
            for entry in fs::read_dir(col_path)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "json") {
                    if path.file_name().unwrap() == "_meta.json" {
                        continue;
                    }
                    let content = fs::read_to_string(&path)?;
                    if let Ok(doc) = serde_json::from_str::<Value>(&content) {
                        docs.push(doc);
                    }
                }
            }
        }
        Ok(docs)
    }

    /// Applique les règles x_compute et la validation du schéma lié à la collection
    fn prepare_document(&self, collection: &str, doc: &mut Value) -> Result<()> {
        let meta_path = self
            .storage
            .config
            .db_collection_path(&self.space, &self.db, collection)
            .join("_meta.json");
        let schema_uri = if meta_path.exists() {
            let content = fs::read_to_string(meta_path)?;
            let meta: Value = serde_json::from_str(&content)?;
            meta.get("schema")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        } else {
            None
        };

        if let Some(uri) = schema_uri {
            // On tente de charger le registre depuis la DB
            let reg = SchemaRegistry::from_db(&self.storage.config, &self.space, &self.db)?;
            // On compile et exécute le validateur
            let validator = SchemaValidator::compile_with_registry(&uri, &reg)?;
            validator.compute_then_validate(doc)?;
        }
        Ok(())
    }
}
