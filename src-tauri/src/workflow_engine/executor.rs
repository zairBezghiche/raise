use super::{ExecutionStatus, NodeType, WorkflowNode};
use crate::utils::Result;
use serde_json::Value;

pub struct WorkflowExecutor;

impl Default for WorkflowExecutor {
    fn default() -> Self {
        Self::new()
    }
}
impl WorkflowExecutor {
    pub fn new() -> Self {
        Self
    }

    /// Exécute un nœud spécifique (Unité de travail)
    pub async fn execute_node(
        &self,
        node: &WorkflowNode,
        _context: &Value,
    ) -> Result<ExecutionStatus> {
        tracing::info!("⚙️ Exécution du nœud : {} ({:?})", node.name, node.r#type);

        match node.r#type {
            NodeType::Task => {
                // Simulation d'une tâche (ex: Appel Agent IA)
                // Ici, on connectera plus tard le module `ai::agents`
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                tracing::info!("✅ Tâche '{}' terminée.", node.name);
                Ok(ExecutionStatus::Completed)
            }

            NodeType::Decision => {
                // Évaluation logique (ex: if context.score > 50)
                // Pour l'instant, on passe toujours
                Ok(ExecutionStatus::Completed)
            }

            NodeType::GateHitl => {
                // Pause explicite pour Humain
                tracing::warn!(
                    "⏸️ Workflow en pause : Attente validation humaine pour '{}'",
                    node.name
                );
                Ok(ExecutionStatus::Paused)
            }

            NodeType::CallMcp => {
                // Appel d'outil externe
                Ok(ExecutionStatus::Completed)
            }

            NodeType::End => Ok(ExecutionStatus::Completed),

            _ => {
                tracing::warn!("Type de nœud non implémenté : {:?}", node.r#type);
                Ok(ExecutionStatus::Completed)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_execute_standard_task() {
        let executor = WorkflowExecutor::new();
        let node = WorkflowNode {
            id: "1".into(),
            r#type: NodeType::Task,
            name: "Simple Task".into(),
            params: json!({}),
        };

        let result = executor.execute_node(&node, &json!({})).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ExecutionStatus::Completed);
    }

    #[tokio::test]
    async fn test_execute_hitl_gate() {
        let executor = WorkflowExecutor::new();
        let node = WorkflowNode {
            id: "2".into(),
            r#type: NodeType::GateHitl,
            name: "Human Approval".into(),
            params: json!({}),
        };

        // Une porte HITL doit mettre le workflow en PAUSE
        let result = executor.execute_node(&node, &json!({})).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ExecutionStatus::Paused);
    }
}
