use athena_core::config::{load_config, save_config};
use std::io::{self, Write};

pub fn run_config_show() {
    let config = load_config();
    println!("\nCurrent Hermes Configuration:");
    println!("═════════════════════════════════");
    println!("  Profile:          {:?}", config.active_profile.as_deref().unwrap_or("default"));
    println!("  Active Provider:  {}", config.model.provider);
    println!("  Default Model:    {}", config.model.default);
    println!("  Terminal Backend: {}", config.terminal_backend);
    println!("  Max Iterations:   {}", config.agent.max_iterations);
    println!("  Telegram Enabled: {}", config.gateway.telegram_enabled);
    println!("  Discord Enabled:  {}", config.gateway.discord_enabled);
    println!("  Slack Enabled:    {}", config.gateway.slack_enabled);
    println!("  WhatsApp Enabled: {}", config.gateway.whatsapp_enabled);
    println!("  Config Version:   {}", config.config_version);
    println!("═════════════════════════════════\n");
}

pub fn run_config_edit() {
    println!("\nEdit Configuration Fields");
    println!("═══════════════════════════\n");

    let mut config = load_config();

    print!("  Change Terminal Backend (current: {}) [local/docker/ssh/modal]: ", config.terminal_backend);
    io::stdout().flush().ok();
    let mut backend = String::new();
    io::stdin().read_line(&mut backend).ok();
    let backend = backend.trim();
    if !backend.is_empty() && (backend == "local" || backend == "docker" || backend == "ssh" || backend == "modal") {
        config.terminal_backend = backend.to_string();
        println!("  ✓ Terminal backend updated.");
    }

    print!("  Change Max Iterations (current: {}): ", config.agent.max_iterations);
    io::stdout().flush().ok();
    let mut iterations = String::new();
    io::stdin().read_line(&mut iterations).ok();
    let iterations = iterations.trim();
    if !iterations.is_empty() {
        if let Ok(num) = iterations.parse::<u32>() {
            config.agent.max_iterations = num;
            println!("  ✓ Max iterations updated.");
        }
    }

    match save_config(&config) {
        Ok(()) => println!("\n✓ Config successfully updated at ~/.hermes/config.yaml"),
        Err(e) => println!("\n✗ Failed to save config: {}", e),
    }
}
