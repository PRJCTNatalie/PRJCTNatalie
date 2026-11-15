use crate::tools::registry::{Tool, ToolError};
use async_trait::async_trait;
use serde_json::{json, Value};

pub struct ReadFileTool;

#[async_trait]
impl Tool for ReadFileTool {
    fn name(&self) -> &str {
        "read_file"
    }

    fn description(&self) -> &str {
        "Read a UTF-8 text file from disk and return its contents (truncated to 16 KiB)."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "Path to the file to read." }
            },
            "required": ["path"]
        })
    }

    async fn call(&self, args: Value) -> Result<String, ToolError> {
        let path = args
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArgs("missing 'path'".into()))?;
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| ToolError::Execution(format!("{path}: {e}")))?;
        const LIMIT: usize = 16 * 1024;
        if content.len() > LIMIT {
            Ok(format!("{}…\n[truncated]", &content[..LIMIT]))
        } else {
            Ok(content)
        }
    }
}

pub struct ListDirTool;

#[async_trait]
impl Tool for ListDirTool {
    fn name(&self) -> &str {
        "list_dir"
    }

    fn description(&self) -> &str {
        "List the entries of a directory."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "Directory path." }
            },
            "required": ["path"]
        })
    }

    async fn call(&self, args: Value) -> Result<String, ToolError> {
        let path = args
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArgs("missing 'path'".into()))?;
        let mut entries = tokio::fs::read_dir(path)
            .await
            .map_err(|e| ToolError::Execution(format!("{path}: {e}")))?;
        let mut out = Vec::new();
        while let Ok(Some(e)) = entries.next_entry().await {
            let name = e.file_name().to_string_lossy().to_string();
            let kind = if e.file_type().await.map(|t| t.is_dir()).unwrap_or(false) {
                "dir"
            } else {
                "file"
            };
            out.push(format!("{kind}\t{name}"));
        }
        out.sort();
        Ok(out.join("\n"))
    }
}

