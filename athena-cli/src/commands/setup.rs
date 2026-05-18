use athena_core::config::{load_config, save_config, has_any_provider_configured, get_env_value};
use std::io::{self, Write};

/// Interactive setup wizard — walks the user through initial configuration.
pub fn run_setup() {
    println!();
    println!("⚕ Hermes Setup Wizard");
    println!("═══════════════════════");
    println!();

    let hermes_home = athena_core::paths::display_hermes_home();
    println!("  Hermes home: {}", hermes_home);
    println!();

    let mut config = load_config();

    // Section 1: Model & Provider
    setup_model_provider(&mut config);

    // Section 2: Terminal backend
    setup_terminal_backend(&mut config);

    // Section 3: Agent settings
    setup_agent_settings(&mut config);

    // Section 4: Gateway (messaging platforms)
    setup_gateway(&mut config);

    // Save
    match save_config(&config) {
        Ok(()) => {
            println!();
            println!("✓ Configuration saved to {}/config.yaml", hermes_home);
        }
        Err(e) => {
            eprintln!("✗ Failed to save config: {}", e);
        }
    }

    print_setup_summary(&config);
}

fn setup_model_provider(config: &mut athena_core::config::HermesConfig) {
    println!("─── Model & Provider ───────────────────────────────");
    println!();

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

    println!("  Select your inference provider:");
    for (i, (_, label)) in providers.iter().enumerate() {
        let marker = if !config.model.provider.is_empty()
            && config.model.provider == providers[i].0
        {
            " ←"
        } else {
            ""
        };
        println!("    {}. {}{}", i + 1, label, marker);
    }
    println!();

    let choice = prompt_number("  Choice", 1, providers.len());
    let (slug, _label) = providers[choice - 1];
    config.model.provider = slug.to_string();

    // Prompt for API key
    let env_key = match slug {
        "openai" => "OPENAI_API_KEY",
        "anthropic" => "ANTHROPIC_API_KEY",
        "openrouter" => "OPENROUTER_API_KEY",
        "google" => "GOOGLE_API_KEY",
        "deepseek" => "DEEPSEEK_API_KEY",
        "groq" => "GROQ_API_KEY",
        "mistral" => "MISTRAL_API_KEY",
        "xai" => "XAI_API_KEY",
        "local" => "",
        _ => "",
    };

    if !env_key.is_empty() {
        let current = get_env_value(env_key);
        if let Some(ref val) = current {
            let masked = mask_key(val);
            println!("  Current API key: {}", masked);
            print!("  Update? [y/N] ");
            io::stdout().flush().ok();
            let mut input = String::new();
            io::stdin().read_line(&mut input).ok();
            if input.trim().to_lowercase() != "y" {
                println!("  ✓ Keeping existing key");
            } else {
                prompt_and_save_key(env_key);
            }
        } else {
            prompt_and_save_key(env_key);
        }
    } else if slug == "local" {
        print!("  Base URL (e.g. http://localhost:11434/v1): ");
        io::stdout().flush().ok();
        let mut url = String::new();
        io::stdin().read_line(&mut url).ok();
        let url = url.trim();
        if !url.is_empty() {
            let _ = athena_core::config::save_env_value("OPENAI_BASE_URL", url);
            println!("  ✓ Base URL saved");
        }
    }

    // Prompt for default model
    let default_model = match slug {
        "openai" => "gpt-4o",
        "anthropic" => "claude-sonnet-4-20250514",
        "openrouter" => "openai/gpt-4o",
        "google" => "gemini-2.5-pro",
        "deepseek" => "deepseek-chat",
        "groq" => "llama-3.3-70b-versatile",
        "mistral" => "mistral-large-latest",
        "xai" => "grok-3",
        _ => "gpt-4o",
    };

    print!("  Default model [{}]: ", default_model);
    io::stdout().flush().ok();
    let mut model_input = String::new();
    io::stdin().read_line(&mut model_input).ok();
    let model_input = model_input.trim();
    config.model.default = if model_input.is_empty() {
        default_model.to_string()
    } else {
        model_input.to_string()
    };

    println!("  ✓ Provider: {}, Model: {}", slug, config.model.default);
    println!();
}

fn setup_terminal_backend(config: &mut athena_core::config::HermesConfig) {
    println!("─── Terminal Backend ───────────────────────────────");
    println!();

    let backends = ["local", "docker", "ssh", "modal"];
    println!("  Where should Hermes run terminal commands?");
    for (i, b) in backends.iter().enumerate() {
        let marker = if config.terminal_backend == *b { " ←" } else { "" };
        println!("    {}. {}{}", i + 1, b, marker);
    }
    println!();

    let choice = prompt_number("  Choice", 1, backends.len());
    config.terminal_backend = backends[choice - 1].to_string();
    println!("  ✓ Terminal backend: {}", config.terminal_backend);
    println!();
}

fn setup_agent_settings(config: &mut athena_core::config::HermesConfig) {
    println!("─── Agent Settings ────────────────────────────────");
    println!();

    print!("  Max iterations per turn [{}]: ", config.agent.max_iterations);
    io::stdout().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();
    let input = input.trim();
    if !input.is_empty() {
        if let Ok(n) = input.parse::<u32>() {
            config.agent.max_iterations = n;
        }
    }

    println!("  ✓ Max iterations: {}", config.agent.max_iterations);
    println!();
}

fn setup_gateway(config: &mut athena_core::config::HermesConfig) {
    println!("─── Messaging Platforms ────────────────────────────");
    println!();

    // Telegram
    print!("  Enable Telegram? [y/N]: ");
    io::stdout().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();
    if input.trim().to_lowercase() == "y" {
        config.gateway.telegram_enabled = true;
        if get_env_value("TELEGRAM_BOT_TOKEN").is_none() {
            prompt_and_save_key("TELEGRAM_BOT_TOKEN");
        } else {
            println!("  ✓ Telegram token already configured");
        }
    }

    // Discord
    print!("  Enable Discord? [y/N]: ");
    io::stdout().flush().ok();
    input.clear();
    io::stdin().read_line(&mut input).ok();
    if input.trim().to_lowercase() == "y" {
        config.gateway.discord_enabled = true;
        if get_env_value("DISCORD_BOT_TOKEN").is_none() {
            prompt_and_save_key("DISCORD_BOT_TOKEN");
        } else {
            println!("  ✓ Discord token already configured");
        }
    }

    println!();
}

fn print_setup_summary(config: &athena_core::config::HermesConfig) {
    println!();
    println!("═══════════════════════════════════════════════════");
    println!("  Setup Summary");
    println!("═══════════════════════════════════════════════════");
    println!("  Provider:         {}", config.model.provider);
    println!("  Model:            {}", config.model.default);
    println!("  Terminal:         {}", config.terminal_backend);
    println!("  Max iterations:   {}", config.agent.max_iterations);
    println!("  Telegram:         {}", if config.gateway.telegram_enabled { "enabled" } else { "disabled" });
    println!("  Discord:          {}", if config.gateway.discord_enabled { "enabled" } else { "disabled" });
    println!("═══════════════════════════════════════════════════");
    println!();
    println!("  You're all set! Run 'hermes' or 'hermes chat' to start.");
    println!();
}

// ─── Helpers ────────────────────────────────────────────────────────────────

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

fn prompt_and_save_key(env_key: &str) {
    print!("  {}: ", env_key);
    io::stdout().flush().ok();
    let mut key = String::new();
    io::stdin().read_line(&mut key).ok();
    let key = key.trim();
    if !key.is_empty() {
        match athena_core::config::save_env_value(env_key, key) {
            Ok(()) => println!("  ✓ Saved {}", env_key),
            Err(e) => eprintln!("  ✗ Failed to save: {}", e),
        }
    } else {
        println!("  Skipped");
    }
}

fn mask_key(key: &str) -> String {
    if key.len() <= 8 {
        "****".to_string()
    } else {
        format!("{}…{}", &key[..4], &key[key.len()-4..])
    }
}
