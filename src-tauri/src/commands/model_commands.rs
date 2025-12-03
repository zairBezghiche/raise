// FICHIER : src-tauri/src/commands/model_commands.rs

use crate::json_db::storage::StorageEngine;
use crate::model_engine::loader::ModelLoader;
use crate::model_engine::types::ProjectModel;
use tauri::{command, State};

/// Charge l'intégralité du modèle en mémoire pour analyse.
/// Cette commande peut être lourde, elle est déléguée à un thread bloquant.
#[command]
pub async fn load_project_model(
    storage: State<'_, StorageEngine>,
    space: String,
    db: String,
) -> Result<ProjectModel, String> {
    // On clone le moteur pour en avoir une copie "possédée" (Owned) indépendante de Tauri
    let storage_engine = storage.inner().clone();

    // On délègue le travail lourd à un thread dédié
    let model = tauri::async_runtime::spawn_blocking(move || {
        let loader = ModelLoader::from_engine(&storage_engine, &space, &db);

        // CORRECTION ICI : load_full_model au lieu de load_full_project
        loader.load_full_model()
    })
    .await
    .map_err(|e| format!("Thread error: {e}"))?
    .map_err(|e| format!("Load error: {e}"))?;

    Ok(model)
}
