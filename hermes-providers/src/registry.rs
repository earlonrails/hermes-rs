use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::{debug, warn};

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
        let profile = provider.profile();
        
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

/// Global registry instance
lazy_static::lazy_static! {
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
