// FICHIER : src-tauri/src/main.rs
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env;
use std::path::PathBuf;
use std::sync::Mutex; // Mutex Standard (Synchronous) pour AppState
use tauri::Manager;
// 1. On donne un alias explicite au Mutex Async
use tokio::sync::Mutex as AsyncMutex;

use genaptitude::commands::{
    ai_commands, blockchain_commands, codegen_commands, cognitive_commands, genetics_commands,
    json_db_commands, model_commands, traceability_commands, utils_commands, workflow_commands,
};
use genaptitude::json_db::storage::{JsonDbConfig, StorageEngine};

// Import des structures d'état
use genaptitude::commands::workflow_commands::WorkflowStore;
use genaptitude::model_engine::types::ProjectModel;
use genaptitude::AppState; // Contient std::sync::Mutex<ProjectModel>

fn main() {
    // Initialisation des logs & config via utils (optionnel ici si fait dans lib.rs, mais sûr)
    genaptitude::utils::init_logging();
    let _ = genaptitude::utils::AppConfig::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            // --- 1. CONFIGURATION DB ---
            let db_root = if let Ok(env_path) = env::var("PATH_GENAPTITUDE_DOMAIN") {
                println!("📂 Utilisation de la DB personnalisée : {}", env_path);
                PathBuf::from(env_path)
            } else {
                let app_data_dir = app
                    .path()
                    .app_data_dir()
                    .expect("failed to get app data dir");
                let path = app_data_dir.join("genaptitude_db");
                println!("📂 Utilisation de la DB système par défaut : {:?}", path);
                path
            };

            let config = JsonDbConfig::new(db_root);
            let storage = StorageEngine::new(config);

            // Enregistrement du StorageEngine
            app.manage(storage);

            // --- 2. CONFIGURATION ÉTATS GLOBAUX ---

            // A. AppState (Modèle Arcadia) -> Utilise std::sync::Mutex
            app.manage(AppState {
                model: Mutex::new(ProjectModel::default()),
            });

            // B. WorkflowStore (Moteur Workflow) -> Utilise AsyncMutex (Tokio)
            // C'est ici que l'alias est utilisé pour éviter le conflit
            app.manage(AsyncMutex::new(WorkflowStore::default()));

            // C. Blockchain (Clients)
            // genaptitude::blockchain::ensure_innernet_state(app, "default");
            let app_handle = app.handle();
            genaptitude::blockchain::ensure_innernet_state(app_handle, "default");
            // Note: FabricClient doit être géré ici aussi si nécessaire

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // --- GESTION DATABASE ---
            json_db_commands::jsondb_create_db,
            json_db_commands::jsondb_drop_db,
            json_db_commands::jsondb_create_collection,
            json_db_commands::jsondb_list_collections,
            json_db_commands::jsondb_drop_collection,
            json_db_commands::jsondb_create_index,
            json_db_commands::jsondb_drop_index,
            json_db_commands::jsondb_insert_document,
            json_db_commands::jsondb_get_document,
            json_db_commands::jsondb_update_document,
            json_db_commands::jsondb_delete_document,
            json_db_commands::jsondb_list_all,
            json_db_commands::jsondb_execute_query,
            json_db_commands::jsondb_execute_sql,
            // --- MODEL & ARCHITECTURE ---
            model_commands::load_project_model,
            // --- IA & AGENTS ---
            ai_commands::ai_chat,
            // --- BLOCKCHAIN ---
            blockchain_commands::fabric_ping,
            blockchain_commands::fabric_submit_transaction,
            blockchain_commands::fabric_query_transaction,
            blockchain_commands::fabric_get_history,
            blockchain_commands::vpn_network_status,
            blockchain_commands::vpn_connect,
            blockchain_commands::vpn_disconnect,
            blockchain_commands::vpn_list_peers,
            blockchain_commands::vpn_add_peer,
            blockchain_commands::vpn_ping_peer,
            blockchain_commands::vpn_check_installation,
            // --- OPTIMISATION ---
            genetics_commands::run_genetic_optimization,
            // --- CODEGEN ---
            codegen_commands::generate_source_code,
            // --- COGNITIVE (WASM) ---
            cognitive_commands::run_consistency_analysis,
            // --- TRAÇABILITÉ ---
            traceability_commands::analyze_impact,
            traceability_commands::run_compliance_audit,
            traceability_commands::get_traceability_matrix,
            traceability_commands::get_element_neighbors,
            // --- UTILITAIRES ---
            utils_commands::get_app_info,
            // --- WORKFLOW ENGINE ---
            workflow_commands::register_workflow,
            workflow_commands::start_workflow,
            workflow_commands::resume_workflow,
            workflow_commands::get_workflow_state
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
