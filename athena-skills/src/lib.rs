pub mod store;
pub mod manager;

pub use store::*;
pub use manager::*;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub instructions: String,
    pub created_at: i64,
    pub updated_at: i64,
}
