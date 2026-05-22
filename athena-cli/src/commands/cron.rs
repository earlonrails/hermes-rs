use std::process::{Command, Stdio};
use std::io::{self, Write};

pub fn run_cron() {
    println!("\nAthena Cron Job Scheduler");
    println!("═══════════════════════════\n");
    println!("This utility manages periodic background agent queries via the system crontab.");
    println!();
    println!("  1. List active Athena cron jobs");
    println!("  2. Add a new scheduled query");
    println!("  3. Remove all Athena cron jobs");
    println!("  4. Exit");
    println!();

    print!("  Choice [1-4]: ");
    io::stdout().flush().ok();

    let mut choice = String::new();
    io::stdin().read_line(&mut choice).ok();
    let choice = choice.trim().parse::<usize>().unwrap_or(4);

    match choice {
        1 => {
            println!("\nActive crontab entries for Athena:");
            let output = Command::new("crontab")
                .arg("-l")
                .output();

            match output {
                Ok(out) => {
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    if stderr.contains("no crontab for") {
                        println!("  (No crontab exists for the current user)");
                    } else if out.status.success() {
                        let stdout = String::from_utf8_lossy(&out.stdout);
                        let athena_jobs: Vec<&str> = stdout
                            .lines()
                            .filter(|line| line.contains("athena"))
                            .collect();

                        if athena_jobs.is_empty() {
                            println!("  No active Athena cron jobs found.");
                        } else {
                            for job in athena_jobs {
                                println!("  • {}", job);
                            }
                        }
                    } else {
                        println!("  Failed to read crontab: {}", stderr.trim());
                    }
                }
                Err(e) => println!("  ✗ Error invoking crontab command: {}", e),
            }
        }
        2 => {
            println!("\nAdd Scheduled Query");
            println!("-------------------");
            print!("  Enter cron schedule (e.g. '0 * * * *' for hourly, '0 9 * * *' for daily at 9am): ");
            io::stdout().flush().ok();
            let mut schedule = String::new();
            io::stdin().read_line(&mut schedule).ok();
            let schedule = schedule.trim().to_string();

            if schedule.is_empty() {
                println!("  Schedule cannot be empty.");
                return;
            }

            print!("  Enter the query for the agent (e.g. 'check server health'): ");
            io::stdout().flush().ok();
            let mut query = String::new();
            io::stdin().read_line(&mut query).ok();
            let query = query.trim().to_string();

            if query.is_empty() {
                println!("  Query cannot be empty.");
                return;
            }

            // Get current crontab
            let mut current_cron = String::new();
            if let Ok(out) = Command::new("crontab").arg("-l").output() {
                if out.status.success() {
                    current_cron = String::from_utf8_lossy(&out.stdout).to_string();
                }
            }

            // Path to Athena binary
            let exe_path = std::env::current_exe()
                .unwrap_or_else(|_| std::path::PathBuf::from("athena"));

            // Build the cron line:
            let new_job = format!(
                "{} {} query \"{}\" >> ~/.athena/logs/cron.log 2>&1\n",
                schedule,
                exe_path.display(),
                query
            );

            current_cron.push_str(&new_job);

            // Write back to crontab
            let child = Command::new("crontab")
                .arg("-")
                .stdin(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn();

            match child {
                Ok(mut c) => {
                    if let Some(mut stdin) = c.stdin.take() {
                        let _ = stdin.write_all(current_cron.as_bytes());
                    }
                    match c.wait_with_output() {
                        Ok(out) => {
                            if out.status.success() {
                                println!("  ✓ Successfully added scheduled job!");
                                println!("  Job: {}", new_job.trim());
                            } else {
                                println!("  ✗ Failed to update crontab: {}", String::from_utf8_lossy(&out.stderr).trim());
                            }
                        }
                        Err(e) => println!("  ✗ Error waiting for crontab update: {}", e),
                    }
                }
                Err(e) => println!("  ✗ Error starting crontab process: {}", e),
            }
        }
        3 => {
            print!("\nAre you sure you want to remove ALL scheduled Athena cron jobs? [y/N]: ");
            io::stdout().flush().ok();
            let mut confirm = String::new();
            io::stdin().read_line(&mut confirm).ok();
            if confirm.trim().to_lowercase() != "y" {
                println!("Cancelled.");
                return;
            }

            // Get current crontab
            let mut current_cron = String::new();
            if let Ok(out) = Command::new("crontab").arg("-l").output() {
                if out.status.success() {
                    current_cron = String::from_utf8_lossy(&out.stdout).to_string();
                }
            }

            // Filter out lines containing Athena
            let filtered_lines: Vec<&str> = current_cron
                .lines()
                .filter(|line| !line.contains("athena"))
                .collect();

            let mut new_cron = filtered_lines.join("\n");
            if !new_cron.is_empty() {
                new_cron.push('\n');
            }

            // Write back to crontab
            let child = Command::new("crontab")
                .arg("-")
                .stdin(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn();

            match child {
                Ok(mut c) => {
                    if let Some(mut stdin) = c.stdin.take() {
                        let _ = stdin.write_all(new_cron.as_bytes());
                    }
                    match c.wait_with_output() {
                        Ok(out) => {
                            if out.status.success() {
                                println!("  ✓ All Athena cron jobs have been removed.");
                            } else {
                                println!("  ✗ Failed to clear crontab: {}", String::from_utf8_lossy(&out.stderr).trim());
                            }
                        }
                        Err(e) => println!("  ✗ Error waiting for crontab clear: {}", e),
                    }
                }
                Err(e) => println!("  ✗ Error starting crontab process: {}", e),
            }
        }
        _ => {}
    }
}
