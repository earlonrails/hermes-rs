use athena_providers::providers::mistral::MistralProvider;
use athena_providers::LLMProvider;

#[tokio::main]
async fn main() {
// let _ = dotenvy::dotenv();
    let provider = MistralProvider::new();
    
    // Simulate what Mistral does in create_chat_completion:
    let profile = provider.profile().clone();
    println!("Mistral Profile Env Vars: {:?}", profile.env_vars);
    println!("Mistral Base URL: {:?}", profile.base_url);
    
    let mut resolved_key = None;
    for env_var in &profile.env_vars {
        if let Some(val) = athena_core::config::get_env_value(env_var) {
            println!("Found env var {} = {}", env_var, val);
            resolved_key = Some(val);
            break;
        } else {
            println!("Did not find env var {}", env_var);
        }
    }
    
    println!("Resolved Key: {:?}", resolved_key.map(|s| format!("{}...", &s[0..4])));
}
