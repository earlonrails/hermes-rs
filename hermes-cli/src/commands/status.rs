use hermes_core::config::{load_config, get_env_value};
use std::process::Command;

pub fn run_status() {
    println!("\nHermes Component Status");
    println!("═════════════════════════\n");

    let config = load_config();

    // 1. LLM Provider Connection Status
    println!("  LLM Providers:");
    let providers = [
        ("OpenAI API", "OPENAI_API_KEY"),
        ("Anthropic API", "ANTHROPIC_API_KEY"),
        ("OpenRouter API", "OPENROUTER_API_KEY"),
        ("Google API", "GOOGLE_API_KEY"),
    ];

    for (name, env_var) in &providers {
        let is_configured = get_env_value(env_var).is_some();
        let status = if is_configured { "✓ Configured" } else { "✗ Not configured" };
        println!("    • {:<18} : {}", name, status);
    }
    println!();

    // 2. Gateway/Messaging Platforms Status
    println!("  Gateways:");
    println!("    • Telegram Gateway   : {}", if config.gateway.telegram_enabled { "✓ Enabled" } else { "✗ Disabled" });
    println!("    • Discord Gateway    : {}", if config.gateway.discord_enabled { "✓ Enabled" } else { "✗ Disabled" });
    println!("    • WhatsApp Gateway   : {}", if config.gateway.whatsapp_enabled { "✓ Enabled" } else { "✗ Disabled" });
    println!();

    // 3. Execution Environment dependencies
    println!("  Dependencies:");
    
    // Check docker
    let docker_check = Command::new("docker").arg("--version").output();
    let docker_status = match docker_check {
        Ok(out) => {
            let version = String::from_utf8_lossy(&out.stdout).trim().to_string();
            format!("✓ Installed ({})", version)
        }
        Err(_) => "✗ Not installed".to_string(),
    };
    println!("    • Docker Engine      : {}", docker_status);

    // Check git
    let git_check = Command::new("git").arg("--version").output();
    let git_status = match git_check {
        Ok(out) => {
            let version = String::from_utf8_lossy(&out.stdout).trim().to_string();
            format!("✓ Installed ({})", version)
        }
        Err(_) => "✗ Not installed".to_string(),
    };
    println!("    • Git CLI            : {}", git_status);

    // Check node
    let node_check = Command::new("node").arg("--version").output();
    let node_status = match node_check {
        Ok(out) => {
            let version = String::from_utf8_lossy(&out.stdout).trim().to_string();
            format!("✓ Installed ({})", version)
        }
        Err(_) => "✗ Not installed".to_string(),
    };
    println!("    • Node.js            : {}", node_status);

    println!("\nAll status checks completed.");
}
