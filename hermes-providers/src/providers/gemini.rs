use async_openai::{
    Client,
    config::OpenAIConfig,
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

use crate::base::*;
use crate::error::*;
use crate::registry::register_provider;

/// Gemini provider profile
pub fn gemini_profile() -> ProviderProfile {
    ProviderProfile {
        name: "gemini".to_string(),
        api_mode: ApiMode::ChatCompletions,
        aliases: vec!["google".to_string(), "gemini-api".to_string()],
        display_name: "Google Gemini".to_string(),
        description: "Google Gemini via OpenAI compatibility layer".to_string(),
        signup_url: "https://aistudio.google.com/app/apikey".to_string(),
        env_vars: vec!["GEMINI_API_KEY".to_string()],
        base_url: "https://generativelanguage.googleapis.com/v1beta/openai".to_string(),
        models_url: "https://generativelanguage.googleapis.com/v1beta/openai/models".to_string(),
        auth_type: AuthType::ApiKey,
        supports_health_check: true,
        fallback_models: vec![
            "gemini-1.5-pro".to_string(),
            "gemini-1.5-flash".to_string(),
        ],
        hostname: "generativelanguage.googleapis.com".to_string(),
        default_headers: HashMap::new(),
        fixed_temperature: None,
        default_max_tokens: None,
        default_aux_model: "gemini-1.5-flash".to_string(),
    }
}

/// Gemini provider implementation
pub struct GeminiProvider {
    profile: ProviderProfile,
}

impl GeminiProvider {
    pub fn new() -> Self {
        Self {
            profile: gemini_profile(),
        }
    }
}

#[async_trait]
impl LLMProvider for GeminiProvider {
    fn profile(&self) -> &ProviderProfile {
        &self.profile
    }
    
    async fn fetch_models(
        &self,
        api_key: Option<&str>,
        _timeout: f64,
    ) -> Result<Vec<String>, ProviderError> {
        let client = reqwest::Client::new();
        let mut request = client.request(reqwest::Method::GET, &self.profile.models_url);
        
        let key = api_key.map(|k| k.to_string())
            .or_else(|| std::env::var("GEMINI_API_KEY").ok());
            
        if let Some(k) = key {
            request = request.header("Authorization", format!("Bearer {}", k));
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
            
        let models = data.get("data").and_then(|d| d.as_array()).unwrap_or(&Vec::new());
        Ok(models.iter()
            .filter_map(|m| m.get("id").and_then(|id| id.as_str()).map(|s| s.to_string()))
            .collect())
    }
    
    async fn create_chat_completion(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, ProviderError> {
        let openai_provider = super::openai::OpenAIProvider::new_with_profile(
            None, // Will use async-openai default environment variable or we should inject it
            Some(self.profile.base_url.clone()),
            self.profile.clone(),
        );
        
        openai_provider.create_chat_completion(request).await
    }
    
    async fn create_chat_completion_stream(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionStream, ProviderError> {
        let openai_provider = super::openai::OpenAIProvider::new_with_profile(
            None,
            Some(self.profile.base_url.clone()),
            self.profile.clone(),
        );
        
        openai_provider.create_chat_completion_stream(request).await
    }
}

/// Register the Gemini provider
pub fn register() {
    register_provider(Arc::new(GeminiProvider::new()));
}
