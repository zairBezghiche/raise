//! JSON-DB Tauri commands
//!
//! Ces commandes exposent les opérations principales (CRUD) via Tauri.
//! Elles utilisent désormais le StorageEngine injecté pour bénéficier du cache.

use serde_json::Value;
use tauri::{command, State};

use crate::json_db::query::{QueryEngine, QueryInput, QueryResult};
use crate::json_db::transactions::TransactionManager;

use crate::json_db::{
    collections::manager::CollectionsManager,
    storage::{file_storage, StorageEngine},
};

// -----------------------------
// Helpers
// -----------------------------

/// Helper pour obtenir un manager lié (space, db) à partir du State global.
/// Tente d'ouvrir la DB, et si elle n'existe pas, la crée.
fn mgr<'a>(
    storage: &'a State<StorageEngine>,
    space: &str,
    db: &str,
) -> Result<CollectionsManager<'a>, String> {
    // On accède à la config via le moteur de stockage
    let config = &storage.config;

    // Logique "Open OR Create"
    if file_storage::open_db(config, space, db).is_err() {
        file_storage::create_db(config, space, db).map_err(|e| e.to_string())?;
    }

    // On passe le StorageEngine (déréférencé du State) au manager
    // CollectionsManager::new attend &StorageEngine
    Ok(CollectionsManager::new(storage.inner(), space, db))
}

// -----------------------------
// Commandes Collections
// -----------------------------

/// Crée une collection si manquante
#[command]
pub fn jsondb_create_collection(
    storage: State<StorageEngine>,
    space: String,
    db: String,
    collection: String,
    schema: Option<String>,
) -> Result<(), String> {
    let m = mgr(&storage, &space, &db)?;
    m.create_collection(&collection, schema)
        .map_err(|e| e.to_string())
}

/// Supprime une collection (dossier)
#[command]
pub fn jsondb_drop_collection(
    storage: State<StorageEngine>,
    space: String,
    db: String,
    collection: String,
) -> Result<(), String> {
    let m = mgr(&storage, &space, &db)?;
    m.drop_collection(&collection).map_err(|e| e.to_string())
}

/// Liste des collections
#[command]
pub fn jsondb_list_collections(
    storage: State<StorageEngine>,
    space: String,
    db: String,
) -> Result<Vec<String>, String> {
    let m = mgr(&storage, &space, &db)?;
    m.list_collection_names().map_err(|e| e.to_string())
}

// -----------------------------
// Commandes CRUD
// -----------------------------

/// Insert avec schéma
#[command]
pub fn jsondb_insert_with_schema(
    storage: State<StorageEngine>,
    space: String,
    db: String,
    schema_rel: String,
    mut doc: Value,
) -> Result<Value, String> {
    let m = mgr(&storage, &space, &db)?;
    m.insert_with_schema(&schema_rel, doc.take())
        .map_err(|e| e.to_string())
}

/// Upsert avec schéma
#[command]
pub fn jsondb_upsert_with_schema(
    storage: State<StorageEngine>,
    space: String,
    db: String,
    schema_rel: String,
    mut doc: Value,
) -> Result<Value, String> {
    let m = mgr(&storage, &space, &db)?;
    m.upsert_with_schema(&schema_rel, doc.take())
        .map_err(|e| e.to_string())
}

/// Insert direct (sans schéma)
#[command]
pub fn jsondb_insert_raw(
    storage: State<StorageEngine>,
    space: String,
    db: String,
    collection: String,
    doc: Value,
) -> Result<(), String> {
    let m = mgr(&storage, &space, &db)?;
    m.insert_raw(&collection, &doc).map_err(|e| e.to_string())
}

/// Update avec schéma
#[command]
pub fn jsondb_update_with_schema(
    storage: State<StorageEngine>,
    space: String,
    db: String,
    schema_rel: String,
    mut doc: Value,
) -> Result<Value, String> {
    let m = mgr(&storage, &space, &db)?;
    m.update_with_schema(&schema_rel, doc.take())
        .map_err(|e| e.to_string())
}

/// Update direct (sans schéma)
#[command]
pub fn jsondb_update_raw(
    storage: State<StorageEngine>,
    space: String,
    db: String,
    collection: String,
    doc: Value,
) -> Result<(), String> {
    let m = mgr(&storage, &space, &db)?;
    m.update_raw(&collection, &doc).map_err(|e| e.to_string())
}

/// Lecture par id
#[command]
pub fn jsondb_get(
    storage: State<StorageEngine>,
    space: String,
    db: String,
    collection: String,
    id: String,
) -> Result<Value, String> {
    let m = mgr(&storage, &space, &db)?;
    m.get(&collection, &id).map_err(|e| e.to_string())
}

/// Suppression par id
#[command]
pub fn jsondb_delete(
    storage: State<StorageEngine>,
    space: String,
    db: String,
    collection: String,
    id: String,
) -> Result<(), String> {
    let m = mgr(&storage, &space, &db)?;
    m.delete(&collection, &id).map_err(|e| e.to_string())
}

/// Liste des IDs d’une collection
#[command]
pub fn jsondb_list_ids(
    storage: State<StorageEngine>,
    space: String,
    db: String,
    collection: String,
) -> Result<Vec<String>, String> {
    let m = mgr(&storage, &space, &db)?;
    m.list_ids(&collection).map_err(|e| e.to_string())
}

/// Liste de tous les documents d’une collection
#[command]
pub fn jsondb_list_all(
    storage: State<StorageEngine>,
    space: String,
    db: String,
    collection: String,
) -> Result<Vec<Value>, String> {
    let m = mgr(&storage, &space, &db)?;
    m.list_all(&collection).map_err(|e| e.to_string())
}

/// Rechargement du registre de schémas (force le refresh du cache interne du manager)
#[command]
pub fn jsondb_refresh_registry(
    storage: State<StorageEngine>,
    space: String,
    db: String,
) -> Result<(), String> {
    let m = mgr(&storage, &space, &db)?;
    m.refresh_registry().map_err(|e| e.to_string())
}

// -----------------------------
// Alias pour compatibilité
// -----------------------------

#[command]
pub fn jsondb_insert(
    storage: State<StorageEngine>,
    space: String,
    db: String,
    schema_rel: String,
    doc: Value,
) -> Result<Value, String> {
    jsondb_insert_with_schema(storage, space, db, schema_rel, doc)
}

#[command]
pub fn jsondb_upsert(
    storage: State<StorageEngine>,
    space: String,
    db: String,
    schema_rel: String,
    doc: Value,
) -> Result<Value, String> {
    jsondb_upsert_with_schema(storage, space, db, schema_rel, doc)
}

// -----------------------------
// Moteur de Requêtes (Async)
// -----------------------------

#[command]
pub async fn jsondb_query_collection(
    storage: State<'_, StorageEngine>,
    space: String,
    db: String,
    _bucket: String,
    query_json: String,
) -> Result<QueryResult, String> {
    let query_input: QueryInput = match serde_json::from_str(&query_json) {
        Ok(q) => q,
        Err(e) => return Err(format!("Requête JSON invalide : {}", e)),
    };

    let m = mgr(&storage, &space, &db)?;
    let engine = QueryEngine::new(&m);

    match engine.execute_query(query_input).await {
        Ok(result) => Ok(result),
        Err(e) => Err(format!("Erreur d'exécution de la requête : {}", e)),
    }
}

// -----------------------------
// Transactions (ACID)
// -----------------------------

#[derive(serde::Deserialize)]
pub struct TransactionRequest {
    pub operations: Vec<OperationRequest>,
}

#[derive(serde::Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum OperationRequest {
    Insert { collection: String, doc: Value },
    Update { collection: String, doc: Value },
    Delete { collection: String, id: String },
}

#[command]
pub fn jsondb_execute_transaction(
    storage: State<StorageEngine>,
    space: String,
    db: String,
    request: TransactionRequest,
) -> Result<(), String> {
    // On utilise la config injectée dans le moteur de stockage
    let cfg = &storage.config;

    if crate::json_db::storage::file_storage::open_db(cfg, &space, &db).is_err() {
        return Err(format!("Database {}/{} does not exist", space, db));
    }

    // TransactionManager utilise encore JsonDbConfig directement
    let tm = TransactionManager::new(cfg, &space, &db);

    tm.execute(|tx| {
        for op in request.operations {
            match op {
                OperationRequest::Insert {
                    collection,
                    mut doc,
                } => {
                    let id = match doc.get("id").and_then(|v| v.as_str()) {
                        Some(s) => s.to_string(),
                        None => uuid::Uuid::new_v4().to_string(),
                    };

                    if let Some(obj) = doc.as_object_mut() {
                        obj.insert("id".to_string(), serde_json::Value::String(id.clone()));
                    }
                    tx.add_insert(&collection, &id, doc);
                }
                OperationRequest::Update { collection, doc } => {
                    let id = doc
                        .get("id")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                        .ok_or_else(|| anyhow::anyhow!("Missing id for update"))?;
                    tx.add_update(&collection, &id, None, doc);
                }
                OperationRequest::Delete { collection, id } => {
                    tx.add_delete(&collection, &id, None);
                }
            }
        }
        Ok(())
    })
    .map_err(|e| e.to_string())?;

    Ok(())
}
