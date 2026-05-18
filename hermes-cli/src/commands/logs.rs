use std::fs;
use std::io::{self, BufRead, BufReader};
use hermes_core::paths::get_hermes_home;

pub fn run_logs() {
    println!("\nHermes Log Viewer");
    println!("═══════════════════\n");

    let log_dir = get_hermes_home().join("logs");
    if !log_dir.exists() {
        println!("No logs directory found at {}.", log_dir.display());
        return;
    }

    let mut entries = Vec::new();
    if let Ok(dir_entries) = fs::read_dir(&log_dir) {
        for entry in dir_entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(filename) = path.file_name().and_then(|f| f.to_str()).map(String::from) {
                    if filename.contains("log") {
                        if let Ok(metadata) = entry.metadata() {
                            if let Ok(modified) = metadata.modified() {
                                entries.push((path.clone(), filename, modified));
                            }
                        }
                    }
                }
            }
        }
    }

    if entries.is_empty() {
        println!("No log files found in {}.", log_dir.display());
        return;
    }

    // Sort by modified time descending (newest first)
    entries.sort_by(|a, b| b.2.cmp(&a.2));

    println!("Select a log file to view:");
    for (i, (_, filename, _)) in entries.iter().enumerate() {
        println!("  {}. {}", i + 1, filename);
    }
    println!();

    print!("  Choice [1-{}]: ", entries.len());
    use std::io::Write;
    io::stdout().flush().ok();

    let mut choice = String::new();
    io::stdin().read_line(&mut choice).ok();
    let choice = choice.trim().parse::<usize>().unwrap_or(1);

    if choice == 0 || choice > entries.len() {
        println!("Invalid choice.");
        return;
    }

    let (selected_path, selected_name, _) = &entries[choice - 1];
    println!("\n--- Showing last 50 lines of {} ---", selected_name);

    if let Ok(file) = fs::File::open(selected_path) {
        let reader = BufReader::new(file);
        let lines: Vec<String> = reader.lines().flatten().collect();
        let start = lines.len().saturating_sub(50);
        for line in &lines[start..] {
            println!("{}", line);
        }
        if lines.is_empty() {
            println!("(Empty file)");
        }
    } else {
        println!("Failed to open log file.");
    }
}
