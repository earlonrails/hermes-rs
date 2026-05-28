use std::sync::Arc;
use athena_tools::ToolRegistry;

/// State provided to the Wasm guest from the Rust host
pub struct AthenaHost {
    pub registry: Option<Arc<ToolRegistry>>,
}

impl Default for AthenaHost {
    fn default() -> Self {
        Self::new()
    }
}

impl AthenaHost {
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
    fn test_athena_host_new() {
        let host = AthenaHost::new();
        assert!(host.registry.is_none());
    }

    #[test]
    fn test_athena_host_with_registry() {
        let registry = Arc::new(ToolRegistry::new());
        let host = AthenaHost::with_registry(registry.clone());
        assert!(host.registry.is_some());
        assert_eq!(Arc::strong_count(&registry), 2);
    }
}

// Rust guideline compliant 2026-02-21
