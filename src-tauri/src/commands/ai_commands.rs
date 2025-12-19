use crate::ai::agents::intent_classifier::{EngineeringIntent, IntentClassifier};
use crate::ai::agents::{
    business_agent::BusinessAgent,
    data_agent::DataAgent,
    epbs_agent::EpbsAgent,
    hardware_agent::HardwareAgent,
    software_agent::SoftwareAgent,
    system_agent::SystemAgent,
    transverse_agent::TransverseAgent,
    Agent,
    AgentContext,
    AgentResult, // Import AgentResult
};

use crate::ai::context::retriever::SimpleRetriever;
use crate::ai::llm::client::{LlmBackend, LlmClient};
use crate::json_db::storage::StorageEngine;
use crate::model_engine::loader::ModelLoader;

use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{command, State};

/// Commande principale : Retourne un AgentResult structuré
#[command]
pub async fn ai_chat(
    storage: State<'_, StorageEngine>,
    user_input: String,
) -> Result<AgentResult, String> {
    // <--- Retourne AgentResult au lieu de String

    // ... (Configuration : identique) ...
    let mode_dual =
        env::var("GENAPTITUDE_MODE_DUAL").unwrap_or_else(|_| "false".to_string()) == "true";
    let gemini_key = env::var("GENAPTITUDE_GEMINI_KEY").unwrap_or_default();
    let model_name = env::var("GENAPTITUDE_MODEL_NAME").ok();
    let local_url =
        env::var("GENAPTITUDE_LOCAL_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let domain_path = env::var("PATH_GENAPTITUDE_DOMAIN")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap().join("data"));
    let dataset_path = env::var("PATH_GENAPTITUDE_DATASET")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap().join("dataset"));
    let client = LlmClient::new(&local_url, &gemini_key, model_name.clone());

    let ctx = AgentContext::new(
        Arc::new(storage.inner().clone()),
        client.clone(),
        domain_path,
        dataset_path,
    );

    let classifier = IntentClassifier::new(client.clone());
    let intent = classifier.classify(&user_input).await;

    // ... (Routage) ...
    let result = match intent {
        EngineeringIntent::DefineBusinessUseCase { .. } => {
            let agent = BusinessAgent::new();
            agent.process(&ctx, &intent).await
        }
        EngineeringIntent::CreateElement { ref layer, .. } if layer == "SA" => {
            let agent = SystemAgent::new();
            agent.process(&ctx, &intent).await
        }
        EngineeringIntent::CreateElement {
            ref layer,
            ref element_type,
            ..
        } if layer == "LA" || element_type.to_lowercase().contains("software") => {
            let agent = SoftwareAgent::new();
            agent.process(&ctx, &intent).await
        }
        EngineeringIntent::CreateElement { ref layer, .. } if layer == "PA" => {
            let agent = HardwareAgent::new();
            agent.process(&ctx, &intent).await
        }
        EngineeringIntent::CreateElement { ref layer, .. } if layer == "EPBS" => {
            let agent = EpbsAgent::new();
            agent.process(&ctx, &intent).await
        }
        EngineeringIntent::CreateElement { ref layer, .. } if layer == "DATA" => {
            let agent = DataAgent::new();
            agent.process(&ctx, &intent).await
        }
        EngineeringIntent::CreateElement { ref layer, .. } if layer == "TRANSVERSE" => {
            let agent = TransverseAgent::new();
            agent.process(&ctx, &intent).await
        }
        EngineeringIntent::GenerateCode { .. } => {
            let agent = SoftwareAgent::new();
            agent.process(&ctx, &intent).await
        }

        // Mode CHAT (RAG)
        EngineeringIntent::Unknown | EngineeringIntent::Chat => {
            // ... (Logique RAG existante, simplifiée pour l'exemple) ...
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
            let use_cloud = mode_dual && !gemini_key.is_empty();
            let (backend, _) = if use_cloud {
                (LlmBackend::GoogleGemini, "Gemini")
            } else {
                (LlmBackend::LocalLlama, "Local")
            };

            let system_prompt = format!("Tu es GenAptitude. Contexte:\n{}", context_data);
            let response = client
                .ask(backend, &system_prompt, &user_input)
                .await
                .map_err(|e| e.to_string())?;

            // On renvoie un AgentResult "Text Only"
            Ok(Some(AgentResult::text(response)))
        }

        _ => Ok(Some(AgentResult::text("Commande non gérée.".to_string()))),
    };

    // Gestion finale des erreurs
    match result {
        Ok(Some(res)) => Ok(res),
        Ok(None) => Ok(AgentResult::text("Aucune action effectuée.".to_string())),
        Err(e) => Err(e.to_string()),
    }
}
