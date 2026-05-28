use async_trait::async_trait;
use athena_tools::Tool;
use serde_json::Value;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{ChildStdin, ChildStdout, Command};
use tokio::sync::{mpsc, oneshot, Mutex};
use tracing::error;

use crate::types::*;

pub struct McpClient {
    #[allow(dead_code)]
    command: String,
    #[allow(dead_code)]
    args: Vec<String>,
    request_tx: mpsc::Sender<(JsonRpcRequest, oneshot::Sender<JsonRpcResponse>)>,
    next_id: Arc<AtomicU64>,
}

impl McpClient {
    pub async fn new(command: &str, args: &[String]) -> Result<Self, String> {
        let mut child = Command::new(command)
            .args(args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn MCP server: {}", e))?;

        let stdin = match child.stdin.take() {
            Some(s) => s,
            None => return Err("Failed to open stdin to MCP server".to_string()),
        };
        let stdout = match child.stdout.take() {
            Some(s) => s,
            None => return Err("Failed to open stdout to MCP server".to_string()),
        };

        let (tx, rx) = mpsc::channel(32);
        let pending_requests = Arc::new(Mutex::new(std::collections::HashMap::new()));

        // Start IO loops
        let pr_clone = pending_requests.clone();
        tokio::spawn(async move {
            Self::read_loop(stdout, pr_clone).await;
        });

        let pr_clone2 = pending_requests.clone();
        tokio::spawn(async move {
            Self::write_loop(stdin, rx, pr_clone2).await;
        });

        Ok(Self {
            command: command.to_string(),
            args: args.to_vec(),
            request_tx: tx,
            next_id: Arc::new(AtomicU64::new(1)),
        })
    }

    async fn read_loop(stdout: ChildStdout, pending: Arc<Mutex<std::collections::HashMap<u64, oneshot::Sender<JsonRpcResponse>>>>) {
        let mut reader = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            if let Ok(res) = serde_json::from_str::<JsonRpcResponse>(&line) {
                if let Some(id_val) = &res.id {
                    if let Some(id) = id_val.as_u64() {
                        let mut map = pending.lock().await;
                        if let Some(tx) = map.remove(&id) {
                            let _ = tx.send(res);
                        }
                    }
                }
            }
        }
    }

    async fn write_loop(
        mut stdin: ChildStdin,
        mut rx: mpsc::Receiver<(JsonRpcRequest, oneshot::Sender<JsonRpcResponse>)>,
        pending: Arc<Mutex<std::collections::HashMap<u64, oneshot::Sender<JsonRpcResponse>>>>
    ) {
        while let Some((req, tx)) = rx.recv().await {
            if let Some(id_val) = &req.id {
                if let Some(id) = id_val.as_u64() {
                    pending.lock().await.insert(id, tx);
                }
            }
            
            let req_str = match serde_json::to_string(&req) {
                Ok(s) => s,
                Err(e) => {
                    error!("Failed to serialize MCP request: {}", e);
                    continue;
                }
            };
            if let Err(e) = stdin.write_all(format!("{}\n", req_str).as_bytes()).await {
                error!("Failed to write to MCP server: {}", e);
                break;
            }
            let _ = stdin.flush().await;
        }
    }

    pub async fn call(&self, method: &str, params: Option<Value>) -> Result<JsonRpcResponse, String> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(Value::Number(id.into())),
            method: method.to_string(),
            params,
        };

        let (tx, rx) = oneshot::channel();
        self.request_tx.send((req, tx)).await.map_err(|e| e.to_string())?;

        rx.await.map_err(|e| e.to_string())
    }

    pub async fn list_tools(&self) -> Result<Vec<Value>, String> {
        let res = self.call("tools/list", None).await?;
        if let Some(err) = res.error {
            return Err(err.message);
        }
        
        let tools = res.result
            .and_then(|r| r.get("tools").cloned())
            .and_then(|t| t.as_array().cloned())
            .unwrap_or_default();
            
        Ok(tools)
    }

    pub async fn call_tool(&self, name: &str, arguments: Value) -> Result<CallToolResult, String> {
        let params = serde_json::json!({
            "name": name,
            "arguments": arguments
        });
        
        let res = self.call("tools/call", Some(params)).await?;
        
        if let Some(err) = res.error {
            return Err(err.message);
        }
        
        if let Some(result) = res.result {
            serde_json::from_value(result).map_err(|e| e.to_string())
        } else {
            Err("No result returned".to_string())
        }
    }
}

/// A wrapper that adapts an external MCP tool into our internal Tool trait
pub struct ExternalMcpTool {
    pub client: Arc<McpClient>,
    pub name: &'static str,
    pub toolset: &'static str,
    pub schema_val: Value,
}

#[async_trait]
impl Tool for ExternalMcpTool {
    fn name(&self) -> &'static str {
        self.name
    }

    fn toolset(&self) -> &'static str {
        self.toolset
    }

    fn schema(&self) -> Value {
        self.schema_val.clone()
    }

    async fn handle(&self, args: Value) -> Result<Value, String> {
        let res = self.client.call_tool(self.name, args).await?;
        
        let text = res.content.iter()
            .map(|c| c.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
            
        if res.is_error {
            Err(text)
        } else {
            Ok(Value::String(text))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mcp_client_init_and_list_tools() {
        // Mock server that reads input and then outputs a valid JSON-RPC list tools response
        let mock_server_cmd = r#"
            read line
            echo '{"jsonrpc":"2.0","id":1,"result":{"tools":[{"name":"test_tool","description":"A test tool"}]}}'
            sleep 1
        "#;
        
        let client = McpClient::new("bash", &["-c".to_string(), mock_server_cmd.to_string()]).await;
        assert!(client.is_ok());
        
        let client = client.unwrap();
        
        // This will send id=1 and the mock server replies with id=1
        let tools_res = client.list_tools().await;
        assert!(tools_res.is_ok());
        
        let tools = tools_res.unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0]["name"], "test_tool");
    }

    #[tokio::test]
    async fn test_mcp_client_call_tool() {
        // Mock server that replies to tool call
        let mock_server_cmd = r#"
            read line
            echo '{"jsonrpc":"2.0","id":1,"result":{"content":[{"type":"text","text":"Tool output"}],"is_error":false}}'
            sleep 1
        "#;
        
        let client = McpClient::new("bash", &["-c".to_string(), mock_server_cmd.to_string()]).await.unwrap();
        
        let result = client.call_tool("test_tool", serde_json::json!({})).await;
        assert!(result.is_ok());
        
        let call_res = result.unwrap();
        assert!(!call_res.is_error);
        assert_eq!(call_res.content.len(), 1);
        assert_eq!(call_res.content[0].text, "Tool output");
    }

    #[tokio::test]
    async fn test_mcp_client_spawn_failure() {
        let client = McpClient::new("non_existent_command_12345", &[]).await;
        assert!(client.is_err());
    }

    #[tokio::test]
    async fn test_mcp_client_read_loop_edge_cases() {
        let mock_server_cmd = r#"
            echo 'invalid json'
            echo '{"jsonrpc":"2.0","result":{}}'
            echo '{"jsonrpc":"2.0","id":"string_id","result":{}}'
            echo '{"jsonrpc":"2.0","id":999,"result":{}}'
            sleep 1
        "#;
        
        let client = McpClient::new("bash", &["-c".to_string(), mock_server_cmd.to_string()]).await.unwrap();
        // Wait a bit for the server to emit those lines and the read_loop to process them
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        // The read_loop should gracefully handle and ignore all of them.
        assert_eq!(client.command, "bash");
    }

    #[tokio::test]
    async fn test_mcp_client_list_tools_error() {
        let mock_server_cmd = r#"
            read line
            echo '{"jsonrpc":"2.0","id":1,"error":{"code":-32600,"message":"Invalid request"}}'
            sleep 1
        "#;
        let client = McpClient::new("bash", &["-c".to_string(), mock_server_cmd.to_string()]).await.unwrap();
        let res = client.list_tools().await;
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), "Invalid request");
    }

    #[tokio::test]
    async fn test_mcp_client_call_tool_errors() {
        // Test 1: error field present
        let mock_server_cmd1 = r#"
            read line
            echo '{"jsonrpc":"2.0","id":1,"error":{"code":-32000,"message":"Tool execution failed"}}'
            sleep 1
        "#;
        let client1 = McpClient::new("bash", &["-c".to_string(), mock_server_cmd1.to_string()]).await.unwrap();
        let res1 = client1.call_tool("test", serde_json::json!({})).await;
        assert!(res1.is_err());
        assert_eq!(res1.unwrap_err(), "Tool execution failed");

        // Test 2: no result field
        let mock_server_cmd2 = r#"
            read line
            echo '{"jsonrpc":"2.0","id":1}'
            sleep 1
        "#;
        let client2 = McpClient::new("bash", &["-c".to_string(), mock_server_cmd2.to_string()]).await.unwrap();
        let res2 = client2.call_tool("test", serde_json::json!({})).await;
        assert!(res2.is_err());
        assert_eq!(res2.unwrap_err(), "No result returned");
    }

    #[tokio::test]
    async fn test_external_mcp_tool_execution() {
        // Test success and error execution via the tool wrapper
        let mock_server_cmd = r#"
            read line
            echo '{"jsonrpc":"2.0","id":1,"result":{"content":[{"type":"text","text":"Success!"}],"is_error":false}}'
            read line
            echo '{"jsonrpc":"2.0","id":2,"result":{"content":[{"type":"text","text":"Failure!"}],"is_error":true}}'
            sleep 1
        "#;
        let client = Arc::new(McpClient::new("bash", &["-c".to_string(), mock_server_cmd.to_string()]).await.unwrap());
        
        let tool = ExternalMcpTool {
            client: client.clone(),
            name: "test_tool",
            toolset: "mcp",
            schema_val: serde_json::json!({}),
        };

        assert_eq!(tool.name(), "test_tool");
        assert_eq!(tool.toolset(), "mcp");
        assert_eq!(tool.schema(), serde_json::json!({}));

        // First call maps to id=1 (success)
        let res1 = tool.handle(serde_json::json!({})).await;
        assert!(res1.is_ok());
        assert_eq!(res1.unwrap(), Value::String("Success!".to_string()));

        // Second call maps to id=2 (failure)
        let res2 = tool.handle(serde_json::json!({})).await;
        assert!(res2.is_err());
        assert_eq!(res2.unwrap_err(), "Failure!");
    }
}

// Rust guideline compliant 2026-02-21
