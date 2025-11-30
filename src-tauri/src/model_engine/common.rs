use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Identifiant unique (UUID v4)
pub type Uuid = String;

/// Chaîne internationalisée (FR/EN)
/// Correspond à `i18nString` du schéma JSON
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum I18nString {
    /// Cas simple : une seule chaîne (langue par défaut)
    Simple(String),
    /// Cas complet : dictionnaire par langue
    Localized(HashMap<String, String>),
}

impl Default for I18nString {
    fn default() -> Self {
        I18nString::Simple(String::new())
    }
}

/// Référence vers un autre élément (URI ou UUID)
/// Correspond à `Ref` du schéma
pub type ElementRef = String;

/// Socle technique commun à toutes les entités
/// Correspond à `base.schema.json`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseEntity {
    #[serde(rename = "$schema", skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,

    #[serde(default)]
    pub id: Uuid,

    #[serde(rename = "createdAt")]
    pub created_at: String, // RFC3339

    #[serde(rename = "updatedAt")]
    pub updated_at: String, // RFC3339
}
