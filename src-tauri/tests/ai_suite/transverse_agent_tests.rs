use crate::common::init_ai_test_env;
use genaptitude::ai::agents::intent_classifier::EngineeringIntent;
use genaptitude::ai::agents::{transverse_agent::TransverseAgent, Agent, AgentContext};
use genaptitude::ai::llm::client::LlmClient;
use std::sync::Arc;

#[tokio::test]
#[ignore]
async fn test_transverse_agent_ivvq_cycle() {
    dotenvy::dotenv().ok();
    let env = init_ai_test_env();

    // Config Robuste
    let api_key = std::env::var("GENAPTITUDE_GEMINI_KEY").unwrap_or_default();
    let local_url = std::env::var("GENAPTITUDE_LOCAL_URL")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());
    let model_name = std::env::var("GENAPTITUDE_MODEL_NAME").ok();

    if !env.client.ping_local().await && api_key.is_empty() {
        println!("⚠️ SKIPPED: Pas d'IA disponible.");
        return;
    }

    let client = LlmClient::new(&local_url, &api_key, model_name);
    let test_root = env.storage.config.data_root.clone();
    let ctx = AgentContext::new(
        Arc::new(env.storage.clone()),
        client.clone(),
        test_root.clone(),
        test_root.join("dataset"),
    );

    let agent = TransverseAgent::new();

    // 1. CRÉATION EXIGENCE
    let intent_req = EngineeringIntent::CreateElement {
        layer: "TRANSVERSE".to_string(),
        element_type: "Requirement".to_string(),
        name: "Performance Démarrage".to_string(),
    };
    println!("✨ [1/3] Création Exigence...");
    let res_req = agent.process(&ctx, &intent_req).await;
    assert!(res_req.is_ok());

    // 2. CRÉATION TEST PROCEDURE
    let intent_test = EngineeringIntent::CreateElement {
        layer: "TRANSVERSE".to_string(),
        element_type: "TestProcedure".to_string(),
        name: "Test Temps Démarrage".to_string(),
    };
    println!("✨ [2/3] Création Procédure de Test...");
    let res_test = agent.process(&ctx, &intent_test).await;
    assert!(res_test.is_ok());

    // 3. CRÉATION CAMPAGNE
    let intent_camp = EngineeringIntent::CreateElement {
        layer: "TRANSVERSE".to_string(),
        element_type: "TestCampaign".to_string(),
        name: "Campagne V1.0".to_string(),
    };
    println!("✨ [3/3] Création Campagne...");
    let res_camp = agent.process(&ctx, &intent_camp).await;
    assert!(res_camp.is_ok());

    // VÉRIFICATION PHYSIQUE
    tokio::time::sleep(std::time::Duration::from_millis(2000)).await;

    // Check Requirement
    let req_dir = test_root.join("un2/transverse/requirements");
    let mut found_req = false;
    if req_dir.exists() {
        for entry in std::fs::read_dir(&req_dir).unwrap() {
            if let Ok(e) = entry {
                let content = std::fs::read_to_string(e.path())
                    .unwrap_or_default()
                    .to_lowercase();
                if content.contains("req-sys") || content.contains("exigence") {
                    found_req = true;
                    println!("✅ Exigence validée : {:?}", e.file_name());
                }
            }
        }
    }
    assert!(found_req, "Exigence non trouvée.");

    // Check Test Procedure
    let proc_dir = test_root.join("un2/transverse/tests/procedures");
    let mut found_proc = false;
    if proc_dir.exists() {
        for entry in std::fs::read_dir(&proc_dir).unwrap() {
            if let Ok(e) = entry {
                let content = std::fs::read_to_string(e.path())
                    .unwrap_or_default()
                    .to_lowercase();
                // On vérifie que l'IA a bien généré des étapes (steps)
                if content.contains("steps") && content.contains("action") {
                    found_proc = true;
                    println!("✅ Procédure validée : {:?}", e.file_name());
                }
            }
        }
    }
    assert!(found_proc, "Procédure de test non trouvée ou sans étapes.");
}
