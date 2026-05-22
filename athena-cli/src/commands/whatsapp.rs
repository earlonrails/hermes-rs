use athena_core::config::{load_config, save_config};
use cliclack::{intro, select, outro, outro_cancel, note};
use anyhow::Result;

pub fn run_whatsapp() -> Result<()> {
    intro("Athena WhatsApp Integration")?;
    note("Info", "WhatsApp integration utilizes the whatsapp-web.js bridge to interact with clients.")?;

    let mut config = load_config();
    let status = if config.gateway.whatsapp_enabled { "ENABLED" } else { "DISABLED" };

    println!("Options:");
    let choice: usize = select(format!("Current Status: {}", status))
        .item(1, "Enable WhatsApp gateway", "")
        .item(2, "Disable WhatsApp gateway", "")
        .item(3, "Generate QR code pairing instructions", "")
        .item(4, "Exit", "")
        .interact()?;

    match choice {
        1 => {
            config.gateway.whatsapp_enabled = true;
            if save_config(&config).is_ok() {
                outro("WhatsApp gateway has been enabled.")?;
            } else {
                outro_cancel("Failed to save config.")?;
            }
        }
        2 => {
            config.gateway.whatsapp_enabled = false;
            if save_config(&config).is_ok() {
                outro("WhatsApp gateway has been disabled.")?;
            } else {
                outro_cancel("Failed to save config.")?;
            }
        }
        3 => {
            let instructions = "1. Ensure Node.js and whatsapp-web.js are installed on your gateway system.\n2. Run the gateway service using: 'athena gateway --platform whatsapp'.\n3. Scan the terminal-rendered QR code with your WhatsApp mobile application.";
            note("Pairing Instructions", instructions)?;
        }
        _ => { outro("Goodbye!")?; }
    }
    
    Ok(())
}
