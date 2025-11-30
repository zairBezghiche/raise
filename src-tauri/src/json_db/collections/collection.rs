//! Primitives collections : gestion des dossiers et fichiers JSON d’une collection.
//! Pas de logique x_compute/validate ici — uniquement persistance et I/O.

use anyhow::{Context, Result};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

use crate::json_db::storage::file_storage::atomic_write;
use crate::json_db::storage::JsonDbConfig;

/// Racine des collections : {db_root}/collections/{collection}
pub fn collection_root(cfg: &JsonDbConfig, space: &str, db: &str, collection: &str) -> PathBuf {
    cfg.db_root(space, db).join("collections").join(collection)
}

/// Fichier d’un document : {collection_root}/{id}.json
fn doc_path(cfg: &JsonDbConfig, space: &str, db: &str, collection: &str, id: &str) -> PathBuf {
    collection_root(cfg, space, db, collection).join(format!("{id}.json"))
}

/// S’assure que la collection existe (création récursive).
pub fn create_collection_if_missing(
    cfg: &JsonDbConfig,
    space: &str,
    db: &str,
    collection: &str,
) -> Result<()> {
    let root = collection_root(cfg, space, db, collection);
    fs::create_dir_all(&root).with_context(|| format!("create_dir_all {}", root.display()))?;
    Ok(())
}

/// Lit un document par son ID.
pub fn read_document(
    cfg: &JsonDbConfig,
    space: &str,
    db: &str,
    collection: &str,
    id: &str,
) -> Result<Value> {
    let path = doc_path(cfg, space, db, collection, id);
    let content = fs::read_to_string(&path)
        .with_context(|| format!("Document introuvable : {}/{}", collection, id))?;

    let doc: Value = serde_json::from_str(&content)
        .with_context(|| format!("JSON invalide : {}", path.display()))?;

    Ok(doc)
}

// --- FONCTIONS TRANSACTION MANAGER ---

pub fn create_document(
    cfg: &JsonDbConfig,
    space: &str,
    db: &str,
    collection: &str,
    id: &str,
    document: &Value,
) -> Result<()> {
    create_collection_if_missing(cfg, space, db, collection)?;
    let path = doc_path(cfg, space, db, collection, id);
    let content = serde_json::to_string_pretty(document)?;
    atomic_write(path, content.as_bytes())?;
    Ok(())
}

pub fn update_document(
    cfg: &JsonDbConfig,
    space: &str,
    db: &str,
    collection: &str,
    id: &str,
    document: &Value,
) -> Result<()> {
    create_document(cfg, space, db, collection, id, document)
}

pub fn delete_document(
    cfg: &JsonDbConfig,
    space: &str,
    db: &str,
    collection: &str,
    id: &str,
) -> Result<()> {
    let path = doc_path(cfg, space, db, collection, id);
    if path.exists() {
        fs::remove_file(&path).with_context(|| format!("Suppression {}", path.display()))?;
    }
    Ok(())
}

// --- AJOUT : Suppression de collection ---
pub fn drop_collection(cfg: &JsonDbConfig, space: &str, db: &str, collection: &str) -> Result<()> {
    let root = collection_root(cfg, space, db, collection);
    if root.exists() {
        fs::remove_dir_all(&root)
            .with_context(|| format!("Suppression collection {}", root.display()))?;
    }
    Ok(())
}

// --- FONCTIONS UTILITAIRES ---

pub fn list_document_ids(
    cfg: &JsonDbConfig,
    space: &str,
    db: &str,
    collection: &str,
) -> Result<Vec<String>> {
    let root = collection_root(cfg, space, db, collection);
    let mut out = Vec::new();

    if !root.exists() {
        return Ok(out);
    }

    for e in fs::read_dir(&root)? {
        let e = e?;
        let p = e.path();
        if p.is_file() && p.extension().and_then(|s| s.to_str()) == Some("json") {
            if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                out.push(stem.to_string());
            }
        }
    }
    out.sort();
    Ok(out)
}

pub fn list_documents(
    cfg: &JsonDbConfig,
    space: &str,
    db: &str,
    collection: &str,
) -> Result<Vec<Value>> {
    let ids = list_document_ids(cfg, space, db, collection)?;
    let mut docs = Vec::with_capacity(ids.len());
    for id in ids {
        if let Ok(doc) = read_document(cfg, space, db, collection, &id) {
            docs.push(doc);
        }
    }
    Ok(docs)
}

pub fn list_collection_names_fs(cfg: &JsonDbConfig, space: &str, db: &str) -> Result<Vec<String>> {
    let root = cfg.db_root(space, db).join("collections");
    let mut out = Vec::new();
    if !root.exists() {
        return Ok(out);
    }
    for e in fs::read_dir(root)? {
        let e = e?;
        let ty = e.file_type()?;
        if ty.is_dir() {
            if let Ok(name) = e.file_name().into_string() {
                out.push(name);
            }
        }
    }
    out.sort();
    Ok(out)
}
