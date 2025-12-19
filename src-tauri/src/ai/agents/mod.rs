pub mod business_agent;
pub mod context;
pub mod data_agent;
pub mod epbs_agent;
pub mod hardware_agent;
pub mod intent_classifier;
pub mod software_agent;
pub mod system_agent;
pub mod transverse_agent;

#[cfg(test)]
mod tests;

pub use self::context::AgentContext;

use anyhow::Result;
use async_trait::async_trait;
use serde::Serialize;
use std::fmt;

use self::intent_classifier::EngineeringIntent;

/// Représente un élément créé ou modifié par un agent
#[derive(Debug, Clone, Serialize)]
pub struct CreatedArtifact {
    pub id: String,
    pub name: String,
    pub layer: String,        // "SA", "LA", "TRANSVERSE"...
    pub element_type: String, // "Function", "Requirement"...
    pub path: String,         // Chemin relatif ou absolu pour ouverture
}

/// Résultat structuré d'une action d'agent
#[derive(Debug, Clone, Serialize)]
pub struct AgentResult {
    /// Message conversationnel pour l'utilisateur
    pub message: String,
    /// Liste des artefacts techniques produits
    pub artifacts: Vec<CreatedArtifact>,
}

impl AgentResult {
    /// Helper pour créer une réponse textuelle simple (Chat/RAG)
    pub fn text(msg: String) -> Self {
        Self {
            message: msg,
            artifacts: vec![],
        }
    }
}

impl fmt::Display for AgentResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[async_trait]
pub trait Agent: Send + Sync {
    /// Identifiant unique de l'agent (ex: "system_architect")
    fn id(&self) -> &'static str;

    /// Exécute l'intention et retourne un résultat structuré
    async fn process(
        &self,
        ctx: &AgentContext,
        intent: &EngineeringIntent,
    ) -> Result<Option<AgentResult>>; // <-- Changement de signature ici (String -> AgentResult)
}
