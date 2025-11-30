use crate::json_db::storage::JsonDbConfig;
use crate::json_db::transactions::manager::TransactionManager;
use serde_json::json;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_transaction_commit_success() {
    let dir = tempdir().unwrap();
    let config = JsonDbConfig {
        data_root: dir.path().to_path_buf(),
    };
    let space = "test_space";
    let db = "test_db";

    fs::create_dir_all(config.db_root(space, db).join("users")).unwrap();

    let tm = TransactionManager::new(&config, space, db);

    let doc = json!({"name": "Alice", "age": 30});
    let id = "user1";

    let res = tm.execute(|tx| {
        tx.add_insert("users", id, doc.clone());
        Ok(())
    });

    assert!(res.is_ok());

    let doc_path = config
        .db_collection_path(space, db, "users")
        .join("user1.json");
    assert!(doc_path.exists());
}

#[test]
fn test_transaction_rollback_on_error() {
    let dir = tempdir().unwrap();
    let config = JsonDbConfig {
        data_root: dir.path().to_path_buf(),
    };
    let space = "test_space";
    let db = "test_db";

    fs::create_dir_all(config.db_root(space, db).join("users")).unwrap();

    let tm = TransactionManager::new(&config, space, db);

    let res = tm.execute(|tx| {
        tx.add_insert("users", "user2", json!({"name": "Bob"}));
        Err(anyhow::anyhow!("Oups"))
    });

    assert!(res.is_err());

    let doc_path = config
        .db_collection_path(space, db, "users")
        .join("user2.json");
    assert!(!doc_path.exists());
}

#[test]
fn test_wal_persistence() {
    let dir = tempdir().unwrap();
    let config = JsonDbConfig {
        data_root: dir.path().to_path_buf(),
    };
    let tm = TransactionManager::new(&config, "s", "d");

    // Le dossier WAL doit être créé
    let _ = tm.execute(|_| Ok(()));
    assert!(config.db_root("s", "d").join("wal").exists());
}
