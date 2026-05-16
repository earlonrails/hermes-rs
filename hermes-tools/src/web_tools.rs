use crate::registry::Tool;
use async_trait::async_trait;
use serde_json::{json, Value};

pub struct ReadUrlTool;

#[async_trait]
impl Tool for ReadUrlTool {
    fn name(&self) -> &'static str { "read_url" }
    fn toolset(&self) -> &'static str { "web" }
    fn schema(&self) -> Value {
        json!({
            "description": "Fetch content from a URL via HTTP request.",
            "parameters": {
                "type": "object",
                "properties": {
                    "url": { "type": "string", "description": "The URL to fetch." }
                },
                "required": ["url"]
            }
        })
    }
    async fn handle(&self, args: Value) -> Result<Value, String> {
        let url = match args.get("url").and_then(|v| v.as_str()) {
            Some(u) => u,
            None => return Ok(json!({ "error": "Missing or invalid 'url' argument" })),
        };

        match reqwest::get(url).await {
            Ok(resp) => {
                match resp.text().await {
                    Ok(text) => Ok(json!({ "success": true, "content": text })),
                    Err(e) => Ok(json!({ "error": format!("Failed to read response body: {}", e) })),
                }
            }
            Err(e) => Ok(json!({ "error": format!("Failed to execute request: {}", e) })),
        }
    }
}

inventory::submit!(crate::registry::RegisteredTool { factory: || std::sync::Arc::new(ReadUrlTool) });
