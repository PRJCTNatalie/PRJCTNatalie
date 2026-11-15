//! PRJCTNatalie — foundation of the Natalie AI agent.
//!
//! This crate exposes the building blocks (config, LLM client, tools, agent
//! loop) so they can be reused by the CLI binary, the HTTP server, and any
//! future frontends.

pub mod agent;
pub mod config;
pub mod llm;
pub mod tools;

pub mod cli;
pub mod server;

pub use agent::Agent;
pub use config::Config;
pub use llm::client::OpenAiClient;
pub use tools::registry::ToolRegistry;
