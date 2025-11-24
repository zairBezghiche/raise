// src-tauri/tests/json_db_suite/json_db_lifecycle.rs

use std::fs;

use genaptitude::json_db::storage::file_storage::{create_db, drop_db, open_db, DropMode};

// On réutilise l’environnement partagé
use crate::common::{init_test_env, TEST_DB, TEST_SPACE};

/// Cycle complet : create → open → drop (Soft puis Hard)
#[test]
fn db_lifecycle_minimal() {
    // 1) Domaine de test
    // La variable `env` assure le nettoyage automatique à la fin du scope via Drop
    let env = init_test_env();
    let cfg = &env.cfg;

    let space = TEST_SPACE;
    let db = TEST_DB;

    // CREATE
    let handle = create_db(cfg, space, db).expect("create_db doit réussir");
    assert!(handle.root.is_dir(), "db root doit exister physiquement");

    let index_path = cfg.index_path(space, db);
    assert!(
        index_path.is_file(),
        "le fichier d'index (ex: {db}.json) doit exister"
    );

    // OPEN
    let opened = open_db(cfg, space, db).expect("open_db doit réussir");
    assert_eq!(opened.space, space);
    assert_eq!(opened.database, db);
    assert_eq!(opened.root, handle.root);

    // DROP (Soft) → renommage
    drop_db(cfg, space, db, DropMode::Soft).expect("drop_db soft doit réussir");
    assert!(
        !handle.root.exists(),
        "après soft drop, le dossier original ne doit plus exister"
    );

    // Vérifie qu’un dossier renommé `<db>.deleted-<ts>` existe
    let mut found_soft = false;
    // On utilise cfg.space_root pour lister le contenu de l'espace
    for entry in fs::read_dir(cfg.space_root(space)).expect("ls space_root") {
        let p = entry.expect("dirent").path();
        let name = p.file_name().unwrap().to_string_lossy().to_string();
        // Le dossier doit commencer par le nom de la db et contenir le marqueur deleted
        if name.starts_with(db) && name.contains(".deleted-") && p.is_dir() {
            found_soft = true;
            break;
        }
    }
    assert!(
        found_soft,
        "le dossier renommé *.deleted-<ts> doit exister après un soft drop"
    );

    // Re-crée puis DROP (Hard) → suppression définitive
    let handle2 = create_db(cfg, space, db).expect("recreate_db doit réussir");
    assert!(handle2.root.exists());

    drop_db(cfg, space, db, DropMode::Hard).expect("drop_db hard doit réussir");

    assert!(
        !cfg.db_root(space, db).exists(),
        "après hard drop, la DB doit être supprimée définitivement"
    );
}
