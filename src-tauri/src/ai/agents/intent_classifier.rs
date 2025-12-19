use crate::ai::llm::client::{LlmBackend, LlmClient};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "intent")]
pub enum EngineeringIntent {
    #[serde(rename = "define_business_use_case")]
    DefineBusinessUseCase {
        domain: String,
        process_name: String,
        description: String,
    },
    #[serde(rename = "create_element")]
    CreateElement {
        layer: String,
        element_type: String,
        name: String,
    },
    #[serde(rename = "create_relationship")]
    CreateRelationship {
        source_name: String,
        target_name: String,
        relation_type: String,
    },
    #[serde(rename = "generate_code")]
    GenerateCode {
        language: String,
        #[serde(alias = "content", alias = "code", default)]
        context: String,
        filename: String,
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
        let system_prompt = "Tu es le Dispatcher IA de GenAptitude.
        Ton rôle est de classifier l'intention de l'utilisateur en JSON STRICT.

        RÈGLES DE ROUTAGE :
        - 'Classe', 'Donnée', 'Enum' -> layer: DATA, element_type: Class/DataType
        - 'Composant', 'Service' -> layer: LA, element_type: Component
        - 'Fonction' -> layer: SA, element_type: Function
        - 'Serveur', 'Calculateur', 'Hardware' -> layer: PA, element_type: PhysicalNode
        - 'Exigence', 'Test' -> layer: TRANSVERSE

        EXEMPLES :
        - 'Défini la classe Client' -> { \"intent\": \"create_element\", \"layer\": \"DATA\", \"element_type\": \"Class\", \"name\": \"Client\" }
        - 'Ajoute un serveur SQL' -> { \"intent\": \"create_element\", \"layer\": \"PA\", \"element_type\": \"PhysicalNode\", \"name\": \"SQL Server\" }
        
        JSON UNIQUEMENT. PAS DE MARKDOWN.";

        let response = self
            .llm
            .ask(LlmBackend::LocalLlama, system_prompt, user_input)
            .await
            .unwrap_or_else(|_| "{}".to_string());

        let clean_json = extract_json(&response);

        match serde_json::from_str::<EngineeringIntent>(&clean_json) {
            Ok(mut intent) => {
                // --- CORRECTION ET FORÇAGE DES COUCHES ---
                if let EngineeringIntent::CreateElement {
                    ref mut layer,
                    ref element_type,
                    ..
                } = intent
                {
                    // 1. Priorité absolue aux mots-clés techniques
                    let et_lower = element_type.to_lowercase();

                    if et_lower.contains("class")
                        || et_lower.contains("datatype")
                        || et_lower.contains("enum")
                    {
                        *layer = "DATA".to_string();
                    } else if et_lower.contains("requirement") || et_lower.contains("exigence") {
                        *layer = "TRANSVERSE".to_string();
                    }

                    // 2. Fallback si le layer est vide ou incorrect
                    if layer.is_empty() || layer == "Unknown" {
                        *layer = match et_lower.as_str() {
                            "actor" | "acteur" | "operationalactor" => "OA".to_string(),
                            "function" | "fonction" | "systemfunction" => "SA".to_string(),
                            "component" | "composant" | "logicalcomponent" => "LA".to_string(),
                            _ => "SA".to_string(),
                        };
                    }
                }
                intent
            }
            Err(e) => {
                println!("⚠️ Erreur parsing JSON Intent: {}", e);
                println!("   Input LLM: {}", clean_json);
                EngineeringIntent::Unknown
            }
        }
    }
}

fn extract_json(text: &str) -> String {
    let start_index = match text.find('{') {
        Some(i) => i,
        None => return text.to_string(),
    };
    let end_index = match text.rfind('}') {
        Some(i) => i,
        None => return text[start_index..].to_string(),
    };
    if end_index > start_index {
        return text[start_index..=end_index].trim().to_string();
    }
    text.trim().to_string()
}
