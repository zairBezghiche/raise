use crate::utils::{AppConfig, AppError, Result};
use serde::Serialize;
use tauri::command;

/// Structure de réponse renvoyée au Frontend
#[derive(Debug, Serialize)]
pub struct SystemInfoResponse {
    pub app_version: String,
    pub env_mode: String,
    pub api_status: String,
    pub database_path: String,
}

/// Commande Tauri : Récupère les informations système
/// Retourne un Result<SystemInfoResponse, AppError> qui sera sérialisé en JSON ou string d'erreur.
#[command]
pub async fn get_app_info() -> Result<SystemInfoResponse> {
    // 1. Log structuré (visible si RUST_LOG=info ou debug)
    tracing::info!("📥 Commande reçue : get_app_info");

    // 2. Accès sécurisé à la configuration
    let config = AppConfig::get();

    // 3. Exemple de logique métier (ex: vérifier si l'API répond)
    // Ici on simule juste une vérification de config
    if config.llm_api_url.is_empty() {
        tracing::error!("URL de l'API LLM manquante !");
        return Err(AppError::Config("URL API LLM non configurée".to_string()));
    }

    // 4. Construction de la réponse
    let response = SystemInfoResponse {
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        env_mode: config.env_mode.clone(),
        api_status: format!("Connecté à {}", config.llm_api_url),
        database_path: config.database_root.to_string_lossy().to_string(),
    };

    tracing::debug!("✅ Réponse envoyée : {:?}", response);
    Ok(response)
}
