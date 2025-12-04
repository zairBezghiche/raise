use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;

// --- ENUMS & CONFIGURATION ---

/// Choix du moteur d'intelligence
#[derive(Debug, Clone, Copy)]
pub enum LlmBackend {
    /// Inférence locale (Rapide, Gratuit, Privé)
    LocalLlama,
    /// Inférence Cloud (Puissant, Raisonnement complexe)
    GoogleGemini,
}

// --- DTOs COMMUNS (Interne) ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

// --- DTOs SPECIFIQUES OPENAI / LLAMA.CPP ---

#[derive(Debug, Serialize)]
struct OpenAiRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct OpenAiResponse {
    choices: Vec<OpenAiChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAiChoice {
    message: Message,
    #[serde(default)]
    #[allow(dead_code)]
    finish_reason: Option<String>,
}

// --- DTOs SPECIFIQUES GOOGLE GEMINI ---

#[derive(Debug, Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_instruction: Option<GeminiPartWrapper>,
    generation_config: GeminiConfig,
}

// CORRECTION ICI : Ajout de Deserialize car utilisé dans la réponse
#[derive(Debug, Serialize, Deserialize)]
struct GeminiContent {
    role: String,
    parts: Vec<GeminiPart>,
}

// CORRECTION ICI : Ajout de Deserialize par sécurité/cohérence
#[derive(Debug, Serialize, Deserialize)]
struct GeminiPartWrapper {
    parts: GeminiPart,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiPart {
    text: String,
}

#[derive(Debug, Serialize)]
struct GeminiConfig {
    temperature: f32,
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<GeminiCandidate>>,
}

#[derive(Debug, Deserialize)]
struct GeminiCandidate {
    content: GeminiContent,
}

// --- CLIENT IA UNIFIÉ ---

#[derive(Clone, Debug)]
pub struct LlmClient {
    // Config Local
    local_url: String,
    local_model: String,

    // Config Gemini
    gemini_api_key: String,
    gemini_model: String,

    // HTTP Client partagé
    http_client: reqwest::Client,
}

impl LlmClient {
    /// Constructeur Dual-Mode
    pub fn new(local_url: &str, gemini_key: &str, gemini_model_name: Option<String>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(300))
            .build()
            .unwrap_or_default();

        // Utilise le modèle demandé ou une valeur par défaut sûre
        let cloud_model = gemini_model_name.unwrap_or_else(|| "gemini-1.5-pro".to_string());

        Self {
            local_url: local_url.trim_end_matches('/').to_string(),
            local_model: "mistral-7b".to_string(),
            gemini_api_key: gemini_key.to_string(),
            gemini_model: cloud_model,
            http_client: client,
        }
    }

    /// Point d'entrée unique : route vers le bon backend
    pub async fn ask(
        &self,
        backend: LlmBackend,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<String> {
        match backend {
            LlmBackend::LocalLlama => self.call_local_llama(system_prompt, user_prompt).await,
            LlmBackend::GoogleGemini => self.call_google_gemini(system_prompt, user_prompt).await,
        }
    }

    // --- IMPLÉMENTATION LOCALE (OpenAI Compatible) ---
    async fn call_local_llama(&self, system: &str, user: &str) -> Result<String> {
        let endpoint = format!("{}/v1/chat/completions", self.local_url);

        let messages = vec![
            Message {
                role: "system".to_string(),
                content: system.to_string(),
            },
            Message {
                role: "user".to_string(),
                content: user.to_string(),
            },
        ];

        let body = OpenAiRequest {
            model: self.local_model.clone(),
            messages,
            temperature: 0.7,
            stream: false,
        };

        let res = self.http_client.post(&endpoint).json(&body).send().await?;

        if !res.status().is_success() {
            return Err(anyhow::anyhow!("Erreur Llama Local: {}", res.text().await?));
        }

        let data: OpenAiResponse = res.json().await?;
        Ok(data
            .choices
            .first()
            .context("Pas de réponse locale")?
            .message
            .content
            .clone())
    }

    // --- IMPLÉMENTATION GOOGLE (Gemini REST API) ---
    async fn call_google_gemini(&self, system: &str, user: &str) -> Result<String> {
        if self.gemini_api_key.is_empty() || self.gemini_api_key == "YOUR_GEMINI_KEY" {
            return Err(anyhow::anyhow!("Clé API Gemini non configurée"));
        }

        let endpoint = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.gemini_model, self.gemini_api_key
        );

        // Gemini structure
        let body = GeminiRequest {
            system_instruction: Some(GeminiPartWrapper {
                parts: GeminiPart {
                    text: system.to_string(),
                },
            }),
            contents: vec![GeminiContent {
                role: "user".to_string(),
                parts: vec![GeminiPart {
                    text: user.to_string(),
                }],
            }],
            generation_config: GeminiConfig { temperature: 0.3 },
        };

        let res = self.http_client.post(&endpoint).json(&body).send().await?;

        if !res.status().is_success() {
            return Err(anyhow::anyhow!(
                "Erreur Gemini Cloud: {}",
                res.text().await?
            ));
        }

        let data: GeminiResponse = res.json().await?;

        if let Some(candidates) = data.candidates {
            if let Some(first) = candidates.first() {
                if let Some(part) = first.content.parts.first() {
                    return Ok(part.text.clone());
                }
            }
        }

        Err(anyhow::anyhow!("Gemini a répondu mais sans contenu texte."))
    }

    pub async fn ping_local(&self) -> bool {
        self.http_client.get(&self.local_url).send().await.is_ok()
    }
}
