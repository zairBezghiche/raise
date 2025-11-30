use crate::json_db::storage::StorageEngine;
use crate::model_engine::loader::ProjectLoader;
use crate::model_engine::model::ProjectModel;
use tauri::{command, State};

/// Charge l'intégralité du modèle en mémoire pour analyse.
/// Cette commande peut être lourde, elle est déléguée à un thread bloquant.
#[command]
pub async fn load_project_model(
    storage: State<'_, StorageEngine>,
    space: String,
    db: String,
) -> Result<ProjectModel, String> {
    // CORRECTION : On clone le moteur pour en avoir une copie "possédée" (Owned)
    // Cela permet de le passer au thread 'static de spawn_blocking.
    // Le clonage est léger car le cache interne utilise Arc.
    let storage_engine = storage.inner().clone();

    // On délègue le travail lourd à un thread dédié
    let model = tauri::async_runtime::spawn_blocking(move || {
        // storage_engine est maintenant déplacé (move) dans le thread
        let loader = ProjectLoader::new(&storage_engine, &space, &db);
        loader.load_full_project()
    })
    .await
    .map_err(|e| format!("Thread error: {e}"))?
    .map_err(|e| format!("Load error: {e}"))?;

    Ok(model)
}
