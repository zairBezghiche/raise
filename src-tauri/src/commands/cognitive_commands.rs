use crate::plugins::cognitive::CognitiveManager;
use std::path::PathBuf;
use tauri::{AppHandle, Manager}; // Manager est nécessaire pour .path()

#[tauri::command]
pub async fn run_consistency_analysis(
    app_handle: AppHandle,
    model_json: serde_json::Value,
) -> Result<String, String> {
    // 1. Instanciation du moteur
    let manager = CognitiveManager::new();

    // 2. Résolution du chemin (Logique Hybride Dev/Prod)
    // L'utilisation de 'if cfg!' au lieu de '#[cfg]' permet au compilateur de valider
    // les deux branches, ce qui supprime les warnings "unused variable".
    let plugin_path = if cfg!(debug_assertions) {
        // --- MODE DÉVELOPPEMENT ---
        // On utilise la variable d'environnement de compilation pour localiser la source
        // 'env!' est résolu à la compilation, c'est sûr et performant.
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent() // Remonte de 'src-tauri' vers la racine du projet
            .unwrap()
            .join("wasm-modules/analyzers/consistency_basic.wasm")
    } else {
        // --- MODE PRODUCTION ---
        // Ici 'app_handle' est utilisé, donc le warning disparaît.
        app_handle
            .path()
            .resource_dir()
            .unwrap_or(PathBuf::from("."))
            .join("wasm-modules/analyzers/consistency_basic.wasm")
    };

    println!("🤖 Exécution du bloc cognitif : {:?}", plugin_path);

    // Sécurité : Vérification avant exécution
    if !plugin_path.exists() {
        return Err(format!(
            "ERREUR CRITIQUE: Le fichier WASM est introuvable à ce chemin : {:?}",
            plugin_path
        ));
    }

    // 3. Exécution via le CognitiveManager
    manager
        .run_block(&plugin_path, &model_json)
        .map_err(|e| e.to_string())
}
