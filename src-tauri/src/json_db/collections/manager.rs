//! CollectionsManager : façade orientée instance (cfg, space, db)
//! - cache le SchemaRegistry
//! - expose des méthodes CRUD cohérentes (avec et sans schéma)
//! - centralise les chemins cibles de collection (dérivés du schéma)

use anyhow::{anyhow, Context, Result};
use serde_json::Value;
use std::sync::RwLock;

use crate::json_db::{
    schema::{SchemaRegistry, SchemaValidator},
    storage::JsonDbConfig,
};

// Imports des primitives de collection (FS)
use super::collection::create_collection_if_missing;
use super::collection::delete_document as delete_document_fs;
use super::collection::drop_collection as drop_collection_fs;
use super::collection::list_collection_names_fs;
use super::collection::list_document_ids as list_document_ids_fs;
use super::collection::list_documents as list_documents_fs;
use super::collection::read_document as read_document_fs;

// Imports restaurés et utilisés
use super::collection::persist_insert;
use super::collection::persist_update;
use super::collection_from_schema_rel;

/// Manager lié à un couple (space, db)
#[derive(Debug)]
pub struct CollectionsManager<'a> {
    cfg: &'a JsonDbConfig,
    space: String,
    db: String,
    // RwLock pour la mutabilité interne thread-safe
    registry: RwLock<Option<SchemaRegistry>>,
}

impl<'a> CollectionsManager<'a> {
    /// Construit un manager (le registre est lazy, créé au premier usage)
    pub fn new(cfg: &'a JsonDbConfig, space: &str, db: &str) -> Self {
        Self {
            cfg,
            space: space.to_string(),
            db: db.to_string(),
            registry: RwLock::new(None),
        }
    }

    /// (Re)charge explicitement le registre depuis la DB (forces refresh)
    pub fn refresh_registry(&self) -> Result<()> {
        let reg = SchemaRegistry::from_db(self.cfg, &self.space, &self.db)?;

        // Utilisation de write() pour muter
        *self
            .registry
            .write()
            .map_err(|e| anyhow!("RwLock poisoned on write: {}", e))? = Some(reg);

        Ok(())
    }

    /// Helper interne pour s'assurer que le registre est chargé.
    /// Ne retourne rien, s'assure juste que le RwLock contient Some.
    fn ensure_registry_loaded(&self) -> Result<()> {
        // Vérification rapide en lecture
        let is_none = {
            let guard = self
                .registry
                .read()
                .map_err(|e| anyhow!("RwLock poisoned on read: {}", e))?;
            guard.is_none()
        };

        // Si vide, on charge (avec verrou d'écriture)
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
    // Collections (dossiers)
    // ---------------------------

    pub fn create_collection(&self, collection_name: &str) -> Result<()> {
        create_collection_if_missing(self.cfg, &self.space, &self.db, collection_name)
    }

    pub fn drop_collection(&self, collection_name: &str) -> Result<()> {
        drop_collection_fs(self.cfg, &self.space, &self.db, collection_name)
    }

    // ---------------------------
    // Inserts / Updates
    // ---------------------------

    /// Insert avec schéma :
    /// - x_compute + validate (préremplit $schema, id, createdAt, updatedAt si manquants)
    /// - persist en FS (échec si id existe)
    pub fn insert_with_schema(&self, schema_rel: &str, mut doc: Value) -> Result<Value> {
        let validator = self.compile(schema_rel)?;
        validator.compute_then_validate(&mut doc)?;

        // Utilisation de collection_from_schema_rel importé
        let collection = collection_from_schema_rel(schema_rel);
        self.create_collection(&collection)?;

        // Utilisation de persist_insert importé
        persist_insert(self.cfg, &self.space, &self.db, &collection, &doc)?;
        Ok(doc)
    }

    /// Insert direct (sans schéma). À utiliser si déjà conforme.
    pub fn insert_raw(&self, collection: &str, doc: &Value) -> Result<()> {
        self.create_collection(collection)?;
        persist_insert(self.cfg, &self.space, &self.db, collection, doc)
    }

    /// Update avec schéma : recompute + validate + persist (remplace par id ; erreur si absent)
    pub fn update_with_schema(&self, schema_rel: &str, mut doc: Value) -> Result<Value> {
        let validator = self.compile(schema_rel)?;
        validator.compute_then_validate(&mut doc)?;

        let collection = collection_from_schema_rel(schema_rel);
        // Utilisation de persist_update importé
        persist_update(self.cfg, &self.space, &self.db, &collection, &doc)?;
        Ok(doc)
    }

    /// Update direct (sans schéma). Remplacement complet (erreur si absent).
    pub fn update_raw(&self, collection: &str, doc: &Value) -> Result<()> {
        persist_update(self.cfg, &self.space, &self.db, collection, doc)
    }

    // ---------------------------
    // Lecture / Suppression / Listes
    // ---------------------------

    pub fn get(&self, collection: &str, id: &str) -> Result<Value> {
        read_document_fs(self.cfg, &self.space, &self.db, collection, id)
    }

    pub fn delete(&self, collection: &str, id: &str) -> Result<()> {
        delete_document_fs(self.cfg, &self.space, &self.db, collection, id)
    }

    pub fn list_ids(&self, collection: &str) -> Result<Vec<String>> {
        list_document_ids_fs(self.cfg, &self.space, &self.db, collection)
    }

    pub fn list_all(&self, collection: &str) -> Result<Vec<Value>> {
        list_documents_fs(self.cfg, &self.space, &self.db, collection)
    }

    // ---------------------------
    // Helpers pratiques
    // ---------------------------

    /// Déduit le nom de collection à partir d’un schéma et liste les ids.
    pub fn list_ids_for_schema(&self, schema_rel: &str) -> Result<Vec<String>> {
        let collection = collection_from_schema_rel(schema_rel);
        self.list_ids(&collection)
    }

    /// Upsert basé schéma : insert si absent, sinon update (selon présence de `id`)
    pub fn upsert_with_schema(&self, schema_rel: &str, doc: Value) -> Result<Value> {
        // On clone le doc pour l'essai d'insertion, car insert_with_schema le consomme
        match self.insert_with_schema(schema_rel, doc.clone()) {
            Ok(stored) => Ok(stored),
            Err(_e) => self.update_with_schema(schema_rel, doc),
        }
    }

    /// Renvoie le (space, db) courants
    pub fn context(&self) -> (&str, &str) {
        (&self.space, &self.db)
    }

    pub fn list_collection_names(&self) -> Result<Vec<String>> {
        list_collection_names_fs(self.cfg, &self.space, &self.db)
    }
}
