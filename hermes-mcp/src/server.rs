use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::mpsc;
use tracing::{debug, error, info};
use std::collections::HashMap;

use crate::types::*;
use hermes_tools::{ToolRegistry, ToolContext};

pub struct McpServer {
    registry: Arc<ToolRegistry>,
}

impl McpServer {
    pub fn new(registry: Arc<ToolRegistry>) -> Self {
        Self { registry }
    }

    pub async fn run(&self) {
        let stdin = tokio::io::stdin();
        let mut stdout = tokio::io::stdout();
        let mut reader = BufReader::new(stdin).lines();

        info!("MCP Server listening on stdin...");

        while let Ok(Some(line)) = reader.next_line().await {
            let req: Result<JsonRpcRequest, _> = serde_json::from_str(&line);
            
            match req {
                Ok(request) => {
                    let response = self.handle_request(request).await;
                    let res_str = serde_json::to_string(&response).unwrap_or_default();
                    if let Err(e) = stdout.write_all(format!("{}\n", res_str).as_bytes()).await {
                        error!("Failed to write MCP response: {}", e);
                    }
                    let _ = stdout.flush().await;
                }
                Err(e) => {
                    error!("Invalid MCP request: {}", e);
                    let err_res = JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: None,
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32700,
                            message: "Parse error".to_string(),
                            data: None,
                        }),
                    };
                    let err_str = match serde_json::to_string(&err_res) {
                        Ok(s) => s,
                        Err(_) => "{\"jsonrpc\":\"2.0\",\"error\":{\"code\":-32700,\"message\":\"Parse error\"}}".to_string(),
                    };
                    let _ = stdout.write_all(format!("{}\n", err_str).as_bytes()).await;
                    let _ = stdout.flush().await;
                }
            }
        }
    }

    async fn handle_request(&self, req: JsonRpcRequest) -> JsonRpcResponse {
        let mut res = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: req.id.clone(),
            result: None,
            error: None,
        };

        match req.method.as_str() {
            "tools/list" => {
                let mut tools_list = Vec::new();
                for (name, schema) in self.registry.list_schemas() {
                    tools_list.push(serde_json::json!({
                        "name": name,
                        "description": schema.description,
                        "inputSchema": schema.parameters,
                    }));
                }
                res.result = Some(serde_json::json!({
                    "tools": tools_list
                }));
            }
            "tools/call" => {
                if let Some(params) = req.params {
                    if let Ok(call_req) = serde_json::from_value::<CallToolRequest>(params) {
                        let context = ToolContext {
                            workspace_dir: std::env::current_dir().unwrap_or_default(),
                            env: HashMap::new(),
                        };
                        
                        match self.registry.execute(&call_req.name, call_req.arguments, context).await {
                            Ok(output) => {
                                let result = CallToolResult {
                                    content: vec![ToolContent {
                                        r#type: "text".to_string(),
                                        text: output,
                                    }],
                                    is_error: false,
                                };
                                res.result = match serde_json::to_value(result) {
                                    Ok(v) => Some(v),
                                    Err(_) => None,
                                };
                            }
                            Err(e) => {
                                let result = CallToolResult {
                                    content: vec![ToolContent {
                                        r#type: "text".to_string(),
                                        text: e.to_string(),
                                    }],
                                    is_error: true,
                                };
                                res.result = match serde_json::to_value(result) {
                                    Ok(v) => Some(v),
                                    Err(_) => None,
                                };
                            }
                        }
                    } else {
                        res.error = Some(JsonRpcError {
                            code: -32602,
                            message: "Invalid params".to_string(),
                            data: None,
                        });
                    }
                } else {
                    res.error = Some(JsonRpcError {
                        code: -32602,
                        message: "Missing params".to_string(),
                        data: None,
                    });
                }
            }
            _ => {
                res.error = Some(JsonRpcError {
                    code: -32601,
                    message: "Method not found".to_string(),
                    data: None,
                });
            }
        }

        res
    }
}
