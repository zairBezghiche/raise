use super::{
    executor::WorkflowExecutor, state_machine::WorkflowStateMachine, ExecutionStatus,
    WorkflowDefinition, WorkflowInstance,
};
use crate::utils::Result;
use std::collections::HashMap;

pub struct WorkflowScheduler {
    executor: WorkflowExecutor,
    definitions: HashMap<String, WorkflowDefinition>,
}

impl Default for WorkflowScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkflowScheduler {
    pub fn new() -> Self {
        Self {
            executor: WorkflowExecutor::new(),
            definitions: HashMap::new(),
        }
    }

    pub fn register_workflow(&mut self, def: WorkflowDefinition) {
        self.definitions.insert(def.id.clone(), def);
    }

    pub async fn run_step(&self, instance: &mut WorkflowInstance) -> Result<bool> {
        let def = self.definitions.get(&instance.workflow_id).ok_or_else(|| {
            crate::utils::AppError::NotFound(format!("WorkflowDef {}", instance.workflow_id))
        })?;

        let sm = WorkflowStateMachine::new(def.clone());

        if instance.status == ExecutionStatus::Completed
            || instance.status == ExecutionStatus::Failed
        {
            return Ok(false);
        }

        let next_nodes_ids = sm.next_runnable_nodes(instance);

        if next_nodes_ids.is_empty() {
            // Si on est ici, c'est qu'il n'y a rien à faire.
            // Si le statut est Running, c'est peut-être qu'on vient de finir une branche
            // et qu'il n'y a plus rien après. On devrait peut-être clore le workflow ?
            // Pour l'instant, on laisse la logique de transition gérer la clôture.
            return Ok(false);
        }

        for node_id in next_nodes_ids {
            let node = def.nodes.iter().find(|n| n.id == node_id).unwrap();

            instance
                .node_states
                .insert(node_id.clone(), ExecutionStatus::Running);

            let result = self
                .executor
                .execute_node(node, &serde_json::Value::Null)
                .await?;

            sm.transition(instance, &node_id, result)?;

            if result == ExecutionStatus::Paused {
                instance.status = ExecutionStatus::Paused;
                return Ok(false);
            }
        }

        Ok(true)
    }

    pub fn resume_node(
        &self,
        instance: &mut WorkflowInstance,
        node_id: &str,
        approved: bool,
    ) -> Result<()> {
        if instance.node_states.get(node_id) != Some(&ExecutionStatus::Paused) {
            return Err(crate::utils::AppError::System(anyhow::anyhow!(
                "Ce nœud n'est pas en pause"
            )));
        }

        let new_status = if approved {
            ExecutionStatus::Completed
        } else {
            ExecutionStatus::Failed
        };

        // 1. Mettre à jour le nœud
        instance.node_states.insert(node_id.to_string(), new_status);

        // 2. CORRECTION : Recalculer l'état global du workflow
        // Au lieu de forcer "Running", on vérifie s'il reste du travail.
        if let Some(def) = self.definitions.get(&instance.workflow_id) {
            let sm = WorkflowStateMachine::new(def.clone());

            if new_status == ExecutionStatus::Failed {
                instance.status = ExecutionStatus::Failed;
                instance
                    .logs
                    .push(format!("Node {} rejected/failed via resume.", node_id));
            } else {
                // On vérifie s'il y a une suite
                let next = sm.next_runnable_nodes(instance);
                if next.is_empty() {
                    // Plus de suite => Terminé !
                    instance.status = ExecutionStatus::Completed;
                    instance
                        .logs
                        .push("Workflow completed successfully (after resume).".to_string());
                } else {
                    // Il reste des choses à faire
                    instance.status = ExecutionStatus::Running;
                }
            }
        } else {
            // Fallback (ne devrait pas arriver si l'instance est valide)
            instance.status = ExecutionStatus::Running;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow_engine::{NodeType, WorkflowEdge, WorkflowNode};
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_scheduler_run_cycle() {
        let mut scheduler = WorkflowScheduler::new();

        // Workflow : Start (Task) -> Approval (GateHitl)
        let def = WorkflowDefinition {
            id: "wf-test".into(),
            entry: "start".into(),
            nodes: vec![
                WorkflowNode {
                    id: "start".into(),
                    r#type: NodeType::Task,
                    name: "Start".into(),
                    params: Default::default(),
                },
                WorkflowNode {
                    id: "approval".into(),
                    r#type: NodeType::GateHitl,
                    name: "Approve".into(),
                    params: Default::default(),
                },
            ],
            edges: vec![WorkflowEdge {
                from: "start".into(),
                to: "approval".into(),
                condition: None,
            }],
        };

        scheduler.register_workflow(def);

        let mut instance = WorkflowInstance::new("wf-test", HashMap::new());

        // ÉTAPE 1 : Exécution de 'start'
        let progress = scheduler.run_step(&mut instance).await.unwrap();
        assert!(progress);
        assert_eq!(
            instance.node_states.get("start"),
            Some(&ExecutionStatus::Completed)
        );

        // ÉTAPE 2 : Exécution de 'approval' (HITL)
        let progress = scheduler.run_step(&mut instance).await.unwrap();
        assert!(!progress);

        // Le nœud est bien en PAUSE
        assert_eq!(
            instance.node_states.get("approval"),
            Some(&ExecutionStatus::Paused)
        );
        assert_eq!(instance.status, ExecutionStatus::Paused);

        // ÉTAPE 3 : Intervention Humaine (Resume)
        scheduler
            .resume_node(&mut instance, "approval", true)
            .expect("resume failed");

        assert_eq!(
            instance.node_states.get("approval"),
            Some(&ExecutionStatus::Completed)
        );

        // CORRECTION DU TEST : Maintenant, resume_node est intelligent.
        // Comme 'approval' n'a pas d'enfants, le workflow doit passer directement à Completed.
        assert_eq!(instance.status, ExecutionStatus::Completed);

        // ÉTAPE 4 : Tentative de continuer (ne doit rien faire)
        let progress = scheduler.run_step(&mut instance).await.unwrap();
        assert!(!progress);
    }
}
