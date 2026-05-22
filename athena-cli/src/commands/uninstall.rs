use std::io::{self, Write};
use std::fs;

pub fn run_uninstall() {
    println!("\nUninstall Athena Agent");
    println!("════════════════════════\n");
    println!("This will remove the ~/.athena config directory, skills, database, and settings.");
    print!("Are you sure you want to proceed? [y/N]: ");
    io::stdout().flush().ok();

    let mut confirm = String::new();
    io::stdin().read_line(&mut confirm).ok();
    if confirm.trim().to_lowercase() != "y" {
        println!("Uninstall cancelled.");
        return;
    }

    let home = athena_core::paths::get_athena_home();
    if home.exists() {
        match fs::remove_dir_all(&home) {
            Ok(()) => println!("✓ Removed directory {}", home.display()),
            Err(e) => println!("✗ Failed to remove home directory: {}", e),
        }
    } else {
        println!("Home directory {} does not exist.", home.display());
    }

    println!("\nTo completely uninstall, please remove the 'athena' binary from your PATH/cargo bin directory.");
}
