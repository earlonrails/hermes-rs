use std::fs;
use std::io::{self, Write};
use serde::{Deserialize, Serialize};
use athena_core::paths::get_hermes_home;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct WebhookData {
    subscriptions: Vec<Subscription>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Subscription {
    name: String,
    url: String,
    secret: String,
    active: bool,
}

pub fn run_webhook() {
    println!("\nHermes Webhook Subscriptions");
    println!("══════════════════════════════\n");
    println!("Manage incoming and outgoing HTTP webhook hooks for third-party service dispatch.");
    println!();

    let webhook_file = get_hermes_home().join("webhooks.json");
    let mut data = if webhook_file.exists() {
        let content = fs::read_to_string(&webhook_file).unwrap_or_default();
        serde_json::from_str::<WebhookData>(&content).unwrap_or_default()
    } else {
        WebhookData::default()
    };

    println!("Options:");
    println!("  1. List registered webhooks");
    println!("  2. Register a new webhook");
    println!("  3. Toggle webhook status");
    println!("  4. Delete a webhook");
    println!("  5. Exit");
    println!();

    print!("  Choice [1-5]: ");
    io::stdout().flush().ok();

    let mut choice = String::new();
    io::stdin().read_line(&mut choice).ok();
    let choice = choice.trim().parse::<usize>().unwrap_or(5);

    match choice {
        1 => {
            println!("\nRegistered Webhooks:");
            if data.subscriptions.is_empty() {
                println!("  No webhooks registered.");
            } else {
                for (i, sub) in data.subscriptions.iter().enumerate() {
                    let status = if sub.active { "ACTIVE" } else { "INACTIVE" };
                    println!(
                        "  {}. {} -> {} [{}] (Secret: {})",
                        i + 1,
                        sub.name,
                        sub.url,
                        status,
                        sub.secret
                    );
                }
            }
        }
        2 => {
            println!("\nRegister New Webhook");
            print!("  Enter custom name: ");
            io::stdout().flush().ok();
            let mut name = String::new();
            io::stdin().read_line(&mut name).ok();
            let name = name.trim().to_string();

            print!("  Enter callback URL: ");
            io::stdout().flush().ok();
            let mut url = String::new();
            io::stdin().read_line(&mut url).ok();
            let url = url.trim().to_string();

            if name.is_empty() || url.is_empty() {
                println!("  ✗ Name and URL cannot be empty.");
                return;
            }

            let chars: Vec<char> = "abcdefghijklmnopqrstuvwxyz0123456789".chars().collect();
            let mut secret = String::new();
            for _ in 0..16 {
                let idx = rand::random::<usize>() % chars.len();
                secret.push(chars[idx]);
            }

            data.subscriptions.push(Subscription {
                name,
                url,
                secret,
                active: true,
            });

            if let Ok(serialized) = serde_json::to_string_pretty(&data) {
                let _ = fs::write(&webhook_file, serialized);
                println!("  ✓ Webhook successfully registered.");
            }
        }
        3 => {
            if data.subscriptions.is_empty() {
                println!("\n  No webhooks to toggle.");
                return;
            }

            println!("\nSelect a webhook to toggle active status:");
            for (i, sub) in data.subscriptions.iter().enumerate() {
                println!("  {}. {} [{}]", i + 1, sub.name, if sub.active { "ACTIVE" } else { "INACTIVE" });
            }
            println!();

            print!("  Choice [1-{}]: ", data.subscriptions.len());
            io::stdout().flush().ok();
            let mut sub_choice = String::new();
            io::stdin().read_line(&mut sub_choice).ok();
            let sub_choice = sub_choice.trim().parse::<usize>().unwrap_or(0);

            if sub_choice < 1 || sub_choice > data.subscriptions.len() {
                println!("  ✗ Invalid choice.");
                return;
            }

            data.subscriptions[sub_choice - 1].active = !data.subscriptions[sub_choice - 1].active;
            if let Ok(serialized) = serde_json::to_string_pretty(&data) {
                let _ = fs::write(&webhook_file, serialized);
                println!("  ✓ Successfully toggled status.");
            }
        }
        4 => {
            if data.subscriptions.is_empty() {
                println!("\n  No webhooks to delete.");
                return;
            }

            println!("\nSelect a webhook to delete:");
            for (i, sub) in data.subscriptions.iter().enumerate() {
                println!("  {}. {}", i + 1, sub.name);
            }
            println!();

            print!("  Choice [1-{}]: ", data.subscriptions.len());
            io::stdout().flush().ok();
            let mut sub_choice = String::new();
            io::stdin().read_line(&mut sub_choice).ok();
            let sub_choice = sub_choice.trim().parse::<usize>().unwrap_or(0);

            if sub_choice < 1 || sub_choice > data.subscriptions.len() {
                println!("  ✗ Invalid choice.");
                return;
            }

            let removed = data.subscriptions.remove(sub_choice - 1);
            if let Ok(serialized) = serde_json::to_string_pretty(&data) {
                let _ = fs::write(&webhook_file, serialized);
                println!("  ✓ Webhook {} deleted successfully.", removed.name);
            }
        }
        _ => {}
    }
}
