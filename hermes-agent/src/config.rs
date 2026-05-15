use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub provider: Option<String>,
    pub api_mode: Option<String>,
    pub model: String,
    pub max_iterations: usize,
    pub tool_delay_ms: u64,
    pub enabled_toolsets: Vec<String>,
    pub disabled_toolsets: Vec<String>,
    pub save_trajectories: bool,
    pub verbose_logging: bool,
    pub quiet_mode: bool,
    pub platform: Option<String>,
    // Many more to come...
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            base_url: None,
            api_key: None,
            provider: None,
            api_mode: None,
            model: "anthropic/claude-opus-4.6".to_string(),
            max_iterations: 90,
            tool_delay_ms: 1000,
            enabled_toolsets: vec![],
            disabled_toolsets: vec![],
            save_trajectories: false,
            verbose_logging: false,
            quiet_mode: false,
            platform: None,
        }
    }
}
