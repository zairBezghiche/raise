use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fs, path::PathBuf};

use crate::json_db::{
    collections::collection, // Nécessite que collection_root soit public
    storage::{self, JsonDbConfig},
};

use super::{btree, hash, paths, IndexDefinition, IndexType};

/// Structure qui représente la configuration d'une collection,
/// contenant les définitions d'index.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CollectionConfig {
    pub schema_rel: String, // ex: "actors/actor.schema.json"
    #[serde(default)]
    pub indexes: Vec<IndexDefinition>,
}

/// Chemin vers le fichier de configuration de la collection (collection_root/_config.json)
fn collection_config_path(cfg: &JsonDbConfig, space: &str, db: &str, collection: &str) -> PathBuf {
    collection::collection_root(cfg, space, db, collection).join("_config.json")
}

/// Lit la configuration d'une collection pour obtenir la liste de ses index.
fn get_collection_index_definitions(
    cfg: &JsonDbConfig,
    space: &str,
    db: &str,
    collection: &str,
) -> Result<Vec<IndexDefinition>> {
    let path = collection_config_path(cfg, space, db, collection);

    // Si le fichier n'existe pas, ou si la collection est vide de définition (c'est rare mais possible),
    // nous retournons les index par défaut (ID).
    if !path.exists() {
        return Ok(vec![
            // Index 'id' par défaut (nécessaire pour la cohérence des documents)
            IndexDefinition {
                name: "id".to_string(),
                field_path: "/id".to_string(),
                index_type: IndexType::Hash,
                unique: true,
            },
        ]);
    }

    let content = fs::read_to_string(&path)
        .with_context(|| format!("Lecture config collection {}", path.display()))?;

    let config: CollectionConfig = serde_json::from_str(&content)
        .with_context(|| format!("Désérialisation config collection {}", path.display()))?;

    // On ajoute l'index 'id' s'il n'est pas déjà défini dans le fichier de config
    let has_id_index = config.indexes.iter().any(|def| def.name == "id");
    let mut definitions = config.indexes;

    if !has_id_index {
        definitions.push(IndexDefinition {
            name: "id".to_string(),
            field_path: "/id".to_string(),
            index_type: IndexType::Hash,
            unique: true,
        });
    }

    Ok(definitions)
}

/// Crée le dossier des index et le fichier de configuration de la collection.
pub fn create_collection_indexes(
    cfg: &JsonDbConfig,
    space: &str,
    db: &str,
    collection: &str,
    schema_rel: &str,
) -> Result<()> {
    // 1. Créer le dossier _indexes
    let indexes_root = paths::indexes_root(cfg, space, db, collection);
    fs::create_dir_all(&indexes_root)
        .with_context(|| format!("Création répertoire index {}", indexes_root.display()))?;

    // 2. Créer le fichier _config.json de la collection (avec index par défaut)
    let config = CollectionConfig {
        schema_rel: schema_rel.to_string(),
        indexes: vec![IndexDefinition {
            name: "id".to_string(),
            field_path: "/id".to_string(),
            index_type: IndexType::Hash,
            unique: true,
        }],
    };

    let config_path = collection_config_path(cfg, space, db, collection);
    storage::file_storage::atomic_write_json(&config_path, &serde_json::to_value(config)?)?;

    Ok(())
}

/// Gère l'ajout, la mise à jour, ou la suppression d'une entrée dans tous les index pertinents.
/// C'est le point d'entrée principal appelé par les opérations CRUD.
pub fn update_indexes(
    cfg: &JsonDbConfig,
    space: &str,
    db: &str,
    collection: &str,
    doc_id: &str,
    old_doc: Option<&Value>, // Document avant modification (pour retirer l'ancienne entrée d'index)
    new_doc: Option<&Value>, // Document après modification (pour ajouter la nouvelle entrée d'index)
) -> Result<()> {
    // Charger les définitions d'index pour cette collection.
    let definitions = get_collection_index_definitions(cfg, space, db, collection)?;

    // Itérer sur chaque définition d'index et appeler le module d'implémentation.
    for def in definitions {
        match def.index_type {
            IndexType::BTree => btree::update_btree_index(
                cfg, space, db, collection, &def, doc_id, old_doc, new_doc,
            )
            .with_context(|| format!("Échec mise à jour BTree index: {}", def.name))?,

            IndexType::Hash => {
                hash::update_hash_index(cfg, space, db, collection, &def, doc_id, old_doc, new_doc)
                    .with_context(|| format!("Échec mise à jour Hash index: {}", def.name))?
            } // Text => // À implémenter plus tard
        }
    }

    Ok(())
}
