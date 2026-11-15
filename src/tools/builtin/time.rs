use crate::tools::registry::{Tool, ToolError};
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};

pub struct TimeTool;

#[async_trait]
impl Tool for TimeTool {
    fn name(&self) -> &str {
        "current_time"
    }

    fn description(&self) -> &str {
        "Return the current UTC date and time in RFC3339 format."
    }

    fn parameters(&self) -> Value {
        json!({ "type": "object", "properties": {} })
    }

    async fn call(&self, _args: Value) -> Result<String, ToolError> {
        Ok(Utc::now().to_rfc3339())
    }
}
