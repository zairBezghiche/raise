pub mod executor;
pub mod scheduler;
pub mod state_machine;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

// Re-exports
pub use executor::WorkflowExecutor;
pub use scheduler::WorkflowScheduler;
pub use state_machine::WorkflowStateMachine;

/// Type d'un nœud dans le graphe (correspond au schema JSON)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    Task,       // Tâche standard (ex: Agent IA)
    Decision,   // Branchement conditionnel
    Parallel,   // Exécution simultanée
    GateHitl,   // Pause pour validation humaine
    GatePolicy, // Vérification automatique de règles
    CallMcp,    // Appel outil externe (Model Context Protocol)
    End,        // Fin du flux
}

/// Statut d'exécution d'une instance ou d'un nœud
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Paused,  // En attente (HITL)
    Skipped, // Branche non prise
}

/// Représentation d'une étape du workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowNode {
    pub id: String,
    pub r#type: NodeType,
    pub name: String,
    #[serde(default)]
    pub params: Value,
}

/// Représentation d'une transition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowEdge {
    pub from: String,
    pub to: String,
    pub condition: Option<String>, // Script rhai ou json-logic
}

/// Définition statique du Workflow (le "Moule")
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub id: String,
    pub nodes: Vec<WorkflowNode>,
    pub edges: Vec<WorkflowEdge>,
    pub entry: String, // ID du nœud de départ
}

/// Instance dynamique (l'Exécution en cours)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowInstance {
    pub id: String,
    pub workflow_id: String,
    pub status: ExecutionStatus,
    /// État de chaque nœud : NodeID -> Status
    pub node_states: HashMap<String, ExecutionStatus>,
    /// Mémoire contextuelle du workflow (Variables)
    pub context: HashMap<String, Value>,
    /// Logs d'exécution
    pub logs: Vec<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl WorkflowInstance {
    pub fn new(workflow_id: &str, context: HashMap<String, Value>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            workflow_id: workflow_id.to_string(),
            status: ExecutionStatus::Pending,
            node_states: HashMap::new(),
            context,
            logs: Vec::new(),
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_instance_creation() {
        let context = HashMap::from([("user".to_string(), serde_json::json!("Alice"))]);
        let instance = WorkflowInstance::new("wf-onboarding", context);

        assert_eq!(instance.workflow_id, "wf-onboarding");
        assert_eq!(instance.status, ExecutionStatus::Pending);
        assert!(!instance.id.is_empty(), "L'ID doit être généré (UUID)");
        assert!(instance.created_at > 0, "Le timestamp doit être défini");
        assert!(
            instance.context.contains_key("user"),
            "Le contexte doit être préservé"
        );
    }

    #[test]
    fn test_serialization() {
        let node = WorkflowNode {
            id: "node_1".to_string(),
            r#type: NodeType::Task,
            name: "Task 1".to_string(),
            params: serde_json::json!({}),
        };
        let json = serde_json::to_string(&node).unwrap();
        assert!(json.contains("\"type\":\"task\"")); // Vérifie snake_case
    }
}
