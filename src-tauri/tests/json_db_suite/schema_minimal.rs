// FICHIER : src-tauri/tests/json_db_suite/schema_minimal.rs

use crate::common::{ensure_db_exists, init_test_env, TEST_DB, TEST_SPACE};
use genaptitude::json_db::schema::{SchemaRegistry, SchemaValidator};
// use genaptitude::json_db::storage::file_storage; // Plus nécessaire
use serde_json::json;

#[test]
fn schema_instantiate_validate_minimal() {
    let test_env = init_test_env();
    let cfg = &test_env.cfg;

    let space = TEST_SPACE;
    let db = TEST_DB;

    // 1) S'assurer que la DB existe et contient les schémas
    ensure_db_exists(cfg, space, db);

    // 2) Charger le registre (depuis la DB peuplée)
    let reg = SchemaRegistry::from_db(cfg, space, db).expect("registry from DB");

    // URI du schéma à tester
    let root_uri = reg.uri("actors/actor.schema.json");

    // Si le schéma n'est pas trouvé (mock failed), on skip ou on fail proprement
    if reg.get_by_uri(&root_uri).is_none() {
        // On peut tenter de créer un schéma à la volée pour que le test passe même en mode dégradé
        // Mais ensure_db_exists devrait avoir fait le travail (copie ou mock)
        panic!("Schéma introuvable dans le registre de test: {}", root_uri);
    }

    let validator =
        SchemaValidator::compile_with_registry(&root_uri, &reg).expect("compile failed");

    // 3) Document minimal
    let mut doc = json!({
      "handle": "devops-engineer",
      "displayName": "Ingénieur DevOps",
      "label": { "fr": "Ingénieur DevOps", "en": "DevOps Engineer" },
      "emoji": "🛠️",
      "kind": "human",
      "tags": ["core"]
    });

    // 4) Compute + Validate
    validator
        .compute_then_validate(&mut doc)
        .expect("compute + validate failed");

    // 5) Vérifications
    assert!(
        doc.get("id").is_some() || doc.get("_id").is_some(),
        "id manquant"
    );
    assert!(doc.get("createdAt").is_some(), "createdAt manquant");
    assert!(doc.get("updatedAt").is_some(), "updatedAt manquant");
}
