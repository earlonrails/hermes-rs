use athena_core::config::{get_env_value, save_env_value, remove_env_value};
use std::io::{self, Write};

pub fn run_auth() {
    println!("\nHermes Authentication Pool Manager");
    println!("═════════════════════════════════════\n");
    println!("Manage pooled API credentials across your workspace profiles.");
    println!();

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

    println!("Current credentials pool status:");
    for (i, (name, env_var)) in providers.iter().enumerate() {
        let status = if let Some(val) = get_env_value(env_var) {
            if val.len() > 8 {
                format!("✓ Configured ({}...{})", &val[..4], &val[val.len()-4..])
            } else {
                "✓ Configured (****)".to_string()
            }
        } else {
            "✗ Not configured".to_string()
        };
        println!("  {}. {:<12} : {}", i + 1, name, status);
    }
    println!();

    println!("Options:");
    println!("  1. Add/update a credential");
    println!("  2. Remove a credential");
    println!("  3. Exit");
    println!();

    print!("  Choice [1-3]: ");
    io::stdout().flush().ok();

    let mut choice = String::new();
    io::stdin().read_line(&mut choice).ok();
    let choice = choice.trim().parse::<usize>().unwrap_or(3);

    match choice {
        1 => {
            print!("\n  Select provider number to update [1-{}]: ", providers.len());
            io::stdout().flush().ok();
            let mut prov_choice = String::new();
            io::stdin().read_line(&mut prov_choice).ok();
            let prov_choice = prov_choice.trim().parse::<usize>().unwrap_or(0);

            if prov_choice < 1 || prov_choice > providers.len() {
                println!("  ✗ Invalid choice.");
                return;
            }

            let (name, env_var) = providers[prov_choice - 1];
            print!("  Enter API key for {}: ", name);
            io::stdout().flush().ok();
            let mut key = String::new();
            io::stdin().read_line(&mut key).ok();
            let key = key.trim();

            if !key.is_empty() {
                match save_env_value(env_var, key) {
                    Ok(()) => println!("  ✓ Credential saved successfully!"),
                    Err(e) => println!("  ✗ Failed to save API key: {}", e),
                }
            }
        }
        2 => {
            print!("\n  Select provider number to remove [1-{}]: ", providers.len());
            io::stdout().flush().ok();
            let mut prov_choice = String::new();
            io::stdin().read_line(&mut prov_choice).ok();
            let prov_choice = prov_choice.trim().parse::<usize>().unwrap_or(0);

            if prov_choice < 1 || prov_choice > providers.len() {
                println!("  ✗ Invalid choice.");
                return;
            }

            let (name, env_var) = providers[prov_choice - 1];
            print!("  Are you sure you want to remove credential for {}? [y/N]: ", name);
            io::stdout().flush().ok();
            let mut confirm = String::new();
            io::stdin().read_line(&mut confirm).ok();
            if confirm.trim().to_lowercase() == "y" {
                match remove_env_value(env_var) {
                    Ok(()) => println!("  ✓ Credential removed successfully!"),
                    Err(e) => println!("  ✗ Failed to remove API key: {}", e),
                }
            }
        }
        _ => {}
    }
}
