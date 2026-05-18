use std::fs;
use std::io::{self, Write};
use hermes_core::paths::get_default_hermes_root;

pub fn run_profile() {
    println!("\nHermes Profiles Manager");
    println!("═════════════════════════\n");
    println!("Manage isolated Hermes workspace profiles.");
    println!();

    let root = get_default_hermes_root();
    let profiles_dir = root.join("profiles");
    if !profiles_dir.exists() {
        let _ = fs::create_dir_all(&profiles_dir);
    }

    let active_profile_path = root.join("active_profile");
    let active = if active_profile_path.exists() {
        fs::read_to_string(&active_profile_path).unwrap_or_else(|_| "default".to_string()).trim().to_string()
    } else {
        "default".to_string()
    };

    println!("Current Active Profile: {}", active);
    println!();

    let mut profiles = vec!["default".to_string()];
    if let Ok(entries) = fs::read_dir(&profiles_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    profiles.push(name.to_string());
                }
            }
        }
    }

    println!("Available Profiles:");
    for (i, p) in profiles.iter().enumerate() {
        let marker = if p == &active { " (active)" } else { "" };
        println!("  {}. {}{}", i + 1, p, marker);
    }
    println!();

    println!("Options:");
    println!("  1. Switch active profile");
    println!("  2. Create new isolated profile");
    println!("  3. Delete a profile");
    println!("  4. Exit");
    println!();

    print!("  Choice [1-4]: ");
    io::stdout().flush().ok();

    let mut choice = String::new();
    io::stdin().read_line(&mut choice).ok();
    let choice = choice.trim().parse::<usize>().unwrap_or(4);

    match choice {
        1 => {
            print!("  Select profile number to switch to [1-{}]: ", profiles.len());
            io::stdout().flush().ok();
            let mut p_choice = String::new();
            io::stdin().read_line(&mut p_choice).ok();
            let p_choice = p_choice.trim().parse::<usize>().unwrap_or(0);

            if p_choice < 1 || p_choice > profiles.len() {
                println!("  ✗ Invalid choice.");
                return;
            }

            let selected = &profiles[p_choice - 1];
            if fs::write(&active_profile_path, selected).is_ok() {
                println!("  ✓ Successfully switched active profile to: {}.", selected);
            } else {
                println!("  ✗ Failed to save active profile switch.");
            }
        }
        2 => {
            print!("  Enter new profile name: ");
            io::stdout().flush().ok();
            let mut name = String::new();
            io::stdin().read_line(&mut name).ok();
            let name = name.trim().to_string();

            if name.is_empty() || name == "default" {
                println!("  ✗ Invalid profile name.");
                return;
            }

            let new_profile_dir = profiles_dir.join(&name);
            if new_profile_dir.exists() {
                println!("  ✗ Profile '{}' already exists.", name);
                return;
            }

            if fs::create_dir_all(&new_profile_dir).is_ok() {
                let _ = fs::create_dir_all(new_profile_dir.join("skills"));
                let _ = fs::create_dir_all(new_profile_dir.join("plugins"));
                let _ = fs::create_dir_all(new_profile_dir.join("sessions"));
                
                let _ = fs::write(&active_profile_path, &name);
                println!("  ✓ Created and switched to new isolated profile: {}.", name);
            } else {
                println!("  ✗ Failed to create isolated profile.");
            }
        }
        3 => {
            print!("  Select profile number to delete [1-{}]: ", profiles.len());
            io::stdout().flush().ok();
            let mut p_choice = String::new();
            io::stdin().read_line(&mut p_choice).ok();
            let p_choice = p_choice.trim().parse::<usize>().unwrap_or(0);

            if p_choice < 1 || p_choice > profiles.len() {
                println!("  ✗ Invalid choice.");
                return;
            }

            let selected = &profiles[p_choice - 1];
            if selected == "default" {
                println!("  ✗ The 'default' profile cannot be deleted.");
                return;
            }

            if selected == &active {
                println!("  ✗ Cannot delete currently active profile. Switch profiles first.");
                return;
            }

            print!("  Are you sure you want to delete profile '{}'? [y/N]: ", selected);
            io::stdout().flush().ok();
            let mut confirm = String::new();
            io::stdin().read_line(&mut confirm).ok();
            if confirm.trim().to_lowercase() == "y" {
                let del_dir = profiles_dir.join(selected);
                if fs::remove_dir_all(del_dir).is_ok() {
                    println!("  ✓ Profile '{}' deleted successfully.", selected);
                } else {
                    println!("  ✗ Failed to delete profile folder.");
                }
            }
        }
        _ => {}
    }
}
