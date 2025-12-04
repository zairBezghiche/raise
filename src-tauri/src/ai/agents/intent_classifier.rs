use crate::ai::llm::client::{LlmBackend, LlmClient};
use serde::{Deserialize, Serialize};

// 1. Définition des intentions possibles
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "intent", content = "params")]
pub enum EngineeringIntent {
    #[serde(rename = "create_element")]
    CreateElement {
        layer: String,
        element_type: String,
        name: String,
    },
    #[serde(rename = "chat")]
    Chat,
    #[serde(rename = "unknown")]
    Unknown,
}

pub struct IntentClassifier {
    llm: LlmClient,
}

impl IntentClassifier {
    pub fn new(llm: LlmClient) -> Self {
        Self { llm }
    }

    pub async fn classify(&self, user_input: &str) -> EngineeringIntent {
        let system_prompt = r#"
        Tu es le moteur de classification de GenAptitude.
        Ta seule tâche est d'analyser la demande de l'utilisateur et de retourner un objet JSON strict.
        
        FORMATS ACCEPTÉS :
        1. Pour créer un élément : { "intent": "create_element", "params": { "layer": "OA"|"SA"|"LA"|"PA", "element_type": "Actor"|"Function"|"Component", "name": "Nom" } }
        2. Pour discuter : { "intent": "chat", "params": null }

        RÈGLES :
        - UNIQUEMENT le JSON. Pas de phrase.
        - Déduis la couche si absente.
        "#;

        match self
            .llm
            .ask(LlmBackend::LocalLlama, system_prompt, user_input)
            .await
        {
            Ok(raw_response) => {
                let json_str = extract_json(&raw_response);

                // CORRECTION : Nettoyage des échappements Markdown parasites (ex: create\_element -> create_element)
                let clean_json = json_str.replace(r"\_", "_");

                println!("🔍 Intent JSON nettoyé: {}", clean_json);

                match serde_json::from_str::<EngineeringIntent>(&clean_json) {
                    Ok(intent) => intent,
                    Err(e) => {
                        println!("⚠️ Erreur parsing intent: {}", e);
                        // On affiche le JSON fautif pour debug
                        println!("   Contenu reçu: {}", clean_json);
                        EngineeringIntent::Chat
                    }
                }
            }
            Err(_) => EngineeringIntent::Unknown,
        }
    }
}

/// Helper pour extraire le JSON d'un bloc Markdown éventuel
fn extract_json(text: &str) -> String {
    let start_tag = "```json";
    let end_tag = "```";

    if let Some(start) = text.find(start_tag) {
        if let Some(end_offset) = text[start + start_tag.len()..].find(end_tag) {
            let start_content = start + start_tag.len();
            let end_content = start_content + end_offset;
            return text[start_content..end_content].trim().to_string();
        }
    }
    // Si pas de balises, on suppose que tout le texte est du JSON
    text.trim().to_string()
}
