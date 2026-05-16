use std::sync::Arc;
use hermes_tools::ToolRegistry;

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
