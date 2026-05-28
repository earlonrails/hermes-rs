use athena_core::config::{load_config, save_config, get_env_value};
use cliclack::{intro, outro, outro_cancel, select, input, password, confirm, note};
use anyhow::Result;

pub fn run_setup() -> Result<()> {
    intro("⚕ Athena Setup Wizard")?;

    let athena_home = athena_core::paths::display_athena_home();
    note("Athena home", &athena_home)?;

    let mut config = load_config();

    setup_model_provider(&mut config)?;
    setup_terminal_backend(&mut config)?;
    setup_agent_settings(&mut config)?;
    setup_gateway(&mut config)?;

    match save_config(&config) {
        Ok(()) => {
            outro(format!("✓ Configuration saved to {}/config.yaml", athena_home))?;
        }
        Err(e) => {
            outro_cancel(format!("✗ Failed to save config: {}", e))?;
        }
    }

    Ok(())
}

pub fn apply_model_provider(config: &mut athena_core::config::AthenaConfig, slug: &str, default_model: &str) {
    config.model.provider = slug.to_string();
    config.model.default = default_model.to_string();
}

pub fn get_env_key_for_provider(slug: &str) -> &'static str {
    match slug {
        "openai" => "OPENAI_API_KEY",
        "anthropic" => "ANTHROPIC_API_KEY",
        "openrouter" => "OPENROUTER_API_KEY",
        "google" => "GOOGLE_API_KEY",
        "deepseek" => "DEEPSEEK_API_KEY",
        "groq" => "GROQ_API_KEY",
        "mistral" => "MISTRAL_API_KEY",
        "xai" => "XAI_API_KEY",
        _ => "",
    }
}

pub fn get_default_model_for_provider(slug: &str) -> &'static str {
    match slug {
        "openai" => "gpt-4o",
        "anthropic" => "claude-sonnet-4-20250514",
        "openrouter" => "openai/gpt-4o",
        "google" => "gemini-2.5-pro",
        "deepseek" => "deepseek-chat",
        "groq" => "llama-3.3-70b-versatile",
        "mistral" => "mistral-large-latest",
        "xai" => "grok-3",
        _ => "gpt-4o",
    }
}

pub fn apply_terminal_backend(config: &mut athena_core::config::AthenaConfig, backend: &str) {
    config.terminal_backend = backend.to_string();
}

pub fn apply_agent_iterations(config: &mut athena_core::config::AthenaConfig, max_iter: u32) {
    config.agent.max_iterations = max_iter;
}

pub fn apply_gateway_settings(config: &mut athena_core::config::AthenaConfig, enable_telegram: bool, enable_discord: bool) {
    config.gateway.telegram_enabled = enable_telegram;
    config.gateway.discord_enabled = enable_discord;
}

fn setup_model_provider(config: &mut athena_core::config::AthenaConfig) -> Result<()> {
    let providers = [
        ("openai", "OpenAI", "GPT-4o, o1, o3, ..."),
        ("anthropic", "Anthropic", "Claude 4, Opus, Sonnet, ..."),
        ("openrouter", "OpenRouter", "Access many providers"),
        ("google", "Google", "Gemini"),
        ("deepseek", "DeepSeek", ""),
        ("groq", "Groq", ""),
        ("mistral", "Mistral AI", ""),
        ("xai", "xAI", "Grok"),
        ("local", "Local", "Custom endpoint"),
    ];

    let mut select_prompt = select("Select your inference provider");
    for (slug, label, hint) in providers.iter() {
        select_prompt = select_prompt.item(*slug, *label, *hint);
    }
    
    if !config.model.provider.is_empty() {
        select_prompt = select_prompt.initial_value(config.model.provider.as_str());
    }
    
    let slug: String = select_prompt.interact()?.to_string();

    let env_key = get_env_key_for_provider(&slug);
    if !env_key.is_empty() {
        let current = get_env_value(env_key);
        if current.is_some() {
            let update: bool = confirm(format!("Current {} exists. Update it?", env_key)).interact()?;
            if update {
                let key: String = password(format!("Enter {} (will be masked)", env_key)).interact()?;
                if !key.is_empty() {
                    let _ = athena_core::config::save_env_value(env_key, &key);
                }
            }
        } else {
            let key: String = password(format!("Enter {} (will be masked)", env_key)).interact()?;
            if !key.is_empty() {
                let _ = athena_core::config::save_env_value(env_key, &key);
            }
        }
    } else if slug == "local" {
        let url: String = input("Base URL")
            .placeholder("http://localhost:11434/v1")
            .interact()?;
        if !url.is_empty() {
            let _ = athena_core::config::save_env_value("OPENAI_BASE_URL", &url);
        }
    }

    let default_model = get_default_model_for_provider(&slug);
    let model_input: String = input("Default model")
        .default_input(default_model)
        .interact()?;
    
    let final_model = if model_input.is_empty() {
        default_model.to_string()
    } else {
        model_input
    };

    apply_model_provider(config, &slug, &final_model);
    Ok(())
}

fn setup_terminal_backend(config: &mut athena_core::config::AthenaConfig) -> Result<()> {
    let mut select_prompt = select("Where should Athena run terminal commands?")
        .item("local", "Local", "Run directly on your machine")
        .item("docker", "Docker", "Run in an isolated container")
        .item("ssh", "SSH", "Run on a remote server")
        .item("modal", "Modal", "Run in Modal cloud");
        
    if !config.terminal_backend.is_empty() {
        select_prompt = select_prompt.initial_value(config.terminal_backend.as_str());
    }
        
    let backend: String = select_prompt.interact()?.to_string();
    apply_terminal_backend(config, &backend);
    Ok(())
}

fn setup_agent_settings(config: &mut athena_core::config::AthenaConfig) -> Result<()> {
    let max_iter: String = input("Max iterations per turn")
        .default_input(&config.agent.max_iterations.to_string())
        .interact()?;
        
    if let Ok(n) = max_iter.parse::<u32>() {
        apply_agent_iterations(config, n);
    }
    
    Ok(())
}

fn setup_gateway(config: &mut athena_core::config::AthenaConfig) -> Result<()> {
    let enable_telegram: bool = confirm("Enable Telegram?").initial_value(config.gateway.telegram_enabled).interact()?;
    
    if enable_telegram {
        if get_env_value("TELEGRAM_BOT_TOKEN").is_none() {
            let key: String = password("TELEGRAM_BOT_TOKEN").interact()?;
            if !key.is_empty() {
                let _ = athena_core::config::save_env_value("TELEGRAM_BOT_TOKEN", &key);
            }
        } else {
            note("Telegram", "Token already configured")?;
        }
    }

    let enable_discord: bool = confirm("Enable Discord?").initial_value(config.gateway.discord_enabled).interact()?;
    
    if enable_discord {
        if get_env_value("DISCORD_BOT_TOKEN").is_none() {
            let key: String = password("DISCORD_BOT_TOKEN").interact()?;
            if !key.is_empty() {
                let _ = athena_core::config::save_env_value("DISCORD_BOT_TOKEN", &key);
            }
        } else {
            note("Discord", "Token already configured")?;
        }
    }

    apply_gateway_settings(config, enable_telegram, enable_discord);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use athena_core::config::AthenaConfig;

    #[test]
    fn test_apply_model_provider() {
        let mut config = AthenaConfig::default();
        apply_model_provider(&mut config, "anthropic", "claude-opus");
        assert_eq!(config.model.provider, "anthropic");
        assert_eq!(config.model.default, "claude-opus");
    }

    #[test]
    fn test_get_env_key_for_provider() {
        assert_eq!(get_env_key_for_provider("openai"), "OPENAI_API_KEY");
        assert_eq!(get_env_key_for_provider("mistral"), "MISTRAL_API_KEY");
        assert_eq!(get_env_key_for_provider("unknown"), "");
    }

    #[test]
    fn test_get_default_model_for_provider() {
        assert_eq!(get_default_model_for_provider("openai"), "gpt-4o");
        assert_eq!(get_default_model_for_provider("google"), "gemini-2.5-pro");
        assert_eq!(get_default_model_for_provider("unknown"), "gpt-4o");
    }

    #[test]
    fn test_apply_terminal_backend() {
        let mut config = AthenaConfig::default();
        apply_terminal_backend(&mut config, "docker");
        assert_eq!(config.terminal_backend, "docker");
    }

    #[test]
    fn test_apply_agent_iterations() {
        let mut config = AthenaConfig::default();
        apply_agent_iterations(&mut config, 42);
        assert_eq!(config.agent.max_iterations, 42);
    }

    #[test]
    fn test_apply_gateway_settings() {
        let mut config = AthenaConfig::default();
        apply_gateway_settings(&mut config, true, false);
        assert!(config.gateway.telegram_enabled);
        assert!(!config.gateway.discord_enabled);
    }
}

// Rust guideline compliant 2026-02-21
