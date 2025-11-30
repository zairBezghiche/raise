use serde::{Deserialize, Serialize};

// --- Imports des modules de couches ---

// 1. Operational Analysis (OA)
// Pas de conflits ici, on peut importer les types directement
use super::arcadia::operational_analysis::{
    OperationalActivity, OperationalActor, OperationalCapability, OperationalEntity,
    OperationalExchange,
};

// 2. System Analysis (SA)
// Conflit potentiel sur FunctionalExchange -> on utilise un alias
use super::arcadia::system_analysis::{
    FunctionalExchange as SaFunctionalExchange, SystemActor, SystemCapability, SystemComponent,
    SystemFunction,
};

// 3. Logical Architecture (LA)
// Conflits sur FunctionalExchange et ComponentExchange -> alias
use super::arcadia::logical_architecture::{
    ComponentExchange as LaComponentExchange, FunctionalExchange as LaFunctionalExchange,
    LogicalActor, LogicalComponent, LogicalFunction, LogicalInterface,
};

// 4. Physical Architecture (PA)
// Conflit sur ComponentExchange -> alias
use super::arcadia::physical_architecture::{
    ComponentExchange as PaComponentExchange, PhysicalActor, PhysicalComponent, PhysicalFunction,
    PhysicalLink,
};

// 5. EPBS
use super::arcadia::epbs::ConfigurationItem;

/// Représentation complète d'un projet Arcadia en mémoire.
/// C'est l'objet racine retourné par la commande `load_project_model`.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ProjectModel {
    pub oa: OperationalAnalysisLayer,
    pub sa: SystemAnalysisLayer,
    pub la: LogicalArchitectureLayer,
    pub pa: PhysicalArchitectureLayer,
    pub epbs: EPBSLayer,

    /// Métadonnées globales (stats, version...)
    pub meta: ProjectMeta,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ProjectMeta {
    pub name: String,
    pub loaded_at: String,
    pub element_count: usize,
}

// --- Sous-structures pour organiser les données par couche ---

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct OperationalAnalysisLayer {
    pub actors: Vec<OperationalActor>,
    pub activities: Vec<OperationalActivity>,
    pub capabilities: Vec<OperationalCapability>,
    pub entities: Vec<OperationalEntity>,
    pub exchanges: Vec<OperationalExchange>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SystemAnalysisLayer {
    pub components: Vec<SystemComponent>, // Souvent un seul (le Système)
    pub actors: Vec<SystemActor>,
    pub functions: Vec<SystemFunction>,
    pub capabilities: Vec<SystemCapability>,

    // Utilisation de l'alias SA
    pub exchanges: Vec<SaFunctionalExchange>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct LogicalArchitectureLayer {
    pub components: Vec<LogicalComponent>,
    pub actors: Vec<LogicalActor>,
    pub functions: Vec<LogicalFunction>,
    pub interfaces: Vec<LogicalInterface>,

    // Utilisation des alias LA
    pub functional_exchanges: Vec<LaFunctionalExchange>,
    pub component_exchanges: Vec<LaComponentExchange>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PhysicalArchitectureLayer {
    pub components: Vec<PhysicalComponent>, // Node & Behavior
    pub actors: Vec<PhysicalActor>,
    pub functions: Vec<PhysicalFunction>,
    pub links: Vec<PhysicalLink>,

    // Utilisation de l'alias PA
    pub component_exchanges: Vec<PaComponentExchange>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct EPBSLayer {
    pub configuration_items: Vec<ConfigurationItem>,
}
