use hermes_core::config::{load_config, save_config, get_env_value, save_env_value};
use std::io::{self, Write};

/// Select default model — starts with provider selection, then model picker.
pub fn run_model() {
    let mut config = load_config();

    let current_model = if config.model.default.is_empty() {
        "(not set)".to_string()
    } else {
        config.model.default.clone()
    };

    let current_provider = if config.model.provider.is_empty() {
        "auto".to_string()
    } else {
        config.model.provider.clone()
    };

    println!();
    println!("  Current model:    {}", current_model);
    println!("  Active provider:  {}", current_provider);
    println!();

    // Provider selection
    let providers = [
        ("openai",      "OpenAI (GPT-4o, o1, o3, ...)"),
        ("anthropic",   "Anthropic (Claude 4, Opus, Sonnet, ...)"),
        ("openrouter",  "OpenRouter (access many providers)"),
        ("google",      "Google (Gemini)"),
        ("deepseek",    "DeepSeek"),
        ("groq",        "Groq"),
        ("mistral",     "Mistral AI"),
        ("xai",         "xAI (Grok)"),
        ("local",       "Local / Custom endpoint"),
    ];

    println!("  Select provider:");
    for (i, (slug, label)) in providers.iter().enumerate() {
        let marker = if current_provider == *slug { " ←" } else { "" };
        println!("    {}. {}{}", i + 1, label, marker);
    }
    println!();

    let choice = prompt_number("  Choice", 1, providers.len());
    let (slug, _) = providers[choice - 1];
    config.model.provider = slug.to_string();

    // Check API key
    let env_key = provider_env_key(slug);
    if !env_key.is_empty() {
        if get_env_value(env_key).is_none() {
            print!("  {} not set. Enter API key: ", env_key);
            io::stdout().flush().ok();
            let mut key = String::new();
            io::stdin().read_line(&mut key).ok();
            let key = key.trim();
            if !key.is_empty() {
                match save_env_value(env_key, key) {
                    Ok(()) => println!("  ✓ Saved {}", env_key),
                    Err(e) => eprintln!("  ✗ {}", e),
                }
            }
        } else {
            println!("  ✓ {} already configured", env_key);
        }
    } else if slug == "local" {
        print!("  Base URL (e.g. http://localhost:11434/v1): ");
        io::stdout().flush().ok();
        let mut url = String::new();
        io::stdin().read_line(&mut url).ok();
        let url = url.trim();
        if !url.is_empty() {
            let _ = save_env_value("OPENAI_BASE_URL", url);
            println!("  ✓ Base URL saved");
        }
    }

    // Model selection
    let default_model = default_model_for_provider(slug);
    print!("  Model [{}]: ", default_model);
    io::stdout().flush().ok();
    let mut model_input = String::new();
    io::stdin().read_line(&mut model_input).ok();
    let model_input = model_input.trim();
    config.model.default = if model_input.is_empty() {
        default_model.to_string()
    } else {
        model_input.to_string()
    };

    match save_config(&config) {
        Ok(()) => {
            println!();
            println!("  ✓ Default model set: {} (provider: {})", config.model.default, config.model.provider);
        }
        Err(e) => eprintln!("  ✗ Failed to save: {}", e),
    }
}

fn provider_env_key(slug: &str) -> &'static str {
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

fn default_model_for_provider(slug: &str) -> &'static str {
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

fn prompt_number(label: &str, min: usize, max: usize) -> usize {
    loop {
        print!("{} [{}-{}]: ", label, min, max);
        io::stdout().flush().ok();
        let mut input = String::new();
        io::stdin().read_line(&mut input).ok();
        match input.trim().parse::<usize>() {
            Ok(n) if n >= min && n <= max => return n,
            _ => println!("  Please enter a number between {} and {}", min, max),
        }
    }
}
