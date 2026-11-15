//! Tool trait and registry.

use crate::llm::types::ToolDefinition;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

/// Errors a tool can produce.
#[derive(Debug, Error)]
pub enum ToolError {
    #[error("unknown tool: {0}")]
    Unknown(String),
    #[error("invalid arguments: {0}")]
    InvalidArgs(String),
    #[error("execution failed: {0}")]
    Execution(String),
}

/// A callable tool exposed to the agent.
#[async_trait]
pub trait Tool: Send + Sync {
    /// Stable, unique name.
    fn name(&self) -> &str;
    /// Human-readable description shown to the model.
    fn description(&self) -> &str;
    /// JSON schema describing the arguments.
    fn parameters(&self) -> Value;
    /// Execute the tool with the given arguments.
    async fn call(&self, args: Value) -> Result<String, ToolError>;
}

/// Holds registered tools keyed by name.
#[derive(Default, Clone)]
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, tool: Arc<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    pub fn definitions(&self) -> Vec<ToolDefinition> {
        self.tools
            .values()
            .map(|t| {
                ToolDefinition::function(t.name(), t.description(), t.parameters())
            })
            .collect()
    }

    pub async fn call(&self, name: &str, args: Value) -> Result<String, ToolError> {
        let tool = self
            .tools
            .get(name)
            .cloned()
            .ok_or_else(|| ToolError::Unknown(name.to_string()))?;
        tool.call(args).await
    }

    pub fn names(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }
}

