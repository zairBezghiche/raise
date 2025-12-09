// FICHIER : src-tauri/src/json_db/collections/manager.rs

use crate::json_db::indexes::IndexManager;
use crate::json_db::jsonld::{JsonLdProcessor, VocabularyRegistry};
use crate::json_db::schema::{SchemaRegistry, SchemaValidator};
use crate::json_db::storage::{file_storage, StorageEngine};
use anyhow::{anyhow, Context, Result};
use serde_json::{json, Value};
use std::fs;
use uuid::Uuid;

use super::collection;

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

    pub fn init_db(&self) -> Result<()> {
        file_storage::create_db(&self.storage.config, &self.space, &self.db)?;
        self.ensure_system_index()
    }

    pub fn ensure_system_index(&self) -> Result<()> {
        let sys_path = self
            .storage
            .config
            .db_root(&self.space, &self.db)
            .join("_system.json");

        let mut system_doc = if sys_path.exists() {
            serde_json::from_str(&fs::read_to_string(&sys_path)?)?
        } else {
            json!({
                "space": self.space,
                "database": self.db,
                "version": 1,
                "collections": {}
            })
        };

        self.save_system_index(&mut system_doc)
    }

    fn save_system_index(&self, doc: &mut Value) -> Result<()> {
        let sys_path = self
            .storage
            .config
            .db_root(&self.space, &self.db)
            .join("_system.json");

        let expected_uri = format!(
            "db://{}/{}/schemas/v1/db/index.schema.json",
            self.space, self.db
        );

        if let Some(obj) = doc.as_object_mut() {
            if !obj.contains_key("$schema") {
                obj.insert("$schema".to_string(), Value::String(expected_uri.clone()));
            }
        }

        let reg = SchemaRegistry::from_db(&self.storage.config, &self.space, &self.db)?;

        // --- DEBUG & DIAGNOSTIC ---
        let found_uri = if reg.get_by_uri(&expected_uri).is_some() {
            Some(expected_uri.clone())
        } else {
            reg.list_uris()
                .into_iter()
                .find(|u| u.ends_with("/db/index.schema.json"))
        };

        if let Some(uri) = found_uri {
            match SchemaValidator::compile_with_registry(&uri, &reg) {
                Ok(validator) => {
                    if let Err(e) = validator.compute_then_validate(doc) {
                        return Err(anyhow!("Index système invalide: {}", e));
                    }
                }
                Err(e) => {
                    return Err(anyhow!("Schéma système corrompu ({}): {}", uri, e));
                }
            }
        } else {
            let mut msg =
                "🔥 CRITIQUE: Impossible de trouver le schéma de l'index système !\n".to_string();
            msg.push_str(&format!("   -> URI Attendue : {}\n", expected_uri));
            msg.push_str(&format!(
                "   -> Chemin Physique scanné : {:?}\n",
                self.storage.config.db_schemas_root(&self.space, &self.db)
            ));
            msg.push_str("   -> Clés disponibles dans le registre :\n");

            let mut keys = reg.list_uris();
            keys.sort();
            if keys.is_empty() {
                msg.push_str("      (REGISTRE VIDE - Aucun fichier .json trouvé)\n");
            } else {
                for k in keys {
                    msg.push_str(&format!("      - {}\n", k));
                }
            }
            panic!("{}", msg);
        }

        fs::write(&sys_path, serde_json::to_string_pretty(doc)?)?;
        Ok(())
    }

    pub fn create_collection(&self, name: &str, schema_uri: Option<String>) -> Result<()> {
        if !self.storage.config.db_root(&self.space, &self.db).exists() {
            self.init_db()?;
        }

        let final_schema_uri = if let Some(uri) = schema_uri {
            uri
        } else {
            self.resolve_schema_from_index(name).unwrap_or_default()
        };

        let col_path = self
            .storage
            .config
            .db_collection_path(&self.space, &self.db, name);
        if !col_path.exists() {
            fs::create_dir_all(&col_path)?;
        }

        let meta = json!({ "schema": final_schema_uri, "indexes": [] });
        let meta_path = col_path.join("_meta.json");
        if !meta_path.exists() {
            fs::write(&meta_path, serde_json::to_string_pretty(&meta)?)?;
        }

        self.update_system_index_collection(name, &final_schema_uri)?;
        Ok(())
    }

    pub fn drop_collection(&self, name: &str) -> Result<()> {
        collection::drop_collection(&self.storage.config, &self.space, &self.db, name)?;
        self.remove_collection_from_system_index(name)?;
        Ok(())
    }

    // --- GESTION DES INDEX (AJOUTÉ) ---
    pub fn create_index(&self, collection: &str, field: &str, kind: &str) -> Result<()> {
        let mut idx_mgr = IndexManager::new(self.storage, &self.space, &self.db);
        idx_mgr.create_index(collection, field, kind)
    }

    pub fn drop_index(&self, collection: &str, field: &str) -> Result<()> {
        let mut idx_mgr = IndexManager::new(self.storage, &self.space, &self.db);
        idx_mgr.drop_index(collection, field)
    }
    // ----------------------------------

    fn resolve_schema_from_index(&self, col_name: &str) -> Result<String> {
        let sys_path = self
            .storage
            .config
            .db_root(&self.space, &self.db)
            .join("_system.json");
        if !sys_path.exists() {
            return Err(anyhow!("Index _system.json introuvable"));
        }
        let content = fs::read_to_string(&sys_path)?;
        let sys_json: Value = serde_json::from_str(&content)?;
        let ptr = format!("/collections/{}/schema", col_name);
        let raw_path = sys_json
            .pointer(&ptr)
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Collection '{}' inconnue", col_name))?;
        let relative_path = if let Some(idx) = raw_path.find("/schemas/v1/") {
            &raw_path[idx + "/schemas/v1/".len()..]
        } else {
            raw_path
        };
        Ok(format!(
            "db://{}/{}/schemas/v1/{}",
            self.space, self.db, relative_path
        ))
    }

    fn update_system_index_collection(&self, col_name: &str, schema_uri: &str) -> Result<()> {
        let sys_path = self
            .storage
            .config
            .db_root(&self.space, &self.db)
            .join("_system.json");
        let mut system_doc = if sys_path.exists() {
            serde_json::from_str(&fs::read_to_string(&sys_path)?)?
        } else {
            json!({ "space": self.space, "database": self.db, "version": 1, "collections": {} })
        };

        if system_doc.get("collections").is_none() {
            system_doc["collections"] = json!({});
        }
        if let Some(cols) = system_doc["collections"].as_object_mut() {
            let existing_items = cols
                .get(col_name)
                .and_then(|c| c.get("items"))
                .cloned()
                .unwrap_or(json!([]));
            cols.insert(
                col_name.to_string(),
                json!({ "schema": schema_uri, "items": existing_items }),
            );
        }
        self.save_system_index(&mut system_doc)?;
        Ok(())
    }

    fn remove_collection_from_system_index(&self, col_name: &str) -> Result<()> {
        let sys_path = self
            .storage
            .config
            .db_root(&self.space, &self.db)
            .join("_system.json");
        if !sys_path.exists() {
            return Ok(());
        }
        let content = fs::read_to_string(&sys_path)?;
        let mut system_doc: Value = serde_json::from_str(&content)?;
        let mut changed = false;
        if let Some(cols) = system_doc
            .get_mut("collections")
            .and_then(|c| c.as_object_mut())
        {
            if cols.remove(col_name).is_some() {
                changed = true;
            }
        }
        if changed {
            self.save_system_index(&mut system_doc)?;
        }
        Ok(())
    }

    fn add_item_to_index(&self, col_name: &str, id: &str) -> Result<()> {
        let sys_path = self
            .storage
            .config
            .db_root(&self.space, &self.db)
            .join("_system.json");
        let mut system_doc = if sys_path.exists() {
            serde_json::from_str(&fs::read_to_string(&sys_path)?)?
        } else {
            json!({ "space": self.space, "database": self.db, "version": 1, "collections": {} })
        };

        if system_doc.get("collections").is_none() {
            system_doc["collections"] = json!({});
        }
        let filename = format!("{}.json", id);

        if let Some(cols) = system_doc["collections"].as_object_mut() {
            if !cols.contains_key(col_name) {
                let schema_guess = self
                    .resolve_schema_from_index(col_name)
                    .ok()
                    .unwrap_or_default();
                cols.insert(
                    col_name.to_string(),
                    json!({ "schema": schema_guess, "items": [] }),
                );
            }
            if let Some(col_entry) = cols.get_mut(col_name) {
                if col_entry.get("items").is_none() {
                    col_entry["items"] = json!([]);
                }
                if let Some(items) = col_entry["items"].as_array_mut() {
                    if !items
                        .iter()
                        .any(|i| i.get("file").and_then(|f| f.as_str()) == Some(&filename))
                    {
                        items.push(json!({ "file": filename }));
                    }
                }
            }
        }
        self.save_system_index(&mut system_doc)?;
        Ok(())
    }

    pub fn list_collections(&self) -> Result<Vec<String>> {
        let root = self
            .storage
            .config
            .db_root(&self.space, &self.db)
            .join("collections");
        let mut cols = Vec::new();
        if root.exists() {
            for entry in fs::read_dir(root)? {
                let entry = entry?;
                if entry.path().is_dir() {
                    if let Some(name) = entry.file_name().to_str() {
                        if !name.starts_with('_') {
                            cols.push(name.to_string());
                        }
                    }
                }
            }
        }
        Ok(cols)
    }

    pub fn list_all(&self, collection: &str) -> Result<Vec<Value>> {
        let col_path = self
            .storage
            .config
            .db_collection_path(&self.space, &self.db, collection);
        let mut docs = Vec::new();
        if !col_path.exists() {
            return Ok(docs);
        }
        for entry in fs::read_dir(&col_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "json") {
                if path.file_name().unwrap() == "_meta.json" {
                    continue;
                }
                let content = fs::read_to_string(&path)?;
                if let Ok(doc) = serde_json::from_str::<Value>(&content) {
                    docs.push(doc);
                }
            }
        }
        Ok(docs)
    }

    pub fn insert_raw(&self, collection: &str, doc: &Value) -> Result<()> {
        let id = doc
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("ID manquant"))?;
        let meta_path = self
            .storage
            .config
            .db_collection_path(&self.space, &self.db, collection)
            .join("_meta.json");
        if !meta_path.exists() {
            let schema_hint = doc
                .get("$schema")
                .and_then(|s| s.as_str())
                .map(|s| s.to_string());
            self.create_collection(collection, schema_hint)?;
        }
        self.storage
            .write_document(&self.space, &self.db, collection, id, doc)?;
        self.add_item_to_index(collection, id)?;
        let mut idx_mgr = IndexManager::new(self.storage, &self.space, &self.db);
        if let Err(e) = idx_mgr.index_document(collection, doc) {
            #[cfg(debug_assertions)]
            eprintln!("⚠️ Indexation secondaire échouée: {}", e);
        }
        Ok(())
    }

    pub fn insert_with_schema(&self, collection: &str, mut doc: Value) -> Result<Value> {
        self.prepare_document(collection, &mut doc)?;
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
        let old_doc = self.get_document(collection, id)?;
        if old_doc.is_none() {
            return Err(anyhow!("Document introuvable"));
        }
        if let Some(obj) = doc.as_object_mut() {
            obj.insert("id".to_string(), Value::String(id.to_string()));
        }
        self.prepare_document(collection, &mut doc)?;
        self.storage
            .write_document(&self.space, &self.db, collection, id, &doc)?;
        let mut idx_mgr = IndexManager::new(self.storage, &self.space, &self.db);
        if let Some(old) = old_doc {
            let _ = idx_mgr.remove_document(collection, &old);
        }
        let _ = idx_mgr.index_document(collection, &doc);
        Ok(doc)
    }

    pub fn delete_document(&self, collection: &str, id: &str) -> Result<bool> {
        let old_doc = self.get_document(collection, id)?;
        self.storage
            .delete_document(&self.space, &self.db, collection, id)?;
        if let Some(doc) = old_doc {
            let mut idx_mgr = IndexManager::new(self.storage, &self.space, &self.db);
            let _ = idx_mgr.remove_document(collection, &doc);
        }
        Ok(true)
    }

    fn prepare_document(&self, collection: &str, doc: &mut Value) -> Result<()> {
        let meta_path = self
            .storage
            .config
            .db_collection_path(&self.space, &self.db, collection)
            .join("_meta.json");
        let schema_uri = if meta_path.exists() {
            let content = fs::read_to_string(&meta_path)?;
            let meta: Value = serde_json::from_str(&content)?;
            meta.get("schema")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        } else {
            None
        };

        if let Some(uri) = schema_uri {
            if !uri.is_empty() {
                if let Some(obj) = doc.as_object_mut() {
                    if !obj.contains_key("$schema") {
                        obj.insert("$schema".to_string(), Value::String(uri.clone()));
                    }
                }
                let reg = SchemaRegistry::from_db(&self.storage.config, &self.space, &self.db)?;
                let validator = SchemaValidator::compile_with_registry(&uri, &reg)?;
                validator.compute_then_validate(doc)?;
            }
        }
        self.apply_semantic_logic(doc)
            .context("Validation sémantique")?;
        Ok(())
    }

    fn apply_semantic_logic(&self, doc: &mut Value) -> Result<()> {
        if let Some(obj) = doc.as_object_mut() {
            if !obj.contains_key("@context") {
                obj.insert(
                    "@context".to_string(),
                    json!({
                            "oa": "https://genaptitude.io/ontology/arcadia/oa#",
                            "sa": "https://genaptitude.io/ontology/arcadia/sa#",
                            "la": "https://genaptitude.io/ontology/arcadia/la#",
                            "pa": "https://genaptitude.io/ontology/arcadia/pa#",
                            "data": "https://genaptitude.io/ontology/arcadia/data#"
                    }),
                );
            }
        }
        let processor = JsonLdProcessor::new();
        if let Some(type_uri) = processor.get_type(doc) {
            let registry = VocabularyRegistry::new();
            let expanded_type = processor.context_manager().expand_term(&type_uri);
            if !registry.has_class(&expanded_type) {
                #[cfg(debug_assertions)]
                println!("⚠️ [Semantic Warning] Type inconnu: {}", expanded_type);
            }
        }
        Ok(())
    }
}
