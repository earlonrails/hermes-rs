use std::fs;
use std::io::{self, Write};
use serde::{Deserialize, Serialize};
use athena_core::paths::get_hermes_home;

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

pub fn run_kanban() {
    println!("\nHermes Collaboration Kanban Board");
    println!("═══════════════════════════════════\n");
    println!("View, assign, and transition multi-agent tasks.");
    println!();

    let kanban_file = get_hermes_home().join("kanban.json");
    let mut board = if kanban_file.exists() {
        let content = fs::read_to_string(&kanban_file).unwrap_or_default();
        serde_json::from_str::<KanbanBoard>(&content).unwrap_or_default()
    } else {
        KanbanBoard::default()
    };

    println!("Options:");
    println!("  1. View Kanban Board");
    println!("  2. Add new Task");
    println!("  3. Move Task status");
    println!("  4. Delete Task");
    println!("  5. Exit");
    println!();

    print!("  Choice [1-5]: ");
    io::stdout().flush().ok();

    let mut choice = String::new();
    io::stdin().read_line(&mut choice).ok();
    let choice = choice.trim().parse::<usize>().unwrap_or(5);

    match choice {
        1 => {
            println!("\n--- KANBAN BOARD ---");
            let columns = ["Todo", "In Progress", "Done"];
            for col in &columns {
                println!("\n  [{}]", col.to_uppercase());
                let col_tasks: Vec<&TaskItem> = board.tasks.iter().filter(|t| t.status == *col).collect();
                if col_tasks.is_empty() {
                    println!("    (No tasks)");
                } else {
                    for task in col_tasks {
                        let assignee = task.assignee.as_deref().unwrap_or("unassigned");
                        println!("    #{}: {} (Assigned to: {})", task.id, task.title, assignee);
                    }
                }
            }
            println!("\n--------------------");
        }
        2 => {
            println!("\nAdd New Task");
            print!("  Enter task title: ");
            io::stdout().flush().ok();
            let mut title = String::new();
            io::stdin().read_line(&mut title).ok();
            let title = title.trim().to_string();

            if title.is_empty() {
                println!("  ✗ Task title cannot be empty.");
                return;
            }

            print!("  Enter assignee (optional): ");
            io::stdout().flush().ok();
            let mut assignee_in = String::new();
            io::stdin().read_line(&mut assignee_in).ok();
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
                println!("  ✓ Successfully added task #{}", next_id);
            }
        }
        3 => {
            if board.tasks.is_empty() {
                println!("\n  No tasks on the board.");
                return;
            }

            print!("  Enter Task ID to move: ");
            io::stdout().flush().ok();
            let mut id_str = String::new();
            io::stdin().read_line(&mut id_str).ok();
            let id = id_str.trim().parse::<usize>().unwrap_or(0);

            if let Some(task) = board.tasks.iter_mut().find(|t| t.id == id) {
                println!("  Current status: {}", task.status);
                println!("  Select new status:");
                println!("    1. Todo");
                println!("    2. In Progress");
                println!("    3. Done");
                print!("    Choice [1-3]: ");
                io::stdout().flush().ok();

                let mut stat_choice = String::new();
                io::stdin().read_line(&mut stat_choice).ok();
                let status = match stat_choice.trim().parse::<usize>().unwrap_or(1) {
                    1 => "Todo",
                    2 => "In Progress",
                    3 => "Done",
                    _ => "Todo",
                };

                task.status = status.to_string();
                if let Ok(serialized) = serde_json::to_string_pretty(&board) {
                    let _ = fs::write(&kanban_file, serialized);
                    println!("  ✓ Task #{} moved to status [{}].", id, status);
                }
            } else {
                println!("  ✗ Task #{} not found.", id);
            }
        }
        4 => {
            if board.tasks.is_empty() {
                println!("\n  No tasks to delete.");
                return;
            }

            print!("  Enter Task ID to delete: ");
            io::stdout().flush().ok();
            let mut id_str = String::new();
            io::stdin().read_line(&mut id_str).ok();
            let id = id_str.trim().parse::<usize>().unwrap_or(0);

            if board.tasks.iter().any(|t| t.id == id) {
                board.tasks.retain(|t| t.id != id);
                if let Ok(serialized) = serde_json::to_string_pretty(&board) {
                    let _ = fs::write(&kanban_file, serialized);
                    println!("  ✓ Task #{} deleted successfully.", id);
                }
            } else {
                println!("  ✗ Task #{} not found.", id);
            }
        }
        _ => {}
    }
}
