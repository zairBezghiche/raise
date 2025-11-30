use crate::common::{ensure_db_exists, init_test_env, TEST_DB, TEST_SPACE};
use genaptitude::json_db::schema::{SchemaRegistry, SchemaValidator};
use serde_json::json;
use uuid::Uuid;

#[test]
fn workunit_compute_then_validate_minimal() {
    let test_env = init_test_env();
    let cfg = &test_env.cfg;
    let space = TEST_SPACE;
    let db = TEST_DB;

    ensure_db_exists(cfg, space, db);

    let reg = SchemaRegistry::from_db(cfg, space, db).expect("registry init failed");
    let root_uri = reg.uri("workunits/workunit.schema.json");

    if reg.get_by_uri(&root_uri).is_none() {
        panic!("Schéma workunit introuvable");
    }

    let validator =
        SchemaValidator::compile_with_registry(&root_uri, &reg).expect("compile workunit failed");

    // CORRECTION : Fournir un objet 'finance' COMPLET pour satisfaire le schéma strict
    let mut doc = json!({
        "code": "WU-DEVOPS-01",
        "name": "DevOps pipeline",
        "status": "draft",
        "version": "1.0.0",
        "finance": {
            "version": "1.0.0",
            "billing_model": "T&M",
            "revenue_scenarios": {},
            "gross_margin": {},
            "summary": {},
            "synthese_build": {
                "controle": {}
            },
            // Requis par le schéma finance
            "activation_credit": {
                "prix_eur": 0,
                "volume": 0,
                "unite": "N/A"
            },
            // Requis par le schéma finance
            "charges_par_sprint": {
                "tjm_eur": 0,
                "sprints": {
                    "sprint_0_1": { "items": [] },
                    "sprint_2": { "items": [] }
                }
            }
        }
    });

    validator
        .compute_then_validate(&mut doc)
        .expect("compute+validate failed");

    let id = doc.get("id").and_then(|v| v.as_str()).expect("id manquant");
    assert!(Uuid::parse_str(id).is_ok());
    assert!(doc.get("createdAt").is_some());
}

#[test]
fn finance_compute_minimal() {
    let env = init_test_env();
    let cfg = &env.cfg;
    let space = TEST_SPACE;
    let db = TEST_DB;

    ensure_db_exists(cfg, space, db);

    let reg = SchemaRegistry::from_db(cfg, space, db).expect("registry init failed");
    let root_uri = reg.uri("workunits/finance.schema.json");

    if reg.get_by_uri(&root_uri).is_none() {
        panic!("Schéma finance introuvable");
    }

    let validator =
        SchemaValidator::compile_with_registry(&root_uri, &reg).expect("compile finance failed");

    // CORRECTION : Ajout de 'charges_par_sprint' et détails 'activation_credit'
    let mut finance = json!({
        "billing_model": "T&M",
        "version": "1.0.0",
        "revenue_scenarios": {},
        "gross_margin": {},
        "summary": {},
        "synthese_build": {
             "controle": {}
        },
        "activation_credit": {
            "prix_eur": 100,
            "volume": 1,
            "unite": "UO"
        },
        "charges_par_sprint": {
            "tjm_eur": 500,
            "sprints": {
                "sprint_0_1": { "items": [] },
                "sprint_2": { "items": [] }
            }
        }
    });

    validator
        .compute_then_validate(&mut finance)
        .expect("compute+validate finance failed");

    // On vérifie que le calcul a bien fonctionné (ex: total_eur calculé)
    let total = finance.pointer("/activation_credit/total_eur");
    assert!(total.is_some(), "Total EUR devrait être calculé");
    assert_eq!(total.unwrap().as_f64(), Some(100.0));
}
