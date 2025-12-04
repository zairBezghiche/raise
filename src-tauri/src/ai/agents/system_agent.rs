use crate::ai::agents::{Agent, EngineeringIntent};
use crate::ai::llm::client::{LlmBackend, LlmClient};
use crate::json_db::collections::manager::CollectionsManager;
use crate::json_db::storage::StorageEngine;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde_json::json;

pub struct SystemAgent {
    llm: LlmClient,
    storage: StorageEngine,
    space: String,
    db: String,
}

impl SystemAgent {
    pub fn new(llm: LlmClient, storage: StorageEngine) -> Self {
        Self {
            llm,
            storage,
            space: "un2".to_string(),
            db: "_system".to_string(),
        }
    }

    async fn generate_description(&self, element_type: &str, name: &str) -> String {
        let prompt = format!(
            "Génère une description technique courte (1 phrase) pour un '{}' nommé '{}' dans un contexte d'ingénierie système. \
            Réponds impérativement en Français. Sois factuel.",
            element_type, name
        );

        self.llm
            .ask(
                LlmBackend::LocalLlama,
                "Tu es un assistant ingénieur.",
                &prompt,
            )
            .await
            .unwrap_or_else(|_| "Description générée par IA.".to_string())
    }
}

#[async_trait]
impl Agent for SystemAgent {
    async fn process(&self, intent: &EngineeringIntent) -> Result<Option<String>> {
        match intent {
            EngineeringIntent::CreateElement {
                layer,
                element_type,
                name,
            } => {
                if layer != "OA" && layer != "SA" {
                    return Ok(None);
                }

                println!("🤖 SystemAgent: Création de {} {}...", layer, name);

                // CORRECTION DES CHEMINS DE SCHÉMAS
                // Ils doivent correspondre à l'arborescence physique dans schemas/v1/
                let (collection, type_uri, schema_rel) =
                    match (layer.as_str(), element_type.as_str()) {
                        ("OA", "Actor") => (
                            "actors",
                            "oa:OperationalActor",
                            "arcadia/oa/actor.schema.json", // Chemin corrigé
                        ),
                        ("OA", "Activity") => (
                            "activities",
                            "oa:OperationalActivity",
                            "arcadia/oa/activity.schema.json", // Chemin corrigé
                        ),
                        ("SA", "Function") => (
                            "functions",
                            "sa:SystemFunction",
                            "arcadia/sa/system-function.schema.json", // Chemin corrigé
                        ),
                        ("SA", "Component") => (
                            "components",
                            "sa:SystemComponent",
                            "arcadia/sa/system-component.schema.json", // Chemin corrigé
                        ),
                        _ => return Err(anyhow!("Type non supporté: {}/{}", layer, element_type)),
                    };

                // URI Absolue pour le registre (utilisée dans _meta.json et _system.json)
                let schema_uri =
                    format!("db://{}/{}/schemas/v1/{}", self.space, self.db, schema_rel);

                // Calcul du chemin relatif pour le champ "$schema" du document
                // Depuis collections/actors/uuid.json vers schemas/v1/...
                let document_schema_ref = format!("../../../schemas/v1/{}", schema_rel);

                let description = self.generate_description(element_type, name).await;

                let doc = json!({
                    // On force le champ $schema pour la portabilité
                    "$schema": document_schema_ref,
                    "@context": {
                        "oa": "https://genaptitude.io/ontology/arcadia/oa#",
                        "sa": "https://genaptitude.io/ontology/arcadia/sa#",
                        "name": "http://www.w3.org/2004/02/skos/core#prefLabel"
                    },
                    "@type": type_uri,
                    "name": name,
                    "description": description,
                    "status": "draft_ai"
                });

                let mgr = CollectionsManager::new(&self.storage, &self.space, &self.db);

                // Création avec la bonne URI de schéma
                mgr.create_collection(collection, Some(schema_uri.clone()))?;

                let saved = mgr.insert_with_schema(collection, doc)?;
                let id = saved.get("id").and_then(|v| v.as_str()).unwrap_or("?");

                Ok(Some(format!(
                    "✅ J'ai créé l'élément **{}** (Type: {}) dans la collection `{}`.\n\n*Description :* _{}_\nID: `{}`", 
                    name, element_type, collection, description, id
                )))
            }
            _ => Ok(None),
        }
    }
}
