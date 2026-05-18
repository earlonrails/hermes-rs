use crate::registry::Tool;
use async_trait::async_trait;
use serde_json::{json, Value};
use tokio::fs;
use std::path::{Path, Component};

fn is_path_safe(path_str: &str) -> bool {
    let path = Path::new(path_str);
    
    for component in path.components() {
        if let Component::ParentDir = component {
            return false;
        }
    }

    if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
        let normalized = file_name.to_lowercase();
        if normalized == ".env" 
            || normalized.contains("key") 
            || normalized.contains("oauth")
            || normalized.contains("secret")
            || normalized.contains("credential")
            || normalized == "passwd" 
            || normalized == "shadow" 
        {
            return false;
        }
    }
    
    true
}

pub struct ReadFileTool;

#[async_trait]
impl Tool for ReadFileTool {
    fn name(&self) -> &'static str {
        "read_file"
    }

    fn toolset(&self) -> &'static str {
        "file_operations"
    }

    fn schema(&self) -> Value {
        json!({
            "description": "Read the contents of a file.",
            "parameters": {
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "The path to the file to read."
                    }
                },
                "required": ["path"]
            }
        })
    }

    async fn handle(&self, args: Value) -> Result<Value, String> {
        let path = match args.get("path").and_then(|v| v.as_str()) {
            Some(p) => p,
            None => return Ok(json!({ "error": "Missing or invalid 'path' argument" })),
        };

        if !is_path_safe(path) {
            return Ok(json!({ "error": "Access denied: Reading this path/file is restricted for security." }));
        }

        match fs::read_to_string(path).await {
            Ok(content) => Ok(json!({ "success": true, "content": content })),
            Err(e) => Ok(json!({ "error": format!("Failed to read file: {}", e) })),
        }
    }
}

pub struct WriteFileTool;

#[async_trait]
impl Tool for WriteFileTool {
    fn name(&self) -> &'static str {
        "write_file"
    }

    fn toolset(&self) -> &'static str {
        "file_operations"
    }

    fn schema(&self) -> Value {
        json!({
            "description": "Write content to a file, completely overwriting it.",
            "parameters": {
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "The path to the file to write."
                    },
                    "content": {
                        "type": "string",
                        "description": "The exact content to write."
                    }
                },
                "required": ["path", "content"]
            }
        })
    }

    async fn handle(&self, args: Value) -> Result<Value, String> {
        let path = match args.get("path").and_then(|v| v.as_str()) {
            Some(p) => p,
            None => return Ok(json!({ "error": "Missing or invalid 'path' argument" })),
        };
        let content = match args.get("content").and_then(|v| v.as_str()) {
            Some(c) => c,
            None => return Ok(json!({ "error": "Missing or invalid 'content' argument" })),
        };

        if !is_path_safe(path) {
            return Ok(json!({ "error": "Access denied: Writing to this path/file is restricted for security." }));
        }

        match fs::write(path, content).await {
            Ok(_) => Ok(json!({ "success": true })),
            Err(e) => Ok(json!({ "error": format!("Failed to write file: {}", e) })),
        }
    }
}

pub struct ListDirTool;

#[async_trait]
impl Tool for ListDirTool {
    fn name(&self) -> &'static str { "list_dir" }
    fn toolset(&self) -> &'static str { "file_operations" }
    fn schema(&self) -> Value {
        json!({
            "description": "List the contents of a directory.",
            "parameters": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "The directory path to list." }
                },
                "required": ["path"]
            }
        })
    }
    async fn handle(&self, args: Value) -> Result<Value, String> {
        let path = match args.get("path").and_then(|v| v.as_str()) {
            Some(p) => p,
            None => return Ok(json!({ "error": "Missing or invalid 'path' argument" })),
        };

        if !is_path_safe(path) {
            return Ok(json!({ "error": "Access denied: Accessing this directory is restricted for security." }));
        }

        let mut entries = match fs::read_dir(path).await {
            Ok(entries) => entries,
            Err(e) => return Ok(json!({ "error": format!("Failed to read directory: {}", e) })),
        };
        let mut items = Vec::new();
        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Ok(file_type) = entry.file_type().await {
                items.push(json!({
                    "name": entry.file_name().to_string_lossy().to_string(),
                    "is_dir": file_type.is_dir(),
                }));
            }
        }
        Ok(json!({ "success": true, "items": items }))
    }
}

pub struct SearchFilesTool;

#[async_trait]
impl Tool for SearchFilesTool {
    fn name(&self) -> &'static str { "search_files" }
    fn toolset(&self) -> &'static str { "file_operations" }
    fn schema(&self) -> Value {
        json!({
            "description": "Search for a pattern in files in a directory.",
            "parameters": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "The directory path to search in." },
                    "pattern": { "type": "string", "description": "The search pattern (e.g. grep query)." }
                },
                "required": ["path", "pattern"]
            }
        })
    }
    async fn handle(&self, args: Value) -> Result<Value, String> {
        let path = match args.get("path").and_then(|v| v.as_str()) {
            Some(p) => p,
            None => return Ok(json!({ "error": "Missing or invalid 'path' argument" })),
        };
        let pattern = match args.get("pattern").and_then(|v| v.as_str()) {
            Some(p) => p,
            None => return Ok(json!({ "error": "Missing or invalid 'pattern' argument" })),
        };

        if !is_path_safe(path) {
            return Ok(json!({ "error": "Access denied: Searching this directory is restricted for security." }));
        }

        let output = std::process::Command::new("grep")
            .arg("-rn")
            .arg(pattern)
            .arg(path)
            .output();

        match output {
            Ok(out) => {
                let result = String::from_utf8_lossy(&out.stdout).to_string();
                Ok(json!({ "success": true, "result": result }))
            }
            Err(e) => Ok(json!({ "error": format!("grep failed: {}", e) })),
        }
    }
}

inventory::submit!(crate::registry::RegisteredTool { factory: || std::sync::Arc::new(ReadFileTool) });
inventory::submit!(crate::registry::RegisteredTool { factory: || std::sync::Arc::new(WriteFileTool) });
inventory::submit!(crate::registry::RegisteredTool { factory: || std::sync::Arc::new(ListDirTool) });
inventory::submit!(crate::registry::RegisteredTool { factory: || std::sync::Arc::new(SearchFilesTool) });
