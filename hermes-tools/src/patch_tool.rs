use crate::registry::{Tool, tool_error, tool_result};
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
