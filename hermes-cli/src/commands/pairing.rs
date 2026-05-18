use std::fs;
use std::io::{self, Write};
use serde::{Deserialize, Serialize};
use hermes_core::paths::get_hermes_home;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct AuthorizedClients {
    clients: Vec<ClientInfo>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ClientInfo {
    id: String,
    platform: String,
    username: Option<String>,
    paired_at: String,
}

pub fn run_pairing() {
    println!("\nHermes Gateway Pairing Manager");
    println!("════════════════════════════════\n");
    println!("Manage authorization for remote messaging gateways (Telegram, Slack, WhatsApp).");
    println!();

    let clients_file = get_hermes_home().join("authorized_clients.json");
    let mut auth_data = if clients_file.exists() {
        let content = fs::read_to_string(&clients_file).unwrap_or_default();
        serde_json::from_str::<AuthorizedClients>(&content).unwrap_or_default()
    } else {
        AuthorizedClients::default()
    };

    println!("Options:");
    println!("  1. Generate a temporary pairing code");
    println!("  2. List currently authorized clients");
    println!("  3. Add a manual client authorization");
    println!("  4. Revoke a client authorization");
    println!("  5. Exit");
    println!();

    print!("  Choice [1-5]: ");
    io::stdout().flush().ok();

    let mut choice = String::new();
    io::stdin().read_line(&mut choice).ok();
    let choice = choice.trim().parse::<usize>().unwrap_or(5);

    match choice {
        1 => {
            let chars: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".chars().collect();
            let mut code = String::new();
            for i in 0..8 {
                if i == 4 {
                    code.push('-');
                }
                let idx = rand::random::<usize>() % chars.len();
                code.push(chars[idx]);
            }
            println!("\n  🔑 Temporary Pairing Code: HERMES-{}", code);
            println!("  Send this code in a DM to your configured Telegram/Slack bot to authorize your account.");
            println!("  (Code is active for this session)");
        }
        2 => {
            println!("\nAuthorized Clients:");
            if auth_data.clients.is_empty() {
                println!("  No authorized clients found.");
            } else {
                for (i, client) in auth_data.clients.iter().enumerate() {
                    let user_str = client.username.as_deref().unwrap_or("unknown");
                    println!(
                        "  {}. ID: {} [{}] (User: @{}, Paired: {})",
                        i + 1,
                        client.id,
                        client.platform,
                        user_str,
                        client.paired_at
                    );
                }
            }
        }
        3 => {
            println!("\nAdd Manual Authorization");
            print!("  Enter Platform (e.g. telegram, slack, whatsapp): ");
            io::stdout().flush().ok();
            let mut platform = String::new();
            io::stdin().read_line(&mut platform).ok();
            let platform = platform.trim().to_string();

            print!("  Enter Chat ID or User ID: ");
            io::stdout().flush().ok();
            let mut client_id = String::new();
            io::stdin().read_line(&mut client_id).ok();
            let client_id = client_id.trim().to_string();

            if client_id.is_empty() {
                println!("  ✗ Chat ID cannot be empty.");
                return;
            }

            print!("  Enter Username (optional): ");
            io::stdout().flush().ok();
            let mut username_in = String::new();
            io::stdin().read_line(&mut username_in).ok();
            let username = if username_in.trim().is_empty() {
                None
            } else {
                Some(username_in.trim().to_string())
            };

            let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            auth_data.clients.push(ClientInfo {
                id: client_id.clone(),
                platform,
                username,
                paired_at: now,
            });

            if let Ok(serialized) = serde_json::to_string_pretty(&auth_data) {
                let _ = fs::write(&clients_file, serialized);
                println!("  ✓ Successfully authorized client: {}.", client_id);
            } else {
                println!("  ✗ Failed to serialize pairing information.");
            }
        }
        4 => {
            if auth_data.clients.is_empty() {
                println!("\n  No authorized clients to revoke.");
                return;
            }

            println!("\nSelect a client number to revoke:");
            for (i, client) in auth_data.clients.iter().enumerate() {
                let user_str = client.username.as_deref().unwrap_or("unknown");
                println!("  {}. {} (User: @{})", i + 1, client.id, user_str);
            }
            println!();

            print!("  Choice [1-{}]: ", auth_data.clients.len());
            io::stdout().flush().ok();
            let mut rev_choice = String::new();
            io::stdin().read_line(&mut rev_choice).ok();
            let rev_choice = rev_choice.trim().parse::<usize>().unwrap_or(0);

            if rev_choice < 1 || rev_choice > auth_data.clients.len() {
                println!("  ✗ Invalid choice.");
                return;
            }

            let removed = auth_data.clients.remove(rev_choice - 1);
            if let Ok(serialized) = serde_json::to_string_pretty(&auth_data) {
                let _ = fs::write(&clients_file, serialized);
                println!("  ✓ Revoked authorization for client: {}.", removed.id);
            }
        }
        _ => {}
    }
}
