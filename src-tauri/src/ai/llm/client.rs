use reqwest::Client;
use serde_json::{json, Value};
use std::time::Duration;

// ✅ AJOUT DE CLONE ICI
#[derive(Clone, Debug)]
pub enum LlmBackend {
    LocalLlama,
    GoogleGemini,
}

// ✅ AJOUT DE CLONE ICI (Indispensable pour vos Agents)
#[derive(Clone)]
pub struct LlmClient {
    local_url: String,
    gemini_key: String,
    model_name: String,
    http_client: Client,
}

impl LlmClient {
    pub fn new(local_url: &str, gemini_key: &str, model_name: Option<String>) -> Self {
        let raw_model = model_name.unwrap_or_else(|| "gemini-1.5-flash-latest".to_string());
        // Nettoyage préventif
        let clean_model = raw_model.replace("models/", "");

        LlmClient {
            local_url: local_url.to_string(),
            gemini_key: gemini_key.to_string(),
            model_name: clean_model,
            http_client: Client::new(),
        }
    }

    pub async fn ping_local(&self) -> bool {
        let url = format!("{}/models", self.local_url);
        match self
            .http_client
            .get(&url)
            .timeout(Duration::from_secs(1))
            .send()
            .await
        {
            Ok(res) => res.status().is_success(),
            Err(_) => false,
        }
    }

    // --- LOGIQUE DE RETRY (Anti-Crash Quota) ---
    pub async fn ask(
        &self,
        backend: LlmBackend,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<String, String> {
        let max_retries = 5;
        let mut attempt = 0;

        loop {
            attempt += 1;

            // On passe 'backend' par référence ou clone selon besoin
            let result = self
                .ask_internal(&backend, system_prompt, user_prompt)
                .await;

            match result {
                Ok(response) => return Ok(response),
                Err(err) => {
                    // Si c'est une 404 (Erreur de nom), on ne réessaie pas, on plante direct pour vous avertir.
                    if err.contains("404") || err.contains("NOT_FOUND") {
                        return Err(format!("❌ ERREUR CONFIGURATION : {}", err));
                    }

                    // Si c'est le Quota (429), on attend.
                    let is_quota = err.contains("429")
                        || err.contains("RESOURCE_EXHAUSTED")
                        || err.contains("quota");

                    if !is_quota || attempt >= max_retries {
                        return Err(err);
                    }

                    let wait = Duration::from_secs(2u64.pow(attempt));
                    println!(
                        "⚠️ Quota API atteint (Tentative {}/{}). Pause de {}s...",
                        attempt,
                        max_retries,
                        wait.as_secs()
                    );
                    tokio::time::sleep(wait).await;
                }
            }
        }
    }

    async fn ask_internal(
        &self,
        backend: &LlmBackend,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<String, String> {
        match backend {
            LlmBackend::LocalLlama => {
                if self.ping_local().await {
                    return self
                        .call_openai_format(&self.local_url, system_prompt, user_prompt)
                        .await;
                }
                println!("⚠️ Local LLM indisponible, bascule sur Gemini...");
                self.call_google_gemini(system_prompt, user_prompt).await
            }
            LlmBackend::GoogleGemini => self.call_google_gemini(system_prompt, user_prompt).await,
        }
    }

    async fn call_openai_format(
        &self,
        base_url: &str,
        sys: &str,
        user: &str,
    ) -> Result<String, String> {
        let url = format!("{}/chat/completions", base_url);
        let body = json!({
            "model": "local-model",
            "messages": [
                { "role": "system", "content": sys },
                { "role": "user", "content": user }
            ],
            "temperature": 0.7
        });

        let res = self
            .http_client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if !res.status().is_success() {
            return Err(format!("Erreur HTTP Local: {}", res.status()));
        }
        let json: Value = res.json().await.map_err(|e| e.to_string())?;
        json["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "Réponse locale malformée".to_string())
    }

    async fn call_google_gemini(&self, sys: &str, user: &str) -> Result<String, String> {
        // Construction URL propre
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model_name, self.gemini_key
        );

        let combined_prompt = format!("{}\n\nInstruction Utilisateur:\n{}", sys, user);
        let body = json!({ "contents": [{ "parts": [{ "text": combined_prompt }] }] });

        let res = self
            .http_client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let status = res.status();
        let text_response = res.text().await.unwrap_or_default();

        if !status.is_success() {
            // On renvoie l'erreur brute pour l'analyse (404, 429...)
            return Err(format!("Erreur Gemini Cloud: {}", text_response));
        }

        let json: Value = serde_json::from_str(&text_response).map_err(|e| e.to_string())?;
        if let Some(candidates) = json.get("candidates") {
            if let Some(first) = candidates.get(0) {
                if let Some(content) = first.get("content") {
                    if let Some(parts) = content.get("parts") {
                        if let Some(text) = parts[0].get("text") {
                            return Ok(text.as_str().unwrap_or("").to_string());
                        }
                    }
                }
            }
        }
        Err("Structure JSON Gemini inattendue".to_string())
    }
}
