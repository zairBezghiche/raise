// FICHIER : src-tauri/src/json_db/jsonld/vocabulary.rs

//! Vocabulaire et ontologie Arcadia
//!
//! Ce module définit les types, classes et propriétés pour chaque phase
//! de la méthode Arcadia (OA, SA, LA, PA, EPBS).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Espace de noms des ontologies Arcadia
pub mod namespaces {
    pub const ARCADIA: &str = "https://genaptitude.io/ontology/arcadia#";
    pub const OA: &str = "https://genaptitude.io/ontology/arcadia/oa#";
    pub const SA: &str = "https://genaptitude.io/ontology/arcadia/sa#";
    pub const LA: &str = "https://genaptitude.io/ontology/arcadia/la#";
    pub const PA: &str = "https://genaptitude.io/ontology/arcadia/pa#";
    pub const EPBS: &str = "https://genaptitude.io/ontology/arcadia/epbs#";
    pub const DATA: &str = "https://genaptitude.io/ontology/arcadia/data#";

    // Standards sémantiques
    pub const RDF: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";
    pub const RDFS: &str = "http://www.w3.org/2000/01/rdf-schema#";
    pub const OWL: &str = "http://www.w3.org/2002/07/owl#";
    pub const XSD: &str = "http://www.w3.org/2001/XMLSchema#";
    pub const DCTERMS: &str = "http://purl.org/dc/terms/";
    pub const PROV: &str = "http://www.w3.org/ns/prov#";
}

/// Constantes des types Arcadia pour le typage fort dans le Model Engine
pub mod arcadia_types {
    // --- OA (Operational Analysis) ---
    pub const OA_ACTOR: &str = "OperationalActor";
    pub const OA_ACTIVITY: &str = "OperationalActivity";
    pub const OA_CAPABILITY: &str = "OperationalCapability";
    pub const OA_ENTITY: &str = "OperationalEntity";
    pub const OA_EXCHANGE: &str = "OperationalExchange";

    // --- SA (System Analysis) ---
    pub const SA_COMPONENT: &str = "SystemComponent";
    pub const SA_FUNCTION: &str = "SystemFunction";
    pub const SA_ACTOR: &str = "SystemActor";
    pub const SA_CAPABILITY: &str = "SystemCapability";
    // Note: FunctionalExchange existe en SA et LA
    pub const SA_EXCHANGE: &str = "FunctionalExchange";

    // --- LA (Logical Architecture) ---
    pub const LA_COMPONENT: &str = "LogicalComponent";
    pub const LA_FUNCTION: &str = "LogicalFunction";
    pub const LA_ACTOR: &str = "LogicalActor";
    pub const LA_INTERFACE: &str = "LogicalInterface";
    pub const LA_EXCHANGE: &str = "FunctionalExchange";

    // --- PA (Physical Architecture) ---
    pub const PA_COMPONENT: &str = "PhysicalComponent";
    pub const PA_FUNCTION: &str = "PhysicalFunction";
    pub const PA_ACTOR: &str = "PhysicalActor";
    pub const PA_LINK: &str = "PhysicalLink";

    // --- EPBS ---
    pub const EPBS_ITEM: &str = "ConfigurationItem";

    /// Helper pour construire l'URI complète (ex: "https://.../oa#OperationalActor")
    pub fn uri(namespace: &str, type_name: &str) -> String {
        format!("{}{}", namespace, type_name)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PropertyType {
    DatatypeProperty,
    ObjectProperty,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Class {
    pub iri: String,
    pub label: String,
    pub comment: String,
    pub sub_class_of: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Property {
    pub iri: String,
    pub label: String,
    pub property_type: PropertyType,
    pub domain: Option<String>,
    pub range: Option<String>,
}

// --- Sous-modules existants (conservés pour la compatibilité) ---

pub mod oa {
    use super::*;
    pub const OPERATIONAL_ACTIVITY: &str = "OperationalActivity";
    pub const OPERATIONAL_CAPABILITY: &str = "OperationalCapability";
    pub const OPERATIONAL_ENTITY: &str = "OperationalEntity";

    pub fn classes() -> Vec<Class> {
        vec![
            Class {
                iri: format!("{}{}", namespaces::OA, OPERATIONAL_ACTIVITY),
                label: "Operational Activity".to_string(),
                comment: "An activity performed by an operational entity".to_string(),
                sub_class_of: None,
            },
            Class {
                iri: format!("{}{}", namespaces::OA, OPERATIONAL_CAPABILITY),
                label: "Operational Capability".to_string(),
                comment: "An ability of an organization to provide a service".to_string(),
                sub_class_of: None,
            },
        ]
    }

    pub fn properties() -> Vec<Property> {
        vec![Property {
            iri: format!("{}involvesActivity", namespaces::OA),
            label: "involves activity".to_string(),
            property_type: PropertyType::ObjectProperty,
            domain: Some(format!("{}{}", namespaces::OA, OPERATIONAL_CAPABILITY)),
            range: Some(format!("{}{}", namespaces::OA, OPERATIONAL_ACTIVITY)),
        }]
    }
}

// --- Registry ---

pub struct VocabularyRegistry {
    classes: HashMap<String, Class>,
    properties: HashMap<String, Property>,
}

impl Default for VocabularyRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl VocabularyRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            classes: HashMap::new(),
            properties: HashMap::new(),
        };
        registry.register_module_oa();
        registry
    }

    fn register_module_oa(&mut self) {
        for cls in oa::classes() {
            self.classes.insert(cls.iri.clone(), cls);
        }
        for prop in oa::properties() {
            self.properties.insert(prop.iri.clone(), prop);
        }
    }

    pub fn get_class(&self, iri: &str) -> Option<&Class> {
        self.classes.get(iri)
    }

    pub fn has_class(&self, iri: &str) -> bool {
        self.classes.contains_key(iri)
    }

    /// Retourne les préfixes standards pour le processeur JSON-LD
    pub fn get_default_prefixes() -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("arcadia".to_string(), namespaces::ARCADIA.to_string());
        map.insert("oa".to_string(), namespaces::OA.to_string());
        map.insert("sa".to_string(), namespaces::SA.to_string());
        map.insert("la".to_string(), namespaces::LA.to_string());
        map.insert("pa".to_string(), namespaces::PA.to_string());
        map.insert("epbs".to_string(), namespaces::EPBS.to_string());
        map.insert("data".to_string(), namespaces::DATA.to_string());

        map.insert("rdf".to_string(), namespaces::RDF.to_string());
        map.insert("rdfs".to_string(), namespaces::RDFS.to_string());
        map.insert("xsd".to_string(), namespaces::XSD.to_string());
        map.insert("dcterms".to_string(), namespaces::DCTERMS.to_string());
        map.insert("prov".to_string(), namespaces::PROV.to_string());
        map
    }

    /// Vérifie si une chaîne est une IRI absolue
    pub fn is_iri(term: &str) -> bool {
        term.starts_with("http://") || term.starts_with("https://") || term.starts_with("urn:")
    }
}
