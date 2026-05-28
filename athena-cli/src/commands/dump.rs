use athena_core::paths::get_athena_home;
use athena_core::config::{load_config, get_env_value};

pub fn run_dump() {
    let version = env!("CARGO_PKG_VERSION");
    let home = get_athena_home();
    let config = load_config();

    let providers = [
        ("OPENAI_API_KEY", "OpenAI"),
        ("ANTHROPIC_API_KEY", "Anthropic"),
        ("OPENROUTER_API_KEY", "OpenRouter"),
        ("GOOGLE_API_KEY", "Google"),
        ("DEEPSEEK_API_KEY", "DeepSeek"),
        ("GROQ_API_KEY", "Groq"),
        ("MISTRAL_API_KEY", "Mistral"),
        ("XAI_API_KEY", "xAI"),
    ];

    println!("### Athena Agent Support Dump");
    println!("- **Version**: `v{}`", version);
    println!("- **OS Platform**: `{}`", std::env::consts::OS);
    println!("- **Architecture**: `{}`", std::env::consts::ARCH);
    println!("- **Home Directory**: `{}`", home.display());
    println!("- **Default Model**: `{}`", config.model.default);

    println!("\n#### Active Credentials:");
    for (env_var, label) in providers {
        let configured = get_env_value(env_var).is_some();
        println!("- **{}**: `{}`", label, if configured { "Configured" } else { "Not Configured" });
    }

    println!("\n#### Paths & Files Presence:");
    println!("- **Config File**: `{}`", if home.join("config.json").exists() { "Present" } else { "Missing" });
    println!("- **Database**: `{}`", if home.join("session.db").exists() { "Present" } else { "Missing" });
    println!("- **Skills Folder**: `{}`", if home.join("skills").exists() { "Present" } else { "Missing" });
    println!("- **Plugins Folder**: `{}`", if home.join("plugins").exists() { "Present" } else { "Missing" });
}

// Rust guideline compliant 2026-02-21
