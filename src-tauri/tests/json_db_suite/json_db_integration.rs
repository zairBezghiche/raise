use serde_json::Value;
use std::fs;

// 💡 Import de get_dataset_root et des constantes de test
use crate::common::{ensure_db_exists, get_dataset_root, init_test_env, TEST_DB, TEST_SPACE};
use genaptitude::json_db::collections::manager::CollectionsManager;
use genaptitude::json_db::storage::JsonDbConfig;

/// Charge l'article de base depuis le dataset externe via le helper commun
fn load_base_article(_cfg: &JsonDbConfig) -> Value {
    let root = get_dataset_root(); // Utilisation du helper partagé
    let article_path = root.join("arcadia/v1/data/articles/article.json");

    // eprintln!("📄 Chargement dataset : {}", article_path.display());

    if !article_path.exists() {
        panic!(
            "❌ Dataset introuvable : {}\nVérifiez votre .env (PATH_GENAPTITUDE_DATASET).",
            article_path.display()
        );
    }

    let raw = fs::read_to_string(&article_path)
        .unwrap_or_else(|e| panic!("Impossible de lire le fichier : {e}"));

    serde_json::from_str(&raw).unwrap_or_else(|e| panic!("JSON invalide : {e}"))
}

#[test]
fn query_get_article_by_id() {
    let test_env = init_test_env();
    let cfg = &test_env.cfg;

    // Initialisation physique
    ensure_db_exists(cfg, TEST_SPACE, TEST_DB);

    // CORRECTION : On passe &test_env.storage au lieu de cfg
    let mgr = CollectionsManager::new(&test_env.storage, TEST_SPACE, TEST_DB);

    let mut doc = load_base_article(cfg);

    if let Some(obj) = doc.as_object_mut() {
        obj.remove("id");
        obj.insert(
            "handle".to_string(),
            Value::String("test-handle-get".to_string()),
        );
    }

    let stored = mgr
        .insert_with_schema("articles/article.schema.json", doc)
        .expect("insert article failed");

    let id = stored
        .get("id")
        .and_then(|v| v.as_str())
        .expect("id manquant");

    let loaded = mgr.get("articles", id).expect("get failed");
    assert_eq!(loaded.get("id").and_then(|v| v.as_str()), Some(id));
}

#[test]
fn query_find_one_article_by_handle() {
    let test_env = init_test_env();
    let cfg = &test_env.cfg;

    ensure_db_exists(cfg, TEST_SPACE, TEST_DB);

    // CORRECTION : On passe &test_env.storage
    let mgr = CollectionsManager::new(&test_env.storage, TEST_SPACE, TEST_DB);

    let base_doc = load_base_article(cfg);
    let target_handle = "handle-002";
    let mut target_id = String::new();

    for i in 1..=3 {
        let mut doc = base_doc.clone();
        let h = format!("handle-{:03}", i);

        if let Some(obj) = doc.as_object_mut() {
            obj.remove("id");
            obj.insert("handle".to_string(), Value::String(h.clone()));
        }

        let stored = mgr
            .insert_with_schema("articles/article.schema.json", doc)
            .expect("insert");

        if h == target_handle {
            target_id = stored.get("id").unwrap().as_str().unwrap().to_string();
        }
    }

    let all = mgr.list_all("articles").expect("list_all");
    let found = all
        .iter()
        .find(|d| d.get("handle").and_then(|v| v.as_str()) == Some(target_handle))
        .expect("article non trouvé");

    assert_eq!(found.get("id").unwrap().as_str().unwrap(), target_id);
}

#[test]
fn query_find_many_with_sort_and_limit_simulated() {
    let test_env = init_test_env();
    let cfg = &test_env.cfg;

    ensure_db_exists(cfg, TEST_SPACE, TEST_DB);

    // CORRECTION : On passe &test_env.storage
    let mgr = CollectionsManager::new(&test_env.storage, TEST_SPACE, TEST_DB);

    let base_doc = load_base_article(cfg);

    for i in 1..=5 {
        let mut doc = base_doc.clone();
        if let Some(obj) = doc.as_object_mut() {
            obj.remove("id");
            obj.insert(
                "handle".to_string(),
                Value::String(format!("sort-{:03}", i)),
            );
        }
        mgr.insert_with_schema("articles/article.schema.json", doc)
            .expect("insert");
    }

    let mut all = mgr.list_all("articles").expect("list_all");

    all.sort_by(|a, b| {
        let ha = a.get("handle").and_then(|v| v.as_str()).unwrap_or("");
        let hb = b.get("handle").and_then(|v| v.as_str()).unwrap_or("");
        hb.cmp(ha)
    });

    let handles: Vec<String> = all
        .iter()
        .take(3)
        .map(|d| d.get("handle").unwrap().as_str().unwrap().to_string())
        .collect();

    assert_eq!(handles, vec!["sort-005", "sort-004", "sort-003"]);
}
