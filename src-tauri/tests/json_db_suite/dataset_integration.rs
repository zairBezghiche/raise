// FICHIER : src-tauri/tests/json_db_suite/dataset_integration.rs

use genaptitude::json_db::collections::manager::CollectionsManager;
use genaptitude::json_db::storage::StorageEngine;
use genaptitude::json_db::test_utils::{
    ensure_db_exists, get_dataset_file, init_test_env, TEST_DB, TEST_SPACE,
};
use std::fs;

#[test]
fn debug_import_exchange_item() {
    let env = init_test_env();
    ensure_db_exists(&env.cfg, &env.space, &env.db);

    let refreshed_storage = StorageEngine::new(env.cfg.clone());
    let mgr = CollectionsManager::new(&refreshed_storage, &env.space, &env.db);

    println!("--- 🛠️  DEBUG IMPORT EXCHANGE ITEM ---");

    let data_path = get_dataset_file(&env.cfg, "arcadia/v1/data/exchange-items/position_gps.json");
    let json_content = fs::read_to_string(&data_path).expect("Lecture donnée impossible");
    let mut json_doc: serde_json::Value =
        serde_json::from_str(&json_content).expect("JSON malformé");

    // URI du schéma (doit exister dans le dossier schemas/v1 copié par init_test_env)
    let schema_rel_path = "arcadia/data/exchange-item.schema.json";
    let db_schema_uri = format!(
        "db://{}/{}/schemas/v1/{}",
        TEST_SPACE, TEST_DB, schema_rel_path
    );

    if let Some(obj) = json_doc.as_object_mut() {
        obj.insert(
            "$schema".to_string(),
            serde_json::Value::String(db_schema_uri.clone()),
        );
    }

    match mgr.create_collection("exchange-items", Some(db_schema_uri)) {
        Ok(_) => println!("✅ Collection créée."),
        Err(e) => panic!("❌ ÉCHEC CREATE_COLLECTION : {}", e),
    }

    match mgr.insert_with_schema("exchange-items", json_doc) {
        Ok(res) => {
            println!("✅ SUCCÈS ! ID: {}", res.get("id").unwrap());
            assert!(res.get("id").is_some());
        }
        Err(e) => panic!("❌ ÉCHEC INSERTION : {}", e),
    }
}
