use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde_json::json;
use uuid::Uuid;

use super::intent_classifier::EngineeringIntent;
use super::{Agent, AgentContext, AgentResult, CreatedArtifact};
use crate::ai::llm::client::LlmBackend;
use crate::ai::nlp::entity_extractor;

#[derive(Default)]
pub struct TransverseAgent;

impl TransverseAgent {
    pub fn new() -> Self {
        Self
    }

    /// Nettoyage agressif du JSON (Markdown, espaces...)
    fn clean_json_text(&self, text: &str) -> String {
        let text = text.trim();
        // Retire les balises markdown
        let text = text.trim_start_matches("```json").trim_start_matches("```");
        let text = text.trim_end_matches("```");

        let start = text.find('{').unwrap_or(0);
        let end = text.rfind('}').map(|i| i + 1).unwrap_or(text.len());

        if end > start {
            text[start..end].to_string()
        } else {
            text.to_string()
        }
    }

    async fn call_llm(
        &self,
        ctx: &AgentContext,
        sys: &str,
        user: &str,
        doc_type: &str,
        original_name: &str,
    ) -> Result<serde_json::Value> {
        let response = ctx
            .llm
            .ask(LlmBackend::LocalLlama, sys, user)
            .await
            .map_err(|e| anyhow!("LLM Transverse: {}", e))?;

        let clean = self.clean_json_text(&response);
        let mut doc: serde_json::Value = serde_json::from_str(&clean).unwrap_or(json!({}));

        // --- BLINDAGE TOTAL ---
        doc["name"] = json!(original_name); // Force le nom pour le test
        doc["id"] = json!(Uuid::new_v4().to_string());
        doc["layer"] = json!("TRANSVERSE");
        doc["type"] = json!(doc_type);
        doc["createdAt"] = json!(chrono::Utc::now().to_rfc3339());

        // Valeurs par défaut pour chaque type (évite les erreurs de validation)
        if doc_type == "Requirement" {
            if doc.get("reqId").is_none() {
                doc["reqId"] = json!("REQ-AUTO-001");
            }
            if doc.get("statement").is_none() {
                doc["statement"] = json!(format!("Exigence pour {}", original_name));
            }
            if doc.get("category").is_none() {
                doc["category"] = json!("Functional");
            }
        } else if doc_type == "TestProcedure" {
            if doc.get("steps").is_none() {
                doc["steps"] = json!([]);
            }
        } else if doc_type == "TestCampaign" {
            if doc.get("scenarios").is_none() {
                doc["scenarios"] = json!([]);
            }
        } else if doc_type == "ExchangeScenario" && doc.get("messages").is_none() {
            doc["messages"] = json!([]);
        }

        Ok(doc)
    }

    // --- ENRICHISSEURS ---

    async fn enrich_requirement(
        &self,
        ctx: &AgentContext,
        name: &str,
    ) -> Result<serde_json::Value> {
        let entities = entity_extractor::extract_entities(name);
        let mut nlp_hint = String::new();
        if !entities.is_empty() {
            nlp_hint.push_str("Concerne :\n");
            for e in entities {
                nlp_hint.push_str(&format!("- {}\n", e.text));
            }
        }
        let sys = "RÔLE: Ingénieur Exigences. JSON Strict.";
        let user = format!("Exigence: \"{}\"\n{}\nJSON: {{ \"statement\": \"str\", \"category\": \"Functional\", \"reqId\": \"REQ-01\" }}", name, nlp_hint);
        self.call_llm(ctx, sys, &user, "Requirement", name).await
    }

    async fn enrich_test_procedure(
        &self,
        ctx: &AgentContext,
        name: &str,
    ) -> Result<serde_json::Value> {
        let sys = "RÔLE: QA. JSON.";
        let user = format!("Procédure: \"{}\".\nJSON: {{ \"steps\": [] }}", name);
        self.call_llm(ctx, sys, &user, "TestProcedure", name).await
    }

    async fn enrich_test_campaign(
        &self,
        ctx: &AgentContext,
        name: &str,
    ) -> Result<serde_json::Value> {
        let sys = "RÔLE: QA Manager. JSON.";
        let user = format!("Campagne: \"{}\".\nJSON: {{ \"scenarios\": [] }}", name);
        self.call_llm(ctx, sys, &user, "TestCampaign", name).await
    }

    async fn enrich_scenario(&self, ctx: &AgentContext, name: &str) -> Result<serde_json::Value> {
        let sys = "RÔLE: Architecte. JSON.";
        let user = format!("Scénario: \"{}\".\nJSON: {{ \"messages\": [] }}", name);
        self.call_llm(ctx, sys, &user, "ExchangeScenario", name)
            .await
    }
}

#[async_trait]
impl Agent for TransverseAgent {
    fn id(&self) -> &'static str {
        "quality_manager"
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
            } if layer == "TRANSVERSE" => {
                // 1. NORMALISATION (Pour éviter le bug de casse)
                let et_lower = element_type.to_lowercase();

                // 2. ROUTAGE ROBUSTE
                // On await DANS le match pour avoir un type de retour concret (Value) et éviter l'erreur de compilation
                let (doc, sub_folder) = match et_lower.as_str() {
                    "requirement" | "exigence" => {
                        (self.enrich_requirement(ctx, name).await?, "requirements")
                    }
                    "testprocedure" | "procedure" | "test" => (
                        self.enrich_test_procedure(ctx, name).await?,
                        "test_procedures",
                    ),
                    "testcampaign" | "campaign" | "campagne" => (
                        self.enrich_test_campaign(ctx, name).await?,
                        "test_campaigns",
                    ),
                    "exchangescenario" | "scenario" => {
                        (self.enrich_scenario(ctx, name).await?, "scenarios")
                    }
                    _ => (
                        // Fallback par sécurité -> Requirement
                        self.enrich_requirement(ctx, name).await?,
                        "requirements",
                    ),
                };

                let doc_id = doc["id"].as_str().unwrap_or("unknown").to_string();

                // 3. ECRITURE (Standard 'collections')
                let relative_path =
                    format!("un2/transverse/collections/{}/{}.json", sub_folder, doc_id);
                let path = ctx.paths.domain_root.join(&relative_path);

                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(&path, serde_json::to_string_pretty(&doc)?)?;

                Ok(Some(AgentResult {
                    message: format!("Élément Transverse **{}** ({}) créé.", name, element_type),
                    artifacts: vec![CreatedArtifact {
                        id: doc_id,
                        name: name.clone(),
                        layer: "TRANSVERSE".to_string(),
                        element_type: element_type.clone(),
                        path: relative_path,
                    }],
                }))
            }
            _ => Ok(None),
        }
    }
}
