use crate::ai::agents::intent_classifier::EngineeringIntent;
use serde_json::json;

#[test]
fn test_intent_deserialization_create_element() {
    // C'est le format exact que le prompt système demande au LLM de générer
    let llm_output = json!({
        "intent": "create_element",
        "params": {
            "layer": "OA",
            "element_type": "Actor",
            "name": "Opérateur"
        }
    });

    let intent: EngineeringIntent = serde_json::from_value(llm_output)
        .expect("Le JSON généré par le LLM doit correspondre à l'Enum Rust");

    match intent {
        EngineeringIntent::CreateElement {
            layer,
            element_type,
            name,
        } => {
            assert_eq!(layer, "OA");
            assert_eq!(element_type, "Actor");
            assert_eq!(name, "Opérateur");
        }
        _ => panic!("Mauvais type d'intention détecté"),
    }
}

#[test]
fn test_intent_deserialization_chat() {
    let llm_output = json!({
        "intent": "chat",
        "params": null
    });

    let intent: EngineeringIntent =
        serde_json::from_value(llm_output).expect("Le parsing du mode Chat a échoué");

    assert!(matches!(intent, EngineeringIntent::Chat));
}
