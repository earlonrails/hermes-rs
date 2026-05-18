use hermes_core::config::{load_config, get_env_value};
use std::process::Command;
use std::fs;

pub fn run_doctor() {
    println!("\nHermes Diagnostics (Doctor)");
    println!("═════════════════════════════\n");

    let mut issues_found = 0;

    // Check 1: ~/.hermes directory exists
    let home = hermes_core::paths::get_hermes_home();
    if home.exists() {
        println!("  ✓ Home directory exists at ~/.hermes");
    } else {
        println!("  ✗ Home directory ~/.hermes does not exist!");
        issues_found += 1;
    }

    // Check 2: config.yaml exists and is valid
    let config_path = hermes_core::paths::get_config_path();
    if config_path.exists() {
        println!("  ✓ Configuration file config.yaml exists");
        let config = load_config();
        if config.model.provider.is_empty() {
            println!("  ! Warning: Default model provider is not set.");
            issues_found += 1;
        } else {
            println!("  ✓ Active provider configured: {}", config.model.provider);
        }
    } else {
        println!("  ✗ config.yaml not found! Run 'hermes setup' to create one.");
        issues_found += 1;
    }

    // Check 3: env file
    let env_path = hermes_core::paths::get_env_path();
    if env_path.exists() {
        println!("  ✓ Environment file .env exists");
    } else {
        println!("  ! Warning: .env file not found.");
    }

    // Check 4: at least one API key configured
    let env_keys = [
        "OPENAI_API_KEY",
        "ANTHROPIC_API_KEY",
        "OPENROUTER_API_KEY",
        "GOOGLE_API_KEY",
    ];
    let mut keys_found = 0;
    for key in &env_keys {
        if get_env_value(key).is_some() {
            keys_found += 1;
        }
    }

    if keys_found > 0 {
        println!("  ✓ {} LLM API key(s) detected.", keys_found);
    } else {
        println!("  ✗ No LLM API keys configured! You won't be able to run queries. Run 'hermes login'.");
        issues_found += 1;
    }

    // Check 5: execution environment dependencies
    let docker_check = Command::new("docker").arg("--version").output();
    if docker_check.is_err() {
        println!("  ! Warning: Docker is not installed or not in PATH. Container environments won't work.");
    }

    if issues_found == 0 {
        println!("\n✓ No critical issues found! Your Hermes installation looks healthy.");
    } else {
        println!("\n✗ Found {} issue(s). Please follow the recommendations above to resolve them.", issues_found);
    }
}
