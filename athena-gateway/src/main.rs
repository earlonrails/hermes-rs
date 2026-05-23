use athena_agent::AIAgent;
use athena_core::logging::{setup_logging, LoggingConfig, Mode};
use athena_tools::ToolRegistry;
use teloxide::prelude::*;
use tracing::{error, info};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_cron_scheduler::{Job, JobScheduler};

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();

    setup_logging(LoggingConfig {
        mode: Some(Mode::Gateway),
        ..Default::default()
    });

    info!("Starting Athena Telegram Gateway...");

    let bot = Bot::from_env();
    let registry = ToolRegistry::new();

    let agent_builder = AIAgent::builder()
        .model("gpt-4o")
        .max_iterations(10);

    // We wrap the builder so we can create fresh agents for users, or we can share an agent instance.
    // For now, let's just create one agent and protect it with a mutex.
    let agent = Arc::new(Mutex::new(agent_builder.build()));

    athena_providers::registry::init_builtin_providers();
    let provider = athena_providers::registry::get_provider("openai").unwrap();

    let arc_registry = Arc::new(registry);

    // Initialize JobScheduler
    let config = athena_core::config::load_config();
    if !config.cron_jobs.is_empty() {
        if let Ok(sched) = JobScheduler::new().await {
            info!("Initializing {} cron jobs from config...", config.cron_jobs.len());
            for cron in config.cron_jobs {
                let job_agent = agent.clone();
                let job_registry = arc_registry.clone();
                let job_provider = provider.clone();
                let query = cron.query.clone();
                let schedule = cron.schedule.clone();
                
                let job = Job::new_async(schedule.as_str(), move |_uuid, mut _l| {
                    let agent_clone = job_agent.clone();
                    let registry_clone = job_registry.clone();
                    let provider_clone = job_provider.clone();
                    let query_clone = query.clone();
                    
                    Box::pin(async move {
                        info!("Executing cron job: '{}'", query_clone);
                        let mut locked_agent = agent_clone.lock().await;
                        match locked_agent.run_conversation(&query_clone, Some("You are a helpful assistant running as a cron job."), &registry_clone, provider_clone).await {
                            Ok(response) => {
                                info!("[Cron Job Completed]\nQuery: {}\nResponse: {}", query_clone, response);
                            }
                            Err(e) => {
                                error!("[Cron Job Error]\nQuery: {}\nError: {}", query_clone, e);
                            }
                        }
                    })
                });

                match job {
                    Ok(j) => {
                        if let Err(e) = sched.add(j).await {
                            error!("Failed to add job '{}': {}", cron.query, e);
                        }
                    }
                    Err(e) => error!("Invalid cron schedule '{}' for query '{}': {}", cron.schedule, cron.query, e),
                }
            }
            if let Err(e) = sched.start().await {
                error!("Failed to start JobScheduler: {}", e);
            } else {
                info!("Cron scheduler started.");
            }
        }
    }

    let handler = Update::filter_message().endpoint(
        |bot: Bot, msg: Message, agent: Arc<Mutex<AIAgent>>, registry: Arc<ToolRegistry>, provider: Arc<dyn athena_providers::LLMProvider + Send + Sync>| async move {
            if let Some(text) = msg.text() {
                let _ = bot.send_message(msg.chat.id, "Thinking...").await;

                let mut locked_agent = agent.lock().await;
                match locked_agent.run_conversation(text, Some("You are a helpful assistant talking over Telegram."), &registry, provider).await {
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
        .dependencies(dptree::deps![agent, arc_registry, provider])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
