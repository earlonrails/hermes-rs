use hermes_agent::AIAgent;
use hermes_core::logging::{setup_logging, LoggingConfig, Mode};
use hermes_tools::ToolRegistry;
use teloxide::prelude::*;
use tracing::{error, info};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();

    setup_logging(LoggingConfig {
        mode: Some(Mode::Gateway),
        ..Default::default()
    });

    info!("Starting Hermes Telegram Gateway...");

    let bot = Bot::from_env();
    let registry = ToolRegistry::new();

    let agent_builder = AIAgent::builder()
        .model("gpt-4o")
        .max_iterations(10);
    
    // We wrap the builder so we can create fresh agents for users, or we can share an agent instance.
    // For now, let's just create one agent and protect it with a mutex.
    let agent = Arc::new(Mutex::new(agent_builder.build()));

    let handler = Update::filter_message().endpoint(
        |bot: Bot, msg: Message, agent: Arc<Mutex<AIAgent>>, registry: Arc<ToolRegistry>| async move {
            if let Some(text) = msg.text() {
                let _ = bot.send_message(msg.chat.id, "Thinking...").await;

                let mut locked_agent = agent.lock().await;
                match locked_agent.run_conversation(text, Some("You are a helpful assistant talking over Telegram."), &registry).await {
                    Ok(response) => {
                        let _ = bot.send_message(msg.chat.id, response).await;
                    }
                    Err(e) => {
                        error!("Agent error: {}", e);
                        let _ = bot.send_message(msg.chat.id, format!("Error: {}", e)).await;
                    }
                }
            }
            respond(())
        },
    );

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![agent, Arc::new(registry)])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
