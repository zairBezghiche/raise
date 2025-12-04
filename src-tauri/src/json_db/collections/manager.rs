// FICHIER : src-tauri/src/json_db/collections/manager.rs

use crate::json_db::indexes::IndexManager;
use crate::json_db::schema::{SchemaRegistry, SchemaValidator};
// AJOUT : Import de file_storage pour accéder à create_db
use crate::json_db::storage::{file_storage, StorageEngine};
use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::fs;
use uuid::Uuid;

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
        // --- CORRECTION CRITIQUE ---
        // On force l'initialisation de la DB parente.
        // C'est CE qui va déclencher le bootstrap des schémas si c'est la base _system.
        file_storage::create_db(&self.storage.config, &self.space, &self.db)?;
        // ---------------------------

        let col_path = self
            .storage
            .config
            .db_collection_path(&self.space, &self.db, name);

        if !col_path.exists() {
            fs::create_dir_all(&col_path)?;
        }

        let uri_str = schema_uri.clone().unwrap_or_default();
        let meta = json!({ "schema": uri_str });
        fs::write(
            col_path.join("_meta.json"),
            serde_json::to_string_pretty(&meta)?,
        )?;

        self.update_system_index_collection(name, &uri_str)?;

        Ok(())
    }

    // ... LE RESTE DU FICHIER RESTE STRICTEMENT IDENTIQUE (update_system_index, insert_with_schema, etc.) ...
    // (Copiez le reste du fichier précédent ici)

    fn update_system_index_collection(&self, col_name: &str, schema_uri: &str) -> Result<()> {
        let sys_path = self
            .storage
            .config
            .db_root(&self.space, &self.db)
            .join("_system.json");
        let mut system_doc = self.load_system_index(&sys_path);

        if let Some(cols) = system_doc["collections"].as_object_mut() {
            let existing_items = cols
                .get(col_name)
                .and_then(|c| c.get("items"))
                .cloned()
                .unwrap_or(json!([]));

            cols.insert(
                col_name.to_string(),
                json!({
                    "schema": schema_uri,
                    "items": existing_items
                }),
            );
        }
        fs::write(sys_path, serde_json::to_string_pretty(&system_doc)?)?;
        Ok(())
    }

    fn add_item_to_index(&self, col_name: &str, id: &str) -> Result<()> {
        let sys_path = self
            .storage
            .config
            .db_root(&self.space, &self.db)
            .join("_system.json");
        let mut system_doc = self.load_system_index(&sys_path);
        let filename = format!("{}.json", id);

        if let Some(cols) = system_doc["collections"].as_object_mut() {
            if let Some(col_entry) = cols.get_mut(col_name) {
                if col_entry.get("items").is_none() {
                    col_entry["items"] = json!([]);
                }
                if let Some(items) = col_entry["items"].as_array_mut() {
                    let exists = items
                        .iter()
                        .any(|item| item.get("file").and_then(|f| f.as_str()) == Some(&filename));
                    if !exists {
                        items.push(json!({ "file": filename }));
                    }
                }
            }
        }
        fs::write(sys_path, serde_json::to_string_pretty(&system_doc)?)?;
        Ok(())
    }

    fn load_system_index(&self, path: &std::path::Path) -> Value {
        if path.exists() {
            let content = fs::read_to_string(path).unwrap_or_default();
            serde_json::from_str(&content).unwrap_or(json!({ "collections": {} }))
        } else {
            json!({ "collections": {} })
        }
    }

    pub fn list_collections(&self) -> Result<Vec<String>> {
        let collections_root = self
            .storage
            .config
            .db_root(&self.space, &self.db)
            .join("collections");
        let mut collections = Vec::new();
        if collections_root.exists() {
            for entry in fs::read_dir(collections_root)? {
                let entry = entry?;
                if entry.path().is_dir() {
                    if let Some(name) = entry.file_name().to_str() {
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

    pub fn insert_raw(&self, collection: &str, doc: &Value) -> Result<()> {
        let id = doc
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("insert_raw: Document sans ID"))?;

        self.storage
            .write_document(&self.space, &self.db, collection, id, doc)?;
        self.add_item_to_index(collection, id)?;
        let mut _idx = IndexManager::new(self.storage, &self.space, &self.db);
        Ok(())
    }

    pub fn insert_with_schema(&self, collection: &str, mut doc: Value) -> Result<Value> {
        if let Err(e) = self.prepare_document(collection, &mut doc) {
            #[cfg(debug_assertions)]
            eprintln!("Schema warning for {}: {}", collection, e);
        }

        if doc.get("id").is_none() {
            if let Some(obj) = doc.as_object_mut() {
                obj.insert("id".to_string(), Value::String(Uuid::new_v4().to_string()));
            }
        }

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
        if let Some(obj) = doc.as_object_mut() {
            obj.insert("id".to_string(), Value::String(id.to_string()));
        }
        let _ = self.prepare_document(collection, &mut doc);
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
            let reg = SchemaRegistry::from_db(&self.storage.config, &self.space, &self.db)?;
            let validator = SchemaValidator::compile_with_registry(&uri, &reg)?;
            validator.compute_then_validate(doc)?;
        }
        Ok(())
    }
}
