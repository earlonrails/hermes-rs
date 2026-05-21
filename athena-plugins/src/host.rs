use std::sync::Arc;
use athena_tools::ToolRegistry;

/// State provided to the Wasm guest from the Rust host
pub struct HermesHost {
    pub registry: Option<Arc<ToolRegistry>>,
}

impl HermesHost {
    pub fn new() -> Self {
        Self {
            registry: None,
        }
    }
    
    pub fn with_registry(registry: Arc<ToolRegistry>) -> Self {
        Self {
            registry: Some(registry),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hermes_host_new() {
        let host = HermesHost::new();
        assert!(host.registry.is_none());
    }

    #[test]
    fn test_hermes_host_with_registry() {
        let registry = Arc::new(ToolRegistry::new());
        let host = HermesHost::with_registry(registry.clone());
        assert!(host.registry.is_some());
        assert_eq!(Arc::strong_count(&registry), 2);
    }
}
