use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde_json::json;
use uuid::Uuid;

use super::intent_classifier::EngineeringIntent;
use super::{Agent, AgentContext, AgentResult, CreatedArtifact};
use crate::ai::llm::client::LlmBackend;

#[derive(Default)]
pub struct EpbsAgent;

impl EpbsAgent {
    pub fn new() -> Self {
        Self {}
    }

    fn extract_json(&self, text: &str) -> String {
        let start_opt = text.find('{');
        let end_opt = text.rfind('}');

        if let (Some(start), Some(end)) = (start_opt, end_opt) {
            if end > start {
                if let Some(slice) = text.get(start..=end) {
                    return slice.trim().to_string();
                }
            }
        }
        text.trim().to_string()
    }

    async fn enrich_configuration_item(
        &self,
        ctx: &AgentContext,
        name: &str,
        raw_type: &str,
    ) -> Result<serde_json::Value> {
        let system_prompt = "Tu es un Gestionnaire de Configuration (EPBS). Détermine le 'kind' et le 'partNumber'. Génère du JSON.";
        let user_prompt = format!("Item: {}. Type: {}. Format JSON strict.", name, raw_type);

        let response = ctx
            .llm
            .ask(LlmBackend::LocalLlama, system_prompt, &user_prompt)
            .await
            .map_err(|e| anyhow!("Erreur LLM: {}", e))?;

        let clean_json = self.extract_json(&response);
        println!("🔍 [DEBUG EPBS CLEAN] : {}", clean_json);

        let mut data: serde_json::Value = serde_json::from_str(&clean_json)
            .unwrap_or(json!({ "name": name, "kind": "Hardware" }));

        // CORRECTION CRITIQUE : On force le nom pour qu'il soit identique à la requête
        // Cela empêche le LLM de renommer "Rack Server" en "Server" et de casser le test
        data["name"] = json!(name);

        data["id"] = json!(Uuid::new_v4().to_string());
        data["layer"] = json!("EPBS");
        data["type"] = json!("ConfigurationItem");
        data["createdAt"] = json!(chrono::Utc::now().to_rfc3339());

        if data.get("partNumber").is_none() {
            data["partNumber"] = json!(format!(
                "GEN-{}",
                Uuid::new_v4()
                    .to_string()
                    .chars()
                    .take(8)
                    .collect::<String>()
            ));
        }

        Ok(data)
    }
}

#[async_trait]
impl Agent for EpbsAgent {
    fn id(&self) -> &'static str {
        "configuration_manager"
    }

    async fn process(
        &self,
        ctx: &AgentContext,
        intent: &EngineeringIntent,
    ) -> Result<Option<AgentResult>> {
        match intent {
            EngineeringIntent::CreateElement {
                layer,
                element_type,
                name,
            } if layer == "EPBS" => {
                let doc = self
                    .enrich_configuration_item(ctx, name, element_type)
                    .await?;

                let doc_id = doc["id"].as_str().unwrap_or("unknown").to_string();
                let pn = doc["partNumber"].as_str().unwrap_or("N/A").to_string();

                let collection = "configuration_items";

                let rel_path = format!("un2/epbs/collections/{}/{}.json", collection, doc_id);
                let path = ctx.paths.domain_root.join(&rel_path);

                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(&path, serde_json::to_string_pretty(&doc)?)?;

                Ok(Some(AgentResult {
                    message: format!("Article de configuration **{}** (P/N: {}) créé.", name, pn),
                    artifacts: vec![CreatedArtifact {
                        id: doc_id,
                        name: name.clone(),
                        layer: "EPBS".to_string(),
                        element_type: "ConfigurationItem".to_string(),
                        path: rel_path,
                    }],
                }))
            }
            _ => Ok(None),
        }
    }
}
