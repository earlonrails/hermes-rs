use hermes_core::config::{get_env_value, save_env_value, remove_env_value};
use std::io::{self, Write};

pub fn run_login() {
    println!("\nAuthenticate with an Inference Provider");
    println!("═════════════════════════════════════════\n");

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

    for (i, (name, _)) in providers.iter().enumerate() {
        println!("  {}. {}", i + 1, name);
    }
    println!();

    print!("  Select provider [1-{}]: ", providers.len());
    io::stdout().flush().ok();
    let mut choice = String::new();
    io::stdin().read_line(&mut choice).ok();
    let choice = choice.trim().parse::<usize>().unwrap_or(0);

    if choice < 1 || choice > providers.len() {
        println!("✗ Invalid choice.");
        return;
    }

    let (name, env_var) = providers[choice - 1];
    let current = get_env_value(env_var);
    if let Some(ref val) = current {
        let masked = if val.len() > 8 {
            format!("{}...{}", &val[..4], &val[val.len()-4..])
        } else {
            "****".to_string()
        };
        println!("  Current API key: {}", masked);
    }

    print!("  Enter new API key for {} (or press Enter to skip): ", name);
    io::stdout().flush().ok();
    let mut key = String::new();
    io::stdin().read_line(&mut key).ok();
    let key = key.trim();

    if !key.is_empty() {
        match save_env_value(env_var, key) {
            Ok(()) => println!("✓ Saved API key for {} to ~/.hermes/.env", name),
            Err(e) => println!("✗ Failed to save API key: {}", e),
        }
    } else {
        println!("  No changes made.");
    }
}

pub fn run_logout() {
    println!("\nClear Provider Authentication");
    println!("═══════════════════════════════\n");

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

    for (i, (name, _)) in providers.iter().enumerate() {
        println!("  {}. {}", i + 1, name);
    }
    println!();

    print!("  Select provider to log out of [1-{}]: ", providers.len());
    io::stdout().flush().ok();
    let mut choice = String::new();
    io::stdin().read_line(&mut choice).ok();
    let choice = choice.trim().parse::<usize>().unwrap_or(0);

    if choice < 1 || choice > providers.len() {
        println!("✗ Invalid choice.");
        return;
    }

    let (name, env_var) = providers[choice - 1];
    if get_env_value(env_var).is_none() {
        println!("✓ Provider {} is already logged out.", name);
        return;
    }

    print!("  Are you sure you want to log out of {}? [y/N]: ", name);
    io::stdout().flush().ok();
    let mut confirm = String::new();
    io::stdin().read_line(&mut confirm).ok();
    if confirm.trim().to_lowercase() == "y" {
        match remove_env_value(env_var) {
            Ok(()) => println!("✓ Logged out of {} successfully.", name),
            Err(e) => println!("✗ Failed to remove API key: {}", e),
        }
    } else {
        println!("  Cancelled.");
    }
}
