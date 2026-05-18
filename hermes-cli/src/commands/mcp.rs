use std::fs;
use std::io::{self, Write};
use serde::{Deserialize, Serialize};
use hermes_core::paths::get_hermes_home;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct McpServersList {
    servers: Vec<McpServerInfo>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct McpServerInfo {
    name: String,
    command: String,
    args: Vec<String>,
    enabled: bool,
}

pub fn run_mcp() {
    println!("\nHermes Model Context Protocol (MCP)");
    println!("═════════════════════════════════════\n");
    println!("Manage external MCP server processes and custom clients.");
    println!();

    let mcp_file = get_hermes_home().join("mcp_servers.json");
    let mut data = if mcp_file.exists() {
        let content = fs::read_to_string(&mcp_file).unwrap_or_default();
        serde_json::from_str::<McpServersList>(&content).unwrap_or_default()
    } else {
        McpServersList::default()
    };

    println!("Options:");
    println!("  1. List configured MCP servers");
    println!("  2. Add external MCP server");
    println!("  3. Toggle MCP server status");
    println!("  4. Remove MCP server");
    println!("  5. Exit");
    println!();

    print!("  Choice [1-5]: ");
    io::stdout().flush().ok();

    let mut choice = String::new();
    io::stdin().read_line(&mut choice).ok();
    let choice = choice.trim().parse::<usize>().unwrap_or(5);

    match choice {
        1 => {
            println!("\nConfigured MCP Servers:");
            if data.servers.is_empty() {
                println!("  No external MCP servers configured.");
            } else {
                for (i, server) in data.servers.iter().enumerate() {
                    let status = if server.enabled { "ENABLED" } else { "DISABLED" };
                    println!(
                        "  {}. {} [{}] - Command: '{}', Args: {:?}",
                        i + 1,
                        server.name,
                        status,
                        server.command,
                        server.args
                    );
                }
            }
        }
        2 => {
            println!("\nAdd External MCP Server");
            print!("  Enter server name: ");
            io::stdout().flush().ok();
            let mut name = String::new();
            io::stdin().read_line(&mut name).ok();
            let name = name.trim().to_string();

            print!("  Enter command executable (e.g. npx, python): ");
            io::stdout().flush().ok();
            let mut command = String::new();
            io::stdin().read_line(&mut command).ok();
            let command = command.trim().to_string();

            if name.is_empty() || command.is_empty() {
                println!("  ✗ Server name and command cannot be empty.");
                return;
            }

            print!("  Enter arguments (comma separated, e.g. -y, @modelcontextprotocol/server-memory): ");
            io::stdout().flush().ok();
            let mut args_in = String::new();
            io::stdin().read_line(&mut args_in).ok();
            let args = args_in
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            data.servers.push(McpServerInfo {
                name,
                command,
                args,
                enabled: true,
            });

            if let Ok(serialized) = serde_json::to_string_pretty(&data) {
                let _ = fs::write(&mcp_file, serialized);
                println!("  ✓ MCP server successfully added.");
            }
        }
        3 => {
            if data.servers.is_empty() {
                println!("\n  No MCP servers configured to toggle.");
                return;
            }

            println!("\nSelect a server to toggle:");
            for (i, s) in data.servers.iter().enumerate() {
                println!("  {}. {} [{}]", i + 1, s.name, if s.enabled { "ENABLED" } else { "DISABLED" });
            }
            println!();

            print!("  Choice [1-{}]: ", data.servers.len());
            io::stdout().flush().ok();
            let mut s_choice = String::new();
            io::stdin().read_line(&mut s_choice).ok();
            let s_choice = s_choice.trim().parse::<usize>().unwrap_or(0);

            if s_choice < 1 || s_choice > data.servers.len() {
                println!("  ✗ Invalid choice.");
                return;
            }

            data.servers[s_choice - 1].enabled = !data.servers[s_choice - 1].enabled;
            if let Ok(serialized) = serde_json::to_string_pretty(&data) {
                let _ = fs::write(&mcp_file, serialized);
                println!("  ✓ Toggled MCP server status.");
            }
        }
        4 => {
            if data.servers.is_empty() {
                println!("\n  No MCP servers to remove.");
                return;
            }

            println!("\nSelect a server to remove:");
            for (i, s) in data.servers.iter().enumerate() {
                println!("  {}. {}", i + 1, s.name);
            }
            println!();

            print!("  Choice [1-{}]: ", data.servers.len());
            io::stdout().flush().ok();
            let mut s_choice = String::new();
            io::stdin().read_line(&mut s_choice).ok();
            let s_choice = s_choice.trim().parse::<usize>().unwrap_or(0);

            if s_choice < 1 || s_choice > data.servers.len() {
                println!("  ✗ Invalid choice.");
                return;
            }

            let removed = data.servers.remove(s_choice - 1);
            if let Ok(serialized) = serde_json::to_string_pretty(&data) {
                let _ = fs::write(&mcp_file, serialized);
                println!("  ✓ Removed MCP server: {}.", removed.name);
            }
        }
        _ => {}
    }
}
