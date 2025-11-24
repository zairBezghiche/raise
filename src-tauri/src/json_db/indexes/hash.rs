use anyhow::{anyhow, Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;

use super::{paths, IndexDefinition, IndexRecord};
use crate::json_db::storage::{file_storage, JsonDbConfig};

// Type interne pour l'index de hachage : Key Value (sérialisée) -> Liste des IDs de documents
// (Nous utilisons Vec<String> pour supporter des index non-uniques)
type HashIndexContent = HashMap<String, Vec<String>>;

/// Tente de charger un index de hachage existant depuis le disque.
fn load_hash_index(
    cfg: &JsonDbConfig,
    space: &str,
    db: &str,
    collection: &str,
    def: &IndexDefinition,
) -> Result<HashIndexContent> {
    let path = paths::index_path(cfg, space, db, collection, &def.name, def.index_type);

    if !path.exists() {
        return Ok(HashMap::new()); // Index vide si le fichier n'existe pas
    }

    // Lire le contenu du fichier (liste de IndexRecord)
    let content =
        fs::read_to_string(&path).with_context(|| format!("Lecture index {}", path.display()))?;

    let records: Vec<IndexRecord> = serde_json::from_str(&content)
        .with_context(|| format!("Désérialisation index {}", path.display()))?;

    // Transformer la liste de records en HashIndexContent
    let mut index = HashMap::new();
    for record in records {
        // La clé doit être sérialisée en String pour être utilisable comme clé de HashMap
        let key_str = record.key.to_string();
        index
            .entry(key_str)
            .or_insert_with(Vec::new)
            .push(record.document_id);
    }

    Ok(index)
}

/// Écrit l'index de hachage en mémoire dans le fichier, en utilisant l'écriture atomique.
fn save_hash_index(
    cfg: &JsonDbConfig,
    space: &str,
    db: &str,
    collection: &str,
    def: &IndexDefinition,
    content: &HashIndexContent,
) -> Result<()> {
    let path = paths::index_path(cfg, space, db, collection, &def.name, def.index_type);

    // Convertir HashIndexContent en Vec<IndexRecord> pour la sérialisation
    let mut records = Vec::new();
    for (key_str, doc_ids) in content.iter() {
        // Tenter de redésérialiser la clé en Value pour stocker un JSON valide
        let key_value: Value =
            serde_json::from_str(key_str).unwrap_or(Value::String(key_str.clone()));

        for doc_id in doc_ids {
            records.push(IndexRecord {
                key: key_value.clone(),
                document_id: doc_id.clone(),
            });
        }
    }

    // Écriture atomique
    file_storage::atomic_write_json(&path, &serde_json::to_value(records)?)
}

/// Met à jour l'index de hachage : retire l'ancienne entrée, ajoute la nouvelle.
pub fn update_hash_index(
    cfg: &JsonDbConfig,
    space: &str,
    db: &str,
    collection: &str,
    def: &IndexDefinition,
    doc_id: &str,
    old_doc: Option<&Value>,
    new_doc: Option<&Value>,
) -> Result<()> {
    let mut index = load_hash_index(cfg, space, db, collection, def)?;

    // 1. Suppression de l'ancienne clé (si c'est un update ou delete)
    if let Some(doc) = old_doc {
        if let Some(old_key) = doc.pointer(&def.field_path) {
            let old_key_str = old_key.to_string();
            if let Some(doc_ids) = index.get_mut(&old_key_str) {
                doc_ids.retain(|id| id != doc_id); // Retire l'ID du document
                if doc_ids.is_empty() {
                    index.remove(&old_key_str);
                }
            }
        }
    }

    // 2. Ajout de la nouvelle clé (si c'est un insert ou update)
    if let Some(doc) = new_doc {
        if let Some(new_key) = doc.pointer(&def.field_path) {
            let new_key_str = new_key.to_string();

            // Vérification de l'unicité
            if def.unique && index.contains_key(&new_key_str) {
                // Sauf si l'ancienne clé est la même que la nouvelle (mise à jour sans changement de clé)
                let is_old_key = old_doc
                    .and_then(|d| d.pointer(&def.field_path))
                    .map(|k| k.to_string() == new_key_str)
                    .unwrap_or(false);

                if !is_old_key {
                    return Err(anyhow!(
                        "Violation d'unicité pour l'index {} avec la clé {}",
                        def.name,
                        new_key_str
                    ));
                }
            }

            // Ajout ou mise à jour
            index
                .entry(new_key_str)
                .or_insert_with(Vec::new)
                .push(doc_id.to_string());
        }
    }

    save_hash_index(cfg, space, db, collection, def, &index)
}
