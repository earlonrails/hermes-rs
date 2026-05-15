mod interactive;

use clap::Parser;
use hermes_agent::AIAgent;
use hermes_core::logging::{setup_logging, LoggingConfig, Mode};
use hermes_tools::ToolRegistry;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Provide a custom workspace path
    #[arg(short, long)]
    workspace: Option<PathBuf>,
}

#[tokio::main]
async fn main() {
    // Load environment variables (.env file)
    let _ = dotenvy::dotenv();

    let args = Args::parse();

    // Setup logging
    let _log_dir = setup_logging(LoggingConfig {
        mode: Some(Mode::Cli),
        ..Default::default()
    });

    let api_key = std::env::var("OPENAI_API_KEY").ok();

    let mut builder = AIAgent::builder()
        .model("gpt-4o")
        .max_iterations(20);

    if let Some(k) = api_key {
        builder = builder.api_key(k);
    }

    let agent = builder.build();

    let registry = ToolRegistry::new();

    interactive::run_interactive_loop(agent, &registry).await;
}
