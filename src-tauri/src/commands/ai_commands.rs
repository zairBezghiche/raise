use crate::ai::agents::intent_classifier::{EngineeringIntent, IntentClassifier};
use crate::ai::agents::{system_agent::SystemAgent, Agent}; // <-- Nouveaux imports
use crate::ai::context::retriever::SimpleRetriever;
use crate::ai::llm::client::{LlmBackend, LlmClient};
use crate::json_db::storage::StorageEngine;
use crate::model_engine::loader::ModelLoader;

use std::env;
use tauri::{command, State};

#[command]
pub async fn ai_chat(
    storage: State<'_, StorageEngine>,
    user_input: String,
) -> Result<String, String> {
    // 1. Config (Inchangé)
    let mode_dual =
        env::var("GENAPTITUDE_MODE_DUAL").unwrap_or_else(|_| "false".to_string()) == "true";
    let gemini_key = env::var("GENAPTITUDE_GEMINI_KEY").unwrap_or_default();
    let model_name = env::var("GENAPTITUDE_MODEL_NAME").ok();
    let local_url =
        env::var("GENAPTITUDE_LOCAL_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());

    let client = LlmClient::new(&local_url, &gemini_key, model_name.clone());

    // 2. Classification d'intention
    let classifier = IntentClassifier::new(client.clone());
    println!("🧠 Analyse de l'intention...");
    let intent = classifier.classify(&user_input).await;

    // 3. ROUTAGE : Action ou Discussion ?
    match intent {
        // CAS A : L'utilisateur veut AGIR (Créer quelque chose)
        EngineeringIntent::CreateElement { .. } => {
            // On instancie l'agent Système
            let sys_agent = SystemAgent::new(client.clone(), storage.inner().clone());

            // On lui demande de traiter l'intention
            if let Some(result_msg) = sys_agent
                .process(&intent)
                .await
                .map_err(|e| e.to_string())?
            {
                return Ok(result_msg);
            }

            Ok("⚠️ Je n'ai pas trouvé d'agent compétent pour cette création.".to_string())
        }

        // CAS B : Discussion / RAG (Code existant)
        EngineeringIntent::Chat | EngineeringIntent::Unknown => {
            // ... (Code RAG existant : chargement modèle, retrieve, prompt) ...
            // COPIEZ ICI VOTRE LOGIQUE RAG PRÉCÉDENTE

            // Pour rappel rapide du bloc RAG :
            let storage_clone = storage.inner().clone();
            let project_model = tauri::async_runtime::spawn_blocking(move || {
                let loader = ModelLoader::from_engine(&storage_clone, "un2", "_system");
                loader.load_full_model()
            })
            .await
            .map_err(|e| e.to_string())?
            .map_err(|e| e.to_string())?;

            let retriever = SimpleRetriever::new(project_model);
            let context_data = retriever.retrieve_context(&user_input);

            let use_cloud = mode_dual && !gemini_key.is_empty() && is_complex_task(&user_input);
            let (backend, display_name) = if use_cloud {
                let name = model_name.unwrap_or_else(|| "Gemini Pro".to_string());
                (LlmBackend::GoogleGemini, format!("☁️ {} (Cloud)", name))
            } else {
                (LlmBackend::LocalLlama, "🏠 Mistral (Local)".to_string())
            };

            let system_prompt = format!("Tu es GenAptitude. Contexte:\n{}", context_data);

            let response = client
                .ask(backend, &system_prompt, &user_input)
                .await
                .map_err(|e| e.to_string())?;

            Ok(format!("**{}**\n\n{}", display_name, response))
        }
    }
}

fn is_complex_task(input: &str) -> bool {
    let keywords = ["sml", "architecture", "analyse", "complexe", "génère"];
    input
        .to_lowercase()
        .split_whitespace()
        .any(|w| keywords.contains(&w))
}
