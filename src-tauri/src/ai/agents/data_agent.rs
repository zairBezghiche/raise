use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde_json::json;
use uuid::Uuid;

use super::intent_classifier::EngineeringIntent;
use super::{Agent, AgentContext, AgentResult, CreatedArtifact};
use crate::ai::llm::client::LlmBackend;

#[derive(Default)]
pub struct DataAgent;

impl DataAgent {
    pub fn new() -> Self {
        Self {}
    }

    async fn call_llm(
        &self,
        ctx: &AgentContext,
        sys: &str,
        user: &str,
        doc_type: &str,
    ) -> Result<serde_json::Value> {
        let response = ctx
            .llm
            .ask(LlmBackend::LocalLlama, sys, user)
            .await
            .map_err(|e| anyhow!("LLM Err: {}", e))?;

        // DEBUG: On affiche la réponse brute pour comprendre pourquoi ça échoue
        println!("🔍 [DEBUG LLM RAW] : {}", response);

        let clean = self.extract_json(&response);
        println!("🧹 [DEBUG JSON CLEAN] : {}", clean);

        let mut data: serde_json::Value = serde_json::from_str(&clean).unwrap_or_else(|_| {
            println!("⚠️ Parsing JSON échoué. Fallback.");
            json!({ "name": "ErrorFallback", "raw": clean })
        });

        data["id"] = json!(Uuid::new_v4().to_string());
        data["layer"] = json!("DATA");
        data["type"] = json!(doc_type);
        data["createdAt"] = json!(chrono::Utc::now().to_rfc3339());
        Ok(data)
    }

    /// Extrait le JSON en nettoyant le Markdown
    fn extract_json(&self, text: &str) -> String {
        // 1. Nettoyage Markdown basique
        let no_markdown = text
            .replace("```json", "")
            .replace("```", "")
            .trim()
            .to_string();

        // 2. Recherche des bornes { }
        let start_opt = no_markdown.find('{');
        let end_opt = no_markdown.rfind('}');

        if let (Some(start), Some(end)) = (start_opt, end_opt) {
            if end > start {
                if let Some(slice) = no_markdown.get(start..=end) {
                    return slice.to_string();
                }
            }
        }

        // Si pas de bornes trouvées, on renvoie le texte nettoyé du markdown
        no_markdown
    }

    async fn enrich_class(&self, ctx: &AgentContext, name: &str) -> Result<serde_json::Value> {
        let sys = "RÔLE: Data Architect. CONSIGNE: Génère uniquement le JSON valide.";
        let user = format!(
            "Génère la Classe Arcadia '{}' en JSON.\nFormat: {{ \"name\": \"{}\", \"description\": \"...\", \"attributes\": [] }}", 
            name, name
        );
        self.call_llm(ctx, sys, &user, "Class").await
    }

    async fn enrich_datatype(&self, ctx: &AgentContext, name: &str) -> Result<serde_json::Value> {
        let sys = "RÔLE: Data Architect. CONSIGNE: Génère uniquement le JSON valide.";
        let user = format!("Génère le DataType '{}' en JSON (Enum ou Structure).", name);
        self.call_llm(ctx, sys, &user, "DataType").await
    }

    async fn enrich_exchange_item(
        &self,
        ctx: &AgentContext,
        name: &str,
    ) -> Result<serde_json::Value> {
        let sys = "RÔLE: Data Architect. CONSIGNE: Génère uniquement le JSON valide.";
        let user = format!("Génère l'ExchangeItem '{}' en JSON.", name);
        self.call_llm(ctx, sys, &user, "ExchangeItem").await
    }
}

#[async_trait]
impl Agent for DataAgent {
    fn id(&self) -> &'static str {
        "data_architect"
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
            } if layer == "DATA" => {
                let (doc, collection) = match element_type.as_str() {
                    "Class" => (self.enrich_class(ctx, name).await?, "classes"),
                    "DataType" => (self.enrich_datatype(ctx, name).await?, "types"),
                    "ExchangeItem" => (
                        self.enrich_exchange_item(ctx, name).await?,
                        "exchange_items",
                    ),
                    _ => return Err(anyhow!("Type inconnu")),
                };

                let doc_id = doc["id"].as_str().unwrap_or("unknown").to_string();

                // Si le nom est ErrorFallback, c'est que le JSON était pourri
                if doc["name"] == "ErrorFallback" {
                    return Ok(Some(AgentResult::text(format!(
                        "⚠️ Échec IA pour {} : JSON invalide reçu du LLM.",
                        name
                    ))));
                }

                let rel_path = format!("un2/data/collections/{}/{}.json", collection, doc_id);
                let path = ctx.paths.domain_root.join(&rel_path);

                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(&path, serde_json::to_string_pretty(&doc)?)?;

                Ok(Some(AgentResult {
                    message: format!("Élément de donnée **{}** ({}) défini.", name, element_type),
                    artifacts: vec![CreatedArtifact {
                        id: doc_id,
                        name: name.clone(),
                        layer: "DATA".to_string(),
                        element_type: element_type.clone(),
                        path: rel_path,
                    }],
                }))
            }
            _ => Ok(None),
        }
    }
}
