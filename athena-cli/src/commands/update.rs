use std::process::Command;

pub fn run_update() {
    println!("\nUpdating Athena Agent...");
    println!("═════════════════════════\n");

    let script_path = "/home/kevin/code/athena/install.sh";

    let mut cmd = Command::new("bash");
    cmd.arg(script_path);

    match cmd.status() {
        Ok(status) => {
            if status.success() {
                println!("\n✓ Athena Agent updated successfully!");
            } else {
                println!("\n✗ Update failed with exit code: {}", status);
            }
        }
        Err(e) => {
            println!("\n✗ Failed to execute install script: {}", e);
        }
    }
}

// Rust guideline compliant 2026-02-21
