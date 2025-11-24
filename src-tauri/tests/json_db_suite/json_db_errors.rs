// src-tauri/tests/json_db_suite/json_db_errors.rs

use genaptitude::json_db::storage::file_storage::{create_db, open_db};

// Helpers et constantes définis dans json_db_suite::mod.rs
use crate::common::{init_test_env, TEST_DB, TEST_SPACE};

/// Vérifie que :
/// - `open_db` échoue si la DB n'existe pas
/// - `create_db` échoue si on essaye de recréer une DB existante
#[test]
fn open_missing_db_fails_and_create_twice_fails() {
    // 1) Initialiser l’environnement de test
    // La variable `env` détient la config et le chemin temporaire.
    // Le dossier sera supprimé automatiquement quand `env` sortira du scope (à la fin de la fonction).
    let env = init_test_env();
    let cfg = &env.cfg;

    let space = TEST_SPACE;
    let db = TEST_DB;

    // 2) open sur DB inexistante → Err
    assert!(
        open_db(cfg, space, db).is_err(),
        "open_db devrait échouer si `{space}/{db}` n'existe pas"
    );

    // 3) create puis create à nouveau

    // Premier appel : Succès attendu
    create_db(cfg, space, db).expect("premier create_db doit réussir");

    // Second appel : Échec attendu (Doublon)
    assert!(
        create_db(cfg, space, db).is_err(),
        "second create_db doit échouer car `{space}/{db}` existe déjà"
    );

    // 4) Cleanup automatique via TestEnv::drop ici
}
