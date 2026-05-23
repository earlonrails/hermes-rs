use cliclack::{intro, select, input, confirm, outro, outro_cancel, note};
use anyhow::Result;
use athena_core::config::{load_config, save_config, CronJob};

pub fn run_cron() -> Result<()> {
    intro("Athena Cron Job Scheduler")?;

    let choice: usize = select("Manage periodic background agent queries via the Gateway")
        .item(1, "List active Athena cron jobs", "")
        .item(2, "Add a new scheduled query", "")
        .item(3, "Remove all Athena cron jobs", "")
        .item(4, "Exit", "")
        .interact()?;

    let mut config = load_config();

    match choice {
        1 => {
            println!("\nActive internal cron jobs for Athena:");
            if config.cron_jobs.is_empty() {
                outro("No active Athena cron jobs found.")?;
            } else {
                let mut msg = String::from("Active Jobs:\n");
                for (i, job) in config.cron_jobs.iter().enumerate() {
                    msg.push_str(&format!("  [{}] '{}' -> {}\n", i, job.schedule, job.query));
                }
                outro(msg.trim_end())?;
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

            config.cron_jobs.push(CronJob {
                schedule: schedule.clone(),
                query: query.clone(),
            });

            if save_config(&config).is_ok() {
                note("Reminder", "You must restart the Athena Gateway to apply new cron schedules.")?;
                outro(format!("Successfully added scheduled job!\nJob: {} -> {}", schedule, query))?;
            } else {
                outro_cancel("Failed to save configuration.")?;
            }
        }
        3 => {
            let confirm_rm: bool = confirm("Are you sure you want to remove ALL scheduled Athena cron jobs?")
                .interact()?;

            if !confirm_rm {
                outro_cancel("Cancelled.")?;
                return Ok(());
            }

            config.cron_jobs.clear();

            if save_config(&config).is_ok() {
                note("Reminder", "You must restart the Athena Gateway to apply changes.")?;
                outro("All Athena cron jobs have been removed.")?;
            } else {
                outro_cancel("Failed to clear cron configuration.")?;
            }
        }
        _ => { outro("Goodbye!")?; }
    }
    
    Ok(())
}
