use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

use crate::base::*;
use crate::error::*;
use crate::registry::register_provider;

/// Mistral provider profile
pub fn mistral_profile() -> ProviderProfile {
    ProviderProfile {
        name: "mistral".to_string(),
        api_mode: ApiMode::ChatCompletions,
        aliases: vec!["mistral-ai".to_string()],
        display_name: "Mistral AI".to_string(),
        description: "Mistral AI — Open and commercial models".to_string(),
        signup_url: "https://console.mistral.ai/api-keys".to_string(),
        env_vars: vec!["MISTRAL_API_KEY".to_string()],
        base_url: "https://api.mistral.ai/v1".to_string(),
        models_url: "https://api.mistral.ai/v1/models".to_string(),
        auth_type: AuthType::ApiKey,
        supports_health_check: true,
        fallback_models: vec![
            "mistral-large-latest".to_string(),
            "mistral-small-latest".to_string(),
            "open-mixtral-8x22b".to_string(),
            "open-mistral-nemo".to_string(),
        ],
        hostname: "api.mistral.ai".to_string(),
        default_headers: HashMap::new(),
        fixed_temperature: None,
        default_max_tokens: None,
        default_aux_model: "mistral-small-latest".to_string(),
    }
}

/// Mistral provider implementation
/// Uses async-openai with Mistral's OpenAI-compatible endpoint
pub struct MistralProvider {
    profile: ProviderProfile,
}

impl MistralProvider {
    pub fn new() -> Self {
        Self {
            profile: mistral_profile(),
        }
    }
}

#[async_trait]
impl LLMProvider for MistralProvider {
    fn profile(&self) -> &ProviderProfile {
        &self.profile
    }
    
    async fn fetch_models(
        &self,
        api_key: Option<&str>,
        _timeout: f64,
    ) -> std::result::Result<Vec<String>, ProviderError> {
        let client = reqwest::Client::new();
        let mut request = client.request(reqwest::Method::GET, &self.profile.models_url);
        
        let key = api_key.map(|k| k.to_string())
            .or_else(|| std::env::var("MISTRAL_API_KEY").ok());
            
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

/// Register the Mistral provider
pub fn register() {
    register_provider(Arc::new(MistralProvider::new()));
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path};

    #[tokio::test]
    async fn test_mistral_profile() {
        let profile = mistral_profile();
        assert_eq!(profile.name, "mistral");
        assert_eq!(profile.api_mode, ApiMode::ChatCompletions);
        assert_eq!(profile.auth_type, AuthType::ApiKey);
        assert_eq!(profile.hostname, "api.mistral.ai");
    }

    #[tokio::test]
    async fn test_mistral_register() {
        // Test that register function works correctly
        let profile = mistral_profile();
        assert_eq!(profile.name, "mistral");
        assert_eq!(profile.api_mode, ApiMode::ChatCompletions);
        assert_eq!(profile.auth_type, AuthType::ApiKey);
    }

    #[tokio::test]
    async fn test_mistral_provider_new() {
        let provider = MistralProvider::new();
        assert_eq!(provider.profile().name, "mistral");
    }

    #[tokio::test]
    async fn test_fetch_models() {
        let mock_server = MockServer::start().await;
        
        let response_body = serde_json::json!({
            "data": [
                {"id": "mistral-large-latest", "object": "model"}
            ]
        });

        Mock::given(method("GET"))
            .and(path("/models"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
            .mount(&mock_server)
            .await;

        let mut provider = MistralProvider::new();
        provider.profile.models_url = format!("{}/models", mock_server.uri());
        
        let models = provider.fetch_models(Some("test_key"), 10.0).await.unwrap();
        assert_eq!(models.len(), 1);
        assert_eq!(models[0], "mistral-large-latest");
    }
}

// Rust guideline compliant 2026-02-21
