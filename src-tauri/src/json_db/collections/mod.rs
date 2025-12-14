//! Façade Collections : API haut niveau pour manipuler les documents

pub mod collection;
pub mod manager;

use anyhow::{Context, Result};
use serde_json::Value;
use std::path::PathBuf;

use crate::json_db::{
    schema::{SchemaRegistry, SchemaValidator},
    storage::JsonDbConfig,
};

// --- Helpers privés ---

fn collection_from_schema_rel(schema_rel: &str) -> String {
    schema_rel
        .split('/')
        .next()
        .unwrap_or("default")
        .to_string()
}

// --- API Publique (Facade) ---

pub fn create_collection(
    cfg: &JsonDbConfig,
    space: &str,
    db: &str,
    collection: &str,
) -> Result<()> {
    collection::create_collection_if_missing(cfg, space, db, collection)
}

pub fn drop_collection(cfg: &JsonDbConfig, space: &str, db: &str, collection: &str) -> Result<()> {
    collection::drop_collection(cfg, space, db, collection)
}

pub fn insert_with_schema(
    cfg: &JsonDbConfig,
    space: &str,
    db: &str,
    schema_rel: &str,
    mut doc: Value,
) -> Result<Value> {
    let reg = SchemaRegistry::from_db(cfg, space, db)?;
    let root_uri = reg.uri(schema_rel);
    let validator = SchemaValidator::compile_with_registry(&root_uri, &reg)?;

    let collection_name = collection_from_schema_rel(schema_rel);

    // --- CORRECTION ICI : Ajout de cfg, space, db ---
    manager::apply_business_rules(
        cfg,              // 1. Config DB (Nouveau)
        space,            // 2. Espace (Nouveau)
        db,               // 3. Nom DB (Nouveau)
        &collection_name, // 4. Nom Collection
        &mut doc,         // 5. Document mutable
        None,             // 6. Ancien doc (None car insertion)
        &reg,             // 7. Registre
        &root_uri,        // 8. URI du schéma
    )
    .context("Rules Engine")?;
    // -----------------------------------------------

    validator.compute_then_validate(&mut doc)?;

    let id = doc
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Document ID manquant"))?;

    collection::update_document(cfg, space, db, &collection_name, id, &doc)?;
    Ok(doc)
}
pub fn insert_raw(
    cfg: &JsonDbConfig,
    space: &str,
    db: &str,
    collection: &str,
    doc: &Value,
) -> Result<()> {
    let id = doc
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Document sans ID"))?;

    collection::create_document(cfg, space, db, collection, id, doc)
}

pub fn update_with_schema(
    cfg: &JsonDbConfig,
    space: &str,
    db: &str,
    schema_rel: &str,
    mut doc: Value,
) -> Result<Value> {
    let reg = SchemaRegistry::from_db(cfg, space, db)?;
    let root_uri = reg.uri(schema_rel);
    let validator = SchemaValidator::compile_with_registry(&root_uri, &reg)?;

    validator.compute_then_validate(&mut doc)?;

    let collection_name = collection_from_schema_rel(schema_rel);
    let id = doc
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Document ID manquant"))?;

    collection::update_document(cfg, space, db, &collection_name, id, &doc)?;
    Ok(doc)
}

pub fn update_raw(
    cfg: &JsonDbConfig,
    space: &str,
    db: &str,
    collection: &str,
    doc: &Value,
) -> Result<()> {
    let id = doc
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Document ID manquant"))?;

    collection::update_document(cfg, space, db, collection, id, doc)
}

pub fn get(cfg: &JsonDbConfig, space: &str, db: &str, collection: &str, id: &str) -> Result<Value> {
    collection::read_document(cfg, space, db, collection, id)
}

pub fn delete(cfg: &JsonDbConfig, space: &str, db: &str, collection: &str, id: &str) -> Result<()> {
    collection::delete_document(cfg, space, db, collection, id)
}

pub fn list_ids(
    cfg: &JsonDbConfig,
    space: &str,
    db: &str,
    collection: &str,
) -> Result<Vec<String>> {
    collection::list_document_ids(cfg, space, db, collection)
}

pub fn list_all(cfg: &JsonDbConfig, space: &str, db: &str, collection: &str) -> Result<Vec<Value>> {
    collection::list_documents(cfg, space, db, collection)
}

pub fn db_root_path(cfg: &JsonDbConfig, space: &str, db: &str) -> PathBuf {
    cfg.db_root(space, db)
}
