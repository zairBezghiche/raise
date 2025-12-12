use super::{ExecutionStatus, WorkflowDefinition, WorkflowInstance};
use crate::utils::Result;

pub struct WorkflowStateMachine {
    definition: WorkflowDefinition,
}

impl WorkflowStateMachine {
    pub fn new(definition: WorkflowDefinition) -> Self {
        Self { definition }
    }

    /// Détermine les prochains nœuds à exécuter
    pub fn next_runnable_nodes(&self, instance: &WorkflowInstance) -> Vec<String> {
        let mut runnable = Vec::new();

        // 1. Si l'instance est nouvelle, on commence par l'entrée
        if instance.node_states.is_empty() {
            return vec![self.definition.entry.clone()];
        }

        // 2. Sinon, on cherche les enfants des nœuds terminés
        for (node_id, status) in &instance.node_states {
            if *status == ExecutionStatus::Completed {
                let children = self.get_children(node_id);
                for child_id in children {
                    // Si le nœud n'a jamais été touché, il est candidat
                    if !instance.node_states.contains_key(&child_id) {
                        // TODO: Vérifier ici si TOUS les parents requis sont finis (pour les Joins)
                        // Pour l'instant, on assume une exécution séquentielle simple
                        runnable.push(child_id);
                    }
                }
            }
        }

        runnable
    }

    /// Retourne les IDs des nœuds enfants
    fn get_children(&self, node_id: &str) -> Vec<String> {
        self.definition
            .edges
            .iter()
            .filter(|e| e.from == node_id)
            .map(|e| e.to.clone())
            .collect()
    }

    /// Met à jour l'état de l'instance après l'exécution d'un nœud
    pub fn transition(
        &self,
        instance: &mut WorkflowInstance,
        node_id: &str,
        result: ExecutionStatus,
    ) -> Result<()> {
        // Mise à jour du nœud
        instance.node_states.insert(node_id.to_string(), result);
        instance.updated_at = chrono::Utc::now().timestamp();

        // Mise à jour globale
        if result == ExecutionStatus::Failed {
            instance.status = ExecutionStatus::Failed;
            instance
                .logs
                .push(format!("Workflow failed at node {}", node_id));
        } else {
            // Est-ce la fin ?
            let next = self.next_runnable_nodes(instance);
            if next.is_empty() {
                // Plus rien à faire -> Terminé
                instance.status = ExecutionStatus::Completed;
                instance
                    .logs
                    .push("Workflow completed successfully.".to_string());
            } else {
                instance.status = ExecutionStatus::Running;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow_engine::{
        ExecutionStatus, NodeType, WorkflowEdge, WorkflowInstance, WorkflowNode,
    };
    use std::collections::HashMap;

    // Helper : Crée un workflow linéaire A -> B
    fn create_linear_workflow() -> WorkflowDefinition {
        WorkflowDefinition {
            id: "wf-linear".to_string(),
            entry: "A".to_string(),
            nodes: vec![
                WorkflowNode {
                    id: "A".to_string(),
                    r#type: NodeType::Task,
                    name: "Step A".to_string(),
                    params: Default::default(),
                },
                WorkflowNode {
                    id: "B".to_string(),
                    r#type: NodeType::Task,
                    name: "Step B".to_string(),
                    params: Default::default(),
                },
            ],
            edges: vec![WorkflowEdge {
                from: "A".to_string(),
                to: "B".to_string(),
                condition: None,
            }],
        }
    }

    #[test]
    fn test_initial_state() {
        let def = create_linear_workflow();
        let sm = WorkflowStateMachine::new(def);
        let instance = WorkflowInstance::new("wf-linear", HashMap::new());

        let next = sm.next_runnable_nodes(&instance);
        assert_eq!(next, vec!["A"], "Le premier nœud doit être l'entrée");
    }

    #[test]
    fn test_transition_a_to_b() {
        let def = create_linear_workflow();
        let sm = WorkflowStateMachine::new(def);
        let mut instance = WorkflowInstance::new("wf-linear", HashMap::new());

        // On simule la fin de A
        sm.transition(&mut instance, "A", ExecutionStatus::Completed)
            .unwrap();

        assert_eq!(
            instance.node_states.get("A"),
            Some(&ExecutionStatus::Completed)
        );

        // Le prochain doit être B
        let next = sm.next_runnable_nodes(&instance);
        assert_eq!(next, vec!["B"]);
    }

    #[test]
    fn test_workflow_completion() {
        let def = create_linear_workflow();
        let sm = WorkflowStateMachine::new(def);
        let mut instance = WorkflowInstance::new("wf-linear", HashMap::new());

        // On termine tout le monde
        instance
            .node_states
            .insert("A".to_string(), ExecutionStatus::Completed);

        // Transition finale sur B
        sm.transition(&mut instance, "B", ExecutionStatus::Completed)
            .unwrap();

        assert_eq!(
            instance.status,
            ExecutionStatus::Completed,
            "Le workflow doit être marqué terminé s'il n'y a plus de suite"
        );
    }
}
