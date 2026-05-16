use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use async_trait::async_trait;

/// API mode types supported by providers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ApiMode {
    /// Standard OpenAI-style chat completions
    ChatCompletions,
    /// Anthropic Messages API
    AnthropicMessages,
    /// Codex-style responses API
    CodexResponses,
    /// Custom/unknown API mode
    Custom(String),
}

impl Default for ApiMode {
    fn default() -> Self {
        ApiMode::ChatCompletions
    }
}

/// Authentication types for providers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AuthType {
    /// API key authentication (Bearer token)
    ApiKey,
    /// OAuth device code flow
    OAuthDeviceCode,
    /// External OAuth
    OAuthExternal,
    /// GitHub Copilot authentication
    Copilot,
    /// AWS SDK authentication
    AwsSdk,
    /// No authentication required
    None,
}

impl Default for AuthType {
    fn default() -> Self {
        AuthType::ApiKey
    }
}

/// Provider profile - declarative configuration for an LLM provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderProfile {
    /// Unique name of the provider
    pub name: String,
    
    /// API mode this provider uses
    #[serde(default)]
    pub api_mode: ApiMode,
    
    /// Alternative names for this provider
    #[serde(default)]
    pub aliases: Vec<String>,
    
    /// Human-readable display name
    #[serde(default)]
    pub display_name: String,
    
    /// Description shown in picker
    #[serde(default)]
    pub description: String,
    
    /// Signup URL for the provider
    #[serde(default)]
    pub signup_url: String,
    
    /// Environment variables for authentication
    #[serde(default)]
    pub env_vars: Vec<String>,
    
    /// Base URL for API requests
    #[serde(default)]
    pub base_url: String,
    
    /// Explicit models endpoint URL
    #[serde(default)]
    pub models_url: String,
    
    /// Authentication type
    #[serde(default)]
    pub auth_type: AuthType,
    
    /// Whether this provider supports health checks
    #[serde(default = "default_true")]
    pub supports_health_check: bool,
    
    /// Fallback models shown when live fetch fails
    #[serde(default)]
    pub fallback_models: Vec<String>,
    
    /// Base hostname for URL-to-provider reverse mapping
    #[serde(default)]
    pub hostname: String,
    
    /// Default headers to include in requests
    #[serde(default)]
    pub default_headers: HashMap<String, String>,
    
    /// Fixed temperature value (None = use caller's default)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fixed_temperature: Option<f32>,
    
    /// Default max tokens
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_max_tokens: Option<u64>,
    
    /// Cheap model for auxiliary tasks (compression, vision, etc.)
    #[serde(default)]
    pub default_aux_model: String,
}

fn default_true() -> bool {
    true
}

impl ProviderProfile {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            api_mode: ApiMode::default(),
            aliases: Vec::new(),
            display_name: String::new(),
            description: String::new(),
            signup_url: String::new(),
            env_vars: Vec::new(),
            base_url: String::new(),
            models_url: String::new(),
            auth_type: AuthType::default(),
            supports_health_check: true,
            fallback_models: Vec::new(),
            hostname: String::new(),
            default_headers: HashMap::new(),
            fixed_temperature: None,
            default_max_tokens: None,
            default_aux_model: String::new(),
        }
    }
    
    /// Get the hostname derived from base_url if not explicitly set
    pub fn get_hostname(&self) -> String {
        if !self.hostname.is_empty() {
            return self.hostname.clone();
        }
        if !self.base_url.is_empty() {
            if let Ok(url) = url::Url::parse(&self.base_url) {
                return url.host().unwrap_or_default().to_string();
            }
        }
        String::new()
    }
    
    /// Prepare messages before sending to the API
    pub fn prepare_messages(&self, messages: Vec<serde_json::Value>) -> Vec<serde_json::Value> {
        messages
    }
    
    /// Build extra body fields for the API request
    pub fn build_extra_body(
        &self,
        session_id: Option<&str>,
        context: &HashMap<String, serde_json::Value>,
    ) -> HashMap<String, serde_json::Value> {
        HashMap::new()
    }
    
    /// Build provider-specific API kwargs
    pub fn build_api_kwargs_extras(
        &self,
        reasoning_config: Option<&HashMap<String, serde_json::Value>>,
        context: &HashMap<String, serde_json::Value>,
    ) -> (HashMap<String, serde_json::Value>, HashMap<String, serde_json::Value>) {
        (HashMap::new(), HashMap::new())
    }
}

/// Trait for LLM providers - async interface for making requests
#[async_trait]
pub trait LLMProvider: Send + Sync {
    /// Get the provider profile
    fn profile(&self) -> &ProviderProfile;
    
    /// Fetch the list of available models from the provider
    async fn fetch_models(
        &self,
        api_key: Option<&str>,
        timeout: f64,
    ) -> Result<Vec<String>, crate::ProviderError>;
    
    /// Create a chat completion request
    async fn create_chat_completion(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, crate::ProviderError>;
    
    /// Create a streaming chat completion request
    async fn create_chat_completion_stream(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionStream, crate::ProviderError>;
}

/// Request for chat completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u64>,
    pub top_p: Option<f32>,
    pub stop: Option<Vec<String>>,
    pub stream: bool,
    pub tools: Option<Vec<ToolDefinition>>,
    pub tool_choice: Option<ToolChoice>,
    pub extra_body: HashMap<String, serde_json::Value>,
}

/// Message in a chat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
    pub name: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub tool_call_id: Option<String>,
}

/// Role of a message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

/// Tool call from the model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub r#type: String,
    pub function: ToolFunction,
}

/// Function definition for a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolFunction {
    pub name: String,
    pub arguments: String,
}

/// Tool definition schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub r#type: String,
    pub function: ToolSchema,
}

/// Schema for a tool function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    pub name: String,
    pub description: Option<String>,
    pub parameters: serde_json::Value,
}

/// Tool choice setting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolChoice {
    None,
    Auto,
    Required,
    Specific(String),
}

/// Response from chat completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub model: String,
    pub created: u64,
    pub choices: Vec<Choice>,
    pub usage: Option<Usage>,
}

/// A single choice in the response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    pub index: usize,
    pub message: ChatMessage,
    pub finish_reason: Option<String>,
}

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
}

/// Streaming response from chat completion
pub struct ChatCompletionStream {
    pub response: Box<dyn futures::Stream<Item = Result<StreamChunk, crate::ProviderError>> + Send + Unpin>,
}

/// A chunk in the streaming response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    pub id: String,
    pub model: String,
    pub created: Option<u64>,
    pub choices: Vec<StreamChoice>,
}

/// A choice in a stream chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChoice {
    pub index: usize,
    pub delta: StreamDelta,
    pub finish_reason: Option<String>,
}

/// Delta for streaming response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamDelta {
    pub role: Option<MessageRole>,
    pub content: Option<String>,
    pub tool_calls: Option<Vec<StreamToolCall>>,
}

/// Tool call in streaming response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamToolCall {
    pub id: Option<String>,
    pub r#type: Option<String>,
    pub function: Option<StreamToolFunction>,
}

/// Function in streaming tool call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamToolFunction {
    pub name: Option<String>,
    pub arguments: Option<String>,
}

/// Sentinel value for omitting temperature
pub const OMIT_TEMPERATURE: Option<f32> = None;
