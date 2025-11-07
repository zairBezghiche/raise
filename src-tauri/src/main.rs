#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{fs, path::PathBuf};
use tauri::{command, Builder};

fn data_dir() -> PathBuf {
    // dossier local "data" à côté du binaire (simple pour un MVP)
    let p = std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("data/schemas");
    let _ = fs::create_dir_all(&p);
    p
}

#[command]
fn register_schema(schema_id: String, schema_json: String) -> Result<(), String> {
    let path = data_dir().join(format!("{schema_id}.json"));
    fs::write(path, schema_json).map_err(|e| e.to_string())
}

#[command]
fn get_schema(schema_id: String) -> Result<String, String> {
    let path = data_dir().join(format!("{schema_id}.json"));
    fs::read_to_string(path).map_err(|e| e.to_string())
}

fn main() {
    Builder::default()
        .invoke_handler(tauri::generate_handler![register_schema, get_schema])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
