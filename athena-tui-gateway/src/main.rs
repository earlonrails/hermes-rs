use athena_agent::AIAgent;
use athena_tools::ToolRegistry;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::io::{stdin, stdout, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::Mutex;
use tracing::error;

#[derive(Serialize, Deserialize, Debug)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<Value>,
    pub id: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RpcResponse {
    pub jsonrpc: String,
    pub result: Option<Value>,
    pub error: Option<Value>,
    pub id: Option<u64>,
}

pub async fn handle_request(
    line: &str,
    agent: Arc<Mutex<AIAgent>>,
    registry: Arc<ToolRegistry>,
    provider: Arc<dyn athena_providers::LLMProvider + Send + Sync>,
) -> Option<RpcResponse> {
    if line.trim().is_empty() {
        return None;
    }

    let req: RpcRequest = match serde_json::from_str(line) {
        Ok(r) => r,
        Err(e) => {
            error!("Invalid JSON RPC: {}", e);
            return None;
        }
    };

    match req.method.as_str() {
        "prompt.submit" => {
            let prompt = req
                .params
                .and_then(|p| p.get("prompt").cloned())
                .and_then(|p| p.as_str().map(|s| s.to_string()))
                .unwrap_or_default();

            let req_id = req.id;
            let mut locked_agent = agent.lock().await;

            let res = locked_agent
                .run_conversation(&prompt, Some("You are Athena TUI."), &registry, provider)
                .await;

            let response_value = match res {
                Ok(content) => serde_json::json!({ "content": content }),
                Err(e) => serde_json::json!({ "error": e }),
            };

            Some(RpcResponse {
                jsonrpc: "2.0".into(),
                result: Some(response_value),
                error: None,
                id: req_id,
            })
        }
        _ => Some(RpcResponse {
            jsonrpc: "2.0".into(),
            result: None,
            error: Some(serde_json::json!({ "message": "Method not found" })),
            id: req.id,
        }),
    }
}

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();

    let stdin = stdin();
    let mut reader = BufReader::new(stdin).lines();
    let _stdout = stdout();

    let registry = Arc::new(ToolRegistry::new());
    let agent_builder = AIAgent::builder().model("gpt-4o").max_iterations(20);
    let agent = Arc::new(Mutex::new(agent_builder.build()));
    
    athena_providers::registry::init_builtin_providers();
    let provider = athena_providers::registry::get_provider("openai").unwrap();

    loop {
        match reader.next_line().await {
            Ok(Some(line)) => {
                let agent_clone = Arc::clone(&agent);
                let reg_clone = Arc::clone(&registry);
                let provider_clone = Arc::clone(&provider);

                tokio::spawn(async move {
                    if let Some(rpc_res) = handle_request(&line, agent_clone, reg_clone, provider_clone).await {
                        let mut out = tokio::io::stdout();
                        let _ = out
                            .write_all(
                                format!("{}\n", serde_json::to_string(&rpc_res).unwrap_or_default())
                                    .as_bytes(),
                            )
                            .await;
                        let _ = out.flush().await;
                    }
                });
            }
            Ok(None) => break,
            Err(e) => {
                error!("Error reading stdin: {}", e);
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rpc_request_deserialization() {
        let json = r#"{"jsonrpc":"2.0","method":"prompt.submit","params":{"prompt":"hello"},"id":1}"#;
        let req: RpcRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.jsonrpc, "2.0");
        assert_eq!(req.method, "prompt.submit");
        assert_eq!(req.id, Some(1));

        let prompt = req.params.unwrap().get("prompt").unwrap().as_str().unwrap().to_string();
        assert_eq!(prompt, "hello");
    }

    #[test]
    fn test_rpc_response_serialization() {
        let res = RpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(serde_json::json!({"content": "hi"})),
            error: None,
            id: Some(1),
        };

        let json = serde_json::to_string(&res).unwrap();
        assert!(json.contains(r#""jsonrpc":"2.0""#));
        assert!(json.contains(r#""content":"hi""#));
        assert!(json.contains(r#""id":1"#));
    }

    #[tokio::test]
    async fn test_handle_request_empty_line() {
        let registry = Arc::new(ToolRegistry::new());
        let agent = Arc::new(Mutex::new(AIAgent::builder().build()));
        let provider = Arc::new(athena_providers::providers::openai::OpenAIProvider::new(None, None));
        let res = handle_request("   \n", agent, registry, provider).await;
        assert!(res.is_none());
    }

    #[tokio::test]
    async fn test_handle_request_invalid_json() {
        let registry = Arc::new(ToolRegistry::new());
        let agent = Arc::new(Mutex::new(AIAgent::builder().build()));
        let provider = Arc::new(athena_providers::providers::openai::OpenAIProvider::new(None, None));
        let res = handle_request("{invalid}", agent, registry, provider).await;
        assert!(res.is_none());
    }

    #[tokio::test]
    async fn test_handle_request_unknown_method() {
        let registry = Arc::new(ToolRegistry::new());
        let agent = Arc::new(Mutex::new(AIAgent::builder().build()));
        let provider = Arc::new(athena_providers::providers::openai::OpenAIProvider::new(None, None));
        let line = r#"{"jsonrpc":"2.0","method":"unknown.method","id":123}"#;
        let res = handle_request(line, agent, registry, provider).await;

        assert!(res.is_some());
        let response = res.unwrap();
        assert_eq!(response.id, Some(123));
        assert!(response.error.is_some());
        assert_eq!(
            response.error.unwrap().get("message").unwrap().as_str().unwrap(),
            "Method not found"
        );
    }
}
