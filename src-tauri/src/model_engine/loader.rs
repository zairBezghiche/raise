// FICHIER : src-tauri/src/model_engine/loader.rs

use crate::json_db::collections::manager::CollectionsManager;
use crate::json_db::jsonld::vocabulary::{arcadia_types, namespaces};
use crate::json_db::jsonld::JsonLdProcessor;
use crate::json_db::storage::StorageEngine;
use anyhow::Result;
use chrono::Utc;
use serde_json::Value;
use std::collections::HashMap;
use tauri::State;

// Import absolu pour la sécurité
use crate::model_engine::types::{
    ArcadiaElement, DataLayer, EPBSLayer, LogicalArchitectureLayer, NameType,
    OperationalAnalysisLayer, PhysicalArchitectureLayer, ProjectMeta, ProjectModel,
    SystemAnalysisLayer,
};

pub struct ModelLoader<'a> {
    manager: CollectionsManager<'a>,
    processor: JsonLdProcessor,
}

impl<'a> ModelLoader<'a> {
    // --- CONSTRUCTEURS UNIFIÉS ---

    /// Pour Tauri (State)
    pub fn new(storage: &'a State<'_, StorageEngine>, space: &str, db: &str) -> Self {
        Self::from_engine(storage.inner(), space, db)
    }

    /// Pour les Commandes (Thread Background) - Rétabli pour model_commands.rs
    pub fn from_engine(storage: &'a StorageEngine, space: &str, db: &str) -> Self {
        Self {
            manager: CollectionsManager::new(storage, space, db),
            processor: JsonLdProcessor::new(),
        }
    }

    /// Pour les Tests (Manager déjà init)
    pub fn new_with_manager(manager: CollectionsManager<'a>) -> Self {
        Self {
            manager,
            processor: JsonLdProcessor::new(),
        }
    }

    // --- MÉTHODE PRINCIPALE ---

    /// Charge tout le modèle (Nom unifié : load_full_model)
    pub fn load_full_model(&self) -> Result<ProjectModel> {
        let mut model = ProjectModel {
            oa: OperationalAnalysisLayer::default(),
            sa: SystemAnalysisLayer::default(),
            la: LogicalArchitectureLayer::default(),
            pa: PhysicalArchitectureLayer::default(),
            epbs: EPBSLayer::default(),
            data: DataLayer::default(),
            meta: ProjectMeta {
                name: format!("{}/{}", self.manager.space, self.manager.db),
                loaded_at: Utc::now().to_rfc3339(),
                element_count: 0,
            },
        };

        if let Ok(collections) = self.manager.list_collections() {
            for col_name in collections {
                if col_name.starts_with('_') {
                    continue;
                }
                if let Ok(docs) = self.manager.list_all(&col_name) {
                    for doc in docs {
                        if let Ok(element) = self.process_document_semantically(doc) {
                            self.dispatch_element(&mut model, element);
                            model.meta.element_count += 1;
                        }
                    }
                }
            }
        }

        Ok(model)
    }

    fn process_document_semantically(&self, doc: Value) -> Result<ArcadiaElement> {
        let expanded = self.processor.expand(&doc);
        let id = self
            .processor
            .get_id(&expanded)
            .unwrap_or_else(|| "unknown".to_string());
        let type_uri = self.processor.get_type(&expanded).unwrap_or_default();
        let compacted = self.processor.compact(&doc);

        let name_val = compacted
            .get("name")
            .or_else(|| compacted.get("http://www.w3.org/2004/02/skos/core#prefLabel"))
            .and_then(|v| v.as_str())
            .unwrap_or("Sans nom");

        let name = NameType::String(name_val.to_string());

        let obj = compacted
            .as_object()
            .ok_or(anyhow::anyhow!("Not an object"))?;
        let mut properties = HashMap::new();
        for (k, v) in obj {
            if !k.starts_with('@') {
                properties.insert(k.clone(), v.clone());
            }
        }

        Ok(ArcadiaElement {
            id,
            name,
            kind: type_uri,
            properties,
        })
    }

    fn dispatch_element(&self, model: &mut ProjectModel, el: ArcadiaElement) {
        let kind = &el.kind;
        if kind == &arcadia_types::uri(namespaces::OA, arcadia_types::OA_ACTOR) {
            model.oa.actors.push(el);
        } else if kind == &arcadia_types::uri(namespaces::OA, arcadia_types::OA_ACTIVITY) {
            model.oa.activities.push(el);
        } else if kind == &arcadia_types::uri(namespaces::OA, arcadia_types::OA_CAPABILITY) {
            model.oa.capabilities.push(el);
        } else if kind == &arcadia_types::uri(namespaces::SA, arcadia_types::SA_FUNCTION) {
            model.sa.functions.push(el);
        } else if kind == &arcadia_types::uri(namespaces::SA, arcadia_types::SA_ACTOR) {
            model.sa.actors.push(el);
        } else if kind == &arcadia_types::uri(namespaces::SA, arcadia_types::SA_COMPONENT) {
            model.sa.components.push(el);
        } else if kind == &arcadia_types::uri(namespaces::LA, arcadia_types::LA_COMPONENT) {
            model.la.components.push(el);
        } else if kind == &arcadia_types::uri(namespaces::PA, arcadia_types::PA_COMPONENT) {
            model.pa.components.push(el);
        } else if kind == &arcadia_types::uri(namespaces::DATA, arcadia_types::EXCHANGE_ITEM) {
            model.data.exchange_items.push(el);
        } else if kind == &arcadia_types::uri(namespaces::DATA, arcadia_types::DATA_CLASS) {
            model.data.classes.push(el);
        } else if kind == &arcadia_types::uri(namespaces::DATA, arcadia_types::DATA_TYPE) {
            model.data.data_types.push(el);
        } else if kind.ends_with("Actor") {
            model.oa.actors.push(el);
        }
    }
}
