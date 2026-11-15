//! OpenAI-compatible LLM client and shared wire types.

pub mod client;
pub mod types;

pub use client::OpenAiClient;
pub use types::*;
