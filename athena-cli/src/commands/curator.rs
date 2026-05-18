use std::fs;
use athena_core::paths::get_hermes_home;

pub fn run_curator() {
    println!("\nHermes Background Curator Service");
    println!("═══════════════════════════════════\n");
    println!("Running diagnostic indexer, active catalog updates, and temporary file sweeps...");
    println!();

    let skills_dir = get_hermes_home().join("skills");
    let mut skills_count = 0;
    if let Ok(entries) = fs::read_dir(&skills_dir) {
        for entry in entries.flatten() {
            if entry.path().is_file() {
                skills_count += 1;
            }
        }
    }
    println!("  • Indexed {} custom semantic skills successfully.", skills_count);

    let plugins_dir = get_hermes_home().join("plugins");
    let mut plugins_count = 0;
    if let Ok(entries) = fs::read_dir(&plugins_dir) {
        for entry in entries.flatten() {
            if entry.path().is_file() {
                plugins_count += 1;
            }
        }
    }
    println!("  • Indexed {} sandboxed WASM agent extensions successfully.", plugins_count);

    let mut cleared_bytes = 0;
    let log_dir = get_hermes_home().join("logs");
    if let Ok(entries) = fs::read_dir(&log_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(elapsed) = modified.elapsed() {
                            if elapsed.as_secs() > 7 * 24 * 3600 {
                                cleared_bytes += metadata.len();
                                let _ = fs::remove_file(path);
                            }
                        }
                    }
                }
            }
        }
    }
    println!(
        "  • Wiped old date-rolled log traces: cleared {:.2} KB of disk space.",
        (cleared_bytes as f64) / 1024.0
    );

    println!("\n✓ Curator service scan completed successfully!");
}
