use crate::registry::Tool;
use async_trait::async_trait;
use serde_json::{json, Value};
use tokio::process::Command;

pub struct TerminalTool;

#[async_trait]
impl Tool for TerminalTool {
    fn name(&self) -> &'static str { "run_command" }
    fn toolset(&self) -> &'static str { "terminal" }
    fn schema(&self) -> Value {
        json!({
            "description": "Execute a terminal command.",
            "parameters": {
                "type": "object",
                "properties": {
                    "command": { "type": "string", "description": "The command to run." }
                },
                "required": ["command"]
            }
        })
    }
    async fn handle(&self, args: Value) -> Result<Value, String> {
        let command = match args.get("command").and_then(|v| v.as_str()) {
            Some(c) => c,
            None => return Ok(json!({ "error": "Missing or invalid 'command' argument" })),
        };

        // For cross-platform compatibility we would typically use a shell.
        // Assuming bash/sh for WSL/Linux environment.
        let output = Command::new("bash")
            .arg("-c")
            .arg(command)
            .output()
            .await;

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
            Err(e) => Ok(json!({ "error": format!("Failed to execute command: {}", e) })),
        }
    }
}

inventory::submit!(crate::registry::RegisteredTool { factory: || std::sync::Arc::new(TerminalTool) });
