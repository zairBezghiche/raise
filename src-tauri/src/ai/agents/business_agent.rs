use super::intent_classifier::EngineeringIntent;
use super::{Agent, AgentContext, AgentResult, CreatedArtifact}; // <--- Imports
use crate::ai::llm::client::LlmBackend;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde_json::json;
use uuid::Uuid;

#[derive(Default)]
pub struct BusinessAgent;

impl BusinessAgent {
    pub fn new() -> Self {
        Self {}
    }

    async fn analyze_business_need(
        &self,
        ctx: &AgentContext,
        domain: &str,
        description: &str,
    ) -> Result<serde_json::Value> {
        let system_prompt = "Tu es un Business Analyst Senior expert en méthode Arcadia. 
        Ton but est d'analyser un besoin métier et d'en extraire les concepts pour la couche Operational Analysis (OA).
        
        Génère un JSON strict avec cette structure :
        {
            \"capability\": { \"name\": \"Nom de la Capacité\", \"description\": \"Objectif haut niveau\" },
            \"actors\": [
                { \"name\": \"Nom Acteur 1\", \"description\": \"Rôle\" },
                { \"name\": \"Nom Acteur 2\", \"description\": \"Rôle\" }
            ]
        }";

        let user_prompt = format!("Domaine: {}\nBesoin: {}", domain, description);

        let response = ctx
            .llm
            .ask(LlmBackend::LocalLlama, system_prompt, &user_prompt)
            .await
            .map_err(|e| anyhow!("Erreur LLM Business: {}", e))?;

        let clean_json = response
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        serde_json::from_str(clean_json)
            .map_err(|e| anyhow!("Le LLM n'a pas généré de JSON valide. {}", e))
    }
}

#[async_trait]
impl Agent for BusinessAgent {
    fn id(&self) -> &'static str {
        "business_analyst"
    }

    async fn process(
        &self,
        ctx: &AgentContext,
        intent: &EngineeringIntent,
    ) -> Result<Option<AgentResult>> {
        // <--- Signature
        if let EngineeringIntent::DefineBusinessUseCase {
            domain,
            process_name,
            description,
        } = intent
        {
            let analysis = self.analyze_business_need(ctx, domain, description).await?;
            let mut artifacts = Vec::new();

            // 1. CAPACITÉ OPÉRATIONNELLE
            if let Some(cap) = analysis.get("capability") {
                let cap_name = cap["name"].as_str().unwrap_or("UnknownCapability");
                let doc_id = Uuid::new_v4().to_string();
                let doc = json!({
                    "id": doc_id,
                    "name": cap_name,
                    "description": cap["description"],
                    "layer": "OA",
                    "type": "OperationalCapability",
                    "domain": domain,
                    "createdAt": chrono::Utc::now().to_rfc3339()
                });

                let rel_path = format!("un2/oa/collections/capabilities/{}.json", doc_id);
                let path = ctx.paths.domain_root.join(&rel_path);

                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(&path, serde_json::to_string_pretty(&doc)?)?;

                artifacts.push(CreatedArtifact {
                    id: doc_id,
                    name: cap_name.to_string(),
                    layer: "OA".to_string(),
                    element_type: "OperationalCapability".to_string(),
                    path: rel_path,
                });
            }

            // 2. ACTEURS
            if let Some(actors) = analysis["actors"].as_array() {
                for actor in actors {
                    let actor_name = actor["name"].as_str().unwrap_or("UnknownActor");
                    let doc_id = Uuid::new_v4().to_string();
                    let doc = json!({
                        "id": doc_id,
                        "name": actor_name,
                        "description": actor["description"],
                        "layer": "OA",
                        "type": "OperationalActor",
                        "createdAt": chrono::Utc::now().to_rfc3339()
                    });

                    let rel_path = format!("un2/oa/collections/actors/{}.json", doc_id);
                    let path = ctx.paths.domain_root.join(&rel_path);

                    if let Some(parent) = path.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    std::fs::write(&path, serde_json::to_string_pretty(&doc)?)?;

                    artifacts.push(CreatedArtifact {
                        id: doc_id,
                        name: actor_name.to_string(),
                        layer: "OA".to_string(),
                        element_type: "OperationalActor".to_string(),
                        path: rel_path,
                    });
                }
            }

            return Ok(Some(AgentResult {
                message: format!(
                    "Analyse métier **{}** terminée. {} éléments identifiés.",
                    process_name,
                    artifacts.len()
                ),
                artifacts,
            }));
        }

        Ok(None)
    }
}
