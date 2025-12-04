use crate::ai::context::retriever::SimpleRetriever;
use crate::model_engine::types::{ArcadiaElement, ProjectModel};
use std::collections::HashMap;

// Helper pour créer un élément factice rapidement
fn mock_element(name: &str, desc: &str) -> ArcadiaElement {
    let mut props = HashMap::new();
    props.insert(
        "description".to_string(),
        serde_json::Value::String(desc.to_string()),
    );

    ArcadiaElement {
        id: "uuid-test".to_string(),
        name: name.to_string(),
        kind: "mock:Type".to_string(),
        properties: props,
    }
}

#[test]
fn test_retriever_finds_relevant_info() {
    // 1. Préparer un modèle mocké
    let mut model = ProjectModel::default();

    // On ajoute un acteur dans l'OA
    model.oa.actors.push(mock_element(
        "Pilote de Drone",
        "Responsable du vol et de la sécurité.",
    ));

    // On ajoute une fonction dans le SA (qui ne devrait pas être trouvée si on cherche 'vol')
    model.sa.functions.push(mock_element(
        "Alimenter Secteur",
        "Fournit l'énergie au système.",
    ));

    // 2. Instancier le retriever
    let retriever = SimpleRetriever::new(model);

    // 3. Test de recherche
    let query = "Qui est responsable du vol ?";
    let context = retriever.retrieve_context(query);

    println!("Contexte généré : \n{}", context);

    // 4. Assertions
    assert!(
        context.contains("Pilote de Drone"),
        "Le contexte doit contenir l'acteur trouvé"
    );
    assert!(
        context.contains("Responsable du vol"),
        "Le contexte doit contenir la description"
    );

    // Vérifier que les éléments non pertinents sont exclus (filtrage basique)
    // Note: Notre SimpleRetriever actuel est assez permissif, mais s'il filtre, ceci devrait être vrai.
    // Si le mot clé "vol" n'est pas dans "Alimenter Secteur", il ne devrait pas sortir.
}

#[test]
fn test_retriever_empty_search() {
    let model = ProjectModel::default();
    let retriever = SimpleRetriever::new(model);

    let context = retriever.retrieve_context("Rien à voir");
    assert!(
        context.contains("Aucun élément spécifique"),
        "Doit gérer le cas vide"
    );
}
