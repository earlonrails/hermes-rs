use hermes_core::config::{load_config, save_config};
use std::io::{self, Write};

pub fn run_whatsapp() {
    println!("\nHermes WhatsApp Integration");
    println!("═════════════════════════════\n");
    println!("WhatsApp integration utilizes the whatsapp-web.js bridge to interact with clients.");
    println!();

    let mut config = load_config();
    println!("Current Status: {}", if config.gateway.whatsapp_enabled { "ENABLED" } else { "DISABLED" });
    println!();

    println!("Options:");
    println!("  1. Enable WhatsApp gateway");
    println!("  2. Disable WhatsApp gateway");
    println!("  3. Generate QR code pairing instructions");
    println!("  4. Exit");
    println!();

    print!("  Choice [1-4]: ");
    io::stdout().flush().ok();

    let mut choice = String::new();
    io::stdin().read_line(&mut choice).ok();
    let choice = choice.trim().parse::<usize>().unwrap_or(4);

    match choice {
        1 => {
            config.gateway.whatsapp_enabled = true;
            if save_config(&config).is_ok() {
                println!("  ✓ WhatsApp gateway has been enabled.");
            }
        }
        2 => {
            config.gateway.whatsapp_enabled = false;
            if save_config(&config).is_ok() {
                println!("  ✓ WhatsApp gateway has been disabled.");
            }
        }
        3 => {
            println!("\n--- Pairing Instructions ---");
            println!("1. Ensure Node.js and whatsapp-web.js are installed on your gateway system.");
            println!("2. Run the gateway service using: 'hermes gateway --platform whatsapp'.");
            println!("3. Scan the terminal-rendered QR code with your WhatsApp mobile application.");
        }
        _ => {}
    }
}
