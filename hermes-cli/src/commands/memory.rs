use hermes_core::config::{get_env_value, save_env_value};
use std::io::{self, Write};

pub fn run_memory() {
    println!("\nHermes External Memory Provider");
    println!("═════════════════════════════════\n");
    println!("Configure Honcho, Qdrant, or Pinecone semantic retrieval nodes.");
    println!();

    let current = get_env_value("MEMORY_BACKEND").unwrap_or_else(|| "local".to_string());
    println!("Current Memory Backend: {}", current);
    println!();

    println!("Options:");
    println!("  1. Set Memory Backend to Local SQLite");
    println!("  2. Set Memory Backend to Honcho Cloud");
    println!("  3. Set Memory Backend to Qdrant Vector DB");
    println!("  4. Exit");
    println!();

    print!("  Choice [1-4]: ");
    io::stdout().flush().ok();

    let mut choice = String::new();
    io::stdin().read_line(&mut choice).ok();
    let choice = choice.trim().parse::<usize>().unwrap_or(4);

    match choice {
        1 => {
            if save_env_value("MEMORY_BACKEND", "local").is_ok() {
                println!("  ✓ Set memory backend to Local SQLite.");
            }
        }
        2 => {
            print!("  Enter Honcho API Key: ");
            io::stdout().flush().ok();
            let mut key = String::new();
            io::stdin().read_line(&mut key).ok();
            let key = key.trim();
            if !key.is_empty() {
                let _ = save_env_value("HONCHO_API_KEY", key);
            }
            if save_env_value("MEMORY_BACKEND", "honcho").is_ok() {
                println!("  ✓ Set memory backend to Honcho Cloud.");
            }
        }
        3 => {
            print!("  Enter Qdrant URL (e.g. http://localhost:6334): ");
            io::stdout().flush().ok();
            let mut url = String::new();
            io::stdin().read_line(&mut url).ok();
            let url = url.trim();
            if !url.is_empty() {
                let _ = save_env_value("QDRANT_URL", url);
            }
            if save_env_value("MEMORY_BACKEND", "qdrant").is_ok() {
                println!("  ✓ Set memory backend to Qdrant Vector DB.");
            }
        }
        _ => {}
    }
}
