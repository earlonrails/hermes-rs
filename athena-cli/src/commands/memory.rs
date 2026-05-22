use athena_core::config::{get_env_value, save_env_value};
use cliclack::{intro, select, input, password, outro, note};
use anyhow::Result;

pub fn run_memory() -> Result<()> {
    intro("Athena External Memory Provider")?;
    note("Info", "Configure Honcho, Qdrant, or Pinecone semantic retrieval nodes.")?;

    let current = get_env_value("MEMORY_BACKEND").unwrap_or_else(|| "local".to_string());
    
    let choice: usize = select(format!("Current Memory Backend: {}", current))
        .item(1, "Set Memory Backend to Local SQLite", "")
        .item(2, "Set Memory Backend to Honcho Cloud", "")
        .item(3, "Set Memory Backend to Qdrant Vector DB", "")
        .item(4, "Exit", "")
        .interact()?;

    match choice {
        1 => {
            if save_env_value("MEMORY_BACKEND", "local").is_ok() {
                outro("Set memory backend to Local SQLite.")?;
            }
        }
        2 => {
            let key: String = password("Enter Honcho API Key (will be masked)").interact()?;
            let key = key.trim();
            if !key.is_empty() {
                let _ = save_env_value("HONCHO_API_KEY", key);
            }
            if save_env_value("MEMORY_BACKEND", "honcho").is_ok() {
                outro("Set memory backend to Honcho Cloud.")?;
            }
        }
        3 => {
            let url: String = input("Enter Qdrant URL")
                .placeholder("http://localhost:6334")
                .interact()?;
            let url = url.trim();
            if !url.is_empty() {
                let _ = save_env_value("QDRANT_URL", url);
            }
            if save_env_value("MEMORY_BACKEND", "qdrant").is_ok() {
                outro("Set memory backend to Qdrant Vector DB.")?;
            }
        }
        _ => { outro("Goodbye!")?; }
    }
    
    Ok(())
}
