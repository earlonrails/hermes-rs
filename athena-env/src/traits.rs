use async_trait::async_trait;
use thiserror::Error;
use std::collections::HashMap;

#[derive(Error, Debug)]
pub enum EnvError {
    #[error("Environment initialization failed: {0}")]
    InitFailed(String),
    
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    
    #[error("Resource not found: {0}")]
    NotFound(String),
    
    #[error("Timeout: {0}")]
    Timeout(String),
}

/// The result of an execution within an environment
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// Configuration for an execution
#[derive(Debug, Clone, Default)]
pub struct ExecutionConfig {
    pub timeout_seconds: Option<u64>,
    pub env_vars: HashMap<String, String>,
    pub working_dir: Option<String>,
}

/// A sandboxed environment where code and tools can be executed
#[async_trait]
pub trait Environment: Send + Sync {
    /// Name or ID of the environment
    fn id(&self) -> &str;
    
    /// Initialize or start the environment
    async fn init(&self) -> Result<(), EnvError>;
    
    /// Execute a command in the environment
    async fn execute(&self, command: &str, config: ExecutionConfig) -> Result<ExecutionResult, EnvError>;
    
    /// Write a file to the environment
    async fn write_file(&self, path: &str, content: &[u8]) -> Result<(), EnvError>;
    
    /// Read a file from the environment
    async fn read_file(&self, path: &str) -> Result<Vec<u8>, EnvError>;
    
    /// Cleanup and destroy the environment
    async fn destroy(&self) -> Result<(), EnvError>;
}

// Rust guideline compliant 2026-02-21
