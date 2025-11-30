//! JSON-DB Tauri commands

use serde::Serialize;
use serde_json::Value;
use tauri::{command, State};

use crate::json_db::collections::manager::CollectionsManager;
use crate::json_db::query::sql::parse_sql;
use crate::json_db::query::{Query, QueryEngine};
use crate::json_db::storage::{file_storage, StorageEngine};

#[derive(Serialize)]
pub struct QueryResponse {
    pub documents: Vec<Value>,
    pub total: usize,
}

fn mgr<'a>(
    storage: &'a State<StorageEngine>,
    space: &str,
    db: &str,
) -> Result<CollectionsManager<'a>, String> {
    let config = &storage.config;
    if file_storage::open_db(config, space, db).is_err() {
        file_storage::create_db(config, space, db).map_err(|e| e.to_string())?;
    }
    Ok(CollectionsManager::new(storage.inner(), space, db))
}

#[command]
pub async fn jsondb_create_collection(
    storage: State<'_, StorageEngine>,
    space: String,
    db: String,
    collection: String,
    schema_uri: Option<String>,
) -> Result<String, String> {
    let manager = mgr(&storage, &space, &db)?;
    manager
        .create_collection(&collection, schema_uri)
        .map_err(|e| e.to_string())?;
    Ok(format!("Collection '{}' created.", collection))
}

#[command]
pub async fn jsondb_list_collections(
    storage: State<'_, StorageEngine>,
    space: String,
    db: String,
) -> Result<Vec<String>, String> {
    let manager = mgr(&storage, &space, &db)?;
    manager.list_collections().map_err(|e| e.to_string())
}

#[command]
pub async fn jsondb_insert_document(
    storage: State<'_, StorageEngine>,
    space: String,
    db: String,
    collection: String,
    document: Value,
) -> Result<Value, String> {
    let manager = mgr(&storage, &space, &db)?;
    let inserted_doc = manager
        .insert_with_schema(&collection, document)
        .map_err(|e| format!("Insert Failed: {}", e))?;
    Ok(inserted_doc)
}

#[command]
pub async fn jsondb_get_document(
    storage: State<'_, StorageEngine>,
    space: String,
    db: String,
    collection: String,
    id: String,
) -> Result<Option<Value>, String> {
    let manager = mgr(&storage, &space, &db)?;
    manager
        .get_document(&collection, &id)
        .map_err(|e| e.to_string())
}

#[command]
pub async fn jsondb_update_document(
    storage: State<'_, StorageEngine>,
    space: String,
    db: String,
    collection: String,
    id: String,
    document: Value,
) -> Result<Value, String> {
    let manager = mgr(&storage, &space, &db)?;
    manager
        .update_document(&collection, &id, document)
        .map_err(|e| format!("Update Failed: {}", e))
}

#[command]
pub async fn jsondb_delete_document(
    storage: State<'_, StorageEngine>,
    space: String,
    db: String,
    collection: String,
    id: String,
) -> Result<bool, String> {
    let manager = mgr(&storage, &space, &db)?;
    manager
        .delete_document(&collection, &id)
        .map_err(|e| e.to_string())
}

#[command]
pub async fn jsondb_execute_query(
    storage: State<'_, StorageEngine>,
    space: String,
    db: String,
    query: Query,
) -> Result<QueryResponse, String> {
    let manager = mgr(&storage, &space, &db)?;
    let engine = QueryEngine::new(&manager);

    let result = engine
        .execute_query(query)
        .await
        .map_err(|e| e.to_string())?;

    Ok(QueryResponse {
        documents: result.documents,
        // CORRECTION : Cast u64 -> usize
        total: result.total_count as usize,
    })
}

#[command]
pub async fn jsondb_execute_sql(
    storage: State<'_, StorageEngine>,
    space: String,
    db: String,
    sql: String,
) -> Result<QueryResponse, String> {
    let manager = mgr(&storage, &space, &db)?;

    let query = parse_sql(&sql).map_err(|e| format!("SQL Parse Error: {}", e))?;

    let engine = QueryEngine::new(&manager);
    let result = engine
        .execute_query(query)
        .await
        .map_err(|e| format!("Execution Error: {}", e))?;

    Ok(QueryResponse {
        documents: result.documents,
        // CORRECTION : Cast u64 -> usize
        total: result.total_count as usize,
    })
}
