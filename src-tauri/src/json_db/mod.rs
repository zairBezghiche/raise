//! Module de gestion de base de données JSON
//!
//! Fonctionnalités:
//! - Collections avec schémas JSON Schema
//! - Support JSON-LD pour contexte sémantique
//! - Indexes pour requêtes rapides
//! - Transactions ACID
//! - Migrations de schémas

use anyhow::Result;
use std::path::Path;

// --- Imports qui définissent la structure JsonDb ---
use self::collections::manager::CollectionsManager;
// CORRECTION : On retire StorageEngine d'ici car il est importé via le `pub use` plus bas
use self::storage::JsonDbConfig;

// Déclarations des modules
pub mod collections;
pub mod indexes;
pub mod jsonld;
pub mod migrations;
pub mod query;
pub mod schema;
pub mod storage;
pub mod transactions;

#[doc(hidden)]
pub mod test_utils;

// ===========================================================================
// STRUCTURE PRINCIPALE : JsonDb
// ===========================================================================

/// La structure principale de la base de données JSON.
/// Agit comme une façade de haut niveau et détient le moteur de stockage (et son cache).
#[derive(Debug, Clone)]
pub struct JsonDb {
    pub storage: StorageEngine,
}

impl JsonDb {
    /// Crée une nouvelle instance de JsonDb en chargeant la configuration.
    pub fn new(repo_root: impl AsRef<Path>) -> Result<Self> {
        let config = JsonDbConfig::from_env(repo_root)?;
        // On initialise le StorageEngine (qui contient le cache)
        let storage = StorageEngine::new(config);
        Ok(Self { storage })
    }

    /// Crée un manager lié à un espace et une base de données spécifiques.
    pub fn collections_manager<'a>(&'a self, space: &str, db: &str) -> CollectionsManager<'a> {
        // On passe le storage engine complet
        CollectionsManager::new(&self.storage, space, db)
    }

    /// Accès direct à la configuration pour compatibilité
    pub fn config(&self) -> &JsonDbConfig {
        &self.storage.config
    }
}

// ===========================================================================
// RÉ-EXPORTATIONS PUBLIQUES (API Facade)
// ===========================================================================

pub use self::query::{QueryEngine, QueryInput, QueryResult};

pub use self::jsonld::JsonLdContext;
pub use self::schema::SchemaValidator;
pub use self::storage::StorageEngine;
pub use self::transactions::TransactionManager;
