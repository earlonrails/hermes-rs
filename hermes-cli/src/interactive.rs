use hermes_agent::AIAgent;
use hermes_tools::ToolRegistry;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use tracing::{error, info};

pub async fn run_interactive_loop(mut agent: AIAgent, registry: &ToolRegistry) {
    let mut rl = match DefaultEditor::new() {
        Ok(rl) => rl,
        Err(e) => {
            error!("Failed to initialize readline: {}", e);
            return;
        }
    };

    println!("Welcome to Hermes Agent (Rust Edition)");
    println!("Type '/quit' to exit.");

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                let input = line.trim();
                if input.is_empty() {
                    continue;
                }

                if input == "/quit" || input == "/exit" {
                    println!("Goodbye!");
                    break;
                }

                let _ = rl.add_history_entry(input);

                println!("Thinking...");
                
                // For now, no persistent history passed in, just a stateless run.
                // In a future PR we will track history.
                match agent.run_conversation(input, Some("You are a helpful assistant."), registry).await {
                    Ok(response) => {
                        println!("\n{}\n", response);
                    }
                    Err(e) => {
                        error!("Agent error: {}", e);
                        println!("Error: {}", e);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                error!("Error reading line: {:?}", err);
                break;
            }
        }
    }
}
