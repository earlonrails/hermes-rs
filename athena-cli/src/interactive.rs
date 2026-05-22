use athena_agent::AIAgent;
use athena_tools::ToolRegistry;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use tracing::error;

pub async fn run_interactive_loop(mut agent: AIAgent, registry: &ToolRegistry, provider: std::sync::Arc<dyn athena_providers::LLMProvider + Send + Sync>) {
    let mut rl = match DefaultEditor::new() {
        Ok(rl) => rl,
        Err(e) => {
            error!("Failed to initialize readline: {}", e);
            return;
        }
    };

    let raw_model = agent.model();
    let provider_str = if raw_model.contains("claude") || raw_model.contains("opus") || raw_model.contains("sonnet") {
        " (Anthropic)"
    } else if raw_model.contains("gpt") {
        " (OpenAI)"
    } else {
        ""
    };
    let model_display = format!("{}{}", raw_model, provider_str);

    println!("🦉 Athena Interactive Agent Session (v{})", env!("CARGO_PKG_VERSION"));
    println!("Active Model: {}", model_display);
    println!("Sandbox Target: Docker (Local container active)");
    println!("Press Ctrl+D or type 'exit' to quit.");
    println!();

    loop {
        let readline = rl.readline("athena> ");
        match readline {
            Ok(line) => {
                let input = line.trim();
                if input.is_empty() {
                    continue;
                }

                if input == "/quit" || input == "/exit" || input == "exit" || input == "quit" {
                    println!("Goodbye!");
                    break;
                }

                let _ = rl.add_history_entry(input);

                println!();
                
                // For now, no persistent history passed in, just a stateless run.
                // In a future PR we will track history.
                match agent.run_conversation(input, Some("You are a helpful assistant."), registry, provider.clone()).await {
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

// Rust guideline compliant 2026-02-21
