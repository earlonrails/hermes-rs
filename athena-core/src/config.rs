use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use crate::paths::{get_config_path, get_env_path};

/// Top-level Hermes configuration, persisted as ~/.hermes/config.yaml.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HermesConfig {
    /// The currently active profile name (default: "default").
    #[serde(default)]
    pub active_profile: Option<String>,

    /// Model configuration.
    #[serde(default)]
    pub model: ModelConfig,

    /// Provider-specific configuration blocks.
    #[serde(default)]
    pub providers: HashMap<String, ProviderConfig>,

    /// Fallback providers tried in order when the primary fails.
    #[serde(default)]
    pub fallback_providers: Vec<String>,

    /// Terminal backend: "local", "docker", "ssh", "modal".
    #[serde(default = "default_terminal_backend")]
    pub terminal_backend: String,

    /// Agent behaviour settings.
    #[serde(default)]
    pub agent: AgentSettings,

    /// Gateway (messaging platform) configuration.
    #[serde(default)]
    pub gateway: GatewayConfig,

    /// Tools configuration.
    #[serde(default)]
    pub tools: ToolsConfig,

    /// Internal config schema version for migrations.
    #[serde(default, rename = "_config_version")]
    pub config_version: u32,
}

fn default_terminal_backend() -> String {
    "local".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelConfig {
    /// Default model identifier, e.g. "gpt-4o".
    #[serde(default)]
    pub default: String,

    /// Active provider slug, e.g. "openai", "anthropic", "openrouter".
    #[serde(default)]
    pub provider: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProviderConfig {
    /// Human-readable name.
    #[serde(default)]
    pub name: String,

    /// Base URL override (for custom / self-hosted endpoints).
    #[serde(default)]
    pub base_url: Option<String>,

    /// Default model for this provider.
    #[serde(default)]
    pub default_model: Option<String>,

    /// API key (prefer .env, but allow inline for convenience).
    #[serde(default)]
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSettings {
    /// Maximum tool-calling iterations per conversation turn.
    #[serde(default = "default_max_iterations")]
    pub max_iterations: u32,

    /// System prompt override.
    #[serde(default)]
    pub system_prompt: Option<String>,

    /// Enable YOLO mode (auto-approve dangerous commands).
    #[serde(default)]
    pub yolo_mode: bool,
}

impl Default for AgentSettings {
    fn default() -> Self {
        Self {
            max_iterations: 20,
            system_prompt: None,
            yolo_mode: false,
        }
    }
}

fn default_max_iterations() -> u32 {
    20
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GatewayConfig {
    /// Telegram bot token (prefer .env).
    #[serde(default)]
    pub telegram_enabled: bool,

    /// Discord bot token (prefer .env).
    #[serde(default)]
    pub discord_enabled: bool,

    /// Slack integration.
    #[serde(default)]
    pub slack_enabled: bool,

    /// WhatsApp bridge.
    #[serde(default)]
    pub whatsapp_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolsConfig {
    /// Tool names that are explicitly disabled.
    #[serde(default)]
    pub disabled: Vec<String>,

    /// Extra tool directories to scan.
    #[serde(default)]
    pub extra_dirs: Vec<String>,
}

// ─── Load / Save ───────────────────────────────────────────────────────────

/// Load the Hermes config from ~/.hermes/config.yaml, or return defaults.
pub fn load_config() -> HermesConfig {
    let path = get_config_path();
    match fs::read_to_string(&path) {
        Ok(contents) => match serde_yaml::from_str(&contents) {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!("Warning: failed to parse {}: {}", path.display(), e);
                HermesConfig::default()
            }
        },
        Err(_) => HermesConfig::default(),
    }
}

/// Save the Hermes config to ~/.hermes/config.yaml.
pub fn save_config(config: &HermesConfig) -> Result<(), String> {
    let path = get_config_path();
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create config dir: {}", e))?;
    }
    let yaml = serde_yaml::to_string(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    fs::write(&path, yaml).map_err(|e| format!("Failed to write config: {}", e))?;
    Ok(())
}

// ─── .env helpers ──────────────────────────────────────────────────────────

/// Read a value from ~/.hermes/.env (simple KEY=VALUE parser).
pub fn get_env_value(key: &str) -> Option<String> {
    // First check process environment
    if let Ok(val) = std::env::var(key) {
        return Some(val);
    }
    // Then check .env file
    let env_path = get_env_path();
    let contents = fs::read_to_string(&env_path).ok()?;
    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            let k = k.trim();
            let v = v.trim().trim_matches('"').trim_matches('\'');
            if k == key {
                return Some(v.to_string());
            }
        }
    }
    None
}

/// Save or update a key in ~/.hermes/.env.
pub fn save_env_value(key: &str, value: &str) -> Result<(), String> {
    let env_path = get_env_path();
    if let Some(parent) = env_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create .env dir: {}", e))?;
    }

    let existing = fs::read_to_string(&env_path).unwrap_or_default();
    let mut found = false;
    let mut lines: Vec<String> = existing
        .lines()
        .map(|line| {
            let trimmed = line.trim();
            if let Some((k, _)) = trimmed.split_once('=') {
                if k.trim() == key {
                    found = true;
                    return format!("{}={}", key, value);
                }
            }
            line.to_string()
        })
        .collect();

    if !found {
        lines.push(format!("{}={}", key, value));
    }

    let output = lines.join("\n") + "\n";
    fs::write(&env_path, output).map_err(|e| format!("Failed to write .env: {}", e))?;

    Ok(())
}

/// Remove a key from ~/.hermes/.env.
pub fn remove_env_value(key: &str) -> Result<(), String> {
    let env_path = get_env_path();
    let existing = fs::read_to_string(&env_path).unwrap_or_default();
    let lines: Vec<&str> = existing
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            if let Some((k, _)) = trimmed.split_once('=') {
                k.trim() != key
            } else {
                true
            }
        })
        .collect();
    let output = lines.join("\n") + "\n";
    fs::write(&env_path, output).map_err(|e| format!("Failed to write .env: {}", e))?;
    Ok(())
}

/// Check if any provider is configured (has an API key set).
pub fn has_any_provider_configured() -> bool {
    let env_keys = [
        "OPENAI_API_KEY",
        "ANTHROPIC_API_KEY",
        "OPENROUTER_API_KEY",
        "GOOGLE_API_KEY",
        "DEEPSEEK_API_KEY",
        "GROQ_API_KEY",
        "MISTRAL_API_KEY",
        "XAI_API_KEY",
        "TOGETHER_API_KEY",
        "FIREWORKS_API_KEY",
    ];
    for key in &env_keys {
        if get_env_value(key).is_some() {
            return true;
        }
    }
    false
}

/// Return the Hermes home directory as a displayable string.
pub fn hermes_home_display() -> String {
    crate::paths::display_hermes_home()
}
