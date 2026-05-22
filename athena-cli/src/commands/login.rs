use athena_core::config::{get_env_value, save_env_value, remove_env_value};
use cliclack::{intro, select, password, confirm, outro, outro_cancel, note};
use anyhow::Result;

pub fn run_login() -> Result<()> {
    intro("Authenticate with an Inference Provider")?;

    let providers = [
        ("openai",      "OPENAI_API_KEY"),
        ("anthropic",   "ANTHROPIC_API_KEY"),
        ("openrouter",  "OPENROUTER_API_KEY"),
        ("google",      "GOOGLE_API_KEY"),
        ("deepseek",    "DEEPSEEK_API_KEY"),
        ("groq",        "GROQ_API_KEY"),
        ("mistral",     "MISTRAL_API_KEY"),
        ("xai",         "XAI_API_KEY"),
    ];

    let mut select_prompt = select("Select provider");
    for (i, (name, _)) in providers.iter().enumerate() {
        select_prompt = select_prompt.item(i, *name, "");
    }
    let choice: usize = select_prompt.interact()?;

    let (name, env_var) = providers[choice];
    let current = get_env_value(env_var);
    if let Some(ref val) = current {
        let masked = if val.len() > 8 {
            format!("{}...{}", &val[..4], &val[val.len()-4..])
        } else {
            "****".to_string()
        };
        note("Current API key", masked)?;
    }

    let key: String = password(format!("Enter new API key for {} (or press Enter to skip)", name)).interact()?;
    let key = key.trim();

    if !key.is_empty() {
        match save_env_value(env_var, key) {
            Ok(()) => outro(format!("Saved API key for {} to ~/.athena/.env", name))?,
            Err(e) => outro_cancel(format!("Failed to save API key: {}", e))?,
        }
    } else {
        outro("No changes made.")?;
    }
    
    Ok(())
}

pub fn run_logout() -> Result<()> {
    intro("Clear Provider Authentication")?;

    let providers = [
        ("openai",      "OPENAI_API_KEY"),
        ("anthropic",   "ANTHROPIC_API_KEY"),
        ("openrouter",  "OPENROUTER_API_KEY"),
        ("google",      "GOOGLE_API_KEY"),
        ("deepseek",    "DEEPSEEK_API_KEY"),
        ("groq",        "GROQ_API_KEY"),
        ("mistral",     "MISTRAL_API_KEY"),
        ("xai",         "XAI_API_KEY"),
    ];

    let mut select_prompt = select("Select provider to log out of");
    for (i, (name, _)) in providers.iter().enumerate() {
        select_prompt = select_prompt.item(i, *name, "");
    }
    let choice: usize = select_prompt.interact()?;

    let (name, env_var) = providers[choice];
    if get_env_value(env_var).is_none() {
        outro(format!("Provider {} is already logged out.", name))?;
        return Ok(());
    }

    let confirm_out: bool = confirm(format!("Are you sure you want to log out of {}?", name)).interact()?;
    if confirm_out {
        match remove_env_value(env_var) {
            Ok(()) => outro(format!("Logged out of {} successfully.", name))?,
            Err(e) => outro_cancel(format!("Failed to remove API key: {}", e))?,
        }
    } else {
        outro_cancel("Cancelled.")?;
    }
    
    Ok(())
}
