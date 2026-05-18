use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use hermes_core::paths::get_hermes_home;
use hermes_core::config::{load_config, get_env_value};

pub fn run_debug() {
    println!("\nHermes Debug & Diagnostic Packager");
    println!("═════════════════════════════════════\n");
    println!("Gathering local environment statistics, workspace directories, and model provider status...");
    println!();

    let mut report = String::new();
    report.push_str("=== HERMES DEBUG REPORT ===\n\n");

    // 1. Version Info
    let version = env!("CARGO_PKG_VERSION");
    let version_line = format!("Hermes Version : v{} (Rust Rewrite)\n", version);
    println!("  • {}", version_line.trim());
    report.push_str(&version_line);

    // 2. OS Info
    let os_line = format!("OS Platform    : {}\n", std::env::consts::OS);
    let arch_line = format!("Architecture   : {}\n", std::env::consts::ARCH);
    println!("  • {}", os_line.trim());
    println!("  • {}", arch_line.trim());
    report.push_str(&os_line);
    report.push_str(&arch_line);

    // 3. Executable Path
    if let Ok(exe_path) = std::env::current_exe() {
        let exe_line = format!("Binary Path    : {}\n", exe_path.display());
        println!("  • {}", exe_line.trim());
        report.push_str(&exe_line);
    }

    // 4. Hermes Home
    let home = get_hermes_home();
    let home_line = format!("Hermes Home    : {}\n", home.display());
    println!("  • {}", home_line.trim());
    report.push_str(&home_line);

    // 5. Config Info
    let config = load_config();
    let default_model_line = format!("Default Model  : {}\n", config.model.default);
    println!("  • {}", default_model_line.trim());
    report.push_str(&default_model_line);

    // 6. Provider status (without showing private keys)
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

    let mut prov_report = String::from("\n=== ACTIVE PROVIDER KEYS (MASKED) ===\n");
    println!("\nChecking API Credentials:");
    for (env_var, label) in providers {
        let status = if let Some(val) = get_env_value(env_var) {
            if val.len() > 8 {
                format!("ACTIVE ({}...{})", &val[..4], &val[val.len()-4..])
            } else {
                "ACTIVE (****)".to_string()
            }
        } else {
            "NOT CONFIGURED".to_string()
        };
        let line = format!("  • {:<12} : {}\n", label, status);
        print!("{}", line);
        prov_report.push_str(&line);
    }
    report.push_str(&prov_report);

    // Write to a local file
    let report_path = PathBuf::from("./hermes-debug-report.txt");
    if let Ok(mut file) = File::create(&report_path) {
        if file.write_all(report.as_bytes()).is_ok() {
            println!("\n✓ Diagnostic report generated successfully!");
            println!("Saved to: {}", report_path.display());
        }
    } else {
        println!("\n✗ Failed to save diagnostic report locally.");
    }
}
