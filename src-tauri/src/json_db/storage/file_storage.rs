// FICHIER : src-tauri/src/json_db/storage/file_storage.rs

use crate::json_db::storage::JsonDbConfig;
use anyhow::{Context, Result};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DropMode {
    Soft,
    Hard,
}

pub fn open_db(config: &JsonDbConfig, space: &str, db: &str) -> Result<()> {
    let db_path = config.db_root(space, db);
    if !db_path.exists() {
        return Err(anyhow::anyhow!("Database does not exist: {:?}", db_path));
    }
    Ok(())
}

pub fn create_db(config: &JsonDbConfig, space: &str, db: &str) -> Result<()> {
    let db_path = config.db_root(space, db);

    // 1. Création de la structure
    if !db_path.exists() {
        fs::create_dir_all(&db_path)?;
    }

    // 2. AUTO-BOOTSTRAP : Si c'est la base système, on assure son intégrité
    if db == "_system" {
        bootstrap_system_db(config, space, db)?;
    }

    Ok(())
}

/// Fonction interne pour garantir l'intégrité de la base système
fn bootstrap_system_db(config: &JsonDbConfig, space: &str, db: &str) -> Result<()> {
    let schemas_dest = config.db_schemas_root(space, db).join("v1");

    if !schemas_dest.exists() {
        // 1. Création du dossier destination
        fs::create_dir_all(&schemas_dest)?;

        let candidates = vec![
            PathBuf::from("schemas/v1"),       // Run root
            PathBuf::from("../schemas/v1"),    // Run src-tauri
            PathBuf::from("../../schemas/v1"), // Run tools
            // AJOUT DE SÉCURITÉ : Chemin absolu basé sur une hypothèse courante ou variable d'env
            PathBuf::from("/home/zair/genaptitude/schemas/v1"),
        ];

        // AJOUT DE LOG DE DEBUG
        println!(
            "🔍 [JSON-DB] Bootstrap: Recherche des schémas dans : {:?}",
            candidates
        );

        if let Some(source) = candidates.iter().find(|p| p.exists()) {
            println!(
                "✅ [JSON-DB] Schémas trouvés dans {:?} -> Copie en cours...",
                source
            );
            copy_dir_recursive(source, &schemas_dest)?;
        } else {
            eprintln!("❌ [JSON-DB] CRITIQUE : Aucun dossier de schémas source trouvé !");
        }
    }
    Ok(())
}

// --- RESTE DU FICHIER INCHANGÉ (CRUD) ---

pub fn drop_db(config: &JsonDbConfig, space: &str, db: &str, mode: DropMode) -> Result<()> {
    let db_path = config.db_root(space, db);
    if !db_path.exists() {
        return Ok(());
    }

    match mode {
        DropMode::Hard => {
            fs::remove_dir_all(&db_path)
                .with_context(|| format!("Failed to remove DB {:?}", db_path))?;
        }
        DropMode::Soft => {
            let timestamp = chrono::Utc::now().timestamp();
            let parent = db_path.parent().unwrap();
            let new_name = format!("{}.deleted-{}", db, timestamp);
            let new_path = parent.join(new_name);
            fs::rename(&db_path, &new_path).with_context(|| "Failed to soft drop DB")?;
        }
    }
    Ok(())
}

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

/// Helper récursif pour copier les dossiers
fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            // On ne copie que les .json
            if src_path.extension().map_or(false, |e| e == "json") {
                fs::copy(&src_path, &dst_path)?;
            }
        }
    }
    Ok(())
}
