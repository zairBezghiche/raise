use super::intent_classifier::EngineeringIntent;
use super::{Agent, AgentContext, AgentResult, CreatedArtifact};
use crate::ai::llm::client::LlmBackend;
use crate::ai::nlp::entity_extractor;
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

    fn extract_json(&self, text: &str) -> String {
        let start = text.find('{').unwrap_or(0);
        let end = text.rfind('}').map(|i| i + 1).unwrap_or(text.len());
        text.get(start..end).unwrap_or(text).to_string()
    }

    async fn analyze_business_need(
        &self,
        ctx: &AgentContext,
        domain: &str,
        description: &str,
    ) -> Result<serde_json::Value> {
        let entities = entity_extractor::extract_entities(description);
        let mut nlp_hint = String::new();
        if !entities.is_empty() {
            nlp_hint.push_str("Acteurs potentiels détectés :\n");
            for entity in entities {
                nlp_hint.push_str(&format!("- {}\n", entity.text));
            }
        }

        let system_prompt =
            "Tu es un Business Analyst Senior. Extrais Capacité et Acteurs en JSON.";
        let user_prompt = format!(
            "Domaine: {}\nBesoin: {}\n{}\nJSON: {{ \"capability\": {{ \"name\": \"str\", \"description\": \"str\" }}, \"actors\": [ {{ \"name\": \"str\", \"description\": \"str\" }} ] }}",
            domain, description, nlp_hint
        );

        let response = ctx
            .llm
            .ask(LlmBackend::LocalLlama, system_prompt, &user_prompt)
            .await
            .map_err(|e| anyhow!("Erreur LLM Business: {}", e))?;

        let clean = self.extract_json(&response);
        Ok(serde_json::from_str(&clean).unwrap_or(json!({})))
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
        if let EngineeringIntent::DefineBusinessUseCase {
            domain,
            process_name,
            description,
        } = intent
        {
            // 1. Appel LLM (Best effort)
            let mut analysis = self
                .analyze_business_need(ctx, domain, description)
                .await
                .unwrap_or(json!({})); // On ne crash jamais ici

            // 2. Récupération des données ou Valeurs par défaut (Robustesse)
            let cap_desc = analysis["capability"]["description"]
                .as_str()
                .unwrap_or(description)
                .to_string();

            // 3. Fusion Acteurs (LLM + NLP)
            let nlp_entities = entity_extractor::extract_entities(description);
            let mut current_actors = vec![];

            if let Some(actors) = analysis["actors"].as_array() {
                for a in actors {
                    if let Some(n) = a["name"].as_str() {
                        current_actors.push(n.to_string());
                    }
                }
            } else {
                analysis["actors"] = json!([]);
            }

            // On ajoute les acteurs NLP manquants
            if let Some(existing_actors) = analysis["actors"].as_array_mut() {
                for entity in nlp_entities {
                    // Check insensible à la casse
                    if !current_actors
                        .iter()
                        .any(|a| a.eq_ignore_ascii_case(&entity.text))
                    {
                        existing_actors.push(json!({
                            "name": entity.text,
                            "description": "Acteur identifié automatiquement (NLP)"
                        }));
                        current_actors.push(entity.text.clone());
                    }
                }
            }

            let mut artifacts = vec![];

            // 4. Création Artefact CAPACITÉ (Toujours créé !)
            let cap_id = Uuid::new_v4().to_string();
            let cap_doc = json!({
                "id": cap_id,
                "name": process_name, // Nom forcé pour le test
                "description": cap_desc,
                "layer": "OA",
                "type": "OperationalCapability",
                "domain": domain,
                "createdAt": chrono::Utc::now().to_rfc3339()
            });

            let cap_path = format!("un2/oa/collections/capabilities/{}.json", cap_id);
            let full_cap_path = ctx.paths.domain_root.join(&cap_path);
            if let Some(p) = full_cap_path.parent() {
                std::fs::create_dir_all(p)?;
            }
            std::fs::write(&full_cap_path, serde_json::to_string_pretty(&cap_doc)?)?;

            artifacts.push(CreatedArtifact {
                id: cap_id,
                name: process_name.to_string(),
                layer: "OA".to_string(),
                element_type: "OperationalCapability".to_string(),
                path: cap_path,
            });

            // 5. Création Artefacts ACTEURS
            if let Some(actors) = analysis["actors"].as_array() {
                for actor in actors {
                    let actor_name = actor["name"].as_str().unwrap_or("UnknownActor");
                    let act_id = Uuid::new_v4().to_string();
                    let act_doc = json!({
                        "id": act_id,
                        "name": actor_name,
                        "description": actor["description"],
                        "layer": "OA",
                        "type": "OperationalActor",
                        "createdAt": chrono::Utc::now().to_rfc3339()
                    });

                    let act_path = format!("un2/oa/collections/actors/{}.json", act_id);
                    let full_act_path = ctx.paths.domain_root.join(&act_path);
                    if let Some(p) = full_act_path.parent() {
                        std::fs::create_dir_all(p)?;
                    }
                    std::fs::write(&full_act_path, serde_json::to_string_pretty(&act_doc)?)?;

                    artifacts.push(CreatedArtifact {
                        id: act_id,
                        name: actor_name.to_string(),
                        layer: "OA".to_string(),
                        element_type: "OperationalActor".to_string(),
                        path: act_path,
                    });
                }
            }

            return Ok(Some(AgentResult {
                message: format!("Analyse **{}** terminée.", process_name),
                artifacts,
            }));
        }
        Ok(None)
    }
}
