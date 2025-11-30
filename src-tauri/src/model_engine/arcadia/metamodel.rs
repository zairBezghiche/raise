use crate::model_engine::common::{ElementRef, I18nString};
use serde::{Deserialize, Serialize};

/// Propriétés fonctionnelles communes (Arcadia Metamodel)
/// Correspond à `BaseProperties` du schéma
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArcadiaProperties {
    #[serde(rename = "xmi_id", skip_serializing_if = "Option::is_none")]
    pub xmi_id: Option<String>,

    pub name: I18nString,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<I18nString>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<I18nString>,

    #[serde(default)]
    pub tags: Vec<String>,

    /// Extensions PVMT (Property Values Management Tool)
    #[serde(rename = "propertyValues", default)]
    pub property_values: Vec<ElementRef>,
}

/// Macro pour faciliter la composition des structures
/// CORRECTION : Ajout du support pour les attributs (#[serde...])
#[macro_export]
macro_rules! arcadia_element {
    (
        $name:ident {
            $(
                $(#[$meta:meta])* // Capture les attributs (ex: #[serde(rename = "...")])
                $field:ident : $type:ty
            ),* $(,)? // Virgule traînante optionnelle
        }
    ) => {
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
        pub struct $name {
            // Socle technique (ID, Dates...)
            #[serde(flatten)]
            pub base: $crate::model_engine::common::BaseEntity,

            // Socle métier (Nom, Desc, Tags...)
            #[serde(flatten)]
            pub props: $crate::model_engine::arcadia::metamodel::ArcadiaProperties,

            // Champs spécifiques déclarés dans l'appel
            $(
                $(#[$meta])* // Ré-applique les attributs capturés sur le champ
                pub $field: $type
            ),*
        }
    };
}
