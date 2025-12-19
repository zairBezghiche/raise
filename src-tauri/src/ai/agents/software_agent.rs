use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde_json::json;
use uuid::Uuid;

use super::intent_classifier::EngineeringIntent;
use super::{Agent, AgentContext, AgentResult, CreatedArtifact}; // <--- Imports
use crate::ai::llm::client::LlmBackend;

#[derive(Default)]
pub struct SoftwareAgent;

impl SoftwareAgent {
    pub fn new() -> Self {
        Self {}
    }

    async fn ask_llm(&self, ctx: &AgentContext, system: &str, user: &str) -> Result<String> {
        ctx.llm
            .ask(LlmBackend::LocalLlama, system, user)
            .await
            .map_err(|e| anyhow!("Erreur LLM : {}", e))
    }

    async fn enrich_logical_component(
        &self,
        ctx: &AgentContext,
        name: &str,
        description: &str,
    ) -> Result<serde_json::Value> {
        let system_prompt = "Tu es un Architecte Logiciel. Génère uniquement du JSON valide.";
        let user_prompt = format!(
            "Crée un objet JSON pour un Composant Logique Arcadia (LA).
            Nom: {}
            Intention: {}
            Schéma: {{ \"name\": \"str\", \"is_abstract\": bool, \"implementation_language\": \"rust\" }}",
            name, description
        );

        let response = self.ask_llm(ctx, system_prompt, &user_prompt).await?;

        let clean_json = response
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        let mut data: serde_json::Value = serde_json::from_str(clean_json)
            .unwrap_or(json!({ "name": name, "description": description }));

        data["id"] = json!(Uuid::new_v4().to_string());
        data["layer"] = json!("LA");
        data["createdAt"] = json!(chrono::Utc::now().to_rfc3339());

        Ok(data)
    }
}

#[async_trait]
impl Agent for SoftwareAgent {
    fn id(&self) -> &'static str {
        "software_engineer"
    }

    async fn process(
        &self,
        ctx: &AgentContext,
        intent: &EngineeringIntent,
    ) -> Result<Option<AgentResult>> {
        // <--- Signature
        match intent {
            EngineeringIntent::CreateElement {
                layer: _,
                element_type,
                name,
            } => {
                let doc = self
                    .enrich_logical_component(ctx, name, &format!("Type: {}", element_type))
                    .await?;
                let doc_id = doc["id"].as_str().unwrap_or("unknown").to_string();

                let relative_path = format!("un2/la/collections/components/{}.json", doc_id);
                let path = ctx.paths.domain_root.join(&relative_path);

                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(&path, serde_json::to_string_pretty(&doc)?)?;

                Ok(Some(AgentResult {
                    message: format!("Composant logiciel **{}** modélisé.", name),
                    artifacts: vec![CreatedArtifact {
                        id: doc_id,
                        name: name.clone(),
                        layer: "LA".to_string(),
                        element_type: "Component".to_string(),
                        path: relative_path,
                    }],
                }))
            }

            EngineeringIntent::GenerateCode {
                language,
                context,
                filename,
            } => {
                let user = format!("Code pour: {}\nLangage: {}", context, language);
                let code = self
                    .ask_llm(ctx, "Expert Code. Pas de markdown.", &user)
                    .await?;

                let clean_code = code.replace("```rust", "").replace("```", "");

                let relative_path = format!("src-gen/{}", filename);
                let path = ctx.paths.domain_root.join(&relative_path);

                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(&path, clean_code.trim())?;

                Ok(Some(AgentResult {
                    message: format!("Code source généré dans **{}**.", filename),
                    artifacts: vec![CreatedArtifact {
                        id: filename.clone(),
                        name: filename.clone(),
                        layer: "CODE".to_string(),
                        element_type: "SourceFile".to_string(),
                        path: relative_path,
                    }],
                }))
            }

            _ => Ok(None),
        }
    }
}
