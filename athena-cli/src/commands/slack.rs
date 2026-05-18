use athena_core::config::{load_config, save_config};
use athena_core::paths::get_hermes_home;
use std::io::{self, Write};
use std::fs;

pub fn run_slack() {
    println!("\nHermes Slack Integration");
    println!("══════════════════════════\n");
    println!("Provides helpers for generating Slack Manifest JSON files and enabling the bot gateway.");
    println!();

    let mut config = load_config();
    println!("Current Status: {}", if config.gateway.slack_enabled { "ENABLED" } else { "DISABLED" });
    println!();

    println!("Options:");
    println!("  1. Enable Slack gateway");
    println!("  2. Disable Slack gateway");
    println!("  3. Generate Slack Manifest JSON");
    println!("  4. Exit");
    println!();

    print!("  Choice [1-4]: ");
    io::stdout().flush().ok();

    let mut choice = String::new();
    io::stdin().read_line(&mut choice).ok();
    let choice = choice.trim().parse::<usize>().unwrap_or(4);

    match choice {
        1 => {
            config.gateway.slack_enabled = true;
            if save_config(&config).is_ok() {
                println!("  ✓ Slack gateway has been enabled.");
            }
        }
        2 => {
            config.gateway.slack_enabled = false;
            if save_config(&config).is_ok() {
                println!("  ✓ Slack gateway has been disabled.");
            }
        }
        3 => {
            let manifest = r#"{
    "display_information": {
        "name": "Hermes Agent"
    },
    "features": {
        "bot_user": {
            "display_name": "Hermes",
            "always_online": true
        }
    },
    "oauth_config": {
        "scopes": {
            "bot": [
                "chat:write",
                "im:history",
                "im:write",
                "app_mentions:read"
            ]
        }
    },
    "settings": {
        "event_subscriptions": {
            "bot_events": [
                "message.im",
                "app_mention"
            ]
        }
    }
}"#;
            let path = get_hermes_home().join("slack-manifest.json");
            match fs::write(&path, manifest) {
                Ok(()) => {
                    println!("  ✓ Generated Slack Manifest template successfully!");
                    println!("  Saved to: {}", path.display());
                }
                Err(e) => {
                    println!("  ✗ Failed to save manifest: {}", e);
                }
            }
        }
        _ => {}
    }
}
