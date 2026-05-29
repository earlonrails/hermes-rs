use std::fs;
use std::io::{self, Write};
use serde::{Deserialize, Serialize};
use athena_core::paths::get_athena_home;
use athena_tools::ToolRegistry;
use athena_mcp::client::{McpClient, ExternalMcpTool};
use athena_mcp::server::McpServer;
use std::sync::Arc;

use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct McpServersList {
    pub servers: Vec<McpServerInfo>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct McpServerInfo {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub enabled: bool,
}

pub fn get_mcp_servers(file_path: &Path) -> McpServersList {
    if file_path.exists() {
        let content = fs::read_to_string(file_path).unwrap_or_default();
        serde_json::from_str::<McpServersList>(&content).unwrap_or_default()
    } else {
        McpServersList::default()
    }
}

pub fn save_mcp_servers(file_path: &Path, data: &McpServersList) -> Result<(), String> {
    if let Ok(serialized) = serde_json::to_string_pretty(data) {
        fs::write(file_path, serialized).map_err(|e| e.to_string())
    } else {
        Err("Failed to serialize".to_string())
    }
}

pub fn add_mcp_server(file_path: &Path, info: McpServerInfo) -> Result<(), String> {
    if info.name.is_empty() || info.command.is_empty() {
        return Err("Server name and command cannot be empty.".to_string());
    }
    let mut data = get_mcp_servers(file_path);
    data.servers.push(info);
    save_mcp_servers(file_path, &data)
}

pub fn toggle_mcp_server(file_path: &Path, index: usize) -> Result<(), String> {
    let mut data = get_mcp_servers(file_path);
    if index >= data.servers.len() {
        return Err("Invalid choice.".to_string());
    }
    data.servers[index].enabled = !data.servers[index].enabled;
    save_mcp_servers(file_path, &data)
}

pub fn remove_mcp_server(file_path: &Path, index: usize) -> Result<McpServerInfo, String> {
    let mut data = get_mcp_servers(file_path);
    if index >= data.servers.len() {
        return Err("Invalid choice.".to_string());
    }
    let removed = data.servers.remove(index);
    save_mcp_servers(file_path, &data)?;
    Ok(removed)
}

pub fn run_mcp() {
    println!("\nAthena Model Context Protocol (MCP)");
    println!("═════════════════════════════════════\n");
    println!("Manage external MCP server processes and custom clients.");
    println!();

    let mcp_file = get_athena_home().join("mcp_servers.json");
    let data = get_mcp_servers(&mcp_file);

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

            print!("  Enter arguments (comma separated, e.g. -y, @modelcontextprotocol/server-memory): ");
            io::stdout().flush().ok();
            let mut args_in = String::new();
            io::stdin().read_line(&mut args_in).ok();
            let args = args_in
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            let info = McpServerInfo {
                name,
                command,
                args,
                enabled: true,
            };

            match add_mcp_server(&mcp_file, info) {
                Ok(_) => println!("  ✓ MCP server successfully added."),
                Err(e) => println!("  ✗ {}", e),
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

            match toggle_mcp_server(&mcp_file, s_choice - 1) {
                Ok(_) => println!("  ✓ Toggled MCP server status."),
                Err(e) => println!("  ✗ {}", e),
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

            match remove_mcp_server(&mcp_file, s_choice - 1) {
                Ok(removed) => println!("  ✓ Removed MCP server: {}.", removed.name),
                Err(e) => println!("  ✗ {}", e),
            }
        }
        _ => {}
    }
}

pub async fn load_mcp_servers_into_registry(registry: &ToolRegistry) {
    let mcp_file = get_athena_home().join("mcp_servers.json");
    if !mcp_file.exists() {
        return;
    }

    let content = fs::read_to_string(&mcp_file).unwrap_or_default();
    let data = serde_json::from_str::<McpServersList>(&content).unwrap_or_default();

    for server_info in data.servers {
        if !server_info.enabled {
            continue;
        }

        println!("📡 Loading MCP Server: {}", server_info.name);
        match McpClient::new(&server_info.command, &server_info.args).await {
            Ok(client) => {
                let client_arc = Arc::new(client);
                if let Ok(tools) = client_arc.list_tools().await {
                    for tool_schema in tools {
                        let tool_name = tool_schema["name"].as_str().unwrap_or("unknown_tool").to_string();
                        let name_static = Box::leak(tool_name.into_boxed_str());
                        let ext_tool = ExternalMcpTool {
                            client: client_arc.clone(),
                            name: name_static,
                            toolset: "mcp",
                            schema_val: tool_schema.clone(),
                        };
                        registry.register(Arc::new(ext_tool)).await;
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to start MCP server '{}': {}", server_info.name, e);
            }
        }
    }
}

pub async fn serve_mcp(registry: Arc<ToolRegistry>) {
    let server = McpServer::new(registry);
    server.run().await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_mcp_server_management() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("mcp_servers.json");

        // Initial state should be empty
        let initial = get_mcp_servers(&file_path);
        assert!(initial.servers.is_empty());

        // Add a server
        let info = McpServerInfo {
            name: "test_server".to_string(),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "test".to_string()],
            enabled: true,
        };
        assert!(add_mcp_server(&file_path, info.clone()).is_ok());

        let after_add = get_mcp_servers(&file_path);
        assert_eq!(after_add.servers.len(), 1);
        assert_eq!(after_add.servers[0].name, "test_server");
        assert!(after_add.servers[0].enabled);

        // Toggle the server
        assert!(toggle_mcp_server(&file_path, 0).is_ok());
        let after_toggle = get_mcp_servers(&file_path);
        assert!(!after_toggle.servers[0].enabled);

        // Remove the server
        let removed = remove_mcp_server(&file_path, 0).unwrap();
        assert_eq!(removed.name, "test_server");

        let after_remove = get_mcp_servers(&file_path);
        assert!(after_remove.servers.is_empty());

        // Test error conditions
        assert!(toggle_mcp_server(&file_path, 0).is_err());
        assert!(remove_mcp_server(&file_path, 0).is_err());

        let empty_info = McpServerInfo {
            name: "".to_string(),
            command: "".to_string(),
            args: vec![],
            enabled: true,
        };
        assert!(add_mcp_server(&file_path, empty_info).is_err());
    }
}

// Rust guideline compliant 2026-02-21
