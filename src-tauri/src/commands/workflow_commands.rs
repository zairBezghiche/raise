use crate::workflow_engine::{
    ExecutionStatus, WorkflowDefinition, WorkflowInstance, WorkflowScheduler,
};
use serde::Serialize;
use std::collections::HashMap;
use tauri::{command, State};
use tokio::sync::Mutex;

/// Structure qui contient l'état global du moteur de workflow.
#[derive(Default)]
pub struct WorkflowStore {
    pub scheduler: WorkflowScheduler,
    pub instances: HashMap<String, WorkflowInstance>,
}

/// DTO pour renvoyer une vue simplifiée au frontend
#[derive(Serialize)]
pub struct WorkflowView {
    pub id: String,
    pub status: ExecutionStatus,
    pub current_nodes: Vec<String>,
    pub logs: Vec<String>,
}

impl From<&WorkflowInstance> for WorkflowView {
    fn from(instance: &WorkflowInstance) -> Self {
        Self {
            id: instance.id.clone(),
            status: instance.status,
            current_nodes: instance.node_states.keys().cloned().collect(),
            logs: instance.logs.clone(),
        }
    }
}

#[command]
pub async fn register_workflow(
    state: State<'_, Mutex<WorkflowStore>>,
    definition: WorkflowDefinition,
) -> Result<String, String> {
    let mut store = state.lock().await;
    let id = definition.id.clone();
    store.scheduler.register_workflow(definition);
    Ok(format!("Workflow '{}' enregistré avec succès.", id))
}

#[command]
pub async fn start_workflow(
    state: State<'_, Mutex<WorkflowStore>>,
    workflow_id: String,
) -> Result<WorkflowView, String> {
    let instance_id = {
        let mut store = state.lock().await;
        let instance = WorkflowInstance::new(&workflow_id, HashMap::new());
        let id = instance.id.clone();
        store.instances.insert(id.clone(), instance);
        id
    };
    run_workflow_loop(state, instance_id).await
}

#[command]
pub async fn resume_workflow(
    state: State<'_, Mutex<WorkflowStore>>,
    instance_id: String,
    node_id: String,
    approved: bool,
) -> Result<WorkflowView, String> {
    {
        // 1. Verrouillage
        let mut guard = state.lock().await;

        // 2. Déstructuration CRITIQUE pour séparer les champs
        // Cela permet d'accéder à 'instances' et 'scheduler' simultanément
        let WorkflowStore {
            scheduler,
            instances,
        } = &mut *guard;

        let instance = instances
            .get_mut(&instance_id)
            .ok_or("Instance introuvable")?;

        // 3. Utilisation de la variable 'scheduler' locale (et non store.scheduler)
        scheduler
            .resume_node(instance, &node_id, approved)
            .map_err(|e| e.to_string())?;
    }

    // On relâche le lock ici (fin du bloc) avant de relancer la boucle
    run_workflow_loop(state, instance_id).await
}

async fn run_workflow_loop(
    state: State<'_, Mutex<WorkflowStore>>,
    instance_id: String,
) -> Result<WorkflowView, String> {
    loop {
        let mut guard = state.lock().await;

        // Même technique ici
        let WorkflowStore {
            scheduler,
            instances,
        } = &mut *guard;

        let instance = instances
            .get_mut(&instance_id)
            .ok_or("Instance introuvable")?;

        let keep_going = scheduler
            .run_step(instance)
            .await
            .map_err(|e| e.to_string())?;

        if !keep_going {
            return Ok(WorkflowView::from(&*instance));
        }
    }
}

#[command]
pub async fn get_workflow_state(
    state: State<'_, Mutex<WorkflowStore>>,
    instance_id: String,
) -> Result<WorkflowView, String> {
    let store = state.lock().await;
    let instance = store
        .instances
        .get(&instance_id)
        .ok_or("Instance introuvable")?;

    Ok(WorkflowView::from(instance))
}
