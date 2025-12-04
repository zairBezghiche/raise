use crate::ai::llm::client::{LlmBackend, LlmClient};

#[test]
fn test_client_instantiation() {
    let _client = LlmClient::new("http://localhost:1234", "dummy-key", None);
    // Le test passe si le client est créé sans paniquer.
    // En Rust, on teste souvent que la logique de construction (trim url, etc.) est saine.
}

#[tokio::test]
#[ignore] // Ce test est ignoré par défaut car il nécessite que Docker tourne
async fn integration_test_local_ping() {
    let client = LlmClient::new("http://localhost:8080", "dummy", None);
    let is_alive = client.ping_local().await;
    assert!(
        is_alive,
        "Le serveur local devrait répondre (Docker lancé ?)"
    );
}

#[tokio::test]
#[ignore] // Ignoré car nécessite une vraie clé API
async fn integration_test_gemini_call() {
    // Récupère la clé depuis l'env pour le test local
    let key = std::env::var("GENAPTITUDE_GEMINI_KEY").unwrap_or_default();
    if key.is_empty() {
        return;
    }

    let client = LlmClient::new(
        "http://localhost:8080",
        &key,
        Some("gemini-1.5-flash".to_string()),
    );
    let res = client
        .ask(
            LlmBackend::GoogleGemini,
            "Tu es un test.",
            "Réponds 'OK' si tu m'entends.",
        )
        .await;

    assert!(res.is_ok());
    println!("Réponse Gemini: {}", res.unwrap());
}
