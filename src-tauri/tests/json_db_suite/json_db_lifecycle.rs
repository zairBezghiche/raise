// src-tauri/tests/json_db_suite/json_db_lifecycle.rs

use crate::common::{init_test_env, TEST_DB, TEST_SPACE};
use genaptitude::json_db::storage::file_storage::{create_db, drop_db, open_db, DropMode};
use std::fs;

#[test]
fn db_lifecycle_minimal() {
    let env = init_test_env();
    let cfg = &env.cfg;
    let space = TEST_SPACE;
    let db = TEST_DB;

    // CREATE
    create_db(cfg, space, db).expect("create_db doit réussir");

    let db_root = cfg.db_root(space, db);
    assert!(db_root.is_dir(), "db root doit exister physiquement");

    // CORRECTION : _ prefix pour unused variable
    let _index_path = cfg.db_root(space, db).join("_system.json");

    let schemas_path = cfg.db_schemas_root(space, db);
    assert!(schemas_path.exists(), "le dossier schemas doit exister");

    // OPEN
    open_db(cfg, space, db).expect("open_db doit réussir");

    // DROP (Soft)
    drop_db(cfg, space, db, DropMode::Soft).expect("drop_db soft doit réussir");
    assert!(
        !db_root.exists(),
        "après soft drop, le dossier original ne doit plus exister"
    );

    // Vérifie qu’un dossier renommé existe
    let mut found_soft = false;
    let space_root = cfg.data_root.join(space);
    for entry in fs::read_dir(&space_root).expect("ls space_root") {
        let p = entry.expect("dirent").path();
        let name = p.file_name().unwrap().to_string_lossy().to_string();
        if name.starts_with(db) && name.contains(".deleted-") && p.is_dir() {
            found_soft = true;
            break;
        }
    }
    assert!(
        found_soft,
        "le dossier renommé *.deleted-<ts> doit exister après un soft drop"
    );

    // Re-crée puis DROP (Hard)
    create_db(cfg, space, db).expect("recreate_db doit réussir");
    assert!(db_root.exists());

    drop_db(cfg, space, db, DropMode::Hard).expect("drop_db hard doit réussir");

    assert!(
        !db_root.exists(),
        "après hard drop, la DB doit être supprimée définitivement"
    );
}
// ... (les autres tests restent inchangés)
