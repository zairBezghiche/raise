use crate::json_db::collections::manager::CollectionsManager;
use crate::json_db::storage::file_storage;
use crate::json_db::test_utils::init_test_env;
use crate::json_db::transactions::TransactionManager;
use serde_json::json;

// Helper pour initialiser une collection avec un schéma valide
fn setup_collection(cm: &CollectionsManager, name: &str) {
    // Option la plus robuste pour le test : créer un fichier schéma minimal
    // CORRECTION : Accès via cm.storage.config
    let schema_path = cm
        .storage
        .config
        .db_schemas_root(&cm.space, &cm.db)
        .join("minimal.json");

    std::fs::write(&schema_path, r#"{"type":"object"}"#).expect("write dummy schema");

    // Maintenant on peut créer la collection liée à ce schéma
    // CORRECTION : Accès via cm.storage.config
    file_storage::create_collection(&cm.storage.config, &cm.space, &cm.db, name, "minimal.json")
        .expect("create collection with schema");

    // On force aussi la création des index
    // CORRECTION : Accès via cm.storage.config
    crate::json_db::indexes::create_collection_indexes(
        &cm.storage.config,
        &cm.space,
        &cm.db,
        name,
        "minimal.json",
    )
    .expect("create indexes");
}

#[test]
fn test_transaction_commit_success() {
    let env = init_test_env();
    let cfg = &env.cfg;
    let space = &env.space;
    let db = &env.db;

    file_storage::create_db(cfg, space, db).expect("create db");

    // CORRECTION : On passe &env.storage au lieu de cfg
    let cm = CollectionsManager::new(&env.storage, space, db);
    setup_collection(&cm, "users");

    let tm = TransactionManager::new(cfg, space, db);

    let result = tm.execute(|tx| {
        tx.add_insert(
            "users",
            "user-1",
            json!({"id": "user-1", "name": "Alice", "balance": 100}),
        );

        tx.add_insert(
            "users",
            "user-2",
            json!({"id": "user-2", "name": "Bob", "balance": 50}),
        );

        Ok(())
    });

    assert!(
        result.is_ok(),
        "La transaction aurait dû réussir : {:?}",
        result.err()
    );

    // Vérifications
    let alice = cm.get("users", "user-1").expect("Alice doit exister");
    assert_eq!(alice["name"], "Alice");

    let index = file_storage::read_index(cfg, space, db).expect("read index");
    let collection_idx = index
        .collections
        .get("users")
        .expect("collection users index");
    assert!(collection_idx.items.iter().any(|i| i.file == "user-1.json"));
}

#[test]
fn test_transaction_rollback_on_error() {
    let env = init_test_env();
    let cfg = &env.cfg;
    let space = &env.space;
    let db = &env.db;

    file_storage::create_db(cfg, space, db).expect("create db");

    // CORRECTION : On passe &env.storage au lieu de cfg
    let cm = CollectionsManager::new(&env.storage, space, db);
    setup_collection(&cm, "users");

    // État initial
    let tm = TransactionManager::new(cfg, space, db);
    tm.execute(|tx| {
        tx.add_insert(
            "users",
            "user-1",
            json!({"id": "user-1", "name": "Alice", "balance": 100}),
        );
        Ok(())
    })
    .expect("Setup initial failed");

    // Transaction qui échoue
    let result = tm.execute(|tx| {
        tx.add_update(
            "users",
            "user-1",
            None,
            json!({"id": "user-1", "name": "Alice", "balance": 0}),
        );
        tx.add_insert(
            "users",
            "user-3",
            json!({"id": "user-3", "name": "Charlie"}),
        );
        anyhow::bail!("Solde insuffisant !")
    });

    assert!(result.is_err());

    let alice = cm.get("users", "user-1").unwrap();
    assert_eq!(alice["balance"], 100, "Rollback réussi");
    assert!(
        cm.get("users", "user-3").is_err(),
        "Charlie ne doit pas exister"
    );
}

#[test]
fn test_wal_persistence() {
    let env = init_test_env();

    file_storage::create_db(&env.cfg, &env.space, &env.db).expect("create db");

    let tm = TransactionManager::new(&env.cfg, &env.space, &env.db);
    // CORRECTION : On passe &env.storage au lieu de &env.cfg
    let cm = CollectionsManager::new(&env.storage, &env.space, &env.db);
    setup_collection(&cm, "logs");

    tm.execute(|tx| {
        tx.add_insert(
            "logs",
            "log-1",
            json!({
                "id": "log-1",
                "msg": "test"
            }),
        );
        Ok(())
    })
    .unwrap();

    let wal_path = env.cfg.db_root(&env.space, &env.db).join("_wal.jsonl");
    let content = std::fs::read_to_string(wal_path).expect("lecture wal");

    assert!(content.contains("log-1"));
    assert!(content.contains("committed"));
}
