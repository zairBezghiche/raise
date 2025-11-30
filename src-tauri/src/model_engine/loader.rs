use crate::json_db::collections::manager::CollectionsManager;
use crate::json_db::storage::StorageEngine;
use crate::model_engine::model::*;
use anyhow::Result;
use serde::de::DeserializeOwned;

pub struct ProjectLoader<'a> {
    manager: CollectionsManager<'a>,
}

impl<'a> ProjectLoader<'a> {
    pub fn new(storage: &'a StorageEngine, space: &str, db: &str) -> Self {
        Self {
            manager: CollectionsManager::new(storage, space, db),
        }
    }

    /// Charge tout le projet en mémoire
    pub fn load_full_project(&self) -> Result<ProjectModel> {
        let mut model = ProjectModel::default();

        // 🟢 OA
        model.oa.actors = self.load_collection("operational-actors")?;
        model.oa.activities = self.load_collection("operational-activities")?;
        model.oa.capabilities = self.load_collection("operational-capabilities")?;
        model.oa.entities = self.load_collection("operational-entities")?;
        // model.oa.exchanges = self.load_collection("operational-exchanges")?;

        // 🟡 SA
        model.sa.components = self.load_collection("system-components")?;
        model.sa.actors = self.load_collection("system-actors")?;
        model.sa.functions = self.load_collection("system-functions")?;
        model.sa.capabilities = self.load_collection("system-capabilities")?;
        // model.sa.exchanges = self.load_collection("functional-exchanges-sa")?;

        // 🔵 LA
        model.la.components = self.load_collection("logical-components")?;
        model.la.functions = self.load_collection("logical-functions")?;
        // ... suite LA ...

        // 🔴 PA
        model.pa.components = self.load_collection("physical-components")?;
        // ... suite PA ...

        // 🟣 EPBS
        model.epbs.configuration_items = self.load_collection("configuration-items")?;

        // Méta
        model.meta.loaded_at = chrono::Utc::now().to_rfc3339();
        model.meta.element_count =
            model.oa.actors.len() + model.sa.functions.len() + model.pa.components.len(); // etc...

        Ok(model)
    }

    /// Helper générique : Charge une collection et la convertit en Vec<T>
    fn load_collection<T: DeserializeOwned>(&self, collection_name: &str) -> Result<Vec<T>> {
        let docs = self.manager.list_all(collection_name).unwrap_or_default();
        let mut items = Vec::new();

        for doc in docs {
            // On utilise serde_json pour transformer la Value générique en Struct typée T
            // Si le schéma JSON est respecté, ça passe. Sinon, on log l'erreur mais on continue.
            match serde_json::from_value::<T>(doc.clone()) {
                Ok(item) => items.push(item),
                Err(e) => {
                    // Ici on pourrait logger : "Failed to parse document in {collection_name}: {e}"
                    // Pour l'instant on ignore silencieusement les documents malformés
                    eprintln!("⚠️ Erreur mapping Rust pour {collection_name}: {e}");
                }
            }
        }
        Ok(items)
    }
}
