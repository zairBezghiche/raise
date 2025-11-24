use crate::common::{init_test_env, TEST_DB, TEST_SPACE};
use genaptitude::json_db::storage::file_storage::{create_db, drop_db, open_db, DropMode};

#[test]
fn drop_is_idempotent_and_recreate_works() {
    let test_env = init_test_env();
    let cfg = &test_env.cfg;

    let space = TEST_SPACE;
    let db = TEST_DB;

    // 1) Drop sur DB inexistante → OK (idempotent)
    // On vérifie que ça ne panique pas et retourne Ok(())
    drop_db(cfg, space, db, DropMode::Soft).expect("soft drop sur DB inexistante devrait réussir");
    drop_db(cfg, space, db, DropMode::Hard).expect("hard drop sur DB inexistante devrait réussir");

    // 2) Cycle de vie : create → open → hard drop
    let h = create_db(cfg, space, db).expect("create doit réussir");

    // Vérification physique
    assert!(
        h.root.exists(),
        "Le dossier racine de la DB doit exister après create"
    );

    // Vérification logique
    let _ = open_db(cfg, space, db).expect("open doit réussir sur une DB existante");

    // Suppression
    drop_db(cfg, space, db, DropMode::Hard).expect("hard drop final doit réussir");

    // Vérification finale
    assert!(
        !cfg.db_root(space, db).exists(),
        "Le dossier racine doit avoir disparu"
    );
}
