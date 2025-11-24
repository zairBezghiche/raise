use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;

use super::{paths, IndexDefinition, IndexRecord};
use crate::json_db::storage::{file_storage, JsonDbConfig};

// Type interne pour l'index B-Tree : Key Value (sérialisée) -> Liste des IDs de documents
// BTreeMap garantit l'ordre des clés (String)
type BTreeIndexContent = BTreeMap<String, Vec<String>>;

// Les fonctions load_btree_index et save_btree_index sont très similaires
// à celles de hash.rs, mais utilisent BTreeMap au lieu de HashMap.

/// Tente de charger un index B-Tree existant depuis le disque.
fn load_btree_index(
    cfg: &JsonDbConfig,
    space: &str,
    db: &str,
    collection: &str,
    def: &IndexDefinition,
) -> Result<BTreeIndexContent> {
    let path = paths::index_path(cfg, space, db, collection, &def.name, def.index_type);

    if !path.exists() {
        return Ok(BTreeMap::new());
    }

    let content =
        fs::read_to_string(&path).with_context(|| format!("Lecture index {}", path.display()))?;

    let records: Vec<IndexRecord> = serde_json::from_str(&content)
        .with_context(|| format!("Désérialisation index {}", path.display()))?;

    let mut index = BTreeMap::new();
    for record in records {
        let key_str = record.key.to_string();
        index
            .entry(key_str)
            .or_insert_with(Vec::new)
            .push(record.document_id);
    }

    Ok(index)
}

/// Écrit l'index B-Tree en mémoire dans le fichier, en utilisant l'écriture atomique.
fn save_btree_index(
    cfg: &JsonDbConfig,
    space: &str,
    db: &str,
    collection: &str,
    def: &IndexDefinition,
    content: &BTreeIndexContent,
) -> Result<()> {
    let path = paths::index_path(cfg, space, db, collection, &def.name, def.index_type);

    let mut records = Vec::new();
    for (key_str, doc_ids) in content.iter() {
        let key_value: Value =
            serde_json::from_str(key_str).unwrap_or(Value::String(key_str.clone()));

        for doc_id in doc_ids {
            records.push(IndexRecord {
                key: key_value.clone(),
                document_id: doc_id.clone(),
            });
        }
    }

    file_storage::atomic_write_json(&path, &serde_json::to_value(records)?)
}

/// Met à jour l'index B-Tree : retire l'ancienne entrée, ajoute la nouvelle.
pub fn update_btree_index(
    cfg: &JsonDbConfig,
    space: &str,
    db: &str,
    collection: &str,
    def: &IndexDefinition,
    doc_id: &str,
    old_doc: Option<&Value>,
    new_doc: Option<&Value>,
) -> Result<()> {
    let mut index = load_btree_index(cfg, space, db, collection, def)?;

    // 1. Suppression de l'ancienne clé
    if let Some(doc) = old_doc {
        if let Some(old_key) = doc.pointer(&def.field_path) {
            let old_key_str = old_key.to_string();
            if let Some(doc_ids) = index.get_mut(&old_key_str) {
                doc_ids.retain(|id| id != doc_id);
                if doc_ids.is_empty() {
                    index.remove(&old_key_str);
                }
            }
        }
    }

    // 2. Ajout de la nouvelle clé
    if let Some(doc) = new_doc {
        if let Some(new_key) = doc.pointer(&def.field_path) {
            let new_key_str = new_key.to_string();

            // NOTE: La vérification d'unicité est moins courante pour le B-Tree,
            // mais si def.unique est vrai, le code de vérification de hash.rs s'applique.

            index
                .entry(new_key_str)
                .or_insert_with(Vec::new)
                .push(doc_id.to_string());
        }
    }

    save_btree_index(cfg, space, db, collection, def, &index)
}
