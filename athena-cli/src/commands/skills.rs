use std::fs;
use std::io::{self, Write};
use athena_core::paths::get_athena_home;

pub fn run_skills() {
    println!("\nAthena Semantic Skills Embeddings");
    println!("═══════════════════════════════════\n");
    println!("Search, install, configure, and manage dynamic skill definitions.");
    println!();

    let skills_dir = get_athena_home().join("skills");
    if !skills_dir.exists() {
        let _ = fs::create_dir_all(&skills_dir);
    }

    println!("Options:");
    println!("  1. List installed skills");
    println!("  2. Register new skill template");
    println!("  3. Uninstall a skill");
    println!("  4. Exit");
    println!();

    print!("  Choice [1-4]: ");
    io::stdout().flush().ok();

    let mut choice = String::new();
    io::stdin().read_line(&mut choice).ok();
    let choice = choice.trim().parse::<usize>().unwrap_or(4);

    match choice {
        1 => {
            println!("\nInstalled Skills:");
            let mut count = 0;
            if let Ok(entries) = fs::read_dir(&skills_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                            println!("  • {} ({})", name, path.display());
                            count += 1;
                        }
                    }
                }
            }
            if count == 0 {
                println!("  No active skills found.");
            }
        }
        2 => {
            println!("\nRegister New Skill");
            print!("  Enter skill identifier (e.g., fetch-docs, notify-slack): ");
            io::stdout().flush().ok();
            let mut name = String::new();
            io::stdin().read_line(&mut name).ok();
            let name = name.trim().to_string();

            if name.is_empty() {
                println!("  ✗ Skill identifier cannot be empty.");
                return;
            }

            let skill_file_name = format!("{}.rs", name);
            let skill_path = skills_dir.join(&skill_file_name);

            let template = format!(
                "// Skill: {}\n// Description: A new custom semantic skill definition\n\npub fn execute() {{\n    println!(\"Executing {} skill...\");\n}}\n",
                name, name
            );

            match fs::write(&skill_path, template) {
                Ok(()) => {
                    println!("  ✓ Registered skill template successfully!");
                    println!("  Created: {}", skill_path.display());
                }
                Err(e) => {
                    println!("  ✗ Failed to create skill template: {}", e);
                }
            }
        }
        3 => {
            print!("\n  Enter skill file name to uninstall (e.g. notify-slack.rs): ");
            io::stdout().flush().ok();
            let mut name = String::new();
            io::stdin().read_line(&mut name).ok();
            let name = name.trim();

            let skill_path = skills_dir.join(name);
            if skill_path.exists() {
                match fs::remove_file(&skill_path) {
                    Ok(()) => println!("  ✓ Skill '{}' uninstalled successfully.", name),
                    Err(e) => println!("  ✗ Failed to remove skill: {}", e),
                }
            } else {
                println!("  ✗ Skill file '{}' does not exist.", name);
            }
        }
        _ => {}
    }
}
