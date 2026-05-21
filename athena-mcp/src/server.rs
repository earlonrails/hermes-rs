use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{error, info};

use crate::types::*;
use athena_tools::ToolRegistry;

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
                let tools = self.registry.get_all_tools().await;
                for tool in tools {
                    let schema = tool.schema();
                    tools_list.push(serde_json::json!({
                        "name": tool.name(),
                        "description": tool.description(),
                        "inputSchema": schema.get("parameters").cloned().unwrap_or(serde_json::Value::Null),
                    }));
                }
                res.result = Some(serde_json::json!({
                    "tools": tools_list
                }));
            }
            "tools/call" => {
                if let Some(params) = req.params {
                    if let Ok(call_req) = serde_json::from_value::<CallToolRequest>(params) {
                        let output = self.registry.dispatch(&call_req.name, call_req.arguments).await;
                        
                        let is_error = if let Ok(v) = serde_json::from_str::<serde_json::Value>(&output) {
                            v.get("error").is_some()
                        } else {
                            false
                        };
                        
                        let result = CallToolResult {
                            content: vec![ToolContent {
                                r#type: "text".to_string(),
                                text: output,
                            }],
                            is_error,
                        };
                        res.result = match serde_json::to_value(result) {
                            Ok(v) => Some(v),
                            Err(_) => None,
                        };
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mcp_server_handle_list_tools() {
        let registry = Arc::new(ToolRegistry::new());
        let server = McpServer::new(registry);
        
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::json!(1)),
            method: "tools/list".to_string(),
            params: None,
        };
        
        let res = server.handle_request(req).await;
        
        assert_eq!(res.jsonrpc, "2.0");
        assert_eq!(res.id, Some(serde_json::json!(1)));
        assert!(res.error.is_none());
        assert!(res.result.is_some());
        
        let result = res.result.unwrap();
        assert!(result.get("tools").is_some());
    }

    #[tokio::test]
    async fn test_mcp_server_handle_call_tool_missing_params() {
        let registry = Arc::new(ToolRegistry::new());
        let server = McpServer::new(registry);
        
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::json!(2)),
            method: "tools/call".to_string(),
            params: None,
        };
        
        let res = server.handle_request(req).await;
        
        assert!(res.error.is_some());
        assert_eq!(res.error.unwrap().message, "Missing params");
    }

    #[tokio::test]
    async fn test_mcp_server_handle_call_tool_invalid_params() {
        let registry = Arc::new(ToolRegistry::new());
        let server = McpServer::new(registry);
        
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::json!(3)),
            method: "tools/call".to_string(),
            params: Some(serde_json::json!({"wrong": "format"})),
        };
        
        let res = server.handle_request(req).await;
        
        assert!(res.error.is_some());
        assert_eq!(res.error.unwrap().message, "Invalid params");
    }

    #[tokio::test]
    async fn test_mcp_server_handle_unknown_method() {
        let registry = Arc::new(ToolRegistry::new());
        let server = McpServer::new(registry);
        
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::json!(4)),
            method: "unknown/method".to_string(),
            params: None,
        };
        
        let res = server.handle_request(req).await;
        
        assert!(res.error.is_some());
        assert_eq!(res.error.unwrap().code, -32601);
    }

    #[tokio::test]
    async fn test_mcp_server_handle_call_tool_success() {
        let registry = Arc::new(ToolRegistry::new());
        let server = McpServer::new(registry);
        
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::json!(5)),
            method: "tools/call".to_string(),
            params: Some(serde_json::json!({
                "name": "non_existent",
                "arguments": {}
            })),
        };
        
        let res = server.handle_request(req).await;
        
        assert!(res.error.is_none());
        assert!(res.result.is_some());
        
        let result = res.result.unwrap();
        let call_result: CallToolResult = serde_json::from_value(result).unwrap();
        
        assert_eq!(call_result.content.len(), 1);
        // Since we dispatched a non-existent tool, the registry should return an error JSON string,
        // which server.rs parses and sets is_error = true.
        assert!(call_result.is_error);
    }

    struct MockBadTool;
    
    #[async_trait::async_trait]
    impl athena_tools::Tool for MockBadTool {
        fn name(&self) -> &'static str { "bad_tool" }
        fn toolset(&self) -> &'static str { "mock" }
        fn schema(&self) -> serde_json::Value { serde_json::json!({}) }
        async fn handle(&self, _args: serde_json::Value) -> Result<serde_json::Value, String> {
            // Wait, handle returns a JSON Value. 
            // The ToolRegistry dispatch method converts the Result to a JSON string!
            // If handle returns Ok(Value::String("not valid json inner string")),
            // the dispatch method returns `not valid json inner string`?
            // Actually, if it returns Ok(v), dispatch does `serde_json::to_string_pretty(&v)`.
            // Wait, if dispatch serializes it, it will ALWAYS be valid JSON!
            // Unless the Value is a string and the tool registry extracts the inner string instead of serializing the Value?
            // Let's just return a String that isn't JSON.
            Ok(serde_json::Value::String("this is not a json object string".to_string()))
        }
    }

    #[tokio::test]
    async fn test_mcp_server_tool_invalid_json_output() {
        let registry = Arc::new(ToolRegistry::new());
        registry.register(Arc::new(MockBadTool)).await;
        
        let server = McpServer::new(registry);
        
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::json!(6)),
            method: "tools/call".to_string(),
            params: Some(serde_json::json!({
                "name": "bad_tool",
                "arguments": {}
            })),
        };
        
        let res = server.handle_request(req).await;
        let result = res.result.unwrap();
        let call_result: CallToolResult = serde_json::from_value(result).unwrap();
        
        // Because "this is not a json object string" is returned, dispatch() may serialize it as `"this is not a json object string"`.
        // parsing `"this is not a json object string"` into Value succeeds (it's a string).
        // BUT v.get("error") will return None! Because it's not an object.
        // So `is_error` will be false!
        assert!(!call_result.is_error);
    }
}
