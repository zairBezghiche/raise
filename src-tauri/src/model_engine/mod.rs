// FICHIER : src-tauri/src/model_engine/mod.rs

pub mod loader;
pub mod types;

// Modules Arcadia
pub mod arcadia;
// (Optionnel) Autres modules capella/transformers s'ils existent
// pub mod capella;
// pub mod transformers;

// Re-exports
pub use loader::ModelLoader;
pub use types::ProjectModel;

// Types communs
pub mod common;
pub mod validators;

// AJOUT : Enregistrement du module de tests
#[cfg(test)]
mod tests;
