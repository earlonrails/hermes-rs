use std::fs;
use cliclack::{intro, confirm, outro, outro_cancel, note};
use anyhow::Result;

pub fn run_uninstall() -> Result<()> {
    intro("Uninstall Athena Agent")?;
    note("Warning", "This will remove the ~/.athena config directory, skills, database, and settings.")?;

    let confirm_un: bool = confirm("Are you sure you want to proceed?").interact()?;

    if !confirm_un {
        outro_cancel("Uninstall cancelled.")?;
        return Ok(());
    }

    let home = athena_core::paths::get_athena_home();
    if home.exists() {
        match fs::remove_dir_all(&home) {
            Ok(()) => outro(format!("Removed directory {}", home.display()))?,
            Err(e) => outro_cancel(format!("Failed to remove home directory: {}", e))?,
        }
    } else {
        outro_cancel(format!("Home directory {} does not exist.", home.display()))?;
    }

    note("Next Steps", "To completely uninstall, please remove the 'athena' binary from your PATH/cargo bin directory.")?;
    Ok(())
}

// Rust guideline compliant 2026-02-21
