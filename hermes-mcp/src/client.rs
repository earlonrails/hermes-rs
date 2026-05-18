use async_trait::async_trait;
use hermes_tools::Tool;
use serde_json::Value;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{ChildStdin, ChildStdout, Command};
use tokio::sync::{mpsc, oneshot, Mutex};
use tracing::error;

use crate::types::*;

pub struct McpClient {
    command: String,
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
