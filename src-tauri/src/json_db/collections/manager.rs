//! CollectionsManager : façade orientée instance (storage, space, db)
//! - cache le SchemaRegistry
//! - expose des méthodes CRUD cohérentes (avec et sans schéma)
//! - centralise les chemins cibles de collection (dérivés du schéma)
//! - Gère automatiquement la cohérence des INDEXES à chaque écriture
//! - Utilise le StorageEngine pour le cache système (_system.json)

use anyhow::{anyhow, Context, Result};
use serde_json::Value;
use std::sync::RwLock;

use crate::json_db::{
    indexes::{create_collection_indexes, update_indexes},
    schema::{SchemaRegistry, SchemaValidator},
    storage::{file_storage, StorageEngine}, // Utilisation du StorageEngine
};

// Imports des primitives de collection (FS)
use super::collection::delete_document as delete_document_fs;
use super::collection::drop_collection as drop_collection_fs;
use super::collection::list_collection_names_fs;
use super::collection::list_document_ids as list_document_ids_fs;
use super::collection::list_documents as list_documents_fs;
use super::collection::read_document as read_document_fs;

use super::collection::persist_insert;
use super::collection::persist_update;
use super::collection_from_schema_rel;

/// Manager lié à un couple (space, db)
#[derive(Debug)]
pub struct CollectionsManager<'a> {
    pub storage: &'a StorageEngine, // Remplacement de cfg par storage pour accès au cache
    pub space: String,
    pub db: String,
    registry: RwLock<Option<SchemaRegistry>>,
}

impl<'a> CollectionsManager<'a> {
    /// Construit un manager (le registre est lazy, créé au premier usage)
    pub fn new(storage: &'a StorageEngine, space: &str, db: &str) -> Self {
        Self {
            storage,
            space: space.to_string(),
            db: db.to_string(),
            registry: RwLock::new(None),
        }
    }

    /// (Re)charge explicitement le registre depuis la DB (forces refresh)
    pub fn refresh_registry(&self) -> Result<()> {
        let reg = SchemaRegistry::from_db(&self.storage.config, &self.space, &self.db)?;
        *self
            .registry
            .write()
            .map_err(|e| anyhow!("RwLock poisoned on write: {}", e))? = Some(reg);
        Ok(())
    }

    /// Helper interne pour s'assurer que le registre est chargé.
    fn ensure_registry_loaded(&self) -> Result<()> {
        let is_none = {
            let guard = self
                .registry
                .read()
                .map_err(|e| anyhow!("RwLock poisoned on read: {}", e))?;
            guard.is_none()
        };
        if is_none {
            self.refresh_registry()?;
        }
        Ok(())
    }

    /// Construit une URI logique complète depuis un chemin relatif de schéma.
    pub fn schema_uri(&self, schema_rel: &str) -> Result<String> {
        self.ensure_registry_loaded()?;
        let guard = self
            .registry
            .read()
            .map_err(|e| anyhow!("RwLock poisoned: {}", e))?;
        let reg = guard.as_ref().context("Registry should be initialized")?;
        Ok(reg.uri(schema_rel).to_string())
    }

    /// Compile un validator pour `schema_rel`
    fn compile(&self, schema_rel: &str) -> Result<SchemaValidator> {
        self.ensure_registry_loaded()?;
        let guard = self
            .registry
            .read()
            .map_err(|e| anyhow!("RwLock poisoned: {}", e))?;
        let reg = guard.as_ref().context("Registry should be initialized")?;
        let root_uri = reg.uri(schema_rel);
        SchemaValidator::compile_with_registry(&root_uri, reg)
    }

    // ---------------------------
    // Collections (dossiers & indexes & _system.json)
    // ---------------------------

    /// Vérifie si la collection (et son index) existe, sinon l'initialise.
    fn ensure_collection_ready(&self, collection: &str, schema_rel: &str) -> Result<()> {
        let root = super::collection::collection_root(
            &self.storage.config,
            &self.space,
            &self.db,
            collection,
        );

        // Si le dossier collection n'existe pas, on l'initialise complètement
        if !root.exists() {
            // Utiliser file_storage pour mettre à jour _system.json et créer le dossier
            // Note: create_collection met à jour _system.json, donc on doit invalider/mettre à jour le cache après
            file_storage::create_collection(
                &self.storage.config,
                &self.space,
                &self.db,
                collection,
                schema_rel,
            )?;
            // On invalide le cache d'index car le fichier _system.json a changé sur disque
            self.storage.invalidate_index(&self.space, &self.db);

            // Création de la config d'index par défaut (id)
            create_collection_indexes(
                &self.storage.config,
                &self.space,
                &self.db,
                collection,
                schema_rel,
            )?;
        } else {
            // Si le dossier existe mais pas la config index, on la crée (migration implicite)
            let config_path = root.join("_config.json");
            if !config_path.exists() {
                create_collection_indexes(
                    &self.storage.config,
                    &self.space,
                    &self.db,
                    collection,
                    schema_rel,
                )?;
            }
        }
        Ok(())
    }

    pub fn create_collection(
        &self,
        collection_name: &str,
        schema_opt: Option<String>,
    ) -> Result<()> {
        // Utilise le schéma fourni ou le schéma générique par défaut
        let schema = schema_opt.unwrap_or_else(|| "sandbox/generic.schema.json".to_string());
        self.ensure_collection_ready(collection_name, &schema)
    }

    pub fn drop_collection(&self, collection_name: &str) -> Result<()> {
        // Drop supprime physiquement et met à jour _system.json (s'il gérait la liste, mais ici drop_collection_fs ne touche que le dossier)
        // Si drop_collection_fs ne touche pas à _system.json, on est bon.
        // Mais pour être sûr, on laisse file_storage gérer.
        drop_collection_fs(&self.storage.config, &self.space, &self.db, collection_name)
    }

    // ---------------------------
    // Inserts / Updates (avec gestion des Index)
    // ---------------------------

    pub fn insert_with_schema(&self, schema_rel: &str, mut doc: Value) -> Result<Value> {
        let validator = self.compile(schema_rel)?;
        validator.compute_then_validate(&mut doc)?;

        let collection = collection_from_schema_rel(schema_rel);

        // 1. S'assurer que la structure existe
        self.ensure_collection_ready(&collection, schema_rel)?;

        // 2. Persistance fichier (atomique)
        persist_insert(
            &self.storage.config,
            &self.space,
            &self.db,
            &collection,
            &doc,
        )?;

        // 3. Mise à jour de l'index principal (_system.json) pour le nouveau fichier
        // OPTIMISATION : On utilise le cache pour lire, et on met à jour le cache après écriture
        {
            // Utilisation du cache via get_index
            let mut idx = self.storage.get_index(&self.space, &self.db)?;

            if let Some(coll_def) = idx.collections.get_mut(&collection) {
                if let Some(id) = doc.get("id").and_then(|v| v.as_str()) {
                    let fname = format!("{}.json", id);
                    if !coll_def.items.iter().any(|i| i.file == fname) {
                        coll_def.items.push(file_storage::DbItemRef { file: fname });

                        // Écriture disque
                        file_storage::write_index(
                            &self.storage.config,
                            &self.space,
                            &self.db,
                            &idx,
                        )?;

                        // Mise à jour du cache
                        self.storage.update_cached_index(&self.space, &self.db, idx);
                    }
                }
            }
        }

        // 4. Mise à jour des index de recherche
        if let Some(id) = doc.get("id").and_then(|v| v.as_str()) {
            update_indexes(
                &self.storage.config,
                &self.space,
                &self.db,
                &collection,
                id,
                None,
                Some(&doc),
            )?;
        }

        Ok(doc)
    }

    pub fn insert_raw(&self, collection: &str, doc: &Value) -> Result<()> {
        self.ensure_collection_ready(collection, "sandbox/generic.schema.json")?;

        persist_insert(&self.storage.config, &self.space, &self.db, collection, doc)?;

        // Mise à jour index _system.json (Cache-Aware)
        {
            let mut idx = self.storage.get_index(&self.space, &self.db)?;
            if let Some(coll_def) = idx.collections.get_mut(collection) {
                if let Some(id) = doc.get("id").and_then(|v| v.as_str()) {
                    let fname = format!("{}.json", id);
                    if !coll_def.items.iter().any(|i| i.file == fname) {
                        coll_def.items.push(file_storage::DbItemRef { file: fname });

                        file_storage::write_index(
                            &self.storage.config,
                            &self.space,
                            &self.db,
                            &idx,
                        )?;
                        self.storage.update_cached_index(&self.space, &self.db, idx);
                    }
                }
            }
        }

        // Mise à jour index recherche
        if let Some(id) = doc.get("id").and_then(|v| v.as_str()) {
            update_indexes(
                &self.storage.config,
                &self.space,
                &self.db,
                collection,
                id,
                None,
                Some(doc),
            )?;
        }
        Ok(())
    }

    pub fn update_with_schema(&self, schema_rel: &str, mut doc: Value) -> Result<Value> {
        let validator = self.compile(schema_rel)?;
        validator.compute_then_validate(&mut doc)?;

        let collection = collection_from_schema_rel(schema_rel);
        let id = doc
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Document missing id"))?;

        let old_doc =
            read_document_fs(&self.storage.config, &self.space, &self.db, &collection, id).ok();

        persist_update(
            &self.storage.config,
            &self.space,
            &self.db,
            &collection,
            &doc,
        )?;

        update_indexes(
            &self.storage.config,
            &self.space,
            &self.db,
            &collection,
            id,
            old_doc.as_ref(),
            Some(&doc),
        )?;

        Ok(doc)
    }

    pub fn update_raw(&self, collection: &str, doc: &Value) -> Result<()> {
        let id = doc
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Document missing id"))?;

        let old_doc =
            read_document_fs(&self.storage.config, &self.space, &self.db, collection, id).ok();

        persist_update(&self.storage.config, &self.space, &self.db, collection, doc)?;

        update_indexes(
            &self.storage.config,
            &self.space,
            &self.db,
            collection,
            id,
            old_doc.as_ref(),
            Some(doc),
        )?;

        Ok(())
    }

    // ---------------------------
    // Lecture / Suppression / Listes
    // ---------------------------

    pub fn get(&self, collection: &str, id: &str) -> Result<Value> {
        read_document_fs(&self.storage.config, &self.space, &self.db, collection, id)
    }

    pub fn delete(&self, collection: &str, id: &str) -> Result<()> {
        let old_doc =
            read_document_fs(&self.storage.config, &self.space, &self.db, collection, id).ok();

        delete_document_fs(&self.storage.config, &self.space, &self.db, collection, id)?;

        if let Some(doc) = old_doc {
            update_indexes(
                &self.storage.config,
                &self.space,
                &self.db,
                collection,
                id,
                Some(&doc),
                None,
            )?;

            // Mise à jour _system.json (retrait) avec Cache
            let mut idx = self.storage.get_index(&self.space, &self.db)?;
            if let Some(coll_def) = idx.collections.get_mut(collection) {
                let fname = format!("{}.json", id);
                // On filtre pour retirer l'élément
                coll_def.items.retain(|i| i.file != fname);

                file_storage::write_index(&self.storage.config, &self.space, &self.db, &idx)?;
                self.storage.update_cached_index(&self.space, &self.db, idx);
            }
        }

        Ok(())
    }

    pub fn list_ids(&self, collection: &str) -> Result<Vec<String>> {
        list_document_ids_fs(&self.storage.config, &self.space, &self.db, collection)
    }

    pub fn list_all(&self, collection: &str) -> Result<Vec<Value>> {
        list_documents_fs(&self.storage.config, &self.space, &self.db, collection)
    }

    // ---------------------------
    // Helpers
    // ---------------------------

    pub fn list_ids_for_schema(&self, schema_rel: &str) -> Result<Vec<String>> {
        let collection = collection_from_schema_rel(schema_rel);
        self.list_ids(&collection)
    }

    pub fn upsert_with_schema(&self, schema_rel: &str, doc: Value) -> Result<Value> {
        match self.insert_with_schema(schema_rel, doc.clone()) {
            Ok(stored) => Ok(stored),
            Err(_e) => self.update_with_schema(schema_rel, doc),
        }
    }

    pub fn context(&self) -> (&str, &str) {
        (&self.space, &self.db)
    }

    pub fn list_collection_names(&self) -> Result<Vec<String>> {
        list_collection_names_fs(&self.storage.config, &self.space, &self.db)
    }

    pub fn get_indexes(
        &self,
        collection: &str,
    ) -> Result<Vec<crate::json_db::indexes::IndexDefinition>> {
        use crate::json_db::indexes::manager::get_collection_index_definitions;
        get_collection_index_definitions(&self.storage.config, &self.space, &self.db, collection)
    }
}
