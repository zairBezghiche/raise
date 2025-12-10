use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// --- 1. Structures de Données Partagées (Le "Langage Commun") ---

/// Représente le modèle de données générique (ex: un graphe, un modèle Capella parsé)
/// Simplifié pour l'exemple, à enrichir avec vos vraies structures Arcadia.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CognitiveModel {
    pub id: String,
    pub elements: HashMap<String, ModelElement>,
    pub metadata: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModelElement {
    pub name: String,
    pub kind: String, // ex: "LogicalComponent", "PhysicalLink"
    pub properties: HashMap<String, String>,
}

/// Le rapport standard renvoyé par un bloc d'analyse
#[derive(Serialize, Deserialize, Debug)]
pub struct AnalysisReport {
    pub block_id: String,
    pub status: AnalysisStatus,
    pub messages: Vec<String>,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum AnalysisStatus {
    Success,
    Warning,
    Failure,
}

// --- 2. Le Trait "CognitiveBlock" (Le Contrat) ---

/// Tout plugin (Bloc) devra implémenter ce trait pour être utilisable par GenAptitude.
/// Note : Pour le WASM pur, nous exposerons des fonctions "extern C",
/// mais ce trait sert à structurer la logique interne Rust.
pub trait CognitiveBlock {
    /// Identifiant unique du bloc (ex: "fr.genaptitude.blocks.consistency")
    fn id(&self) -> &str;

    /// Exécute la logique du bloc sur un modèle donné
    fn execute(&self, model: &CognitiveModel) -> Result<AnalysisReport, String>;
}

// ... (votre code existant struct/trait) ...

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_model_serialization() {
        // 1. Création d'un modèle factice
        let mut elements = HashMap::new();
        elements.insert(
            "elt-1".to_string(),
            ModelElement {
                name: "Test Component".to_string(),
                kind: "LogicalComponent".to_string(),
                properties: HashMap::new(),
            },
        );

        let model = CognitiveModel {
            id: "model-001".to_string(),
            elements,
            metadata: HashMap::new(),
        };

        // 2. Sérialisation (Rust -> JSON)
        let json = serde_json::to_string(&model).expect("La sérialisation a échoué");

        // 3. Vérifications
        assert!(json.contains("model-001"));
        assert!(json.contains("Test Component"));
    }

    #[test]
    fn test_report_deserialization() {
        // Simulation d'un JSON reçu du WASM
        let json_input = r#"{
            "block_id": "test.block",
            "status": "Success",
            "messages": ["Ok"],
            "timestamp": 123456789
        }"#;

        // Désérialisation (JSON -> Rust)
        let report: AnalysisReport =
            serde_json::from_str(json_input).expect("Désérialisation échouée");

        match report.status {
            AnalysisStatus::Success => assert!(true),
            _ => panic!("Mauvais statut récupéré"),
        }
        assert_eq!(report.block_id, "test.block");
    }
}
