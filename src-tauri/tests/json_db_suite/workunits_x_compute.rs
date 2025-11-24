use crate::common::{init_test_env, TEST_DB, TEST_SPACE};

use genaptitude::json_db::{
    schema::{SchemaRegistry, SchemaValidator},
    storage::file_storage,
};

use serde_json::json;
use uuid::Uuid;

#[test]
fn workunit_compute_then_validate_minimal() {
    // 1) Init Env (Nettoyage auto via Drop)
    let test_env = init_test_env();
    let cfg = &test_env.cfg;

    // Utilisation explicite des constantes ou de l'env
    let space = TEST_SPACE;
    let db = TEST_DB;

    // 2) DB idempotente + registre
    // On s'assure que la DB est créée pour que le registre puisse résoudre les chemins relatifs
    file_storage::create_db(cfg, space, db).expect("create_db failed");

    let reg = SchemaRegistry::from_db(cfg, space, db).expect("registry init failed");
    let root_uri = reg.uri("workunits/workunit.schema.json");

    let validator =
        SchemaValidator::compile_with_registry(&root_uri, &reg).expect("compile workunit failed");

    // 3) Document minimal : on fournit seulement les champs métier requis
    let mut doc = json!({
        "code": "WU-DEVOPS-01",
        "name": "DevOps pipeline"
        // id/createdAt/updatedAt/$schema/version seront calculés par x_compute
    });

    // 4) Calcul et Validation
    validator
        .compute_then_validate(&mut doc)
        .expect("compute+validate failed");

    // 5) Vérifications
    // $schema correctement injecté (chemin relatif attendu)
    assert!(
        doc.get("$schema")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .contains("workunits/workunit.schema.json"),
        "$schema incorrect ou absent"
    );

    // id présent et bien formé (UUID)
    let id = doc.get("id").and_then(|v| v.as_str()).expect("id manquant");
    assert!(
        Uuid::parse_str(id).is_ok(),
        "id is not a valid UUID: {}",
        id
    );

    // timestamps présents
    assert!(doc.get("createdAt").is_some(), "createdAt manquant");
    assert!(doc.get("updatedAt").is_some(), "updatedAt manquant");
}

#[test]
fn finance_compute_minimal() {
    // 1) Init Env
    let env = init_test_env();
    let cfg = &env.cfg;

    // CORRECTION : Utiliser l'espace/db de l'environnement initialisé au lieu de hardcoder "un2"
    let space = env.space.as_str();
    let db = env.db.as_str();

    file_storage::create_db(cfg, space, db).expect("create_db failed");

    let reg = SchemaRegistry::from_db(cfg, space, db).expect("registry init failed");
    let root_uri = reg.uri("workunits/finance.schema.json");

    let validator =
        SchemaValidator::compile_with_registry(&root_uri, &reg).expect("compile finance failed");

    // 2) Document métier
    // Adapte la valeur à l'enum réelle du schéma ("T&M", "fixed_price", etc.)
    let mut finance = json!({
        "billing_model": "T&M"
        // le reste sera complété par x_compute/defaults
    });

    // 3) Calcul et Validation
    validator
        .compute_then_validate(&mut finance)
        .expect("compute+validate finance failed");

    // 4) Vérifications
    assert!(
        finance
            .get("$schema")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .contains("workunits/finance.schema.json"),
        "$schema finance incorrect"
    );

    // Vérification d'un champ calculé (si défini dans le schéma)
    // assert!(finance.get("summary").is_some());
}
