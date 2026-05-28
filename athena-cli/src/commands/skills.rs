use std::fs;
use athena_core::paths::get_athena_home;
use cliclack::{intro, select, input, outro, outro_cancel, note};
use anyhow::Result;

pub fn run_skills() -> Result<()> {
    intro("Athena Semantic Skills Embeddings")?;
    note("Info", "Search, install, configure, and manage dynamic skill definitions.")?;

    let skills_dir = get_athena_home().join("skills");
    if !skills_dir.exists() {
        let _ = fs::create_dir_all(&skills_dir);
    }

    let choice: usize = select("Options")
        .item(1, "List installed skills", "")
        .item(2, "Register new skill template", "")
        .item(3, "Uninstall a skill", "")
        .item(4, "Exit", "")
        .interact()?;

    match choice {
        1 => {
            let mut count = 0;
            let mut msg = String::from("Installed Skills:\n");
            if let Ok(entries) = fs::read_dir(&skills_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                            msg.push_str(&format!("  • {} ({})\n", name, path.display()));
                            count += 1;
                        }
                    }
                }
            }
            if count == 0 {
                outro("No active skills found.")?;
            } else {
                outro(msg.trim_end())?;
            }
        }
        2 => {
            let name: String = input("Enter skill identifier")
                .placeholder("fetch-docs, notify-slack")
                .interact()?;
            let name = name.trim().to_string();

            if name.is_empty() {
                outro_cancel("Skill identifier cannot be empty.")?;
                return Ok(());
            }

            let skill_file_name = format!("{}.rs", name);
            let skill_path = skills_dir.join(&skill_file_name);

            let template = format!(
                "// Skill: {}\n// Description: A new custom semantic skill definition\n\npub fn execute() {{\n    println!(\"Executing {} skill...\");\n}}\n",
                name, name
            );

            match fs::write(&skill_path, template) {
                Ok(()) => {
                    note("Success", format!("Registered skill template successfully!\nCreated: {}", skill_path.display()))?;
                }
                Err(e) => {
                    outro_cancel(format!("Failed to create skill template: {}", e))?;
                }
            }
        }
        3 => {
            let name: String = input("Enter skill file name to uninstall")
                .placeholder("notify-slack.rs")
                .interact()?;
            let name = name.trim();

            let skill_path = skills_dir.join(name);
            if skill_path.exists() {
                match fs::remove_file(&skill_path) {
                    Ok(()) => outro(format!("Skill '{}' uninstalled successfully.", name))?,
                    Err(e) => outro_cancel(format!("Failed to remove skill: {}", e))?,
                }
            } else {
                outro_cancel(format!("Skill file '{}' does not exist.", name))?;
            }
        }
        _ => { outro("Goodbye!")?; }
    }
    
    Ok(())
}

// Rust guideline compliant 2026-02-21
