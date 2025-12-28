use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde_json::json;
use uuid::Uuid;

use super::intent_classifier::EngineeringIntent;
use super::{Agent, AgentContext, AgentResult, CreatedArtifact};
use crate::ai::llm::client::LlmBackend;
// 1. AJOUT : Import du module NLP que nous avons validé
use crate::ai::nlp::entity_extractor;

#[derive(Default)]
pub struct SystemAgent;

impl SystemAgent {
    pub fn new() -> Self {
        Self
    }

    async fn enrich_sa_element(
        &self,
        ctx: &AgentContext,
        name: &str,
        element_type: &str,
    ) -> Result<serde_json::Value> {
        // 2. INTELLIGENCE : On utilise l'extracteur d'entités sur le nom fourni
        let entities = entity_extractor::extract_entities(name);

        // 3. CONTEXTE : On construit un guide pour le LLM
        let mut nlp_hint = String::new();
        if !entities.is_empty() {
            nlp_hint.push_str("\n[VOCABULAIRE DÉTECTÉ]:\n");
            for entity in entities {
                nlp_hint.push_str(&format!(
                    "- Terme: '{}' (Catégorie: {:?})\n",
                    entity.text, entity.category
                ));
            }
            nlp_hint.push_str("Utilise ces termes précis dans la description technique.\n");
        }

        let system_prompt = "Tu es un Architecte Système expert (Arcadia/Capella).
        Ton but est de définir un élément de l'analyse système (SA).
        Génère uniquement du JSON valide.";

        // 4. PROMPT : On injecte le 'nlp_hint' dans la demande
        let user_prompt = format!(
            "Crée un objet JSON pour un '{}' (SA).
            Nom: {}
            {} 
            Génère une description technique pertinente en Français.
            Format attendu: {{ \"name\": \"str\", \"description\": \"str\" }}",
            element_type, name, nlp_hint
        );

        let response = ctx
            .llm
            .ask(LlmBackend::LocalLlama, system_prompt, &user_prompt)
            .await
            .map_err(|e| anyhow!("Erreur LLM System: {}", e))?;

        let clean_json = response
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        let mut data: serde_json::Value = serde_json::from_str(clean_json)
            .unwrap_or(json!({ "name": name, "description": "Généré par SystemAgent (Fallback)" }));

        data["id"] = json!(Uuid::new_v4().to_string());
        data["layer"] = json!("SA");
        data["type"] = json!(format!("System{}", element_type));
        data["createdAt"] = json!(chrono::Utc::now().to_rfc3339());

        Ok(data)
    }
}

#[async_trait]
impl Agent for SystemAgent {
    fn id(&self) -> &'static str {
        "system_architect"
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
            } if layer == "SA" => {
                let doc = self.enrich_sa_element(ctx, name, element_type).await?;
                let doc_id = doc["id"].as_str().unwrap_or("unknown").to_string();

                let collection = match element_type.to_lowercase().as_str() {
                    "function" | "fonction" => "functions",
                    "actor" | "acteur" => "actors",
                    "component" | "composant" | "system" | "système" => "components",
                    "capability" | "capacité" => "capabilities",
                    _ => "functions",
                };

                let relative_path = format!("un2/sa/collections/{}/{}.json", collection, doc_id);
                let path = ctx.paths.domain_root.join(&relative_path);

                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(&path, serde_json::to_string_pretty(&doc)?)?;

                Ok(Some(AgentResult {
                    message: format!("J'ai défini l'élément **{}** dans l'analyse système.", name),
                    artifacts: vec![CreatedArtifact {
                        id: doc_id,
                        name: name.clone(),
                        layer: "SA".to_string(),
                        element_type: element_type.clone(),
                        path: relative_path,
                    }],
                }))
            }
            EngineeringIntent::CreateRelationship { .. } => Ok(Some(AgentResult::text(
                "⚠️ SystemAgent: Les relations (DataFlow) sont en cours de migration.".to_string(),
            ))),
            _ => Ok(None),
        }
    }
}
