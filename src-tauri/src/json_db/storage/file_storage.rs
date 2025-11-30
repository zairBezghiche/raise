// FICHIER : src-tauri/src/json_db/storage/file_storage.rs

use crate::json_db::storage::JsonDbConfig;
use anyhow::{Context, Result};
use serde_json::Value;
use std::fs;
use std::path::Path;

// --- AJOUT : Enum DropMode ---
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DropMode {
    Soft, // Renomme en .deleted-<timestamp>
    Hard, // Supprime définitivement
}

/// Vérifie si la DB existe
pub fn open_db(config: &JsonDbConfig, space: &str, db: &str) -> Result<()> {
    let db_path = config.db_root(space, db);
    if !db_path.exists() {
        return Err(anyhow::anyhow!("Database does not exist: {:?}", db_path));
    }
    Ok(())
}

/// Crée l'arborescence de la DB
pub fn create_db(config: &JsonDbConfig, space: &str, db: &str) -> Result<()> {
    let db_path = config.db_root(space, db);
    if !db_path.exists() {
        fs::create_dir_all(&db_path)?;
        fs::create_dir_all(db_path.join("_system").join("schemas"))?;
    }
    Ok(())
}

// --- AJOUT : Fonction drop_db ---
pub fn drop_db(config: &JsonDbConfig, space: &str, db: &str, mode: DropMode) -> Result<()> {
    let db_path = config.db_root(space, db);
    if !db_path.exists() {
        // Idempotence : si n'existe pas, on considère que c'est un succès
        return Ok(());
    }

    match mode {
        DropMode::Hard => {
            fs::remove_dir_all(&db_path)
                .with_context(|| format!("Failed to remove DB {:?}", db_path))?;
        }
        DropMode::Soft => {
            let timestamp = chrono::Utc::now().timestamp();
            let parent = db_path.parent().unwrap(); // Space directory
            let new_name = format!("{}.deleted-{}", db, timestamp);
            let new_path = parent.join(new_name);
            fs::rename(&db_path, &new_path).with_context(|| "Failed to soft drop DB")?;
        }
    }
    Ok(())
}

/// Écrit un document sur le disque (Atomique)
pub fn write_document(
    config: &JsonDbConfig,
    space: &str,
    db: &str,
    collection: &str,
    id: &str,
    doc: &Value,
) -> Result<()> {
    let col_path = config.db_collection_path(space, db, collection);
    if !col_path.exists() {
        fs::create_dir_all(&col_path)?;
    }

    let file_path = col_path.join(format!("{}.json", id));
    let content = serde_json::to_string_pretty(doc)?;

    atomic_write(file_path, content)?;
    Ok(())
}

/// Lit un document depuis le disque
pub fn read_document(
    config: &JsonDbConfig,
    space: &str,
    db: &str,
    collection: &str,
    id: &str,
) -> Result<Option<Value>> {
    let file_path = config
        .db_collection_path(space, db, collection)
        .join(format!("{}.json", id));

    if !file_path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(file_path)?;
    let doc = serde_json::from_str(&content)?;
    Ok(Some(doc))
}

/// Supprime un document
pub fn delete_document(
    config: &JsonDbConfig,
    space: &str,
    db: &str,
    collection: &str,
    id: &str,
) -> Result<()> {
    let file_path = config
        .db_collection_path(space, db, collection)
        .join(format!("{}.json", id));

    if file_path.exists() {
        fs::remove_file(file_path)?;
    }
    Ok(())
}

// --- Helpers Atomiques ---

pub fn atomic_write<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, content: C) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    let temp_path = path.with_extension("tmp");
    fs::write(&temp_path, content)
        .with_context(|| format!("Failed to write temp file {:?}", temp_path))?;
    fs::rename(&temp_path, path)
        .with_context(|| format!("Failed to rename {:?} to {:?}", temp_path, path))?;
    Ok(())
}

pub fn atomic_write_binary<P: AsRef<Path>>(path: P, content: &[u8]) -> Result<()> {
    atomic_write(path, content)
}
