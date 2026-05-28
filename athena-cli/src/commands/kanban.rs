use std::fs;
use serde::{Deserialize, Serialize};
use athena_core::paths::get_athena_home;
use cliclack::{intro, select, input, outro, outro_cancel};
use anyhow::Result;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct KanbanBoard {
    tasks: Vec<TaskItem>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TaskItem {
    id: usize,
    title: String,
    status: String,
    assignee: Option<String>,
}

pub fn run_kanban() -> Result<()> {
    intro("Athena Collaboration Kanban Board")?;

    let kanban_file = get_athena_home().join("kanban.json");
    let mut board = if kanban_file.exists() {
        let content = fs::read_to_string(&kanban_file).unwrap_or_default();
        serde_json::from_str::<KanbanBoard>(&content).unwrap_or_default()
    } else {
        KanbanBoard::default()
    };

    let choice: usize = select("View, assign, and transition multi-agent tasks")
        .item(1, "View Kanban Board", "")
        .item(2, "Add new Task", "")
        .item(3, "Move Task status", "")
        .item(4, "Delete Task", "")
        .item(5, "Exit", "")
        .interact()?;

    match choice {
        1 => {
            let mut output = String::from("\n--- KANBAN BOARD ---\n");
            let columns = ["Todo", "In Progress", "Done"];
            for col in &columns {
                output.push_str(&format!("\n  [{}]\n", col.to_uppercase()));
                let col_tasks: Vec<&TaskItem> = board.tasks.iter().filter(|t| t.status == *col).collect();
                if col_tasks.is_empty() {
                    output.push_str("    (No tasks)\n");
                } else {
                    for task in col_tasks {
                        let assignee = task.assignee.as_deref().unwrap_or("unassigned");
                        output.push_str(&format!("    #{}: {} (Assigned to: {})\n", task.id, task.title, assignee));
                    }
                }
            }
            output.push_str("\n--------------------");
            outro(output)?;
        }
        2 => {
            let title: String = input("Enter task title").interact()?;
            if title.is_empty() {
                outro_cancel("Task title cannot be empty.")?;
                return Ok(());
            }

            let assignee_in: String = input("Enter assignee (optional)").interact()?;
            let assignee = if assignee_in.trim().is_empty() {
                None
            } else {
                Some(assignee_in.trim().to_string())
            };

            let next_id = board.tasks.iter().map(|t| t.id).max().unwrap_or(0) + 1;
            board.tasks.push(TaskItem {
                id: next_id,
                title,
                status: "Todo".to_string(),
                assignee,
            });

            if let Ok(serialized) = serde_json::to_string_pretty(&board) {
                let _ = fs::write(&kanban_file, serialized);
                outro(format!("Successfully added task #{}", next_id))?;
            }
        }
        3 => {
            if board.tasks.is_empty() {
                outro_cancel("No tasks on the board.")?;
                return Ok(());
            }

            let mut select_prompt = select("Select Task to move");
            for task in &board.tasks {
                select_prompt = select_prompt.item(task.id, format!("#{} - {}", task.id, task.title), &task.status);
            }
            let id: usize = select_prompt.interact()?;

            if let Some(task) = board.tasks.iter_mut().find(|t| t.id == id) {
                let status: String = select(format!("Current status: {}. Select new status", task.status))
                    .item("Todo".to_string(), "Todo", "")
                    .item("In Progress".to_string(), "In Progress", "")
                    .item("Done".to_string(), "Done", "")
                    .interact()?;

                task.status = status.clone();
                if let Ok(serialized) = serde_json::to_string_pretty(&board) {
                    let _ = fs::write(&kanban_file, serialized);
                    outro(format!("Task #{} moved to status [{}].", id, status))?;
                }
            } else {
                outro_cancel(format!("Task #{} not found.", id))?;
            }
        }
        4 => {
            if board.tasks.is_empty() {
                outro_cancel("No tasks to delete.")?;
                return Ok(());
            }

            let mut select_prompt = select("Select Task to delete");
            for task in &board.tasks {
                select_prompt = select_prompt.item(task.id, format!("#{} - {}", task.id, task.title), &task.status);
            }
            let id: usize = select_prompt.interact()?;

            if board.tasks.iter().any(|t| t.id == id) {
                board.tasks.retain(|t| t.id != id);
                if let Ok(serialized) = serde_json::to_string_pretty(&board) {
                    let _ = fs::write(&kanban_file, serialized);
                    outro(format!("Task #{} deleted successfully.", id))?;
                }
            } else {
                outro_cancel(format!("Task #{} not found.", id))?;
            }
        }
        _ => { outro("Goodbye!")?; }
    }
    
    Ok(())
}

// Rust guideline compliant 2026-02-21
