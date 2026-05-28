use std::fs;
use std::io::{self, Write};
use athena_core::paths::get_default_athena_root;

use anyhow::{anyhow, Result};
use std::path::Path;

pub fn get_active_profile(root: &Path) -> String {
    let active_profile_path = root.join("active_profile");
    if active_profile_path.exists() {
        fs::read_to_string(&active_profile_path)
            .unwrap_or_else(|_| "default".to_string())
            .trim()
            .to_string()
    } else {
        "default".to_string()
    }
}

pub fn get_available_profiles(root: &Path) -> Vec<String> {
    let profiles_dir = root.join("profiles");
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
    profiles
}

pub fn create_profile(root: &Path, name: &str) -> Result<()> {
    if name.is_empty() || name == "default" {
        return Err(anyhow!("Invalid profile name"));
    }
    
    let profiles_dir = root.join("profiles");
    let new_profile_dir = profiles_dir.join(name);
    
    if new_profile_dir.exists() {
        return Err(anyhow!("Profile already exists"));
    }
    
    fs::create_dir_all(&new_profile_dir)?;
    fs::create_dir_all(new_profile_dir.join("skills"))?;
    fs::create_dir_all(new_profile_dir.join("plugins"))?;
    fs::create_dir_all(new_profile_dir.join("sessions"))?;
    
    let active_profile_path = root.join("active_profile");
    fs::write(&active_profile_path, name)?;
    Ok(())
}

pub fn switch_profile(root: &Path, name: &str) -> Result<()> {
    let active_profile_path = root.join("active_profile");
    fs::write(&active_profile_path, name)?;
    Ok(())
}

pub fn delete_profile(root: &Path, name: &str) -> Result<()> {
    if name == "default" {
        return Err(anyhow!("Cannot delete the 'default' profile"));
    }
    
    let active = get_active_profile(root);
    if name == active {
        return Err(anyhow!("Cannot delete the active profile"));
    }
    
    let profiles_dir = root.join("profiles");
    let del_dir = profiles_dir.join(name);
    
    if del_dir.exists() {
        fs::remove_dir_all(del_dir)?;
    }
    Ok(())
}

pub fn run_profile() {
    println!("\nAthena Profiles Manager");
    println!("═════════════════════════\n");
    println!("Manage isolated Athena workspace profiles.");
    println!();

    let root = get_default_athena_root();
    let profiles_dir = root.join("profiles");
    if !profiles_dir.exists() {
        let _ = fs::create_dir_all(&profiles_dir);
    }

    let active = get_active_profile(&root);
    println!("Current Active Profile: {}", active);
    println!();

    let profiles = get_available_profiles(&root);
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
            if switch_profile(&root, selected).is_ok() {
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

            if create_profile(&root, &name).is_ok() {
                println!("  ✓ Created and switched to new isolated profile: {}.", name);
            } else {
                println!("  ✗ Failed to create isolated profile (or invalid name).");
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
            
            print!("  Are you sure you want to delete profile '{}'? [y/N]: ", selected);
            io::stdout().flush().ok();
            let mut confirm = String::new();
            io::stdin().read_line(&mut confirm).ok();
            if confirm.trim().to_lowercase() == "y" {
                if delete_profile(&root, selected).is_ok() {
                    println!("  ✓ Profile '{}' deleted successfully.", selected);
                } else {
                    println!("  ✗ Failed to delete profile folder.");
                }
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_profile_management() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        
        let active = get_active_profile(root);
        assert_eq!(active, "default");
        
        let profiles = get_available_profiles(root);
        assert_eq!(profiles, vec!["default".to_string()]);
        
        let res = create_profile(root, "test_profile");
        assert!(res.is_ok());
        
        let active = get_active_profile(root);
        assert_eq!(active, "test_profile");
        
        let profiles = get_available_profiles(root);
        assert!(profiles.contains(&"test_profile".to_string()));
        
        let res = switch_profile(root, "default");
        assert!(res.is_ok());
        assert_eq!(get_active_profile(root), "default");
        
        let res = delete_profile(root, "test_profile");
        assert!(res.is_ok());
        
        let profiles = get_available_profiles(root);
        assert!(!profiles.contains(&"test_profile".to_string()));
        
        // Error cases
        let res = create_profile(root, "default");
        assert!(res.is_err());
        
        let res = delete_profile(root, "default");
        assert!(res.is_err());
    }
}

// Rust guideline compliant 2026-02-21
