use crate::common::{init_test_env, TEST_DB, TEST_SPACE};
use genaptitude::json_db::schema::{SchemaRegistry, SchemaValidator};
use genaptitude::json_db::storage::file_storage;
use serde_json::json;

#[test]
fn schema_instantiate_validate_minimal() {
    // 1) Initialisation de l'environnement (nettoyage auto via Drop)
    let test_env = init_test_env();
    let cfg = &test_env.cfg;

    let space = TEST_SPACE;
    let db = TEST_DB;

    // 2) Création de la DB (idempotent, assure que le dossier existe pour le registre)
    // Note: create_db est nécessaire car SchemaRegistry::from_db s'attend à ce que la structure physique existe
    let _ = file_storage::create_db(cfg, space, db).expect("create_db failed");

    // 3) Registre strict DB + compilateur
    // Le registre va charger les schémas présents dans la DB (qui ont été seedés par create_db si configuré,
    // ou qui existent via le lien vers schemas_dev_root dans la config de test)
    let reg = SchemaRegistry::from_db(cfg, space, db).expect("registry from DB");

    // URI du schéma à tester
    let root_uri = reg.uri("actors/actor.schema.json");
    let validator =
        SchemaValidator::compile_with_registry(&root_uri, &reg).expect("compile failed");

    // 4) Document minimal volontairement SANS id/createdAt/updatedAt
    // Ces champs sont marqués 'x_compute' dans le schéma et doivent être générés automatiquement.
    let mut doc = json!({
      "handle": "devops-engineer",
      "displayName": "Ingénieur DevOps",
      "label": { "fr": "Ingénieur DevOps", "en": "DevOps Engineer" },
      "emoji": "🛠️",
      "kind": "human",
      "tags": ["core"]
    });

    // 5) Déclenche les x_compute (uuid_v4, now_ts_ms, etc.) PUIS valide
    validator
        .compute_then_validate(&mut doc)
        .expect("compute + validate failed");

    // 6) Vérifie que les champs calculés existent bien dans le document modifié
    assert!(
        doc.get("id").is_some() || doc.get("_id").is_some(),
        "Un champ d'identifiant (id ou _id) doit avoir été calculé"
    );
    assert!(
        doc.get("createdAt").is_some(),
        "createdAt doit avoir été calculé"
    );
    assert!(
        doc.get("updatedAt").is_some(),
        "updatedAt doit avoir été calculé"
    );

    println!("✅ Document validé et complété : {}", doc);
}
