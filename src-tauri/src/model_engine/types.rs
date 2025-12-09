// FICHIER : src-tauri/src/model_engine/types.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// --- Enums & Types de base ---

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum NameType {
    String(String),
    I18n(HashMap<String, String>),
}

impl Default for NameType {
    fn default() -> Self {
        NameType::String("Sans nom".to_string())
    }
}

impl NameType {
    pub fn as_str(&self) -> &str {
        match self {
            NameType::String(s) => s,
            NameType::I18n(map) => map
                .get("fr")
                .or_else(|| map.get("en"))
                .map(|s| s.as_str())
                .unwrap_or("Sans nom"),
        }
    }
}

// --- Structure Élémentaire ---

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ArcadiaElement {
    pub id: String,
    pub name: NameType, // Le fameux changement de type
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub properties: HashMap<String, serde_json::Value>,
}

// --- Couches ---

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OperationalAnalysisLayer {
    #[serde(default)]
    pub actors: Vec<ArcadiaElement>,
    #[serde(default)]
    pub activities: Vec<ArcadiaElement>,
    #[serde(default)]
    pub capabilities: Vec<ArcadiaElement>,
    #[serde(default)]
    pub entities: Vec<ArcadiaElement>,
    #[serde(default)]
    pub exchanges: Vec<ArcadiaElement>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SystemAnalysisLayer {
    #[serde(default)]
    pub components: Vec<ArcadiaElement>,
    #[serde(default)]
    pub actors: Vec<ArcadiaElement>,
    #[serde(default)]
    pub functions: Vec<ArcadiaElement>,
    #[serde(default)]
    pub capabilities: Vec<ArcadiaElement>,
    #[serde(default)]
    pub exchanges: Vec<ArcadiaElement>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LogicalArchitectureLayer {
    #[serde(default)]
    pub components: Vec<ArcadiaElement>,
    #[serde(default)]
    pub actors: Vec<ArcadiaElement>,
    #[serde(default)]
    pub functions: Vec<ArcadiaElement>,
    #[serde(default)]
    pub interfaces: Vec<ArcadiaElement>,
    #[serde(default)]
    pub exchanges: Vec<ArcadiaElement>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PhysicalArchitectureLayer {
    #[serde(default)]
    pub components: Vec<ArcadiaElement>,
    #[serde(default)]
    pub actors: Vec<ArcadiaElement>,
    #[serde(default)]
    pub functions: Vec<ArcadiaElement>,
    #[serde(default)]
    pub links: Vec<ArcadiaElement>,
    #[serde(default)]
    pub exchanges: Vec<ArcadiaElement>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EPBSLayer {
    #[serde(default)]
    pub configuration_items: Vec<ArcadiaElement>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DataLayer {
    #[serde(default)]
    pub classes: Vec<ArcadiaElement>,
    #[serde(default)]
    pub data_types: Vec<ArcadiaElement>,
    #[serde(default)]
    pub exchange_items: Vec<ArcadiaElement>,
}

// --- Racine ---

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMeta {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub loaded_at: String,
    #[serde(default)]
    pub element_count: usize,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProjectModel {
    #[serde(default)]
    pub oa: OperationalAnalysisLayer,
    #[serde(default)]
    pub sa: SystemAnalysisLayer,
    #[serde(default)]
    pub la: LogicalArchitectureLayer,
    #[serde(default)]
    pub pa: PhysicalArchitectureLayer,
    #[serde(default)]
    pub epbs: EPBSLayer,
    #[serde(default)]
    pub data: DataLayer,
    #[serde(default)]
    pub meta: ProjectMeta,
}
