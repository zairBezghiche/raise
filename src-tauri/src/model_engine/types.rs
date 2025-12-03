// FICHIER : src-tauri/src/model_engine/types.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// La racine d'un projet Arcadia chargé en mémoire
/// Ce modèle agrège toutes les données sans perte d'information (properties: HashMap)
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ProjectModel {
    #[serde(default)]
    pub oa: OperationalAnalysis,
    #[serde(default)]
    pub sa: SystemAnalysis,
    #[serde(default)]
    pub la: LogicalArchitecture,
    #[serde(default)]
    pub pa: PhysicalArchitecture,
    #[serde(default)]
    pub epbs: EPBS,

    /// Métadonnées globales du projet
    #[serde(default)]
    pub meta: ProjectMeta,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ProjectMeta {
    pub element_count: usize,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct OperationalAnalysis {
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

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SystemAnalysis {
    #[serde(default)]
    pub functions: Vec<ArcadiaElement>,
    #[serde(default)]
    pub actors: Vec<ArcadiaElement>,
    #[serde(default)]
    pub capabilities: Vec<ArcadiaElement>,

    // AJOUT : Ce champ manquait pour le SystemComponent (le "Système")
    #[serde(default)]
    pub components: Vec<ArcadiaElement>,

    #[serde(default)]
    pub exchanges: Vec<ArcadiaElement>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct LogicalArchitecture {
    #[serde(default)]
    pub components: Vec<ArcadiaElement>,
    #[serde(default)]
    pub functions: Vec<ArcadiaElement>,
    #[serde(default)]
    pub actors: Vec<ArcadiaElement>,
    #[serde(default)]
    pub interfaces: Vec<ArcadiaElement>,
    #[serde(default)]
    pub exchanges: Vec<ArcadiaElement>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct PhysicalArchitecture {
    #[serde(default)]
    pub components: Vec<ArcadiaElement>, // Node & Behavior
    #[serde(default)]
    pub functions: Vec<ArcadiaElement>,
    #[serde(default)]
    pub actors: Vec<ArcadiaElement>,
    #[serde(default)]
    pub links: Vec<ArcadiaElement>,
    #[serde(default)]
    pub exchanges: Vec<ArcadiaElement>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct EPBS {
    #[serde(default)]
    pub configuration_items: Vec<ArcadiaElement>,
}

/// Élément générique Arcadia (Nœud du graphe)
/// Cette structure est flexible et peut accueillir n'importe quel type JSON-LD.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ArcadiaElement {
    pub id: String,
    pub name: String,

    /// Type sémantique complet (URI, ex: "https://...#OperationalActor")
    #[serde(rename = "type")]
    pub kind: String,

    /// Propriétés dynamiques (champs métiers, relations, extensions PVMT)
    #[serde(flatten)]
    pub properties: HashMap<String, serde_json::Value>,
}
