use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::debug;

use crate::{ProviderProfile, LLMProvider};

/// Global provider registry
pub struct ProviderRegistry {
    profiles: RwLock<HashMap<String, Arc<dyn LLMProvider + Send + Sync>>>, 
    aliases: RwLock<HashMap<String, String>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            profiles: RwLock::new(HashMap::new()),
            aliases: RwLock::new(HashMap::new()),
        }
    }
    
    /// Register a provider
    pub fn register(&self, provider: Arc<dyn LLMProvider + Send + Sync>) {
        let profile = provider.profile().clone();
        
        // Register by name
        if let Ok(mut profiles) = self.profiles.write() {
            profiles.insert(profile.name.clone(), provider);
        }
        
        // Register aliases
        if let Ok(mut aliases) = self.aliases.write() {
            for alias in &profile.aliases {
                aliases.insert(alias.clone(), profile.name.clone());
            }
        }
        
        debug!("Registered provider: {}", profile.name);
    }
    
    /// Get a provider by name or alias
    pub fn get(&self, name: &str) -> Option<Arc<dyn LLMProvider + Send + Sync>> {
        // Check if it's an alias first
        if let Ok(aliases) = self.aliases.read() {
            if let Some(canonical) = aliases.get(name) {
                if let Ok(profiles) = self.profiles.read() {
                    return profiles.get(canonical).cloned();
                }
            }
        }
        
        // Check direct name
        if let Ok(profiles) = self.profiles.read() {
            profiles.get(name).cloned()
        } else {
            None
        }
    }
    
    /// Get a provider profile by name or alias
    pub fn get_profile(&self, name: &str) -> Option<ProviderProfile> {
        self.get(name).map(|p| p.profile().clone())
    }
    
    /// List all registered provider names
    pub fn list_providers(&self) -> Vec<String> {
        if let Ok(profiles) = self.profiles.read() {
            profiles.keys().cloned().collect()
        } else {
            Vec::new()
        }
    }
    
    /// List all registered provider profiles
    pub fn list_provider_profiles(&self) -> Vec<ProviderProfile> {
        if let Ok(profiles) = self.profiles.read() {
            profiles.values().map(|p| p.profile().clone()).collect()
        } else {
            Vec::new()
        }
    }
    
    /// Check if a provider is registered
    pub fn contains(&self, name: &str) -> bool {
        // Check alias first
        if let Ok(aliases) = self.aliases.read() {
            if aliases.contains_key(name) {
                return true;
            }
        }
        if let Ok(profiles) = self.profiles.read() {
            profiles.contains_key(name)
        } else {
            false
        }
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

lazy_static::lazy_static! {
    /// Global registry instance
    pub static ref GLOBAL_REGISTRY: ProviderRegistry = ProviderRegistry::new();
}

/// Register a provider globally
pub fn register_provider(provider: Arc<dyn LLMProvider + Send + Sync>) {
    GLOBAL_REGISTRY.register(provider);
}

/// Get a provider by name or alias from the global registry
pub fn get_provider(name: &str) -> Option<Arc<dyn LLMProvider + Send + Sync>> {
    GLOBAL_REGISTRY.get(name)
}

/// Get a provider profile by name or alias from the global registry
pub fn get_provider_profile(name: &str) -> Option<ProviderProfile> {
    GLOBAL_REGISTRY.get_profile(name)
}

/// List all registered provider names from the global registry
pub fn list_providers() -> Vec<String> {
    GLOBAL_REGISTRY.list_providers()
}

/// List all registered provider profiles from the global registry
pub fn list_provider_profiles() -> Vec<ProviderProfile> {
    GLOBAL_REGISTRY.list_provider_profiles()
}

/// Initialize the registry with built-in providers
pub fn init_builtin_providers() {
    debug!("Initializing built-in providers");
    crate::providers::openai::register();
    crate::providers::anthropic::register();
    crate::providers::openrouter::register();
    crate::providers::mistral::register();
    crate::providers::gemini::register();
    crate::providers::xai::register();
}

/// Macro for easy provider registration
#[macro_export]
macro_rules! register_provider {
    ($provider:expr) => {
        crate::registry::register_provider(Arc::new($provider));
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use crate::{
        ChatCompletionRequest, ChatCompletionResponse, ChatCompletionStream, ProviderError,
    };

    struct MockProvider {
        profile: ProviderProfile,
    }

    impl MockProvider {
        fn new(name: &str, aliases: Vec<&str>) -> Self {
            let mut profile = ProviderProfile::new(name);
            profile.aliases = aliases.into_iter().map(|s| s.to_string()).collect();
            Self { profile }
        }
    }

    #[async_trait]
    impl LLMProvider for MockProvider {
        fn profile(&self) -> &ProviderProfile {
            &self.profile
        }
        
        async fn fetch_models(
            &self,
            _api_key: Option<&str>,
            _timeout: f64,
        ) -> Result<Vec<String>, ProviderError> {
            Ok(vec![])
        }
        
        async fn create_chat_completion(
            &self,
            _request: ChatCompletionRequest,
        ) -> Result<ChatCompletionResponse, ProviderError> {
            Err(ProviderError::ApiRequestFailed("mock".into()))
        }
        
        async fn create_chat_completion_stream(
            &self,
            _request: ChatCompletionRequest,
        ) -> Result<ChatCompletionStream, ProviderError> {
            Err(ProviderError::ApiRequestFailed("mock".into()))
        }
    }

    #[test]
    fn test_registry_basic() {
        let registry = ProviderRegistry::default();
        assert!(!registry.contains("mock_provider"));
        
        let provider = Arc::new(MockProvider::new("mock_provider", vec!["mp1", "mp2"]));
        registry.register(provider);
        
        assert!(registry.contains("mock_provider"));
        assert!(registry.contains("mp1"));
        assert!(registry.contains("mp2"));
        assert!(!registry.contains("mp3"));

        let p1 = registry.get("mock_provider").unwrap();
        assert_eq!(p1.profile().name, "mock_provider");

        let p2 = registry.get("mp1").unwrap();
        assert_eq!(p2.profile().name, "mock_provider");

        let p3 = registry.get_profile("mp2").unwrap();
        assert_eq!(p3.name, "mock_provider");

        let names = registry.list_providers();
        assert_eq!(names.len(), 1);
        assert_eq!(names[0], "mock_provider");

        let profiles = registry.list_provider_profiles();
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].name, "mock_provider");
    }

    #[test]
    fn test_global_registry() {
        let provider = Arc::new(MockProvider::new("global_mock", vec!["g1"]));
        register_provider(provider);

        assert!(get_provider("global_mock").is_some());
        assert!(get_provider_profile("g1").is_some());
        assert!(list_providers().contains(&"global_mock".to_string()));
        assert!(list_provider_profiles().iter().any(|p| p.name == "global_mock"));
    }
}

// Rust guideline compliant 2026-02-21
