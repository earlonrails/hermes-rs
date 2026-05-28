use athena_skills::manager::SkillManager;
use uuid::Uuid;

fn main() {
    let temp_file = std::env::temp_dir().join(format!("test_skill_store_{}.db", Uuid::new_v4()));
    match SkillManager::new(&temp_file) {
        Ok(_) => println!("Success!"),
        Err(e) => println!("Error: {:?}", e),
    }
}

// Rust guideline compliant 2026-02-21
