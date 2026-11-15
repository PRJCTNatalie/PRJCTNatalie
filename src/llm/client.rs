//! HTTP client for an OpenAI-compatible chat completions endpoint.

use crate::llm::types::{ChatRequest, ChatResponse};
use thiserror::Error;

/// Errors produced by the LLM client.
#[derive(Debug, Error)]
pub enum LlmError {
    #[error("http request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("backend returned status {status}: {body}")]
    Status { status: u16, body: String },
    #[error("failed to parse response: {0}")]
    Parse(String),
    #[error("backend returned no choices")]
    NoChoices,
}

/// A thin wrapper around reqwest targeting `/chat/completions`.
#[derive(Clone)]
pub struct OpenAiClient {
    base_url: String,
    api_key: String,
    http: reqwest::Client,
}

impl OpenAiClient {
    pub fn new(base_url: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            api_key: api_key.into(),
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .expect("failed to build http client"),
        }
    }

    pub async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, LlmError> {
        let url = format!("{}/chat/completions", self.base_url);
        let mut builder = self.http.post(&url).json(&request);
        if !self.api_key.is_empty() {
            builder = builder.bearer_auth(&self.api_key);
        }

        let response = builder.send().await?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(LlmError::Status {
                status: status.as_u16(),
                body,
            });
        }

        let parsed: ChatResponse = response
            .json()
            .await
            .map_err(|e| LlmError::Parse(e.to_string()))?;
        if parsed.choices.is_empty() {
            return Err(LlmError::NoChoices);
        }
        Ok(parsed)
    }
}
