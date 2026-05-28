pub mod config;
pub mod paths;
pub mod logging;

pub use config::*;
pub use paths::*;
pub use logging::*;

#[cfg(test)]
pub mod test_utils {
    use std::sync::Mutex;
    pub static ENV_LOCK: Mutex<()> = Mutex::new(());
}

// Rust guideline compliant 2026-02-21
