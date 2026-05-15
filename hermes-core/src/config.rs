use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HermesConfig {
    // Port configurations here eventually
    pub active_profile: Option<String>,
}

pub fn load_config() -> HermesConfig {
    // Stub
    HermesConfig::default()
}
