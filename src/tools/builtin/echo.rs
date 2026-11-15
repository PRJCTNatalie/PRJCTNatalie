use crate::tools::registry::{Tool, ToolError};
use async_trait::async_trait;
use serde_json::{json, Value};

pub struct EchoTool;

#[async_trait]
impl Tool for EchoTool {
    fn name(&self) -> &str {
        "echo"
    }

    fn description(&self) -> &str {
        "Echo back the provided text. Useful for testing tool wiring."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "text": { "type": "string", "description": "Text to echo back." }
            },
            "required": ["text"]
        })
    }

    async fn call(&self, args: Value) -> Result<String, ToolError> {
        let text = args
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArgs("missing 'text'".into()))?;
        Ok(text.to_string())
    }
}
