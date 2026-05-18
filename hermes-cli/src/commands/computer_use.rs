use hermes_core::config::{load_config, save_config};
use std::io::{self, Write};

pub fn run_computer_use() {
    println!("\nHermes Computer Use Backend (CUA)");
    println!("═══════════════════════════════════\n");
    println!("Controls local frame buffer capture, VNC connections, and mouse/keyboard drivers.");
    println!();

    let mut config = load_config();
    println!("YOLO auto-approval mode: {}", if config.agent.yolo_mode { "ENABLED (Caution!)" } else { "DISABLED (Safe)" });
    println!();

    println!("Checking platform compatibility...");
    let os = std::env::consts::OS;
    println!("  • Active OS Platform: {}", os);
    
    let has_xdotool = std::process::Command::new("which")
        .arg("xdotool")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    
    println!("  • Automation Driver (xdotool): {}", if has_xdotool { "AVAILABLE" } else { "NOT FOUND" });
    println!();

    println!("Options:");
    println!("  1. Toggle YOLO auto-approval mode");
    println!("  2. Run pre-flight graphics and VNC diagnostics");
    println!("  3. Exit");
    println!();

    print!("  Choice [1-3]: ");
    io::stdout().flush().ok();

    let mut choice = String::new();
    io::stdin().read_line(&mut choice).ok();
    let choice = choice.trim().parse::<usize>().unwrap_or(3);

    match choice {
        1 => {
            config.agent.yolo_mode = !config.agent.yolo_mode;
            if save_config(&config).is_ok() {
                println!("  ✓ Toggled YOLO mode. New value: {}", config.agent.yolo_mode);
            }
        }
        2 => {
            println!("\nRunning Pre-Flight Graphics Check...");
            if os == "linux" {
                let display = std::env::var("DISPLAY").unwrap_or_else(|_| "none".to_string());
                println!("  • X11 Display Status: {}", display);
                if display == "none" {
                    println!("  ⚠️ Warning: No active X11 display. Frame buffer capture might require a headless Xvfb or VNC setup.");
                } else {
                    println!("  ✓ Active X11 display detected.");
                }
            } else {
                println!("  • Platform diagnostics not yet implemented for non-Linux OS.");
            }
        }
        _ => {}
    }
}
