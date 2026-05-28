use athena_agent::AIAgent;
use athena_core::logging::{setup_logging, LoggingConfig, Mode};
use athena_core::config::CronJob;
use athena_tools::ToolRegistry;
use athena_providers::LLMProvider;
use teloxide::prelude::*;
use tracing::{error, info};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_cron_scheduler::{Job, JobScheduler};

pub async fn setup_cron_jobs(
    sched: &JobScheduler,
    jobs: Vec<CronJob>,
    agent: Arc<Mutex<AIAgent>>,
    registry: Arc<ToolRegistry>,
    provider: Arc<dyn LLMProvider + Send + Sync>
) -> anyhow::Result<()> {
    for cron in jobs {
        let job_agent = agent.clone();
        let job_registry = registry.clone();
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
    
    Ok(())
}

pub async fn process_gateway_message(
    text: &str,
    agent: Arc<Mutex<AIAgent>>,
    registry: Arc<ToolRegistry>,
    provider: Arc<dyn LLMProvider + Send + Sync>
) -> anyhow::Result<String> {
    let mut locked_agent = agent.lock().await;
    let response = locked_agent.run_conversation(
        text,
        Some("You are a helpful assistant talking over Telegram."),
        &registry,
        provider
    ).await.map_err(|e| anyhow::anyhow!("{}", e))?;
    Ok(response)
}

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
            
            if let Err(e) = setup_cron_jobs(&sched, config.cron_jobs, agent.clone(), arc_registry.clone(), provider.clone()).await {
                error!("Error setting up cron jobs: {}", e);
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

                match process_gateway_message(text, agent, registry, provider).await {
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

#[cfg(test)]
mod tests {
    use super::*;
    use athena_providers::LLMProvider;
    use async_trait::async_trait;

    struct DummyProvider {
        profile: athena_providers::ProviderProfile,
    }

    impl Default for DummyProvider {
        fn default() -> Self {
            Self {
                profile: athena_providers::ProviderProfile::new("dummy"),
            }
        }
    }

    #[async_trait::async_trait]
    impl LLMProvider for DummyProvider {
        fn profile(&self) -> &athena_providers::ProviderProfile {
            &self.profile
        }
        
        async fn fetch_models(
            &self,
            _api_key: Option<&str>,
            _timeout: f64,
        ) -> Result<Vec<String>, athena_providers::ProviderError> {
            Ok(vec![])
        }
        
        async fn create_chat_completion(
            &self,
            _request: athena_providers::ChatCompletionRequest,
        ) -> Result<athena_providers::ChatCompletionResponse, athena_providers::ProviderError> {
            Ok(athena_providers::ChatCompletionResponse {
                id: "1".into(),
                model: "dummy".into(),
                choices: vec![
                    athena_providers::Choice {
                        index: 0,
                        message: athena_providers::ChatMessage {
                            role: athena_providers::MessageRole::Assistant,
                            content: "Mock response".into(),
                            name: None,
                            tool_calls: None,
                            tool_call_id: None,
                        },
                        finish_reason: Some("stop".into()),
                    }
                ],
                usage: None,
                created: 0,
            })
        }
        
        async fn create_chat_completion_stream(
            &self,
            _request: athena_providers::ChatCompletionRequest,
        ) -> Result<athena_providers::ChatCompletionStream, athena_providers::ProviderError> {
            Err(athena_providers::ProviderError::ApiRequestFailed("Not implemented".into()))
        }
    }

    #[tokio::test]
    async fn test_process_gateway_message() {
        let agent = Arc::new(Mutex::new(AIAgent::builder().model("dummy-model".to_string()).build()));
        let registry = Arc::new(ToolRegistry::new());
        let provider: Arc<dyn LLMProvider + Send + Sync> = Arc::new(DummyProvider::default());
        
        let result = process_gateway_message("Hello from Telegram", agent, registry, provider).await.unwrap();
        assert_eq!(result, "Mock response");
    }

    #[tokio::test]
    async fn test_setup_cron_jobs() {
        let agent = Arc::new(Mutex::new(AIAgent::builder().model("dummy-model".to_string()).build()));
        let registry = Arc::new(ToolRegistry::new());
        let provider: Arc<dyn LLMProvider + Send + Sync> = Arc::new(DummyProvider::default());
        let sched = JobScheduler::new().await.unwrap();

        let jobs = vec![
            CronJob {
                schedule: "1/10 * * * * *".to_string(), // valid cron
                query: "Test".to_string(),
            },
            CronJob {
                schedule: "invalid cron".to_string(), // invalid cron
                query: "Test".to_string(),
            }
        ];

        let result = setup_cron_jobs(&sched, jobs, agent, registry, provider).await;
        assert!(result.is_ok());
    }
}

// Rust guideline compliant 2026-02-21
