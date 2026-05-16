use rusqlite::{Connection, Result};
use std::path::PathBuf;
use std::sync::Mutex;
use hermes_core::get_hermes_home;

pub const DEFAULT_DB_NAME: &str = "state.db";

pub const SCHEMA_VERSION: i32 = 11;

pub const SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS schema_version (
    version INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY,
    source TEXT NOT NULL,
    user_id TEXT,
    model TEXT,
    model_config TEXT,
    system_prompt TEXT,
    parent_session_id TEXT,
    started_at REAL NOT NULL,
    ended_at REAL,
    end_reason TEXT,
    message_count INTEGER DEFAULT 0,
    tool_call_count INTEGER DEFAULT 0,
    input_tokens INTEGER DEFAULT 0,
    output_tokens INTEGER DEFAULT 0,
    cache_read_tokens INTEGER DEFAULT 0,
    cache_write_tokens INTEGER DEFAULT 0,
    reasoning_tokens INTEGER DEFAULT 0,
    billing_provider TEXT,
    billing_base_url TEXT,
    billing_mode TEXT,
    estimated_cost_usd REAL,
    actual_cost_usd REAL,
    cost_status TEXT,
    cost_source TEXT,
    pricing_version TEXT,
    title TEXT,
    api_call_count INTEGER DEFAULT 0,
    handoff_state TEXT,
    handoff_platform TEXT,
    handoff_error TEXT,
    FOREIGN KEY (parent_session_id) REFERENCES sessions(id)
);

CREATE TABLE IF NOT EXISTS messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL REFERENCES sessions(id),
    role TEXT NOT NULL,
    content TEXT,
    tool_call_id TEXT,
    tool_calls TEXT,
    tool_name TEXT,
    timestamp REAL NOT NULL,
    token_count INTEGER,
    finish_reason TEXT,
    reasoning TEXT,
    reasoning_content TEXT,
    reasoning_details TEXT,
    codex_reasoning_items TEXT,
    codex_message_items TEXT
);

CREATE TABLE IF NOT EXISTS state_meta (
    key TEXT PRIMARY KEY,
    value TEXT
);
"#;

pub struct SessionDB {
    conn: Mutex<Connection>,
}

impl SessionDB {
    pub fn new(db_path: Option<PathBuf>) -> Result<Self> {
        let path = db_path.unwrap_or_else(|| get_hermes_home().join(DEFAULT_DB_NAME));
        
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap_or_default();
        }

        let conn = Connection::open(&path)?;
        
        // Setup WAL mode
        let _ = conn.pragma_update(None, "journal_mode", "WAL");
        let _ = conn.pragma_update(None, "foreign_keys", "ON");

        let db = Self {
            conn: Mutex::new(conn),
        };

        db.init_schema()?;

        Ok(db)
    }

    fn init_schema(&self) -> Result<()> {
        let conn = match self.conn.lock() {
            Ok(c) => c,
            Err(_) => return Err(rusqlite::Error::InvalidQuery),
        };
        conn.execute_batch(SCHEMA_SQL)?;
        Ok(())
    }
}
