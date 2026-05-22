use crate::registry::Tool;
use async_trait::async_trait;
use serde_json::{json, Value};
use tokio::fs;
use tokio::process::Command;
use std::env;

pub struct CodeExecutionTool;

#[async_trait]
impl Tool for CodeExecutionTool {
    fn name(&self) -> &'static str { "execute_code" }
    fn toolset(&self) -> &'static str { "code_execution" }
    fn schema(&self) -> Value {
        json!({
            "description": "Execute a short snippet of code. Supports python and node.",
            "parameters": {
                "type": "object",
                "properties": {
                    "language": { "type": "string", "enum": ["python", "node"], "description": "The programming language." },
                    "code": { "type": "string", "description": "The code to execute." }
                },
                "required": ["language", "code"]
            }
        })
    }
    async fn handle(&self, args: Value) -> Result<Value, String> {
        let language = match args.get("language").and_then(|v| v.as_str()) {
            Some(l) => l,
            None => return Ok(json!({ "error": "Missing or invalid 'language' argument" })),
        };
        let code = match args.get("code").and_then(|v| v.as_str()) {
            Some(c) => c,
            None => return Ok(json!({ "error": "Missing or invalid 'code' argument" })),
        };

        let ext = match language {
            "python" => "py",
            "node" => "js",
            _ => return Ok(json!({ "error": "Unsupported language" })),
        };

        // Create a temporary file
        let mut temp_dir = env::temp_dir();
        temp_dir.push(format!("athena_code_eval_{}.{}", uuid::Uuid::new_v4(), ext));

        if let Err(e) = fs::write(&temp_dir, code).await {
            return Ok(json!({ "error": format!("Failed to write code to temp file: {}", e) }));
        }

        let output = match language {
            "python" => Command::new("python3").arg(&temp_dir).output().await,
            "node" => Command::new("node").arg(&temp_dir).output().await,
            _ => unreachable!(),
        };

        // Clean up
        let _ = fs::remove_file(&temp_dir).await;

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                Ok(json!({
                    "success": out.status.success(),
                    "exit_code": out.status.code(),
                    "stdout": stdout,
                    "stderr": stderr,
                }))
            }
            Err(e) => Ok(json!({ "error": format!("Failed to execute code: {}", e) })),
        }
    }
}

inventory::submit!(crate::registry::RegisteredTool { factory: || std::sync::Arc::new(CodeExecutionTool) });

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_code_execution_tool() {
        let tool = CodeExecutionTool;
        assert_eq!(tool.name(), "execute_code");
        assert_eq!(tool.toolset(), "code_execution");

        let schema = tool.schema();
        assert!(schema.get("description").is_some());
        assert!(schema.get("parameters").is_some());

        let result = tool.handle(json!({})).await.unwrap();
        assert_eq!(result["error"], "Missing or invalid 'language' argument");

        let result = tool.handle(json!({"language": "python"})).await.unwrap();
        assert_eq!(result["error"], "Missing or invalid 'code' argument");

        let result = tool.handle(json!({"language": "unknown", "code": "print()"})).await.unwrap();
        assert_eq!(result["error"], "Unsupported language");
    }
}
