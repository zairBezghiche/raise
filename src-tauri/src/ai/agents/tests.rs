use crate::ai::agents::intent_classifier::EngineeringIntent;
use serde_json::json;

// =============================================================================
// TEST 1 : ANALYSE MÉTIER (OA)
// =============================================================================
#[test]
fn test_intent_deserialization_business_use_case() {
    // Structure plate (Flattened) : Pas de champ "params"
    let llm_output = json!({
        "intent": "define_business_use_case",
        "domain": "Ressources Humaines",
        "process_name": "Gestion Congés",
        "description": "Le salarié pose ses congés..."
    });

    let intent: EngineeringIntent =
        serde_json::from_value(llm_output).expect("Désérialisation BusinessUseCase échouée");

    if let EngineeringIntent::DefineBusinessUseCase {
        domain,
        process_name,
        ..
    } = intent
    {
        assert_eq!(domain, "Ressources Humaines");
        assert_eq!(process_name, "Gestion Congés");
    } else {
        panic!("Mauvais mapping pour DefineBusinessUseCase");
    }
}

// =============================================================================
// TEST 2 : INGÉNIERIE SYSTÈME & LOGICIELLE (SA / LA)
// =============================================================================
#[test]
fn test_intent_deserialization_create_element_sa() {
    let llm_output = json!({
        "intent": "create_element",
        "layer": "SA",
        "element_type": "Function",
        "name": "Démarrer Moteur"
    });

    let intent: EngineeringIntent = serde_json::from_value(llm_output).unwrap();

    match intent {
        EngineeringIntent::CreateElement {
            layer,
            element_type,
            name,
        } => {
            assert_eq!(layer, "SA");
            assert_eq!(element_type, "Function");
            assert_eq!(name, "Démarrer Moteur");
        }
        _ => panic!("Échec match SA"),
    }
}

// =============================================================================
// TEST 3 : HARDWARE (PA)
// =============================================================================
#[test]
fn test_intent_deserialization_hardware() {
    let llm_output = json!({
        "intent": "create_element",
        "layer": "PA",
        "element_type": "PhysicalNode",
        "name": "Carte FPGA"
    });

    let intent: EngineeringIntent = serde_json::from_value(llm_output).unwrap();

    if let EngineeringIntent::CreateElement { layer, .. } = intent {
        assert_eq!(layer, "PA");
    } else {
        panic!("Échec match PA");
    }
}

// =============================================================================
// TEST 4 : EPBS (Configuration Item)
// =============================================================================
#[test]
fn test_intent_deserialization_epbs() {
    let llm_output = json!({
        "intent": "create_element",
        "layer": "EPBS",
        "element_type": "ConfigurationItem",
        "name": "Rack Serveur"
    });

    let intent: EngineeringIntent = serde_json::from_value(llm_output).unwrap();

    if let EngineeringIntent::CreateElement {
        layer,
        element_type,
        ..
    } = intent
    {
        assert_eq!(layer, "EPBS");
        assert_eq!(element_type, "ConfigurationItem");
    } else {
        panic!("Échec match EPBS");
    }
}

// =============================================================================
// TEST 5 : DATA (Classes & Types)
// =============================================================================
#[test]
fn test_intent_deserialization_data() {
    // Cas 1: Classe
    let json_class = json!({
        "intent": "create_element",
        "layer": "DATA",
        "element_type": "Class",
        "name": "Client"
    });
    let intent_class: EngineeringIntent = serde_json::from_value(json_class).unwrap();
    assert!(
        matches!(intent_class, EngineeringIntent::CreateElement { layer, .. } if layer == "DATA")
    );

    // Cas 2: DataType
    let json_type = json!({
        "intent": "create_element",
        "layer": "DATA",
        "element_type": "DataType",
        "name": "Vitesse"
    });
    let intent_type: EngineeringIntent = serde_json::from_value(json_type).unwrap();
    assert!(
        matches!(intent_type, EngineeringIntent::CreateElement { element_type, .. } if element_type == "DataType")
    );
}

// =============================================================================
// TEST 6 : TRANSVERSE (Exigences & Tests)
// =============================================================================
#[test]
fn test_intent_deserialization_transverse() {
    // Cas 1: Exigence
    let json_req = json!({
        "intent": "create_element",
        "layer": "TRANSVERSE",
        "element_type": "Requirement",
        "name": "Latence Max"
    });
    let intent_req: EngineeringIntent = serde_json::from_value(json_req).unwrap();

    match intent_req {
        EngineeringIntent::CreateElement {
            layer,
            element_type,
            name,
        } => {
            assert_eq!(layer, "TRANSVERSE");
            assert_eq!(element_type, "Requirement");
            assert_eq!(name, "Latence Max");
        }
        _ => panic!("Échec match Requirement"),
    }

    // Cas 2: Test Procedure
    let json_test = json!({
        "intent": "create_element",
        "layer": "TRANSVERSE",
        "element_type": "TestProcedure",
        "name": "Test de Charge"
    });
    let intent_test: EngineeringIntent = serde_json::from_value(json_test).unwrap();
    assert!(
        matches!(intent_test, EngineeringIntent::CreateElement { element_type, .. } if element_type == "TestProcedure")
    );
}

// =============================================================================
// TEST 7 : GÉNÉRATION DE CODE
// =============================================================================
#[test]
fn test_intent_deserialization_generate_code() {
    // Note: Le champ 'context' peut être mappé depuis "content" ou "code" grâce aux alias serde
    let llm_output = json!({
        "intent": "generate_code",
        "language": "Rust",
        "filename": "main.rs",
        "code": "Contexte du code à générer..."
    });

    let intent: EngineeringIntent = serde_json::from_value(llm_output).unwrap();

    match intent {
        EngineeringIntent::GenerateCode {
            language,
            filename,
            context,
        } => {
            assert_eq!(language, "Rust");
            assert_eq!(filename, "main.rs");
            assert_eq!(context, "Contexte du code à générer...");
        }
        _ => panic!("Échec match GenerateCode"),
    }
}

// =============================================================================
// TEST 8 : MODE CHAT (RAG)
// =============================================================================
#[test]
fn test_intent_deserialization_chat() {
    // Le mode chat n'a pas de paramètres, mais le JSON peut contenir des champs nulls
    let llm_output = json!({
        "intent": "chat",
        "arg1": null
    });

    let intent: EngineeringIntent = serde_json::from_value(llm_output).expect("Chat failed");
    assert!(matches!(intent, EngineeringIntent::Chat));
}
