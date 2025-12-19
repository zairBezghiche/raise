use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde_json::json;
use uuid::Uuid;
// Suppression de 'use std::fmt;' qui était unused

use super::intent_classifier::EngineeringIntent;
use super::{Agent, AgentContext, AgentResult, CreatedArtifact};
use crate::ai::llm::client::LlmBackend;

#[derive(Default)]
pub struct TransverseAgent;

impl TransverseAgent {
    pub fn new() -> Self {
        Self
    }

    /// Génère une Exigence (Requirement)
    async fn enrich_requirement(
        &self,
        ctx: &AgentContext,
        name: &str,
    ) -> Result<serde_json::Value> {
        let system_prompt = "RÔLE: Ingénieur Exigences (Arcadia).
        CONSIGNE: Génère un objet JSON pour une exigence système. PAS DE CODE.
        Structure: reqId (ex: REQ-SYS-001), statement (phrase 'Le système DOIT...'), rationale, category.";

        let user_prompt = format!(
            r#"
        Exigence: "{}"
        JSON ATTENDU:
        {{
            "name": "{}",
            "reqId": "REQ-SYS-XXX",
            "statement": "Le système doit...",
            "rationale": "Pourquoi cette exigence ?",
            "category": "Functional"
        }}
        "#,
            name, name
        );

        self.call_llm(
            ctx,
            system_prompt,
            &user_prompt,
            "Requirement",
            "requirements",
        )
        .await
    }

    /// Génère une Procédure de Test (TestProcedure)
    async fn enrich_test_procedure(
        &self,
        ctx: &AgentContext,
        name: &str,
    ) -> Result<serde_json::Value> {
        let system_prompt = "RÔLE: Ingénieur QA / Testeur (IVVQ).
        CONSIGNE: Génère une procédure de test structurée JSON. PAS DE CODE.
        Défini une liste d'étapes (steps) avec actions et résultats attendus.";

        let user_prompt = format!(
            r#"
        Procédure de Test: "{}"
        JSON ATTENDU:
        {{
            "name": "{}",
            "testType": "Validation",
            "steps": [
                {{ "order": 1, "action": "Démarrer le système", "expectedResult": "Led verte allumée" }},
                {{ "order": 2, "action": "...", "expectedResult": "..." }}
            ]
        }}
        "#,
            name, name
        );

        self.call_llm(
            ctx,
            system_prompt,
            &user_prompt,
            "TestProcedure",
            "tests/procedures",
        )
        .await
    }

    /// Génère un Scénario (Scenario)
    async fn enrich_scenario(&self, ctx: &AgentContext, name: &str) -> Result<serde_json::Value> {
        let system_prompt = "RÔLE: Architecte Système.
        CONSIGNE: Décris un scénario d'interaction (séquence) JSON. PAS DE CODE.
        Invente des messages simples entre Source et Target.";

        let user_prompt = format!(
            r#"
        Scénario: "{}"
        JSON ATTENDU:
        {{
            "name": "{}",
            "kind": "Functional",
            "messages": [
                {{ "sequenceOrder": 1, "name": "Requête", "source": "User", "target": "System" }},
                {{ "sequenceOrder": 2, "name": "Réponse", "source": "System", "target": "User" }}
            ]
        }}
        "#,
            name, name
        );

        self.call_llm(
            ctx,
            system_prompt,
            &user_prompt,
            "ExchangeScenario",
            "scenarios",
        )
        .await
    }

    /// Génère une Campagne de Test
    async fn enrich_test_campaign(
        &self,
        _ctx: &AgentContext,
        name: &str,
    ) -> Result<serde_json::Value> {
        let doc = json!({
            "name": name,
            "targetVersion": "1.0.0",
            "status": "Planned"
        });
        self.finalize_doc(doc, "TestCampaign", "tests/campaigns")
    }

    /// Helper générique LLM + Parsing Robuste
    async fn call_llm(
        &self,
        ctx: &AgentContext,
        sys: &str,
        user: &str,
        doc_type: &str,
        sub_folder: &str,
    ) -> Result<serde_json::Value> {
        let response = ctx
            .llm
            .ask(LlmBackend::LocalLlama, sys, user)
            .await
            .map_err(|e| anyhow!("Erreur LLM Transverse: {}", e))?;

        let clean_json = self.extract_json(&response);
        let doc: serde_json::Value = serde_json::from_str(&clean_json)
            .unwrap_or(json!({ "name": "ErrorFallback", "description": "Parsing Failed" }));

        self.finalize_doc(doc, doc_type, sub_folder)
    }

    /// Finalise le document avec les métadonnées techniques
    fn finalize_doc(
        &self,
        mut doc: serde_json::Value,
        doc_type: &str,
        sub_folder: &str,
    ) -> Result<serde_json::Value> {
        doc["id"] = json!(Uuid::new_v4().to_string());
        doc["layer"] = json!("TRANSVERSE");
        doc["type"] = json!(doc_type);
        doc["createdAt"] = json!(chrono::Utc::now().to_rfc3339());

        doc["_storage_folder"] = json!(sub_folder);

        Ok(doc)
    }

    /// Parsing robuste qui ne plante pas
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
}

#[async_trait]
impl Agent for TransverseAgent {
    fn id(&self) -> &'static str {
        "transverse_manager"
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
                let doc_result = match element_type.as_str() {
                    "Requirement" => self.enrich_requirement(ctx, name).await,
                    "TestProcedure" => self.enrich_test_procedure(ctx, name).await,
                    "ExchangeScenario" | "Scenario" => self.enrich_scenario(ctx, name).await,
                    "TestCampaign" => self.enrich_test_campaign(ctx, name).await,
                    _ => self.enrich_requirement(ctx, name).await,
                };

                let mut doc = doc_result?;

                let doc_id = doc["id"].as_str().unwrap_or("unknown").to_string();

                let sub_folder = doc["_storage_folder"]
                    .as_str()
                    .unwrap_or("misc")
                    .to_string();

                if let Some(obj) = doc.as_object_mut() {
                    obj.remove("_storage_folder");
                }

                let path = ctx
                    .paths
                    .domain_root
                    .join("un2")
                    .join("transverse")
                    .join(&sub_folder) // <--- CORRECTION 1 : Emprunt (&) pour ne pas move
                    .join(format!("{}.json", doc_id));

                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(&path, serde_json::to_string_pretty(&doc)?)?;

                Ok(Some(AgentResult {
                    message: format!(
                        "✅ [Transverse] {} créé : {} (ID: {})",
                        element_type, name, doc_id
                    ),
                    artifacts: vec![CreatedArtifact {
                        id: doc_id.clone(), // <--- CORRECTION 2 : Clone ici
                        name: name.clone(),
                        layer: "TRANSVERSE".to_string(),
                        element_type: element_type.clone(),
                        // sub_folder et doc_id sont maintenant utilisables ici
                        path: format!("un2/transverse/{}/{}.json", sub_folder, doc_id),
                    }],
                }))
            }
            _ => Ok(None),
        }
    }
}
