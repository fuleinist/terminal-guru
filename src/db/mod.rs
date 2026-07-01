use rusqlite::{Connection, params};
use std::path::PathBuf;
use std::sync::Mutex;

/// Database for storing history patterns and suggestions.
pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    /// Open or create the database at the default path.
    pub fn open() -> Result<Self, Box<dyn std::error::Error>> {
        let path = Self::path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(&path)?;
        let db = Database { conn: Mutex::new(conn) };
        db.initialize()?;
        Ok(db)
    }

    /// Open an in-memory database (for testing).
    pub fn open_in_memory() -> Result<Self, Box<dyn std::error::Error>> {
        let conn = Connection::open_in_memory()?;
        let db = Database { conn: Mutex::new(conn) };
        db.initialize()?;
        Ok(db)
    }

    fn path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let base = dirs::data_dir()
            .ok_or_else(|| "Could not find data directory".to_string())?;
        Ok(base.join("terminal-guru").join("patterns.db"))
    }

    fn initialize(&self) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS suggestions (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                kind        TEXT NOT NULL,
                alias       TEXT,
                command     TEXT NOT NULL,
                frequency   INTEGER NOT NULL DEFAULT 0,
                description TEXT NOT NULL,
                applied     INTEGER NOT NULL DEFAULT 0,
                created_at  TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE IF NOT EXISTS history_snapshot (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                command     TEXT NOT NULL,
                shell       TEXT NOT NULL,
                count       INTEGER NOT NULL DEFAULT 1,
                last_seen   TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE IF NOT EXISTS daemon_state (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );"
        )?;
        Ok(())
    }

    /// Record a suggestion.
    pub fn record_suggestion(&self, kind: &str, alias: Option<&str>, command: &str, frequency: usize, description: &str) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO suggestions (kind, alias, command, frequency, description) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![kind, alias, command, frequency as i64, description],
        )?;
        Ok(())
    }

    /// Mark a suggestion as applied.
    pub fn apply_suggestion(&self, id: i64) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        let rows = conn.execute(
            "UPDATE suggestions SET applied = 1 WHERE id = ?1",
            params![id],
        )?;
        if rows == 0 {
            return Err(format!("No suggestion with id {}", id).into());
        }
        Ok(())
    }

    /// List all suggestions.
    pub fn list_suggestions(&self, only_unapplied: bool) -> Result<Vec<SuggestionRow>, Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        let query = if only_unapplied {
            "SELECT id, kind, alias, command, frequency, description, applied, created_at FROM suggestions WHERE applied = 0 ORDER BY frequency DESC"
        } else {
            "SELECT id, kind, alias, command, frequency, description, applied, created_at FROM suggestions ORDER BY created_at DESC"
        };
        let mut stmt = conn.prepare(query)?;
        let rows = stmt.query_map([], |row| {
            Ok(SuggestionRow {
                id: row.get(0)?,
                kind: row.get(1)?,
                alias: row.get(2)?,
                command: row.get(3)?,
                frequency: row.get(4)?,
                description: row.get(5)?,
                applied: row.get(6)?,
                created_at: row.get(7)?,
            })
        })?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    /// Get or set daemon state.
    pub fn get_daemon_state(&self, key: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT value FROM daemon_state WHERE key = ?1")?;
        let mut rows = stmt.query_map(params![key], |row| row.get::<_, String>(0))?;
        Ok(rows.next().transpose()?)
    }

    pub fn set_daemon_state(&self, key: &str, value: &str) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO daemon_state (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }
}

#[derive(Debug, serde::Serialize)]
pub struct SuggestionRow {
    pub id: i64,
    pub kind: String,
    pub alias: Option<String>,
    pub command: String,
    pub frequency: usize,
    pub description: String,
    pub applied: i64,
    pub created_at: String,
}
