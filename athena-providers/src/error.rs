use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProviderError {
    #[error("Provider not found: {0}")]
    ProviderNotFound(String),
    
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    #[error("API request failed: {0}")]
    ApiRequestFailed(String),
    
    #[error("Invalid response format: {0}")]
    InvalidResponseFormat(String),
    
    #[error("Rate limited: {0}")]
    RateLimited(String),
    
    #[error("Model not found: {0}")]
    ModelNotFound(String),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("Streaming error: {0}")]
    StreamingError(String),
    
    #[error("Tool calling not supported by provider")]
    ToolCallingNotSupported,
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, ProviderError>;
