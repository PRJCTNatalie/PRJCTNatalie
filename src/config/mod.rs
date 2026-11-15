//! Runtime configuration.
//!
//! Everything is sourced from environment variables so the binary stays
//! 12-factor friendly and secrets never need to live in source control.

use std::env;

/// Configuration for the Natalie agent.
#[derive(Clone, Debug)]
pub struct Config {
    /// API key for the OpenAI-compatible backend.
    pub api_key: String,
    /// Base URL of the OpenAI-compatible API (e.g. https://api.openai.com/v1).
    pub base_url: String,
    /// Model identifier to request.
    pub model: String,
    /// Sampling temperature.
    pub temperature: f32,
    /// Maximum tokens per completion.
    pub max_tokens: u32,
    /// System prompt that defines Natalie's persona.
    pub system_prompt: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            api_key: env::var("OPENAI_API_KEY").unwrap_or_default(),
            base_url: env::var("OPENAI_BASE_URL")
                .unwrap_or_else(|_| "https://api.openai.com/v1".to_string()),
            model: env::var("NATALIE_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string()),
            temperature: env::var("NATALIE_TEMPERATURE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0.7),
            max_tokens: env::var("NATALIE_MAX_TOKENS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(2048),
            system_prompt: env::var("NATALIE_SYSTEM_PROMPT").unwrap_or_else(|_| {
                "You are Natalie, a helpful, direct, and honest AI assistant. \
                 You think carefully, use tools when they help, and never invent \
                 facts you are unsure about. You are named after your maintainer."
                    .to_string()
            }),
        }
    }

    /// True when an API key has been provided.
    pub fn has_credentials(&self) -> bool {
        !self.api_key.trim().is_empty()
    }
}
