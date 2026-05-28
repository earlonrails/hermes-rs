use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use athena_core::paths::get_athena_home;
use cliclack::{intro, select, input, outro, outro_cancel};
use anyhow::Result;

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct KanbanBoard {
    pub tasks: Vec<TaskItem>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TaskItem {
    pub id: usize,
    pub title: String,
    pub status: String,
    pub assignee: Option<String>,
}

pub fn get_kanban_board(file_path: &Path) -> KanbanBoard {
    if file_path.exists() {
        let content = fs::read_to_string(file_path).unwrap_or_default();
        serde_json::from_str::<KanbanBoard>(&content).unwrap_or_default()
    } else {
        KanbanBoard::default()
    }
}

pub fn save_kanban_board(file_path: &Path, board: &KanbanBoard) -> Result<()> {
    let serialized = serde_json::to_string_pretty(board)?;
    fs::write(file_path, serialized)?;
    Ok(())
}

pub fn add_task(file_path: &Path, title: String, assignee: Option<String>) -> Result<usize> {
    if title.is_empty() {
        return Err(anyhow::anyhow!("Task title cannot be empty."));
    }
    let mut board = get_kanban_board(file_path);
    let next_id = board.tasks.iter().map(|t| t.id).max().unwrap_or(0) + 1;
    board.tasks.push(TaskItem {
        id: next_id,
        title,
        status: "Todo".to_string(),
        assignee,
    });
    save_kanban_board(file_path, &board)?;
    Ok(next_id)
}

pub fn move_task(file_path: &Path, task_id: usize, new_status: String) -> Result<()> {
    let mut board = get_kanban_board(file_path);
    if let Some(task) = board.tasks.iter_mut().find(|t| t.id == task_id) {
        task.status = new_status;
        save_kanban_board(file_path, &board)?;
        Ok(())
    } else {
        Err(anyhow::anyhow!("Task #{} not found.", task_id))
    }
}

pub fn delete_task(file_path: &Path, task_id: usize) -> Result<()> {
    let mut board = get_kanban_board(file_path);
    if board.tasks.iter().any(|t| t.id == task_id) {
        board.tasks.retain(|t| t.id != task_id);
        save_kanban_board(file_path, &board)?;
        Ok(())
    } else {
        Err(anyhow::anyhow!("Task #{} not found.", task_id))
    }
}

pub fn run_kanban() -> Result<()> {
    intro("Athena Collaboration Kanban Board")?;

    let kanban_file = get_athena_home().join("kanban.json");
    let board = get_kanban_board(&kanban_file);

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
            let assignee_in: String = input("Enter assignee (optional)").interact()?;
            let assignee = if assignee_in.trim().is_empty() {
                None
            } else {
                Some(assignee_in.trim().to_string())
            };

            match add_task(&kanban_file, title, assignee) {
                Ok(id) => outro(format!("Successfully added task #{}", id))?,
                Err(e) => outro_cancel(e.to_string())?,
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
            
            if let Some(task) = board.tasks.iter().find(|t| t.id == id) {
                let status: String = select(format!("Current status: {}. Select new status", task.status))
                    .item("Todo".to_string(), "Todo", "")
                    .item("In Progress".to_string(), "In Progress", "")
                    .item("Done".to_string(), "Done", "")
                    .interact()?;
                
                match move_task(&kanban_file, id, status.clone()) {
                    Ok(_) => outro(format!("Task #{} moved to status [{}].", id, status))?,
                    Err(e) => outro_cancel(e.to_string())?,
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

            match delete_task(&kanban_file, id) {
                Ok(_) => outro(format!("Task #{} deleted successfully.", id))?,
                Err(e) => outro_cancel(e.to_string())?,
            }
        }
        _ => { outro("Goodbye!")?; }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_kanban_management() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("kanban.json");

        // Initial state should be empty
        let initial = get_kanban_board(&file_path);
        assert!(initial.tasks.is_empty());

        // Add a task
        let id1 = add_task(&file_path, "First Task".to_string(), None).unwrap();
        assert_eq!(id1, 1);
        
        let board_after_add = get_kanban_board(&file_path);
        assert_eq!(board_after_add.tasks.len(), 1);
        assert_eq!(board_after_add.tasks[0].title, "First Task");
        assert_eq!(board_after_add.tasks[0].status, "Todo");
        
        // Add another task with assignee
        let id2 = add_task(&file_path, "Second Task".to_string(), Some("kevin".to_string())).unwrap();
        assert_eq!(id2, 2);
        
        // Move task
        assert!(move_task(&file_path, id1, "In Progress".to_string()).is_ok());
        let board_after_move = get_kanban_board(&file_path);
        assert_eq!(board_after_move.tasks.iter().find(|t| t.id == id1).unwrap().status, "In Progress");
        
        // Delete task
        assert!(delete_task(&file_path, id1).is_ok());
        let board_after_delete = get_kanban_board(&file_path);
        assert_eq!(board_after_delete.tasks.len(), 1);
        assert_eq!(board_after_delete.tasks[0].id, id2);
        
        // Test error cases
        assert!(add_task(&file_path, "".to_string(), None).is_err()); // empty title
        assert!(move_task(&file_path, 999, "Done".to_string()).is_err()); // not found
        assert!(delete_task(&file_path, 999).is_err()); // not found
    }
}

// Rust guideline compliant 2026-02-21
