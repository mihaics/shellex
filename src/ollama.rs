// Ollama API wrapper
use anyhow::{bail, Context, Result};
use ollama_rs::generation::completion::request::GenerationRequest;
use ollama_rs::Ollama;

pub struct OllamaClient {
    client: Ollama,
    model: String,
    host: String,
    port: u16,
}

impl OllamaClient {
    pub fn new(url: &str, model: &str) -> Result<Self> {
        // Parse host and port from URL like "http://localhost:11434"
        let url_trimmed = url.trim_end_matches('/');
        let (host, port) = if let Some(last_colon) = url_trimmed.rfind(':') {
            let potential_port = &url_trimmed[last_colon + 1..];
            if let Ok(p) = potential_port.parse::<u16>() {
                (url_trimmed[..last_colon].to_string(), p)
            } else {
                (url_trimmed.to_string(), 11434)
            }
        } else {
            (url_trimmed.to_string(), 11434)
        };

        let client = Ollama::new(host.clone(), port);
        Ok(Self {
            client,
            model: model.to_string(),
            host,
            port,
        })
    }

    pub async fn generate(&self, system_prompt: &str, user_prompt: &str) -> Result<String> {
        let request = GenerationRequest::new(self.model.clone(), user_prompt.to_string())
            .system(system_prompt.to_string());

        let response = self
            .client
            .generate(request)
            .await
            .with_context(|| format!(
                "Error: Cannot connect to Ollama at {}:{}. Is it running? (ollama serve)",
                self.host, self.port
            ))?;

        if response.response.trim().is_empty() {
            bail!("Error: Model returned empty response. Try a different model or rephrase.");
        }

        Ok(response.response)
    }

    pub fn url_display(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
