mod interactive;
mod commands;

use clap::Parser;
use clap::Subcommand;
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
    
    /// Model to use (default: gpt-4o)
    #[arg(long, short = 'm')]
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
    
    /// One-shot mode: send a single prompt and print ONLY the final response text to stdout
    #[arg(long, short = 'z')]
    oneshot: Option<String>,
    
    /// Comma-separated toolsets to enable for this invocation
    #[arg(long, short = 't')]
    toolsets: Option<String>,
    
    /// Preload one or more skills for the session
    #[arg(long, short = 's')]
    skills: Option<String>,
    
    /// Resume a previous session by ID or title
    #[arg(long, short = 'r')]
    resume: Option<String>,
    
    /// Resume a session by name, or the most recent if no name given
    #[arg(long, short = 'c')]
    continue_session: Option<Option<String>>,
    
    /// Run in an isolated git worktree (for parallel agents)
    #[arg(long, short = 'w')]
    worktree: bool,
    
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Interactive chat with the agent
    Chat,
    
    /// Select default model and provider
    Model,
    
    /// Manage fallback providers (tried when the primary model fails)
    Fallback,
    
    /// Messaging gateway management
    Gateway,
    
    /// Language Server Protocol management
    Lsp,
    
    /// Interactive setup wizard
    Setup,
    
    /// Set up WhatsApp integration
    Whatsapp,
    
    /// Slack integration helpers (manifest generation, etc.)
    Slack,
    
    /// Authenticate with an inference provider
    Login,
    
    /// Clear authentication for an inference provider
    Logout,
    
    /// Manage pooled provider credentials
    Auth,
    
    /// Show status of all components
    Status,
    
    /// Cron job management
    Cron,
    
    /// Manage dynamic webhook subscriptions
    Webhook,
    
    /// Multi-profile collaboration board (tasks, links, comments)
    Kanban,
    
    /// Inspect and manage shell-script hooks
    Hooks,
    
    /// Check configuration and dependencies
    Doctor,
    
    /// Dump setup summary for support/debugging
    Dump,
    
    /// Debug tools — upload logs and system info for support
    Debug,
    
    /// Back up Hermes home directory to a zip file
    Backup,
    
    /// Inspect / prune / clear ~/.hermes/checkpoints/
    Checkpoints,
    
    /// Restore a Hermes backup from a zip file
    Import,
    
    /// View and edit configuration
    Config,
    
    /// Manage DM pairing codes for user authorization
    Pairing,
    
    /// Search, install, configure, and manage skills
    Skills,
    
    /// Manage plugins — install, update, remove, list
    Plugins,
    
    /// Background skill maintenance (curator) — status, run, pause, pin
    Curator,
    
    /// Configure external memory provider
    Memory,
    
    /// Configure which tools are enabled per platform
    Tools,
    
    /// Manage the Computer Use (cua-driver) backend (macOS)
    ComputerUse,
    
    /// Manage MCP servers and run Hermes as an MCP server
    Mcp,
    
    /// Manage session history (list, rename, export, prune, delete)
    Sessions,
    
    /// Show usage insights and analytics
    Insights,
    
    /// OpenClaw migration tools
    Claw,
    
    /// Show version information
    Version,
    
    /// Update Hermes Agent to the latest version
    Update,
    
    /// Uninstall Hermes Agent
    Uninstall,
    
    /// Run Hermes Agent as an ACP (Agent Client Protocol) server
    Acp,
    
    /// Manage profiles — multiple isolated Hermes instances
    Profile,
    
    /// Print shell completion script (bash, zsh, or fish)
    Completion,
    
    /// Start the web UI dashboard
    Dashboard,
    
    /// View and filter Hermes log files
    Logs,
    
    /// Run a single query
    #[command(alias = "q")]
    Query {
        /// The query to run
        query: String,
        
        /// Comma-separated list of toolsets to enable
        #[arg(long, short = 't')]
        toolsets: Option<String>,
        
        /// Comma-separated list of skills to preload
        #[arg(long, short = 's')]
        skills: Option<String>,
    },
    
    /// List available tools
    ListTools,
    
    /// List available toolsets
    ListToolsets,
    
    /// Show configuration
    ConfigShow,
    
    /// Show help
    Help,
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

    // Initialize agent builder with global options
    let mut builder = AIAgent::builder();
    
    // Set model if provided globally
    if let Some(model) = &args.model {
        builder = builder.model(model);
    } else {
        // Load default model from config
        let cfg = hermes_core::config::load_config();
        if !cfg.model.default.is_empty() {
            builder = builder.model(&cfg.model.default);
        } else {
            builder = builder.model("gpt-4o");
        }
    }
    
    // Set max iterations if provided globally
    if let Some(max_turns) = args.max_turns {
        builder = builder.max_iterations(max_turns as usize);
    } else {
        builder = builder.max_iterations(20);
    }
    
    // Set API key if provided globally
    let api_key = args.api_key.or_else(|| std::env::var("OPENAI_API_KEY").ok());
    if let Some(k) = api_key {
        builder = builder.api_key(k);
    }
    
    // Set base URL if provided globally
    if let Some(base_url) = &args.base_url {
        builder = builder.base_url(base_url);
    }

    match &args.command {
        Some(Commands::Query { query, toolsets, skills }) => {
            let mut agent = builder.build();
            let registry = ToolRegistry::new();
            
            // Process toolsets if provided
            if let Some(toolsets_str) = toolsets {
                println!("Toolsets specified: {}", toolsets_str);
            } else if let Some(global_toolsets) = &args.toolsets {
                println!("Toolsets specified: {}", global_toolsets);
            }
            
            // Process skills if provided
            if let Some(skills_str) = skills {
                println!("Skills specified: {}", skills_str);
            } else if let Some(global_skills) = &args.skills {
                println!("Skills specified: {}", global_skills);
            }
            
            match agent.run_conversation(query, None, &registry).await {
                Ok(response) => {
                    println!("{}", response);
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Some(Commands::ListTools) => {
            println!("Listing tools feature not yet implemented");
        }
        Some(Commands::ListToolsets) => {
            println!("Listing toolsets feature not yet implemented");
        }
        Some(Commands::ConfigShow) => {
            commands::config::run_config_show();
        }
        Some(Commands::Help) => {
            println!("Help not yet implemented");
        }
        Some(Commands::Chat) => {
            // Start interactive chat session
            let agent = builder.build();
            let registry = ToolRegistry::new();
            interactive::run_interactive_loop(agent, &registry).await;
        }
        Some(Commands::Model) => {
            commands::model::run_model();
        }
        Some(Commands::Fallback) => {
            commands::fallback::run_fallback();
        }
        Some(Commands::Gateway) => {
            commands::gateway::run_gateway();
        }
        Some(Commands::Lsp) => {
            commands::lsp::run_lsp();
        }
        Some(Commands::Setup) => {
            commands::setup::run_setup();
        }
        Some(Commands::Whatsapp) => {
            commands::whatsapp::run_whatsapp();
        }
        Some(Commands::Slack) => {
            commands::slack::run_slack();
        }
        Some(Commands::Login) => {
            commands::login::run_login();
        }
        Some(Commands::Logout) => {
            commands::login::run_logout();
        }
        Some(Commands::Auth) => {
            commands::auth::run_auth();
        }
        Some(Commands::Status) => {
            commands::status::run_status();
        }
        Some(Commands::Cron) => {
            commands::cron::run_cron();
        }
        Some(Commands::Webhook) => {
            commands::webhook::run_webhook();
        }
        Some(Commands::Kanban) => {
            commands::kanban::run_kanban();
        }
        Some(Commands::Hooks) => {
            commands::hooks::run_hooks();
        }
        Some(Commands::Doctor) => {
            commands::doctor::run_doctor();
        }
        Some(Commands::Dump) => {
            commands::dump::run_dump();
        }
        Some(Commands::Debug) => {
            commands::debug::run_debug();
        }
        Some(Commands::Backup) => {
            commands::backup::run_backup();
        }
        Some(Commands::Checkpoints) => {
            commands::checkpoints::run_checkpoints();
        }
        Some(Commands::Import) => {
            commands::import::run_import();
        }
        Some(Commands::Config) => {
            commands::config::run_config_edit();
        }
        Some(Commands::Pairing) => {
            commands::pairing::run_pairing();
        }
        Some(Commands::Skills) => {
            commands::skills::run_skills();
        }
        Some(Commands::Plugins) => {
            commands::plugins::run_plugins();
        }
        Some(Commands::Curator) => {
            commands::curator::run_curator();
        }
        Some(Commands::Memory) => {
            commands::memory::run_memory();
        }
        Some(Commands::Tools) => {
            commands::tools::run_tools();
        }
        Some(Commands::ComputerUse) => {
            commands::computer_use::run_computer_use();
        }
        Some(Commands::Mcp) => {
            commands::mcp::run_mcp();
        }
        Some(Commands::Sessions) => {
            commands::sessions::run_sessions();
        }
        Some(Commands::Insights) => {
            commands::insights::run_insights();
        }
        Some(Commands::Claw) => {
            commands::claw::run_claw();
        }
        Some(Commands::Version) => {
            commands::version::run_version();
        }
        Some(Commands::Update) => {
            commands::update::run_update();
        }
        Some(Commands::Uninstall) => {
            commands::uninstall::run_uninstall();
        }
        Some(Commands::Acp) => {
            commands::acp::run_acp();
        }
        Some(Commands::Profile) => {
            commands::profile::run_profile();
        }
        Some(Commands::Completion) => {
            commands::completion::run_completion();
        }
        Some(Commands::Dashboard) => {
            commands::dashboard::run_dashboard();
        }
        Some(Commands::Logs) => {
            commands::logs::run_logs();
        }
        None => {
            // Run interactive mode
            if let Some(oneshot) = &args.oneshot {
                // One-shot mode
                let mut agent = builder.build();
                let registry = ToolRegistry::new();
                
                match agent.run_conversation(oneshot, None, &registry).await {
                    Ok(response) => {
                        println!("{}", response);
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        std::process::exit(1);
                    }
                }
            } else if args.resume.is_some() || args.continue_session.is_some() {
                println!("Session resume not yet implemented");
            } else if args.worktree {
                println!("Worktree mode not yet implemented");
            } else {
                // Regular interactive mode
                let agent = builder.build();
                let registry = ToolRegistry::new();
                interactive::run_interactive_loop(agent, &registry).await;
            }
        }
    }
}
