use athena_core::config::{load_config, save_config};
use std::io::{self, Write};

pub fn run_tools() {
    println!("\nAthena Tools Configuration");
    println!("════════════════════════════\n");
    println!("Configure which local capabilities (filesystem, search, execution) are enabled.");
    println!();

    let mut config = load_config();
    let all_tools = ["filesystem_read", "filesystem_write", "web_search", "command_execution", "browser_automation"];

    println!("Current Tools Status:");
    for (i, tool) in all_tools.iter().enumerate() {
        let disabled = config.tools.disabled.contains(&tool.to_string());
        let status = if disabled { "DISABLED" } else { "ENABLED" };
        println!("  {}. {:<22} : {}", i + 1, tool, status);
    }
    println!();

    println!("Options:");
    println!("  1. Toggle a tool's enabled/disabled status");
    println!("  2. Enable all tools");
    println!("  3. Exit");
    println!();

    print!("  Choice [1-3]: ");
    io::stdout().flush().ok();

    let mut choice = String::new();
    io::stdin().read_line(&mut choice).ok();
    let choice = choice.trim().parse::<usize>().unwrap_or(3);

    match choice {
        1 => {
            print!("  Select tool number to toggle [1-{}]: ", all_tools.len());
            io::stdout().flush().ok();
            let mut tool_choice = String::new();
            io::stdin().read_line(&mut tool_choice).ok();
            let tool_choice = tool_choice.trim().parse::<usize>().unwrap_or(0);

            if tool_choice < 1 || tool_choice > all_tools.len() {
                println!("  ✗ Invalid choice.");
                return;
            }

            let selected = all_tools[tool_choice - 1].to_string();
            if config.tools.disabled.contains(&selected) {
                config.tools.disabled.retain(|t| t != &selected);
                println!("  ✓ Enabled tool: {}.", selected);
            } else {
                config.tools.disabled.push(selected.clone());
                println!("  ✓ Disabled tool: {}.", selected);
            }

            let _ = save_config(&config);
        }
        2 => {
            config.tools.disabled.clear();
            if save_config(&config).is_ok() {
                println!("  ✓ All tools successfully enabled.");
            }
        }
        _ => {}
    }
}
