use serde_json::json;
use serde_json::Value;
use std::fs;

use genaptitude::json_db::{
    collections::manager::CollectionsManager,
    query::{
        ComparisonOperator, Condition, FilterOperator, Query, QueryEngine, QueryFilter, SortField,
        SortOrder,
    },
    storage::JsonDbConfig,
};
// 💡 Ajout de ensure_db_exists dans les imports
use crate::common::{ensure_db_exists, get_dataset_root, init_test_env, TEST_DB, TEST_SPACE};

// Helper local utilisant le helper global
fn load_test_doc(_cfg: &JsonDbConfig) -> Value {
    let root = get_dataset_root();
    let path = root.join("arcadia/v1/data/articles/article.json");

    if !path.exists() {
        panic!("❌ Dataset article.json introuvable : {}", path.display());
    }

    let raw = fs::read_to_string(&path).expect("Lecture impossible");
    serde_json::from_str(&raw).expect("JSON invalide")
}

fn seed_article<'a>(mgr: &'a CollectionsManager<'a>, handle: &str, doc_template: &Value) -> String {
    let mut doc = doc_template.clone();
    if let Some(obj) = doc.as_object_mut() {
        obj.remove("id");
        obj.insert("handle".to_string(), Value::String(handle.to_string()));
    }

    let stored = mgr
        .insert_with_schema("articles/article.schema.json", doc)
        .expect("insert failed");

    stored.get("id").unwrap().as_str().unwrap().to_string()
}

#[tokio::test]
async fn query_get_article_by_id() {
    let test_env = init_test_env();

    // 💡 CORRECTION : Initialisation physique de la DB avant utilisation
    ensure_db_exists(&test_env.cfg, TEST_SPACE, TEST_DB);

    // CORRECTION : On passe &test_env.storage au lieu de cfg
    let mgr = CollectionsManager::new(&test_env.storage, TEST_SPACE, TEST_DB);
    let base_doc = load_test_doc(&test_env.cfg);

    let handle = "query-get-id";
    let id = seed_article(&mgr, handle, &base_doc);

    // Test via Manager.get
    let loaded = mgr.get("articles", &id).expect("get failed");
    assert_eq!(loaded.get("handle").unwrap().as_str(), Some(handle));
}

#[tokio::test]
async fn query_find_one_article_by_handle() {
    let test_env = init_test_env();

    // 💡 CORRECTION : Initialisation physique de la DB
    ensure_db_exists(&test_env.cfg, TEST_SPACE, TEST_DB);

    // CORRECTION : On passe &test_env.storage au lieu de cfg
    let mgr = CollectionsManager::new(&test_env.storage, TEST_SPACE, TEST_DB);
    let base_doc = load_test_doc(&test_env.cfg);

    let handle = "query-find-one";
    seed_article(&mgr, handle, &base_doc);

    let engine = QueryEngine::new(&mgr);

    let filter = QueryFilter {
        operator: FilterOperator::And,
        conditions: vec![Condition {
            field: "handle".to_string(),
            operator: ComparisonOperator::Eq,
            value: json!(handle),
        }],
    };

    let query = Query {
        collection: "articles".to_string(),
        filter: Some(filter),
        sort: None,
        limit: Some(1),
        offset: None,
        projection: None,
    };

    let result = engine.execute_query(query).await.expect("query failed");
    assert!(!result.documents.is_empty());
    assert_eq!(
        result.documents[0].get("handle").unwrap().as_str(),
        Some(handle)
    );
}

#[tokio::test]
async fn query_find_many_with_sort_and_limit() {
    let test_env = init_test_env();

    // 💡 CORRECTION : Initialisation physique de la DB
    ensure_db_exists(&test_env.cfg, TEST_SPACE, TEST_DB);

    // CORRECTION : On passe &test_env.storage au lieu de cfg
    let mgr = CollectionsManager::new(&test_env.storage, TEST_SPACE, TEST_DB);
    let base_doc = load_test_doc(&test_env.cfg);

    for i in 0..5 {
        seed_article(&mgr, &format!("sort-{}", i), &base_doc);
    }

    let engine = QueryEngine::new(&mgr);

    let q = Query {
        collection: "articles".to_string(),
        filter: None,
        sort: Some(vec![SortField {
            field: "handle".to_string(),
            order: SortOrder::Desc,
        }]),
        offset: Some(0),
        limit: Some(3),
        projection: None,
    };

    let result = engine.execute_query(q).await.expect("query failed");

    assert_eq!(result.documents.len(), 3);
    assert_eq!(
        result.documents[0].get("handle").unwrap().as_str(),
        Some("sort-4")
    );
}
