use athena_agent::AIAgent;
use athena_tools::ToolRegistry;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::io::{stdin, stdout, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::Mutex;
use tracing::{error, info};

#[derive(Serialize, Deserialize, Debug)]
struct RpcRequest {
    jsonrpc: String,
    method: String,
    params: Option<Value>,
    id: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
struct RpcResponse {
    jsonrpc: String,
    result: Option<Value>,
    error: Option<Value>,
    id: Option<u64>,
}

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();

    // The Node.js Ink TUI communicates over stdio using newline-delimited JSON RPC.
    let stdin = stdin();
    let mut reader = BufReader::new(stdin).lines();
    let mut stdout = stdout();

    let registry = Arc::new(ToolRegistry::new());
    let agent_builder = AIAgent::builder().model("gpt-4o").max_iterations(20);
    
    // Create an agent protected by a Mutex for sequential execution.
    let agent = Arc::new(Mutex::new(agent_builder.build()));

    loop {
        match reader.next_line().await {
            Ok(Some(line)) => {
                if line.trim().is_empty() { continue; }
                
                let req: RpcRequest = match serde_json::from_str(&line) {
                    Ok(r) => r,
                    Err(e) => {
                        error!("Invalid JSON RPC: {}", e);
                        continue;
                    }
                };

                // Simple routing based on method name
                match req.method.as_str() {
                    "prompt.submit" => {
                        // Extract prompt from params and send to agent
                        let prompt = req.params
                            .and_then(|p| p.get("prompt").cloned())
                            .and_then(|p| p.as_str().map(|s| s.to_string()))
                            .unwrap_or_default();
                        
                        let agent_clone = Arc::clone(&agent);
                        let reg_clone = Arc::clone(&registry);
                        let req_id = req.id;

                        // We could spawn this if we wanted concurrent UI, but the UI expects a response
                        tokio::spawn(async move {
                            let mut locked_agent = agent_clone.lock().await;
                            let res = locked_agent.run_conversation(&prompt, Some("You are Hermes TUI."), &reg_clone).await;
                            
                            let response_value = match res {
                                Ok(content) => serde_json::json!({ "content": content }),
                                Err(e) => serde_json::json!({ "error": e }),
                            };

                            let rpc_res = RpcResponse {
                                jsonrpc: "2.0".into(),
                                result: Some(response_value),
                                error: None,
                                id: req_id,
                            };
                            
                            let mut out = tokio::io::stdout();
                            let _ = out.write_all(format!("{}\n", serde_json::to_string(&rpc_res).unwrap_or_default()).as_bytes()).await;
                            let _ = out.flush().await;
                        });
                    }
                    _ => {
                        // Unhandled method
                        let rpc_res = RpcResponse {
                            jsonrpc: "2.0".into(),
                            result: None,
                            error: Some(serde_json::json!({ "message": "Method not found" })),
                            id: req.id,
                        };
                        let _ = stdout.write_all(format!("{}\n", serde_json::to_string(&rpc_res).unwrap_or_default()).as_bytes()).await;
                        let _ = stdout.flush().await;
                    }
                }
            }
            Ok(None) => {
                // EOF
                break;
            }
            Err(e) => {
                error!("Error reading stdin: {}", e);
                break;
            }
        }
    }
}
