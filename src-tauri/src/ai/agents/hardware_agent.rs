use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde_json::json;
use uuid::Uuid;

use super::intent_classifier::EngineeringIntent;
use super::{Agent, AgentContext, AgentResult, CreatedArtifact}; // <--- Imports
use crate::ai::llm::client::LlmBackend;

#[derive(Default)]
pub struct HardwareAgent;

impl HardwareAgent {
    pub fn new() -> Self {
        Self {}
    }

    fn determine_category(&self, name: &str, element_type: &str) -> &'static str {
        let keywords = format!("{} {}", name, element_type).to_lowercase();
        if keywords.contains("fpga")
            || keywords.contains("asic")
            || keywords.contains("pcb")
            || keywords.contains("carte")
            || keywords.contains("soc")
        {
            "Electronics"
        } else {
            "Infrastructure"
        }
    }

    async fn enrich_physical_node(
        &self,
        ctx: &AgentContext,
        name: &str,
        element_type: &str,
    ) -> Result<serde_json::Value> {
        let category = self.determine_category(name, element_type);
        let specific_instruction = if category == "Electronics" {
            "Contexte: Design Électronique/Hardware. Mentionne: Logic Cells, I/O, Consommation."
        } else {
            "Contexte: Infrastructure IT. Mentionne: CPU, RAM, Storage, OS."
        };

        let system_prompt = "Tu es un Architecte Matériel (Arcadia PA). Génère JSON.";
        let user_prompt = format!(
            "Crée un objet JSON pour un Noeud Physique (PA). Nom: {}. Type: {}. {}. Format: {{ \"name\": \"str\", \"specs\": {{}} }}",
            name, element_type, specific_instruction
        );

        let response = ctx
            .llm
            .ask(LlmBackend::LocalLlama, system_prompt, &user_prompt)
            .await
            .map_err(|e| anyhow!("Erreur LLM Hardware: {}", e))?;

        let clean_json = response
            .trim()
            .trim_start_matches("```json")
            .trim_end_matches("```")
            .trim();
        let mut data: serde_json::Value =
            serde_json::from_str(clean_json).unwrap_or(json!({ "name": name }));

        data["id"] = json!(Uuid::new_v4().to_string());
        data["layer"] = json!("PA");
        data["type"] = json!("PhysicalNode");
        data["nature"] = json!(category);
        data["createdAt"] = json!(chrono::Utc::now().to_rfc3339());

        Ok(data)
    }
}

#[async_trait]
impl Agent for HardwareAgent {
    fn id(&self) -> &'static str {
        "hardware_architect"
    }

    async fn process(
        &self,
        ctx: &AgentContext,
        intent: &EngineeringIntent,
    ) -> Result<Option<AgentResult>> {
        // <--- Signature
        match intent {
            EngineeringIntent::CreateElement {
                layer,
                element_type,
                name,
            } if layer == "PA" => {
                let doc = self.enrich_physical_node(ctx, name, element_type).await?;
                let doc_id = doc["id"].as_str().unwrap_or("unknown").to_string();
                let nature = doc["nature"].as_str().unwrap_or("Hardware").to_string();

                let rel_path = format!("un2/pa/collections/physical_nodes/{}.json", doc_id);
                let path = ctx.paths.domain_root.join(&rel_path);

                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(&path, serde_json::to_string_pretty(&doc)?)?;

                Ok(Some(AgentResult {
                    message: format!("Noeud physique **{}** ({}) provisionné.", name, nature),
                    artifacts: vec![CreatedArtifact {
                        id: doc_id,
                        name: name.clone(),
                        layer: "PA".to_string(),
                        element_type: "PhysicalNode".to_string(),
                        path: rel_path,
                    }],
                }))
            }
            _ => Ok(None),
        }
    }
}
