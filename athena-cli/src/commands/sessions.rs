use std::fs;
use std::io::{self, Write};
use athena_core::paths::get_athena_home;

pub fn run_sessions() {
    println!("\nAthena Sessions History");
    println!("═════════════════════════\n");
    println!("List, rename, export, delete, or prune past conversation histories.");
    println!();

    let sessions_dir = get_athena_home().join("sessions");
    if !sessions_dir.exists() {
        let _ = fs::create_dir_all(&sessions_dir);
    }

    let mut entries = Vec::new();
    if let Ok(dir_entries) = fs::read_dir(&sessions_dir) {
        for entry in dir_entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()).map(String::from) {
                    if let Ok(metadata) = entry.metadata() {
                        let modified = metadata.modified().ok();
                        entries.push((path, name, modified));
                    }
                }
            }
        }
    }

    if entries.is_empty() {
        println!("No past session files found.");
        return;
    }

    entries.sort_by(|a, b| b.2.cmp(&a.2));

    println!("Current session history records:");
    for (i, (_, name, modified)) in entries.iter().enumerate() {
        let date_str = if let Some(time) = modified {
            let datetime: chrono::DateTime<chrono::Local> = (*time).into();
            datetime.format("%Y-%m-%d %H:%M:%S").to_string()
        } else {
            "Unknown date".to_string()
        };
        println!("  {}. {} - (Last active: {})", i + 1, name, date_str);
    }
    println!();

    println!("Options:");
    println!("  1. View session contents");
    println!("  2. Delete a session");
    println!("  3. Prune all sessions");
    println!("  4. Exit");
    println!();

    print!("  Choice [1-4]: ");
    io::stdout().flush().ok();

    let mut choice = String::new();
    io::stdin().read_line(&mut choice).ok();
    let choice = choice.trim().parse::<usize>().unwrap_or(4);

    match choice {
        1 => {
            print!("  Select session number to view [1-{}]: ", entries.len());
            io::stdout().flush().ok();
            let mut s_choice = String::new();
            io::stdin().read_line(&mut s_choice).ok();
            let s_choice = s_choice.trim().parse::<usize>().unwrap_or(0);

            if s_choice < 1 || s_choice > entries.len() {
                println!("  ✗ Invalid choice.");
                return;
            }

            let (path, name, _) = &entries[s_choice - 1];
            println!("\n--- SESSION CONTENT: {} ---", name);
            if let Ok(content) = fs::read_to_string(path) {
                println!("{}", content);
            } else {
                println!("  (Unable to read session content)");
            }
            println!("-----------------------------");
        }
        2 => {
            print!("  Select session number to delete [1-{}]: ", entries.len());
            io::stdout().flush().ok();
            let mut s_choice = String::new();
            io::stdin().read_line(&mut s_choice).ok();
            let s_choice = s_choice.trim().parse::<usize>().unwrap_or(0);

            if s_choice < 1 || s_choice > entries.len() {
                println!("  ✗ Invalid choice.");
                return;
            }

            let (path, name, _) = &entries[s_choice - 1];
            if fs::remove_file(path).is_ok() {
                println!("  ✓ Session '{}' deleted successfully.", name);
            } else {
                println!("  ✗ Failed to delete session.");
            }
        }
        3 => {
            print!("  Are you sure you want to delete ALL sessions? [y/N]: ");
            io::stdout().flush().ok();
            let mut confirm = String::new();
            io::stdin().read_line(&mut confirm).ok();
            if confirm.trim().to_lowercase() == "y" {
                let mut count = 0;
                for (path, _, _) in entries {
                    if fs::remove_file(path).is_ok() {
                        count += 1;
                    }
                }
                println!("  ✓ Successfully cleared {} session history files.", count);
            }
        }
        _ => {}
    }
}
