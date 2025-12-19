# Module Low-Level LLM 🧠

Ce module est la **couche d'abstraction bas niveau** responsable de la communication avec les modèles d'intelligence artificielle.
Il isole le reste de l'application Rust de la complexité des APIs tierces (OpenAI format, Google REST API) et assure la résilience du service.

---

## 🏗️ Architecture & Flux de Données

Le client implémente un pattern **"Smart Fallback"**. Il tente toujours de privilégier l'inférence locale (confidentialité, coût) mais bascule automatiquement et silencieusement vers le Cloud en cas d'indisponibilité.

```text
    [ Application Rust (Agents) ]
                 |
                 v
      +---------------------+
      |      LlmClient      |
      | (Interface Unifiée) |
      +---------------------+
                 |
        1. Tentative LOCAL
                 |
                 v
      /---------------------\
      |   API Locale (HTTP) | <--- Ping / Timeout (2s)
      \---------------------/
         |             |
     [Succès]      [Échec / 404]
         |             |
         |             v
         |     2. Bascule CLOUD (Fallback)
         |             |
         |             v
         |    /-----------------\
         |    |  Google Gemini  | (REST API v1beta)
         |    \-----------------/
         |             |
         |             |
         v             v
      +---------------------+
      |   Réponse Textuelle |
      +---------------------+
```

---

## 📂 Structure des Fichiers

| Fichier              | Responsabilité                                                                                                                                        |
| -------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| **`client.rs`**      | **Cœur du module**. Implémente `LlmClient`, la gestion HTTP (reqwest), la logique de fallback et le nettoyage des noms de modèles (`models/` prefix). |
| `response_parser.rs` | _(Utilitaire)_ Fonctions pour extraire et valider le JSON depuis les blocs de code Markdown (```json) renvoyés par les LLMs.                          |
| `prompts.rs`         | _(Utilitaire)_ Bibliothèque de prompts système (System Prompts) pour spécialiser l'IA (Expert Rust, Expert SQL, Architecte Arcadia).                  |
| `mod.rs`             | Point d'entrée du module, expose les types publics.                                                                                                   |
| `tests.rs`           | Tests d'intégration pour vérifier la connexion aux backends (Local et Cloud) et le parsing.                                                           |

---

## 🚀 Fonctionnalités Clés

### 1. Smart Fallback (Résilience)

Le système est conçu pour le développement hybride :

- **Mode Local (`LocalLlama`)** : Cible par défaut (ex: `localhost:8080/v1/...`). Idéal pour le dev hors-ligne ou la confidentialité.
- **Mode Cloud (`GoogleGemini`)** : S'active si le serveur local ne répond pas sous 2 secondes. Utilise l'API Google Generative Language.

### 2. Normalisation des Modèles Gemini

Le client gère automatiquement les incohérences de nommage de l'API Google.

- Entrée config : `models/gemini-1.5-flash` ou `gemini-1.5-flash`
- Traitement interne : Nettoie le préfixe `models/` pour construire une URL API valide (`.../models/gemini-1.5-flash:generateContent`).

### 3. Typage Fort

Utilise des structures Rust (`struct`) pour sérialiser/désérialiser proprement les requêtes JSON, garantissant que les payloads envoyés à OpenAI ou Google sont toujours conformes.

---

## ⚙️ Configuration

Le client est instancié avec des paramètres provenant généralement des variables d'environnement (`.env`) chargées par le binaire principal.

| Variable Env             | Usage                                                              |
| ------------------------ | ------------------------------------------------------------------ |
| `GENAPTITUDE_MODEL_NAME` | Nom du modèle (ex: `gemini-2.0-flash-001`). Le préfixe est géré.   |
| `GENAPTITUDE_GEMINI_KEY` | Clé API Google (commence par `AIza...`).                           |
| `GENAPTITUDE_LOCAL_URL`  | URL du serveur d'inférence local (ex: `http://localhost:1234/v1`). |

---

## 💻 Exemple d'Utilisation (Rust)

```rust
use crate::ai::llm::client::{LlmClient, LlmBackend};

async fn example() {
    // 1. Instanciation
    let client = LlmClient::new(
        "http://localhost:1234",
        "AIzaSy...",
        Some("gemini-1.5-flash".to_string())
    );

    // 2. Appel (Le fallback est géré en interne si LocalLlama est choisi)
    let reponse = client.ask(
        LlmBackend::LocalLlama, // Tente le local d'abord
        "Tu es un expert Rust.", // System Prompt
        "Génère une struct Client." // User Prompt
    ).await;

    match reponse {
        Ok(text) => println!("Réponse IA : {}", text),
        Err(e) => eprintln!("Erreur critique : {}", e),
    }
}

```
