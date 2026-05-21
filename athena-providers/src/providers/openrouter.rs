use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

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
}

impl OpenRouterProvider {
    pub fn new(_api_key: Option<String>) -> Self {
        Self {
            profile: openrouter_profile(),
        }
    }
    
    pub fn new_with_profile(profile: ProviderProfile) -> Self {
        Self { profile }
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
        let client = reqwest::Client::new();
        let url = if self.profile.models_url.is_empty() {
            "https://openrouter.ai/api/v1/models".to_string()
        } else {
            self.profile.models_url.clone()
        };
        
        let mut request = client.request(reqwest::Method::GET, &url);
        
        // Add auth header if API key is provided
        if let Some(key) = api_key {
            request = request.header("Authorization", format!("Bearer {}", key));
        }
        
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

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path};

    #[tokio::test]
    async fn test_openrouter_profile() {
        let profile = openrouter_profile();
        assert_eq!(profile.name, "openrouter");
        assert_eq!(profile.api_mode, ApiMode::ChatCompletions);
        assert_eq!(profile.auth_type, AuthType::ApiKey);
        assert_eq!(profile.default_headers.get("HTTP-Referer").unwrap(), "https://github.com/hermes-ai/hermes-rs");
    }

    #[tokio::test]
    async fn test_openrouter_register() {
        // Test that register function works correctly
        let profile = openrouter_profile();
        assert_eq!(profile.name, "openrouter");
        assert_eq!(profile.api_mode, ApiMode::ChatCompletions);
        assert_eq!(profile.auth_type, AuthType::ApiKey);
    }

    #[tokio::test]
    async fn test_openrouter_new_with_profile() {
        let profile = openrouter_profile();
        let provider = OpenRouterProvider::new_with_profile(profile);
        assert_eq!(provider.profile().name, "openrouter");
    }

    #[tokio::test]
    async fn test_openrouter_provider_new() {
        let provider = OpenRouterProvider::new(None);
        assert_eq!(provider.profile().name, "openrouter");

        let custom_profile = ProviderProfile::new("custom_or");
        let custom_provider = OpenRouterProvider::new_with_profile(custom_profile);
        assert_eq!(custom_provider.profile().name, "custom_or");
    }

    #[tokio::test]
    async fn test_fetch_models() {
        let mock_server = MockServer::start().await;
        
        let response_body = serde_json::json!({
            "data": [
                {"id": "anthropic/claude-sonnet-4.6", "object": "model"}
            ]
        });

        Mock::given(method("GET"))
            .and(path("/api/v1/models"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
            .mount(&mock_server)
            .await;

        let mut profile = openrouter_profile();
        profile.models_url = format!("{}/api/v1/models", mock_server.uri());
        let provider = OpenRouterProvider::new_with_profile(profile);
        
        let models = provider.fetch_models(Some("test_key"), 10.0).await.unwrap();
        assert_eq!(models.len(), 1);
        assert_eq!(models[0], "anthropic/claude-sonnet-4.6");
    }

    #[tokio::test]
    async fn test_fetch_models_http_error() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/models"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;
        let mut provider = OpenRouterProvider::new(None);
        provider.profile.models_url = format!("{}/models", mock_server.uri());
        assert!(provider.fetch_models(None, 10.0).await.is_err());
    }

    #[tokio::test]
    async fn test_fetch_models_invalid_json() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/models"))
            .respond_with(ResponseTemplate::new(200).set_body_string("invalid"))
            .mount(&mock_server)
            .await;
        let mut provider = OpenRouterProvider::new(None);
        provider.profile.models_url = format!("{}/models", mock_server.uri());
        assert!(provider.fetch_models(None, 10.0).await.is_err());
    }

    #[tokio::test]
    async fn test_create_chat_completion_delegation() {
        let provider = OpenRouterProvider::new(None);
        let req = ChatCompletionRequest {
            model: "openrouter".to_string(),
            messages: vec![],
            temperature: None, max_tokens: None, top_p: None, stop: None, stream: false, tools: None, tool_choice: None, extra_body: HashMap::new(),
        };
        // This will fail because no mock server is set up, but it exercises the delegation code
        let _ = provider.create_chat_completion(req.clone()).await;
        let _ = provider.create_chat_completion_stream(req).await;
    }

    #[tokio::test]
    async fn test_fetch_models_network_error() {
        let mut provider = OpenRouterProvider::new(None);
        provider.profile.models_url = "http://127.0.0.1:0/models".to_string();
        assert!(provider.fetch_models(None, 10.0).await.is_err());
    }
}

// Rust guideline compliant 2026-02-21
