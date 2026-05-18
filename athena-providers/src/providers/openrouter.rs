use async_openai::{
    Client,
    config::OpenAIConfig,
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::debug;

use crate::base::*;
use crate::error::*;
use crate::registry::register_provider;

/// OpenRouter provider profile
pub fn openrouter_profile() -> ProviderProfile {
    let mut default_headers = HashMap::new();
    default_headers.insert("HTTP-Referer".to_string(), "https://github.com/hermes-ai/hermes-rs".to_string());
    default_headers.insert("X-Title".to_string(), "Hermes-RS".to_string());
    
    ProviderProfile {
        name: "openrouter".to_string(),
        api_mode: ApiMode::ChatCompletions,
        aliases: vec!["or".to_string()],
        display_name: "OpenRouter".to_string(),
        description: "OpenRouter — unified API for 200+ models".to_string(),
        signup_url: "https://openrouter.ai/keys".to_string(),
        env_vars: vec!["OPENROUTER_API_KEY".to_string()],
        base_url: "https://openrouter.ai/api/v1".to_string(),
        models_url: "https://openrouter.ai/api/v1/models".to_string(),
        auth_type: AuthType::ApiKey,
        supports_health_check: true,
        fallback_models: vec![
            "anthropic/claude-sonnet-4.6".to_string(),
            "openai/gpt-5.4".to_string(),
            "deepseek/deepseek-chat".to_string(),
            "google/gemini-3-flash-preview".to_string(),
            "qwen/qwen3-plus".to_string(),
        ],
        hostname: "openrouter.ai".to_string(),
        default_headers,
        fixed_temperature: None,
        default_max_tokens: None,
        default_aux_model: String::new(),
    }
}

/// OpenRouter provider implementation
/// Uses async-openai with OpenRouter's OpenAI-compatible endpoint
pub struct OpenRouterProvider {
    profile: ProviderProfile,
    client: Client<OpenAIConfig>,
}

impl OpenRouterProvider {
    pub fn new(api_key: Option<String>) -> Self {
        let mut config = OpenAIConfig::new();
        if let Some(key) = api_key {
            config = config.with_api_key(key);
        }
        config = config.with_api_base("https://openrouter.ai/api/v1");
        
        Self {
            profile: openrouter_profile(),
            client: Client::with_config(config),
        }
    }
    
    pub fn new_with_profile(api_key: Option<String>, profile: ProviderProfile) -> Self {
        let mut config = OpenAIConfig::new();
        if let Some(key) = api_key {
            config = config.with_api_key(key);
        }
        config = config.with_api_base(profile.base_url.clone());
        
        Self { profile, client: Client::with_config(config) }
    }
}

#[async_trait]
impl LLMProvider for OpenRouterProvider {
    fn profile(&self) -> &ProviderProfile {
        &self.profile
    }
    
    async fn fetch_models(
        &self,
        api_key: Option<&str>,
        _timeout: f64,
    ) -> std::result::Result<Vec<String>, ProviderError> {
        // OpenRouter has a public models endpoint that doesn't require auth
        let client = reqwest::Client::new();
        let url = "https://openrouter.ai/api/v1/models";
        
        let mut request = client.request(reqwest::Method::GET, url);
        
        // Add headers
        request = request
            .header("Accept", "application/json")
            .header("HTTP-Referer", "https://github.com/hermes-ai/hermes-rs")
            .header("X-Title", "Hermes-RS");
        
        for (key, value) in &self.profile.default_headers {
            request = request.header(key, value);
        }
        
        let response = request.send().await
            .map_err(|e| ProviderError::ApiRequestFailed(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(ProviderError::ApiRequestFailed(format!(
                "HTTP {}: {}", 
                response.status(), 
                response.text().await.unwrap_or_default()
            )));
        }
        
        let data: serde_json::Value = response.json().await
            .map_err(|e| ProviderError::InvalidResponseFormat(e.to_string()))?;
        
        let default_vec = Vec::new();
        let models = data.get("data").and_then(|d| d.as_array()).unwrap_or(&default_vec);
        
        Ok(models.iter()
            .filter_map(|m| m.get("id").and_then(|id| id.as_str()).map(|s| s.to_string()))
            .collect())
    }
    
    async fn create_chat_completion(
        &self,
        request: ChatCompletionRequest,
    ) -> std::result::Result<ChatCompletionResponse, ProviderError> {
        // OpenRouter uses OpenAI-compatible format but with some extensions
        // We can reuse the OpenAI provider's implementation with a different base URL
        
        // For now, delegate to the OpenAI provider with OpenRouter's base URL
        let openai_provider = super::openai::OpenAIProvider::new_with_profile(
            None,
            Some(self.profile.base_url.clone()),
            self.profile.clone(),
        );
        
        openai_provider.create_chat_completion(request).await
    }
    
    async fn create_chat_completion_stream(
        &self,
        request: ChatCompletionRequest,
    ) -> std::result::Result<ChatCompletionStream, ProviderError> {
        let openai_provider = super::openai::OpenAIProvider::new_with_profile(
            None,
            Some(self.profile.base_url.clone()),
            self.profile.clone(),
        );
        
        openai_provider.create_chat_completion_stream(request).await
    }
}

/// Register the OpenRouter provider
pub fn register() {
    register_provider(Arc::new(OpenRouterProvider::new(None)));
}
