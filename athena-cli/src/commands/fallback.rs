use athena_core::config::{load_config, save_config};
use std::io::{self, Write};

pub fn run_fallback() {
    let mut config = load_config();
    println!("\nFallback Providers Configuration");
    println!("═════════════════════════════════\n");
    println!("Fallback providers are queried sequentially when the primary provider fails.");
    println!("Current fallback list: {:?}", config.fallback_providers);
    println!();

    println!("  1. Add provider to fallbacks");
    println!("  2. Remove provider from fallbacks");
    println!("  3. Clear fallbacks");
    println!("  4. Exit");
    println!();

    print!("  Choice: ");
    io::stdout().flush().ok();
    let mut choice = String::new();
    io::stdin().read_line(&mut choice).ok();
    let choice = choice.trim().parse::<usize>().unwrap_or(4);

    match choice {
        1 => {
            print!("  Enter provider slug (e.g. anthropic, openrouter): ");
            io::stdout().flush().ok();
            let mut slug = String::new();
            io::stdin().read_line(&mut slug).ok();
            let slug = slug.trim().to_string();
            if !slug.is_empty() {
                config.fallback_providers.push(slug.clone());
                let _ = save_config(&config);
                println!("  ✓ Added {} to fallbacks.", slug);
            }
        }
        2 => {
            print!("  Enter provider slug to remove: ");
            io::stdout().flush().ok();
            let mut slug = String::new();
            io::stdin().read_line(&mut slug).ok();
            let slug = slug.trim();
            config.fallback_providers.retain(|p| p != slug);
            let _ = save_config(&config);
            println!("  ✓ Removed {} from fallbacks.", slug);
        }
        3 => {
            config.fallback_providers.clear();
            let _ = save_config(&config);
            println!("  ✓ Fallback list cleared.");
        }
        _ => {}
    }
}
