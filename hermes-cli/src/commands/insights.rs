use std::fs;
use hermes_core::paths::get_hermes_home;

pub fn run_insights() {
    println!("\nHermes Usage Insights & Analytics");
    println!("═══════════════════════════════════\n");
    println!("Token consumption estimates, file metrics, and cost details.");
    println!();

    let sessions_dir = get_hermes_home().join("sessions");
    let mut sessions_count = 0;
    let mut total_session_size = 0;
    if let Ok(entries) = fs::read_dir(&sessions_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                sessions_count += 1;
                if let Ok(meta) = entry.metadata() {
                    total_session_size += meta.len();
                }
            }
        }
    }

    let log_dir = get_hermes_home().join("logs");
    let mut log_size = 0;
    if let Ok(entries) = fs::read_dir(&log_dir) {
        for entry in entries.flatten() {
            if entry.path().is_file() {
                if let Ok(meta) = entry.metadata() {
                    log_size += meta.len();
                }
            }
        }
    }

    println!("Workspace Metrics:");
    println!("  • Total Conversation Sessions : {}", sessions_count);
    println!("  • Accumulated Session Storage  : {:.2} KB", (total_session_size as f64) / 1024.0);
    println!("  • Rolling Debug Logs Size     : {:.2} KB", (log_size as f64) / 1024.0);
    println!();

    let est_tokens = (total_session_size as f64) / 4.0;
    let est_cost = est_tokens * 0.000015;

    println!("Estimated Inference Statistics:");
    println!("  • Estimated Tokens Processed  : {:.0} tokens", est_tokens);
    println!("  • Estimated Model Cost Saved  : ${:.4}", est_cost);
    println!();
}
