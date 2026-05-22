use std::process::{Command, Stdio};
use std::io::Write;
use cliclack::{intro, select, input, confirm, outro, outro_cancel};
use anyhow::Result;

pub fn run_cron() -> Result<()> {
    intro("Athena Cron Job Scheduler")?;

    let choice: usize = select("Manage periodic background agent queries via the system crontab")
        .item(1, "List active Athena cron jobs", "")
        .item(2, "Add a new scheduled query", "")
        .item(3, "Remove all Athena cron jobs", "")
        .item(4, "Exit", "")
        .interact()?;

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
                        outro("No crontab exists for the current user")?;
                    } else if out.status.success() {
                        let stdout = String::from_utf8_lossy(&out.stdout);
                        let athena_jobs: Vec<&str> = stdout
                            .lines()
                            .filter(|line| line.contains("athena"))
                            .collect();

                        if athena_jobs.is_empty() {
                            outro("No active Athena cron jobs found.")?;
                        } else {
                            let mut msg = String::from("Active Jobs:\n");
                            for job in athena_jobs {
                                msg.push_str(&format!("  • {}\n", job));
                            }
                            outro(msg.trim_end())?;
                        }
                    } else {
                        outro_cancel(format!("Failed to read crontab: {}", stderr.trim()))?;
                    }
                }
                Err(e) => { outro_cancel(format!("Error invoking crontab command: {}", e))?; }
            }
        }
        2 => {
            let schedule: String = input("Enter cron schedule")
                .placeholder("0 * * * *")
                .interact()?;

            if schedule.is_empty() {
                outro_cancel("Schedule cannot be empty.")?;
                return Ok(());
            }

            let query: String = input("Enter the query for the agent")
                .placeholder("check server health")
                .interact()?;

            if query.is_empty() {
                outro_cancel("Query cannot be empty.")?;
                return Ok(());
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
                                outro(format!("Successfully added scheduled job!\nJob: {}", new_job.trim()))?;
                            } else {
                                outro_cancel(format!("Failed to update crontab: {}", String::from_utf8_lossy(&out.stderr).trim()))?;
                            }
                        }
                        Err(e) => { outro_cancel(format!("Error waiting for crontab update: {}", e))?; }
                    }
                }
                Err(e) => { outro_cancel(format!("Error starting crontab process: {}", e))?; }
            }
        }
        3 => {
            let confirm_rm: bool = confirm("Are you sure you want to remove ALL scheduled Athena cron jobs?")
                .interact()?;

            if !confirm_rm {
                outro_cancel("Cancelled.")?;
                return Ok(());
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
                                outro("All Athena cron jobs have been removed.")?;
                            } else {
                                outro_cancel(format!("Failed to clear crontab: {}", String::from_utf8_lossy(&out.stderr).trim()))?;
                            }
                        }
                        Err(e) => { outro_cancel(format!("Error waiting for crontab clear: {}", e))?; }
                    }
                }
                Err(e) => { outro_cancel(format!("Error starting crontab process: {}", e))?; }
            }
        }
        _ => { outro("Goodbye!")?; }
    }
    
    Ok(())
}
