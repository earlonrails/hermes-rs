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
    
    /// Single query to execute (then exit)
    #[arg(short = 'q', long)]
    query: Option<String>,
    
    /// Comma-separated list of toolsets to enable
    #[arg(long)]
    toolsets: Option<String>,
    
    /// Comma-separated list of skills to preload
    #[arg(long)]
    skills: Option<String>,
    
    /// Model to use (default: gpt-4o)
    #[arg(long)]
    model: Option<String>,
    
    /// API key for authentication
    #[arg(long)]
    api_key: Option<String>,
    
    /// Base URL for the API
    #[arg(long)]
    base_url: Option<String>,
    
    /// Maximum tool-calling iterations (default: 20)
    #[arg(long)]
    max_turns: Option<u32>,
    
    /// List available tools and exit
    #[arg(long)]
    list_tools: bool,
    
    /// List available toolsets and exit
    #[arg(long)]
    list_toolsets: bool,
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

    // Handle list tools and toolsets
    if args.list_tools {
        let registry = ToolRegistry::new();
        println!("Available tools:");
        for tool in registry.list_tools() {
            println!("  - {}", tool);
        }
        return;
    }
    
    if args.list_toolsets {
        let registry = ToolRegistry::new();
        println!("Available toolsets:");
        for toolset in registry.list_toolsets() {
            println!("  - {}", toolset);
        }
        return;
    }

    // Initialize agent builder
    let mut builder = AIAgent::builder();
    
    // Set model if provided
    if let Some(model) = args.model {
        builder = builder.model(&model);
    } else {
        builder = builder.model("gpt-4o");
    }
    
    // Set max iterations if provided
    if let Some(max_turns) = args.max_turns {
        builder = builder.max_iterations(max_turns);
    } else {
        builder = builder.max_iterations(20);
    }
    
    // Set API key if provided
    let api_key = args.api_key.or_else(|| std::env::var("OPENAI_API_KEY").ok());
    if let Some(k) = api_key {
        builder = builder.api_key(k);
    }
    
    // Set base URL if provided
    if let Some(base_url) = args.base_url {
        builder = builder.base_url(&base_url);
    }
    
    let agent = builder.build();

    // Handle single query mode
    if let Some(query) = args.query {
        // In single query mode, we just run the agent once and exit
        let registry = ToolRegistry::new();
        
        // Process toolsets if provided
        if let Some(toolsets_str) = args.toolsets {
            // This would normally configure the agent with specific toolsets
            // For now, we'll just print a message
            println!("Toolsets specified: {}", toolsets_str);
        }
        
        // Process skills if provided
        if let Some(skills_str) = args.skills {
            // This would normally preload specific skills
            // For now, we'll just print a message
            println!("Skills specified: {}", skills_str);
        }
        
        match agent.run_conversation(&query, None, &registry).await {
            Ok(response) => {
                println!("{}", response);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        return;
    }

    // Otherwise, run interactive mode
    let registry = ToolRegistry::new();

    // Process toolsets if provided
    if let Some(toolsets_str) = args.toolsets {
        // This would normally configure the agent with specific toolsets
        println!("Toolsets specified: {}", toolsets_str);
    }
    
    // Process skills if provided
    if let Some(skills_str) = args.skills {
        // This would normally preload specific skills
        println!("Skills specified: {}", skills_str);
    }

    interactive::run_interactive_loop(agent, &registry).await;
}
