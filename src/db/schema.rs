use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::Path;

/// The current schema version.
///
/// Bump this integer every time the database schema changes.  External tools
/// (e.g. ctf-dl) should read this value via [`get_schema_version`] after
/// opening the database and refuse (or warn) if their expected version does
/// not match.
pub const SCHEMA_VERSION: i32 = 1;

/// Read the schema version stored in the database (`PRAGMA user_version`).
///
/// Returns `0` if the database has never been stamped (i.e. was created before
/// schema versioning was introduced).
pub fn get_schema_version(conn: &Connection) -> Result<i32> {
    conn.query_row("PRAGMA user_version", [], |row| row.get(0))
        .context("Failed to read schema version")
}

/// Stamp the database with the given schema version (`PRAGMA user_version`).
pub fn set_schema_version(conn: &Connection, version: i32) -> Result<()> {
    // PRAGMA user_version does not accept bound parameters, so we format it
    // directly.  The value is always an i32 so there is no injection risk.
    conn.execute_batch(&format!("PRAGMA user_version = {version};"))
        .context("Failed to set schema version")
}

/// SQL schema for the CTF database
pub const SCHEMA: &str = r#"
-- CTFs table: stores information about CTF competitions
CREATE TABLE IF NOT EXISTS ctfs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    url TEXT,
    start_date TEXT,
    end_date TEXT,
    team_name TEXT,
    notes TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(name, start_date)  -- Prevent duplicate events with same name and start date
);

-- Challenges table: stores individual challenges within CTFs
CREATE TABLE IF NOT EXISTS challenges (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ctf_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    category TEXT,
    points INTEGER,
    flag TEXT,
    solved BOOLEAN DEFAULT 0,
    notes TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (ctf_id) REFERENCES ctfs(id) ON DELETE CASCADE
);

-- Templates table: stores metadata about challenge/CTF templates
CREATE TABLE IF NOT EXISTS templates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT,
    path TEXT NOT NULL,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

-- API cache metadata: tracks last fetch times for external APIs
CREATE TABLE IF NOT EXISTS api_cache_metadata (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    last_ctftime_fetch TEXT,
    last_fetch_status TEXT
);

-- Create indexes for better query performance
CREATE INDEX IF NOT EXISTS idx_challenges_ctf_id ON challenges(ctf_id);
CREATE INDEX IF NOT EXISTS idx_challenges_solved ON challenges(solved);
CREATE INDEX IF NOT EXISTS idx_ctfs_created_at ON ctfs(created_at);

-- TODO: Add full-text search support when implementing search feature
-- CREATE VIRTUAL TABLE IF NOT EXISTS challenges_fts USING fts5(
--     name, category, notes, content=challenges, content_rowid=id
-- );
"#;

/// Initialize the database with the schema
pub fn init_database(db_path: &Path) -> Result<Connection> {
    // Check if this is first run (database doesn't exist)
    let is_first_run = !db_path.exists();

    // Ensure the parent directory exists
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)
            .context("Failed to create database directory")?;
    }

    // Open or create the database
    let conn = Connection::open(db_path)
        .context("Failed to open database connection")?;

    // Enable foreign key support (important for CASCADE deletes)
    conn.execute("PRAGMA foreign_keys = ON;", [])
        .context("Failed to enable foreign keys")?;

    // Execute the schema
    conn.execute_batch(SCHEMA)
        .context("Failed to initialize database schema")?;

    // Stamp (or refresh) the schema version so external tools can detect
    // whether their copy of the schema is still compatible.
    set_schema_version(&conn, SCHEMA_VERSION)
        .context("Failed to stamp schema version")?;

    // If this is the first run, import CTFs from CTFtime
    if is_first_run {
        // Try to fetch from CTFtime, fall back to sample data if it fails
        match crate::db::ctftime::fetch_ctftime_events(&conn) {
            Ok(count) if count > 0 => {
                // Successfully imported CTFs
            }
            Ok(_) => {
                populate_sample_data(&conn)?;
            }
            Err(_e) => {
                populate_sample_data(&conn)?;
            }
        }
    }

    Ok(conn)
}

/// Populate the database with sample data on first run
fn populate_sample_data(conn: &Connection) -> Result<()> {
    conn.execute(
        "INSERT INTO ctfs (name, url, start_date, end_date, team_name, notes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        [
            "PicoCTF 2024",
            "https://picoctf.org",
            "2024-03-12",
            "2024-03-26",
            "MyTeam",
            "Great beginner-friendly CTF",
        ],
    )?;

    conn.execute(
        "INSERT INTO ctfs (name, url, start_date, end_date, team_name, notes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        [
            "HackTheBox Cyber Apocalypse",
            "https://ctf.hackthebox.com",
            "2024-04-10",
            "2024-04-14",
            "MyTeam",
            "Annual HTB CTF event",
        ],
    )?;

    Ok(())
}

// TODO: Add schema migration functions when you need to update the schema:
// - get_schema_version() -> Result<i32>
// - migrate_to_version(conn: &Connection, version: i32) -> Result<()>
