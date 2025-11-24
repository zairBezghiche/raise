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
    pub const SKOS: &str = "http://www.w3.org/2004/02/skos/core#";
    pub const DCT: &str = "http://purl.org/dc/terms/";
    pub const FOAF: &str = "http://xmlns.com/foaf/0.1/";
    pub const PROV: &str = "http://www.w3.org/ns/prov#";
}

/// Définition d'une classe dans l'ontologie
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OntologyClass {
    /// IRI de la classe
    pub iri: String,
    /// Label préféré
    pub label: String,
    /// Description de la classe
    pub description: Option<String>,
    /// Classes parentes (super-classes)
    pub super_classes: Vec<String>,
    /// Propriétés définies pour cette classe
    pub properties: Vec<String>,
}

/// Définition d'une propriété dans l'ontologie
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OntologyProperty {
    /// IRI de la propriété
    pub iri: String,
    /// Label préféré
    pub label: String,
    /// Description de la propriété
    pub description: Option<String>,
    /// Domaine (classe(s) source)
    pub domain: Vec<String>,
    /// Range (classe(s) cible)
    pub range: Vec<String>,
    /// Type de propriété (ObjectProperty, DatatypeProperty, etc.)
    pub property_type: PropertyType,
}

/// Type de propriété OWL
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum PropertyType {
    ObjectProperty,
    DatatypeProperty,
    AnnotationProperty,
}

/// Vocabulaire pour Operational Analysis (OA)
pub mod oa {
    use super::*;

    /// Types de classes OA
    pub const OPERATIONAL_ANALYSIS_PACKAGE: &str = "OperationalAnalysisPackage";
    pub const OPERATIONAL_CAPABILITY: &str = "OperationalCapability";
    pub const OPERATIONAL_ACTIVITY: &str = "OperationalActivity";
    pub const OPERATIONAL_ACTOR: &str = "OperationalActor";
    pub const OPERATIONAL_ENTITY: &str = "OperationalEntity";
    pub const OPERATIONAL_PROCESS: &str = "OperationalProcess";
    pub const OPERATIONAL_EXCHANGE: &str = "OperationalExchange";

    /// Retourne toutes les classes OA
    pub fn classes() -> Vec<OntologyClass> {
        vec![
            OntologyClass {
                iri: format!("{}{}", namespaces::OA, OPERATIONAL_CAPABILITY),
                label: "Operational Capability".to_string(),
                description: Some(
                    "A capability that the operational system must provide".to_string(),
                ),
                super_classes: vec![],
                properties: vec![
                    "involvedActivities".to_string(),
                    "involvedActors".to_string(),
                ],
            },
            OntologyClass {
                iri: format!("{}{}", namespaces::OA, OPERATIONAL_ACTIVITY),
                label: "Operational Activity".to_string(),
                description: Some("An activity performed in the operational context".to_string()),
                super_classes: vec![],
                properties: vec![
                    "contributesTo".to_string(),
                    "allocatedTo".to_string(),
                    "incomingExchanges".to_string(),
                    "outgoingExchanges".to_string(),
                    "subActivities".to_string(),
                ],
            },
            OntologyClass {
                iri: format!("{}{}", namespaces::OA, OPERATIONAL_ACTOR),
                label: "Operational Actor".to_string(),
                description: Some("An entity that performs operational activities".to_string()),
                super_classes: vec![],
                properties: vec![
                    "allocatedActivities".to_string(),
                    "allocatedEntities".to_string(),
                ],
            },
            OntologyClass {
                iri: format!("{}{}", namespaces::OA, OPERATIONAL_ENTITY),
                label: "Operational Entity".to_string(),
                description: Some("An operational entity in the system".to_string()),
                super_classes: vec![],
                properties: vec![
                    "subEntities".to_string(),
                    "incomingExchanges".to_string(),
                    "outgoingExchanges".to_string(),
                ],
            },
            OntologyClass {
                iri: format!("{}{}", namespaces::OA, OPERATIONAL_EXCHANGE),
                label: "Operational Exchange".to_string(),
                description: Some(
                    "An exchange of information or material between activities or entities"
                        .to_string(),
                ),
                super_classes: vec![],
                properties: vec![
                    "source".to_string(),
                    "target".to_string(),
                    "exchangedItems".to_string(),
                ],
            },
        ]
    }

    /// Retourne toutes les propriétés OA
    pub fn properties() -> Vec<OntologyProperty> {
        vec![
            OntologyProperty {
                iri: format!("{}involvesActivity", namespaces::OA),
                label: "involves activity".to_string(),
                description: Some("Links a capability to activities that realize it".to_string()),
                domain: vec![format!("{}{}", namespaces::OA, OPERATIONAL_CAPABILITY)],
                range: vec![format!("{}{}", namespaces::OA, OPERATIONAL_ACTIVITY)],
                property_type: PropertyType::ObjectProperty,
            },
            OntologyProperty {
                iri: format!("{}involvesActor", namespaces::OA),
                label: "involves actor".to_string(),
                description: Some("Links a capability to actors involved in it".to_string()),
                domain: vec![format!("{}{}", namespaces::OA, OPERATIONAL_CAPABILITY)],
                range: vec![format!("{}{}", namespaces::OA, OPERATIONAL_ACTOR)],
                property_type: PropertyType::ObjectProperty,
            },
            OntologyProperty {
                iri: format!("{}contributesToCapability", namespaces::OA),
                label: "contributes to capability".to_string(),
                description: Some(
                    "Links an activity to capabilities it contributes to".to_string(),
                ),
                domain: vec![format!("{}{}", namespaces::OA, OPERATIONAL_ACTIVITY)],
                range: vec![format!("{}{}", namespaces::OA, OPERATIONAL_CAPABILITY)],
                property_type: PropertyType::ObjectProperty,
            },
            OntologyProperty {
                iri: format!("{}allocatedTo", namespaces::OA),
                label: "allocated to".to_string(),
                description: Some(
                    "Links an activity to the actor or entity it is allocated to".to_string(),
                ),
                domain: vec![format!("{}{}", namespaces::OA, OPERATIONAL_ACTIVITY)],
                range: vec![
                    format!("{}{}", namespaces::OA, OPERATIONAL_ACTOR),
                    format!("{}{}", namespaces::OA, OPERATIONAL_ENTITY),
                ],
                property_type: PropertyType::ObjectProperty,
            },
            OntologyProperty {
                iri: format!("{}exchangeSource", namespaces::OA),
                label: "exchange source".to_string(),
                description: Some("The source of an exchange".to_string()),
                domain: vec![format!("{}{}", namespaces::OA, OPERATIONAL_EXCHANGE)],
                range: vec![
                    format!("{}{}", namespaces::OA, OPERATIONAL_ACTIVITY),
                    format!("{}{}", namespaces::OA, OPERATIONAL_ENTITY),
                ],
                property_type: PropertyType::ObjectProperty,
            },
            OntologyProperty {
                iri: format!("{}exchangeTarget", namespaces::OA),
                label: "exchange target".to_string(),
                description: Some("The target of an exchange".to_string()),
                domain: vec![format!("{}{}", namespaces::OA, OPERATIONAL_EXCHANGE)],
                range: vec![
                    format!("{}{}", namespaces::OA, OPERATIONAL_ACTIVITY),
                    format!("{}{}", namespaces::OA, OPERATIONAL_ENTITY),
                ],
                property_type: PropertyType::ObjectProperty,
            },
        ]
    }
}

/// Vocabulaire pour System Analysis (SA)
pub mod sa {
    use super::*;

    /// Types de classes SA
    pub const SYSTEM_ANALYSIS_PACKAGE: &str = "SystemAnalysisPackage";
    pub const SYSTEM_CAPABILITY: &str = "SystemCapability";
    pub const SYSTEM_FUNCTION: &str = "SystemFunction";
    pub const SYSTEM_ACTOR: &str = "SystemActor";
    pub const SYSTEM_COMPONENT: &str = "SystemComponent";
    pub const FUNCTIONAL_EXCHANGE: &str = "FunctionalExchange";

    /// Retourne toutes les classes SA
    pub fn classes() -> Vec<OntologyClass> {
        vec![
            OntologyClass {
                iri: format!("{}{}", namespaces::SA, SYSTEM_CAPABILITY),
                label: "System Capability".to_string(),
                description: Some("A capability that the system must provide".to_string()),
                super_classes: vec![],
                properties: vec![
                    "realizedCapabilities".to_string(),
                    "involvedFunctions".to_string(),
                ],
            },
            OntologyClass {
                iri: format!("{}{}", namespaces::SA, SYSTEM_FUNCTION),
                label: "System Function".to_string(),
                description: Some("A function performed by the system".to_string()),
                super_classes: vec![],
                properties: vec!["realizesActivities".to_string(), "allocatedTo".to_string()],
            },
            OntologyClass {
                iri: format!("{}{}", namespaces::SA, SYSTEM_ACTOR),
                label: "System Actor".to_string(),
                description: Some("An external actor interacting with the system".to_string()),
                super_classes: vec![],
                properties: vec![],
            },
            OntologyClass {
                iri: format!("{}{}", namespaces::SA, SYSTEM_COMPONENT),
                label: "System Component".to_string(),
                description: Some("A component of the system".to_string()),
                super_classes: vec![],
                properties: vec!["allocatedFunctions".to_string()],
            },
        ]
    }
}

/// Vocabulaire pour Logical Architecture (LA)
pub mod la {
    use super::*;

    /// Types de classes LA
    pub const LOGICAL_ARCHITECTURE_PACKAGE: &str = "LogicalArchitecturePackage";
    pub const LOGICAL_FUNCTION: &str = "LogicalFunction";
    pub const LOGICAL_COMPONENT: &str = "LogicalComponent";
    pub const LOGICAL_INTERFACE: &str = "LogicalInterface";
    pub const COMPONENT_EXCHANGE: &str = "ComponentExchange";

    /// Retourne toutes les classes LA
    pub fn classes() -> Vec<OntologyClass> {
        vec![
            OntologyClass {
                iri: format!("{}{}", namespaces::LA, LOGICAL_FUNCTION),
                label: "Logical Function".to_string(),
                description: Some("A function in the logical architecture".to_string()),
                super_classes: vec![],
                properties: vec!["realizesFunctions".to_string()],
            },
            OntologyClass {
                iri: format!("{}{}", namespaces::LA, LOGICAL_COMPONENT),
                label: "Logical Component".to_string(),
                description: Some("A logical component in the architecture".to_string()),
                super_classes: vec![],
                properties: vec![
                    "realizesComponents".to_string(),
                    "providedInterfaces".to_string(),
                    "requiredInterfaces".to_string(),
                ],
            },
        ]
    }
}

/// Vocabulaire pour Physical Architecture (PA)
pub mod pa {
    use super::*;

    /// Types de classes PA
    pub const PHYSICAL_ARCHITECTURE_PACKAGE: &str = "PhysicalArchitecturePackage";
    pub const PHYSICAL_FUNCTION: &str = "PhysicalFunction";
    pub const PHYSICAL_COMPONENT: &str = "PhysicalComponent";
    pub const PHYSICAL_NODE: &str = "PhysicalNode";
    pub const PHYSICAL_LINK: &str = "PhysicalLink";

    /// Retourne toutes les classes PA
    pub fn classes() -> Vec<OntologyClass> {
        vec![
            OntologyClass {
                iri: format!("{}{}", namespaces::PA, PHYSICAL_COMPONENT),
                label: "Physical Component".to_string(),
                description: Some("A physical component of the system".to_string()),
                super_classes: vec![],
                properties: vec![
                    "realizesLogicalComponents".to_string(),
                    "deployedOn".to_string(),
                ],
            },
            OntologyClass {
                iri: format!("{}{}", namespaces::PA, PHYSICAL_NODE),
                label: "Physical Node".to_string(),
                description: Some("A physical node where components are deployed".to_string()),
                super_classes: vec![],
                properties: vec!["deployedComponents".to_string()],
            },
        ]
    }
}

/// Vocabulaire pour EPBS (End Product Breakdown Structure)
pub mod epbs {
    use super::*;

    /// Types de classes EPBS
    pub const EPBS_PACKAGE: &str = "EPBSPackage";
    pub const CONFIGURATION_ITEM: &str = "ConfigurationItem";
    pub const SYSTEM_PART: &str = "SystemPart";
    pub const PRODUCT_LINE: &str = "ProductLine";

    /// Retourne toutes les classes EPBS
    pub fn classes() -> Vec<OntologyClass> {
        vec![OntologyClass {
            iri: format!("{}{}", namespaces::EPBS, CONFIGURATION_ITEM),
            label: "Configuration Item".to_string(),
            description: Some("An item in the end product breakdown".to_string()),
            super_classes: vec![],
            properties: vec![
                "implementsPhysicalComponents".to_string(),
                "subItems".to_string(),
            ],
        }]
    }
}

/// Registre de vocabulaire pour toutes les phases Arcadia
#[derive(Debug, Clone)]
pub struct VocabularyRegistry {
    classes: HashMap<String, OntologyClass>,
    properties: HashMap<String, OntologyProperty>,
}

impl VocabularyRegistry {
    /// Crée un nouveau registre avec tous les vocabulaires Arcadia
    pub fn new() -> Self {
        let mut registry = Self {
            classes: HashMap::new(),
            properties: HashMap::new(),
        };

        // Enregistrer toutes les classes
        for class in oa::classes() {
            registry.classes.insert(class.iri.clone(), class);
        }
        for class in sa::classes() {
            registry.classes.insert(class.iri.clone(), class);
        }
        for class in la::classes() {
            registry.classes.insert(class.iri.clone(), class);
        }
        for class in pa::classes() {
            registry.classes.insert(class.iri.clone(), class);
        }
        for class in epbs::classes() {
            registry.classes.insert(class.iri.clone(), class);
        }

        // Enregistrer toutes les propriétés
        for prop in oa::properties() {
            registry.properties.insert(prop.iri.clone(), prop);
        }

        registry
    }

    /// Récupère une classe par IRI
    pub fn get_class(&self, iri: &str) -> Option<&OntologyClass> {
        self.classes.get(iri)
    }

    /// Récupère une propriété par IRI
    pub fn get_property(&self, iri: &str) -> Option<&OntologyProperty> {
        self.properties.get(iri)
    }

    /// Récupère toutes les classes
    pub fn all_classes(&self) -> Vec<&OntologyClass> {
        self.classes.values().collect()
    }

    /// Récupère toutes les propriétés
    pub fn all_properties(&self) -> Vec<&OntologyProperty> {
        self.properties.values().collect()
    }

    /// Vérifie si une classe existe
    pub fn has_class(&self, iri: &str) -> bool {
        self.classes.contains_key(iri)
    }

    /// Vérifie si une propriété existe
    pub fn has_property(&self, iri: &str) -> bool {
        self.properties.contains_key(iri)
    }
}

impl Default for VocabularyRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oa_classes() {
        let classes = oa::classes();
        assert!(!classes.is_empty());

        let capability = classes
            .iter()
            .find(|c| c.label == "Operational Capability")
            .expect("Should have OperationalCapability");

        assert!(capability.iri.contains("OperationalCapability"));
    }

    #[test]
    fn test_oa_properties() {
        let properties = oa::properties();
        assert!(!properties.is_empty());

        let involves_activity = properties
            .iter()
            .find(|p| p.label == "involves activity")
            .expect("Should have involvesActivity");

        assert_eq!(
            involves_activity.property_type,
            PropertyType::ObjectProperty
        );
    }

    #[test]
    fn test_vocabulary_registry() {
        let registry = VocabularyRegistry::new();

        let oa_capability_iri = format!("{}{}", namespaces::OA, oa::OPERATIONAL_CAPABILITY);
        assert!(registry.has_class(&oa_capability_iri));

        let class = registry.get_class(&oa_capability_iri);
        assert!(class.is_some());
    }

    #[test]
    fn test_namespaces() {
        assert_eq!(
            namespaces::ARCADIA,
            "https://genaptitude.io/ontology/arcadia#"
        );
        assert_eq!(
            namespaces::OA,
            "https://genaptitude.io/ontology/arcadia/oa#"
        );
        assert_eq!(
            namespaces::RDF,
            "http://www.w3.org/1999/02/22-rdf-syntax-ns#"
        );
    }
}
