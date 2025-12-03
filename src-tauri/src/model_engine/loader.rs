// FICHIER : src-tauri/src/model_engine/loader.rs

use crate::json_db::collections::manager::CollectionsManager;
use crate::json_db::jsonld::vocabulary::{arcadia_types, namespaces};
use crate::json_db::jsonld::JsonLdProcessor;
use crate::json_db::storage::StorageEngine;
use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use tauri::State;

use super::types::{ArcadiaElement, ProjectModel};

pub struct ModelLoader<'a> {
    manager: CollectionsManager<'a>,
    processor: JsonLdProcessor,
}

impl<'a> ModelLoader<'a> {
    /// Constructeur principal utilisé par les commandes Tauri
    pub fn new(storage: &'a State<'_, StorageEngine>, space: &str, db: &str) -> Self {
        Self::from_engine(storage.inner(), space, db)
    }

    /// Constructeur découplé (utilisable pour les tests d'intégration sans contexte Tauri)
    pub fn from_engine(storage: &'a StorageEngine, space: &str, db: &str) -> Self {
        Self {
            manager: CollectionsManager::new(storage, space, db),
            processor: JsonLdProcessor::new(),
        }
    }

    /// Charge l'intégralité du modèle en mémoire en scannant toutes les collections
    /// CORRECTION : Retrait de 'async' car les opérations sous-jacentes (std::fs) sont synchrones/bloquantes
    pub fn load_full_model(&self) -> Result<ProjectModel> {
        let mut model = ProjectModel::default();

        // On parcourt toutes les collections de la DB
        let collections = self.manager.list_collections()?;

        for col_name in collections {
            // On ignore les collections systèmes ou de métadonnées pures
            if col_name.starts_with('_') {
                continue;
            }

            let docs = self.manager.list_all(&col_name)?;

            for doc in docs {
                // Analyse Sémantique via JSON-LD
                // On utilise l'expansion pour garantir que le type est bien reconnu
                if let Ok(element) = self.process_document_semantically(doc) {
                    self.dispatch_element(&mut model, element);
                }
            }
        }

        Ok(model)
    }

    /// Transforme un document JSON brut en Élément Arcadia typé sémantiquement
    fn process_document_semantically(&self, doc: Value) -> Result<ArcadiaElement> {
        // 1. Expansion pour normaliser les types (ex: "oa:Actor" -> "https://.../oa#OperationalActor")
        let expanded = self.processor.expand(&doc);

        // 2. Récupération de l'ID et du Type canonique (URI complète)
        let id = self
            .processor
            .get_id(&expanded)
            .unwrap_or_else(|| "unknown".to_string());

        let type_uri = self.processor.get_type(&expanded).unwrap_or_default();

        // 3. Extraction des propriétés
        let compacted = self.processor.compact(&doc);

        // Récupération du nom (supporte "name", "oa:name", ou "skos:prefLabel")
        let name = compacted
            .get("name")
            .or_else(|| compacted.get("http://www.w3.org/2004/02/skos/core#prefLabel"))
            .and_then(|v| v.as_str())
            .unwrap_or("Sans nom")
            .to_string();

        let obj = compacted
            .as_object()
            .ok_or(anyhow::anyhow!("Not an object"))?;

        let mut properties = HashMap::new();
        for (k, v) in obj {
            // On exclut les mots-clés JSON-LD
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

    /// Range l'élément dans la bonne couche du modèle selon son Type URI exact
    fn dispatch_element(&self, model: &mut ProjectModel, el: ArcadiaElement) {
        // --- OA ---
        if el.kind == arcadia_types::uri(namespaces::OA, arcadia_types::OA_ACTOR) {
            model.oa.actors.push(el);
        } else if el.kind == arcadia_types::uri(namespaces::OA, arcadia_types::OA_ACTIVITY) {
            model.oa.activities.push(el);
        } else if el.kind == arcadia_types::uri(namespaces::OA, arcadia_types::OA_CAPABILITY) {
            model.oa.capabilities.push(el);
        }
        // --- SA ---
        else if el.kind == arcadia_types::uri(namespaces::SA, arcadia_types::SA_FUNCTION) {
            model.sa.functions.push(el);
        } else if el.kind == arcadia_types::uri(namespaces::SA, arcadia_types::SA_ACTOR) {
            model.sa.actors.push(el);
        } else if el.kind == arcadia_types::uri(namespaces::SA, arcadia_types::SA_COMPONENT) {
            model.sa.components.push(el);
        }
        // --- LA ---
        else if el.kind == arcadia_types::uri(namespaces::LA, arcadia_types::LA_COMPONENT) {
            model.la.components.push(el);
        }
        // --- PA ---
        else if el.kind == arcadia_types::uri(namespaces::PA, arcadia_types::PA_COMPONENT) {
            model.pa.components.push(el);
        }
        // Fallback pour compatibilité ascendante
        else if el.kind.ends_with("Actor") {
            model.oa.actors.push(el);
        }
    }
}
