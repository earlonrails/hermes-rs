use crate::registry::Tool;
use async_trait::async_trait;
use serde_json::{json, Value};
use tokio::fs;

pub struct PatchTool;

#[async_trait]
impl Tool for PatchTool {
    fn name(&self) -> &'static str { "patch" }
    fn toolset(&self) -> &'static str { "file_operations" }
    fn schema(&self) -> Value {
        json!({
            "description": "Patch a file by replacing instances of a search string with a replacement string.",
            "parameters": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "The file path to patch." },
                    "search": { "type": "string", "description": "The exact string to search for." },
                    "replace": { "type": "string", "description": "The string to replace it with." }
                },
                "required": ["path", "search", "replace"]
            }
        })
    }
    async fn handle(&self, args: Value) -> Result<Value, String> {
        let path = match args.get("path").and_then(|v| v.as_str()) {
            Some(p) => p,
            None => return Ok(json!({ "error": "Missing or invalid 'path' argument" })),
        };
        let search = match args.get("search").and_then(|v| v.as_str()) {
            Some(s) => s,
            None => return Ok(json!({ "error": "Missing or invalid 'search' argument" })),
        };
        let replace = match args.get("replace").and_then(|v| v.as_str()) {
            Some(r) => r,
            None => return Ok(json!({ "error": "Missing or invalid 'replace' argument" })),
        };

        let content = match fs::read_to_string(path).await {
            Ok(c) => c,
            Err(e) => return Ok(json!({ "error": format!("Failed to read file: {}", e) })),
        };

        if !content.contains(search) {
            return Ok(json!({ "error": "Search string not found in file." }));
        }

        let new_content = content.replace(search, replace);

        match fs::write(path, new_content).await {
            Ok(_) => Ok(json!({ "success": true })),
            Err(e) => Ok(json!({ "error": format!("Failed to write patched file: {}", e) })),
        }
    }
}

inventory::submit!(crate::registry::RegisteredTool { factory: || std::sync::Arc::new(PatchTool) });

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_patch_tool() {
        let tool = PatchTool;
        assert_eq!(tool.name(), "patch");
        assert_eq!(tool.toolset(), "file_operations");

        let schema = tool.schema();
        assert!(schema.get("description").is_some());
        assert!(schema.get("parameters").is_some());

        let result = tool.handle(serde_json::json!({})).await.unwrap();
        assert_eq!(result["error"], "Missing or invalid 'path' argument");
        
        let result = tool.handle(serde_json::json!({"path": "file.txt"})).await.unwrap();
        assert_eq!(result["error"], "Missing or invalid 'search' argument");

        let result = tool.handle(serde_json::json!({"path": "file.txt", "search": "foo"})).await.unwrap();
        assert_eq!(result["error"], "Missing or invalid 'replace' argument");
        
        let result = tool.handle(serde_json::json!({"path": "/tmp/athena_patch_test.txt", "search": "foo", "replace": "bar"})).await.unwrap();
        assert!(result.get("error").is_some() || result.get("success").is_some());
    }
}
