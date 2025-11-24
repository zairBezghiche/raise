// Fichier : src-tauri/src/json_db/transactions/manager.rs

use crate::json_db::storage::JsonDbConfig;
use anyhow::Result;

/// Représente une transaction pour la DB (lecture/écriture).
/// Note : Cette structure ne contient plus les champs temporaires (id, staging_root)
/// car les opérations de persistence sont désormais atomiques au niveau fichier.
#[derive(Debug)]
pub struct Transaction {
    pub cfg: JsonDbConfig, // Doit être détenu (owned)
    pub space: String,
    pub db: String,
}

impl Transaction {
    /// Crée un nouveau contexte de transaction.
    /// Accepte une référence à la configuration et la clone (pour posséder sa propre copie).
    pub fn new(cfg: &JsonDbConfig, space: &str, db: &str) -> Self {
        Self {
            cfg: cfg.clone(), // CORRECTION: Clone pour respecter la signature (pub cfg: JsonDbConfig)
            space: space.to_string(),
            db: db.to_string(),
        }
    }

    /// Ouvre la transaction (no-op pour l'instant).
    #[allow(clippy::unused_self)]
    pub fn begin(&self) -> Result<()> {
        Ok(())
    }

    /// Applique les changements (no-op pour l'instant).
    #[allow(clippy::unused_self)]
    pub fn commit(&self) -> Result<()> {
        Ok(())
    }

    /// Annule les changements (no-op pour l'instant).
    #[allow(clippy::unused_self)]
    pub fn rollback(&self) -> Result<()> {
        Ok(())
    }
}
