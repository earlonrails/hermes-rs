use async_trait::async_trait;
use reqwest::Client;

use crate::traits::{Environment, ExecutionConfig, ExecutionResult, EnvError};

pub struct ModalEnv {
    id: String,
    client: Client,
    endpoint: String,
    token: String,
}

impl ModalEnv {
    pub fn new(id: impl Into<String>, endpoint: impl Into<String>, token: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            client: Client::new(),
            endpoint: endpoint.into(),
            token: token.into(),
        }
    }
}

#[async_trait]
impl Environment for ModalEnv {
    fn id(&self) -> &str {
        &self.id
    }
    
    async fn init(&self) -> Result<(), EnvError> {
        // Modal functions spin up instantly, no explicit init required usually
        Ok(())
    }
    
    async fn execute(&self, command: &str, _config: ExecutionConfig) -> Result<ExecutionResult, EnvError> {
        let body = serde_json::json!({
            "command": command
        });
        
        let res = self.client.post(&self.endpoint)
            .bearer_auth(&self.token)
            .json(&body)
            .send()
            .await
            .map_err(|e| EnvError::ExecutionFailed(e.to_string()))?;
            
        if !res.status().is_success() {
            return Err(EnvError::ExecutionFailed(res.status().to_string()));
        }
        
        let data: serde_json::Value = res.json().await
            .map_err(|e| EnvError::ExecutionFailed(e.to_string()))?;
            
        Ok(ExecutionResult {
            exit_code: data["exit_code"].as_i64().unwrap_or(-1) as i32,
            stdout: data["stdout"].as_str().unwrap_or_default().to_string(),
            stderr: data["stderr"].as_str().unwrap_or_default().to_string(),
        })
    }
    
    async fn write_file(&self, _path: &str, _content: &[u8]) -> Result<(), EnvError> {
        Err(EnvError::ExecutionFailed("Not implemented for Modal".to_string()))
    }
    
    async fn read_file(&self, _path: &str) -> Result<Vec<u8>, EnvError> {
        Err(EnvError::ExecutionFailed("Not implemented for Modal".to_string()))
    }
    
    async fn destroy(&self) -> Result<(), EnvError> {
        Ok(())
    }
}
