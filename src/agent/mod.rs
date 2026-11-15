//! The agent loop: drives the model, executes tool calls, accumulates history.

use crate::config::Config;
use crate::llm::client::OpenAiClient;
use crate::llm::types::{ChatRequest, Message};
use crate::tools::builtin;
use crate::tools::registry::ToolRegistry;
use thiserror::Error;

/// Errors from the agent loop.
#[derive(Debug, Error)]
pub enum AgentError {
    #[error("llm error: {0}")]
    Llm(#[from] crate::llm::client::LlmError),
    #[error("no response from model")]
    NoResponse,
    #[error("exceeded maximum tool rounds ({0})")]
    TooManyToolRounds(usize),
}

/// The Natalie agent.
pub struct Agent {
    config: Config,
    client: OpenAiClient,
    tools: ToolRegistry,
    messages: Vec<Message>,
    max_tool_rounds: usize,
}

impl Agent {
    pub fn new(config: Config) -> Self {
        let mut tools = ToolRegistry::new();
        builtin::register_all(&mut tools);
        let client = OpenAiClient::new(config.base_url.clone(), config.api_key.clone());
        let mut agent = Self {
            config,
            client,
            tools,
            messages: Vec::new(),
            max_tool_rounds: 10,
        };
        agent.messages.push(Message::system(agent.config.system_prompt.clone()));
        agent
    }

    /// Access the underlying tool registry (e.g. to add custom tools).
    pub fn tools(&mut self) -> &mut ToolRegistry {
        &mut self.tools
    }

    /// Reset conversation history, keeping the system prompt.
    pub fn reset(&mut self) {
        self.messages.clear();
        self.messages.push(Message::system(self.config.system_prompt.clone()));
    }

    /// Send one user turn and return the final assistant reply.
    pub async fn chat(&mut self, user_input: &str) -> Result<String, AgentError> {
        self.messages.push(Message::user(user_input));

        for round in 0..self.max_tool_rounds {
            let request = ChatRequest {
                model: self.config.model.clone(),
                messages: self.messages.clone(),
                tools: Some(self.tools.definitions()),
                tool_choice: Some("auto".into()),
                temperature: self.config.temperature,
                max_tokens: self.config.max_tokens,
                stream: false,
            };

            let response = self.client.chat(request).await?;
            let choice = response.choices.into_iter().next().ok_or(AgentError::NoResponse)?;
            let msg = choice.message;

            if let Some(calls) = msg.tool_calls.clone() {
                if calls.is_empty() {
                    // Treat as final.
                    let content = msg.content.clone().unwrap_or_default();
                    self.messages.push(Message::assistant(content.clone()));
                    return Ok(content);
                }
                // Record assistant message carrying tool_calls.
                self.messages.push(Message::assistant_tool_calls(calls.clone()));

                for call in calls {
                    let args: serde_json::Value =
                        serde_json::from_str(&call.function.arguments).unwrap_or(serde_json::json!({}));
                    tracing::debug!(tool = %call.function.name, "executing tool");
                    let result = match self.tools.call(&call.function.name, args).await {
                        Ok(r) => r,
                        Err(e) => format!("tool error: {e}"),
                    };
                    self.messages
                        .push(Message::tool(call.id.clone(), call.function.name.clone(), result));
                }
                // Continue loop for next model turn.
                let _ = round;
                continue;
            }

            let content = msg.content.clone().unwrap_or_default();
            self.messages.push(Message::assistant(content.clone()));
            return Ok(content);
        }

        Err(AgentError::TooManyToolRounds(self.max_tool_rounds))
    }
}
