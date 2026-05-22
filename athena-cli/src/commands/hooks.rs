use std::fs;
use std::io::{self, Write};
use athena_core::paths::get_athena_home;

pub fn run_hooks() {
    println!("\nAthena Shell-Script Hooks");
    println!("═══════════════════════════\n");
    println!("Inspect and manage local event scripts executed before or after agent turns.");
    println!();

    let hooks_dir = get_athena_home().join("hooks");
    if !hooks_dir.exists() {
        let _ = fs::create_dir_all(&hooks_dir);
    }

    println!("Options:");
    println!("  1. List active shell hooks");
    println!("  2. Install a new shell hook");
    println!("  3. Delete a shell hook");
    println!("  4. Exit");
    println!();

    print!("  Choice [1-4]: ");
    io::stdout().flush().ok();

    let mut choice = String::new();
    io::stdin().read_line(&mut choice).ok();
    let choice = choice.trim().parse::<usize>().unwrap_or(4);

    match choice {
        1 => {
            println!("\nInstalled Shell Hooks:");
            let mut count = 0;
            if let Ok(entries) = fs::read_dir(&hooks_dir) {
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
                println!("  No shell hooks installed.");
            }
        }
        2 => {
            println!("\nInstall New Hook");
            print!("  Enter hook name (e.g. pre-step, post-step): ");
            io::stdout().flush().ok();
            let mut name = String::new();
            io::stdin().read_line(&mut name).ok();
            let name = name.trim().to_string();

            if name.is_empty() {
                println!("  ✗ Hook name cannot be empty.");
                return;
            }

            print!("  Enter shell script content (e.g. 'echo \"hello\"'): ");
            io::stdout().flush().ok();
            let mut script = String::new();
            io::stdin().read_line(&mut script).ok();
            let script = script.trim().to_string();

            let hook_path = hooks_dir.join(&name);
            match fs::write(&hook_path, script) {
                Ok(()) => {
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        if let Ok(meta) = fs::metadata(&hook_path) {
                            let mut perms = meta.permissions();
                            perms.set_mode(0o755);
                            let _ = fs::set_permissions(&hook_path, perms);
                        }
                    }
                    println!("  ✓ Hook '{}' successfully created and made executable!", name);
                }
                Err(e) => {
                    println!("  ✗ Failed to write hook: {}", e);
                }
            }
        }
        3 => {
            print!("\n  Enter hook name to delete: ");
            io::stdout().flush().ok();
            let mut name = String::new();
            io::stdin().read_line(&mut name).ok();
            let name = name.trim();

            let hook_path = hooks_dir.join(name);
            if hook_path.exists() {
                match fs::remove_file(&hook_path) {
                    Ok(()) => println!("  ✓ Hook '{}' deleted successfully.", name),
                    Err(e) => println!("  ✗ Failed to delete hook: {}", e),
                }
            } else {
                println!("  ✗ Hook '{}' does not exist.", name);
            }
        }
        _ => {}
    }
}
