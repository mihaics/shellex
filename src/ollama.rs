// Ollama-compatible chat API client (/api/chat works on Ollama and compatible servers)
use anyhow::{bail, Context, Result};
use serde::Deserialize;
use serde_json::json;
use std::io::{stderr, IsTerminal, Write};
use std::time::Duration;

const REQUEST_TIMEOUT_SECS: u64 = 120;

pub struct OllamaClient {
    http: reqwest::Client,
    base_url: String,
    model: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    message: ChatMessage,
}

#[derive(Deserialize)]
struct ChatMessage {
    content: String,
}

/// Shows an hourglass on stderr while a request is in flight; clears on drop
/// so the indicator disappears on both success and error paths.
struct WaitIndicator(bool);

impl WaitIndicator {
    fn start() -> Self {
        let show = stderr().is_terminal();
        if show {
            eprint!("\u{23f3}");
            stderr().flush().ok();
        }
        WaitIndicator(show)
    }
}

impl Drop for WaitIndicator {
    fn drop(&mut self) {
        if self.0 {
            eprint!("\r\x1b[K");
            stderr().flush().ok();
        }
    }
}

fn timeout_error(base_url: &str) -> anyhow::Error {
    anyhow::anyhow!(
        "Error: Request to {} timed out after {}s. Model too slow or server hung.",
        base_url,
        REQUEST_TIMEOUT_SECS
    )
}

impl OllamaClient {
    pub fn new(url: &str, model: &str) -> Result<Self> {
        let http = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .build()
            .context("Error: Failed to build HTTP client")?;
        Ok(Self {
            http,
            base_url: url.trim_end_matches('/').to_string(),
            model: model.to_string(),
        })
    }

    pub async fn generate(&self, system_prompt: &str, user_prompt: &str) -> Result<String> {
        let body = json!({
            "model": self.model,
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": user_prompt},
            ],
            "stream": false,
        });

        let _wait = WaitIndicator::start();
        let response = match self
            .http
            .post(format!("{}/api/chat", self.base_url))
            .json(&body)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) if e.is_timeout() => return Err(timeout_error(&self.base_url)),
            Err(e) => {
                return Err(e).with_context(|| {
                    format!(
                        "Error: Cannot connect to Ollama at {}. Is it running? (ollama serve)",
                        self.base_url
                    )
                })
            }
        };

        if !response.status().is_success() {
            let status = response.status();
            // Best-effort body read: on failure we still report the status.
            let text = response.text().await.unwrap_or_default();
            bail!("Error: Server returned {}: {}", status, text.trim());
        }

        let parsed: ChatResponse = match response.json().await {
            Ok(p) => p,
            Err(e) if e.is_timeout() => return Err(timeout_error(&self.base_url)),
            Err(e) => return Err(e).context("Error: Unexpected response from /api/chat"),
        };

        if parsed.message.content.trim().is_empty() {
            bail!("Error: Model returned empty response. Try a different model or rephrase.");
        }

        Ok(parsed.message.content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    #[test]
    fn test_new_trims_trailing_slash() {
        let client = OllamaClient::new("http://localhost:11434/", "m").unwrap();
        assert_eq!(client.base_url, "http://localhost:11434");
    }

    #[test]
    fn test_chat_response_deserializes() {
        let json = r#"{"model":"m","message":{"role":"assistant","content":"ls -la"},"done":true}"#;
        let parsed: ChatResponse = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.message.content, "ls -la");
    }

    /// One-shot HTTP server: accepts a single connection, reads the request,
    /// writes `raw` as the response, then closes. Returns the base URL.
    async fn one_shot_server(raw: String) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let (mut sock, _) = listener.accept().await.unwrap();
            let mut buf = vec![0u8; 65536];
            let _ = sock.read(&mut buf).await.unwrap();
            sock.write_all(raw.as_bytes()).await.unwrap();
            sock.shutdown().await.ok();
        });
        format!("http://{}", addr)
    }

    fn http_200(json_body: &str) -> String {
        format!(
            "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
            json_body.len(),
            json_body
        )
    }

    #[tokio::test]
    async fn test_generate_happy_path() {
        let body = r#"{"message":{"role":"assistant","content":"ls -la"},"done":true}"#;
        let url = one_shot_server(http_200(body)).await;
        let client = OllamaClient::new(&url, "m").unwrap();
        let out = client.generate("sys", "user").await.unwrap();
        assert_eq!(out, "ls -la");
    }

    #[tokio::test]
    async fn test_generate_empty_content_errors() {
        let body = r#"{"message":{"role":"assistant","content":"  "},"done":true}"#;
        let url = one_shot_server(http_200(body)).await;
        let client = OllamaClient::new(&url, "m").unwrap();
        let err = client.generate("sys", "user").await.unwrap_err();
        assert!(err.to_string().contains("empty response"));
    }

    #[tokio::test]
    async fn test_generate_http_error_status() {
        let raw =
            "HTTP/1.1 404 Not Found\r\ncontent-length: 15\r\nconnection: close\r\n\r\nmodel not found"
                .to_string();
        let url = one_shot_server(raw).await;
        let client = OllamaClient::new(&url, "m").unwrap();
        let err = client.generate("sys", "user").await.unwrap_err();
        assert!(err.to_string().contains("404"));
    }

    #[tokio::test]
    async fn test_generate_malformed_json_errors() {
        let url = one_shot_server(http_200("not json")).await;
        let client = OllamaClient::new(&url, "m").unwrap();
        let err = client.generate("sys", "user").await.unwrap_err();
        assert!(err.to_string().contains("Unexpected response"));
    }

    #[tokio::test]
    async fn test_generate_connection_refused_errors() {
        // Bind-then-drop to get a port nothing listens on.
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        drop(listener);
        let client = OllamaClient::new(&format!("http://{}", addr), "m").unwrap();
        let err = client.generate("sys", "user").await.unwrap_err();
        assert!(err.to_string().contains("Cannot connect"));
    }
}
