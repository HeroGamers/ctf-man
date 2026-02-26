use super::models::{Challenge, ChallengeImport, Ctf};
use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};

/// Database wrapper providing high-level query methods
pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }

    /// Open (or create) the ctf-man database at the canonical path and
    /// initialise the schema.
    ///
    /// This is the most convenient way for external tools to obtain a
    /// `Database` handle without having to locate the file themselves:
    ///
    /// ```rust,no_run
    /// let db = ctf_man::db::Database::open().unwrap();
    /// ```
    pub fn open() -> Result<Self> {
        let db_path = crate::config::get_database_path()?;
        let conn = crate::db::schema::init_database(&db_path)?;
        Ok(Self::new(conn))
    }

    /// Return the schema version stamped in this database (`PRAGMA user_version`).
    ///
    /// External tools can compare this against [`crate::db::SCHEMA_VERSION`]
    /// to detect forward-incompatible schema changes.
    pub fn schema_version(&self) -> Result<i32> {
        crate::db::schema::get_schema_version(&self.conn)
    }

    // ==================== CTF Queries ====================

    /// Get all CTFs from the database
    pub fn get_all_ctfs(&self) -> Result<Vec<Ctf>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, url, start_date, end_date, team_name, notes, created_at
             FROM ctfs
             ORDER BY created_at DESC",
        )?;

        let ctfs = stmt
            .query_map([], |row| {
                Ok(Ctf {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    url: row.get(2)?,
                    start_date: row.get(3)?,
                    end_date: row.get(4)?,
                    team_name: row.get(5)?,
                    notes: row.get(6)?,
                    created_at: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(ctfs)
    }

    /// Get active and upcoming CTFs (end_date is today or in the future, or has no end_date)
    pub fn get_active_ctfs(&self) -> Result<Vec<Ctf>> {
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

        let mut stmt = self.conn.prepare(
            "SELECT id, name, url, start_date, end_date, team_name, notes, created_at
             FROM ctfs
             WHERE end_date IS NULL OR end_date >= ?
             ORDER BY CASE WHEN start_date IS NOT NULL THEN start_date ELSE created_at END ASC",
        )?;

        let ctfs = stmt
            .query_map(params![today], |row| {
                Ok(Ctf {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    url: row.get(2)?,
                    start_date: row.get(3)?,
                    end_date: row.get(4)?,
                    team_name: row.get(5)?,
                    notes: row.get(6)?,
                    created_at: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(ctfs)
    }

    /// Get archived CTFs (end_date is in the past)
    pub fn get_archived_ctfs(&self) -> Result<Vec<Ctf>> {
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

        let mut stmt = self.conn.prepare(
            "SELECT id, name, url, start_date, end_date, team_name, notes, created_at
             FROM ctfs
             WHERE end_date IS NOT NULL AND end_date < ?
             ORDER BY end_date DESC",
        )?;

        let ctfs = stmt
            .query_map(params![today], |row| {
                Ok(Ctf {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    url: row.get(2)?,
                    start_date: row.get(3)?,
                    end_date: row.get(4)?,
                    team_name: row.get(5)?,
                    notes: row.get(6)?,
                    created_at: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(ctfs)
    }

    /// Insert a new CTF into the database
    pub fn insert_ctf(&self, ctf: &Ctf) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO ctfs (name, url, start_date, end_date, team_name, notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                ctf.name,
                ctf.url,
                ctf.start_date,
                ctf.end_date,
                ctf.team_name,
                ctf.notes,
            ],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Get a CTF by its ID
    pub fn get_ctf_by_id(&self, id: i64) -> Result<Ctf> {
        self.conn
            .query_row(
                "SELECT id, name, url, start_date, end_date, team_name, notes, created_at
             FROM ctfs
             WHERE id = ?",
                params![id],
                |row| {
                    Ok(Ctf {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        url: row.get(2)?,
                        start_date: row.get(3)?,
                        end_date: row.get(4)?,
                        team_name: row.get(5)?,
                        notes: row.get(6)?,
                        created_at: row.get(7)?,
                    })
                },
            )
            .map_err(|e| e.into())
    }

    // TODO: Add more CTF operations:
    // - update_ctf(&self, ctf: &Ctf) -> Result<()>

    // ==================== Challenge Queries ====================

    /// Get all challenges for a specific CTF
    pub fn get_challenges_for_ctf(&self, ctf_id: i64) -> Result<Vec<Challenge>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, ctf_id, name, category, points, flag, solved, notes, created_at
             FROM challenges
             WHERE ctf_id = ?
             ORDER BY category, name",
        )?;

        let challenges = stmt
            .query_map(params![ctf_id], |row| {
                Ok(Challenge {
                    id: row.get(0)?,
                    ctf_id: row.get(1)?,
                    name: row.get(2)?,
                    category: row.get(3)?,
                    points: row.get(4)?,
                    flag: row.get(5)?,
                    solved: row.get(6)?,
                    notes: row.get(7)?,
                    created_at: row.get(8)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(challenges)
    }

    /// Insert a new challenge
    pub fn insert_challenge(&self, challenge: &Challenge) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO challenges (ctf_id, name, category, points, flag, solved, notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                challenge.ctf_id,
                challenge.name,
                challenge.category,
                challenge.points,
                challenge.flag,
                challenge.solved,
                challenge.notes,
            ],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Delete a challenge by ID
    pub fn delete_challenge(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM challenges WHERE id = ?", params![id])?;
        Ok(())
    }

    // TODO: Add more challenge operations:
    // - update_challenge(&self, challenge: &Challenge) -> Result<()>

    /// Toggle the solved status of a challenge
    pub fn toggle_solved(&self, id: i64) -> Result<()> {
        self.conn.execute(
            "UPDATE challenges SET solved = NOT solved WHERE id = ?",
            params![id],
        )?;
        Ok(())
    }

    // - search_challenges(&self, query: &str) -> Result<Vec<Challenge>>

    // ==================== API Cache Metadata Queries ====================

    /// Get the last CTFtime fetch timestamp (ISO 8601 format)
    pub fn get_last_ctftime_fetch(&self) -> Result<Option<String>> {
        let result = self.conn.query_row(
            "SELECT last_ctftime_fetch FROM api_cache_metadata WHERE id = 1",
            [],
            |row| row.get(0),
        );

        match result {
            Ok(timestamp) => Ok(timestamp),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Update the last CTFtime fetch timestamp and status
    pub fn update_ctftime_fetch_metadata(&self, timestamp: &str, status: &str) -> Result<()> {
        // Use INSERT OR REPLACE to handle first-time insertion
        self.conn.execute(
            "INSERT OR REPLACE INTO api_cache_metadata (id, last_ctftime_fetch, last_fetch_status)
             VALUES (1, ?1, ?2)",
            params![timestamp, status],
        )?;
        Ok(())
    }

    /// Check if CTFtime data should be refreshed (more than 12 hours since last fetch)
    pub fn should_refresh_ctftime(&self) -> Result<bool> {
        use chrono::{DateTime, Duration, Utc};

        let last_fetch = self.get_last_ctftime_fetch()?;

        match last_fetch {
            None => Ok(true), // Never fetched before
            Some(timestamp) => {
                // Parse the timestamp and check if 12 hours have passed
                match DateTime::parse_from_rfc3339(&timestamp) {
                    Ok(last_time) => {
                        let now = Utc::now();
                        let elapsed = now.signed_duration_since(last_time.with_timezone(&Utc));
                        Ok(elapsed > Duration::hours(12))
                    }
                    Err(_) => Ok(true), // Invalid timestamp, trigger refresh
                }
            }
        }
    }

    /// Fetch CTFtime events and update cache metadata
    pub fn fetch_ctftime_with_cache(&self) -> Result<usize> {
        crate::db::ctftime::fetch_and_cache_ctftime_events(self, &self.conn)
    }

    // ==================== Import API ====================

    /// Find-or-create a CTF by URL, then insert any challenges not yet present
    /// for that CTF.
    ///
    /// This is the primary entry point for external importers such as ctf-dl.
    /// The operation is fully idempotent:
    ///
    /// * The CTF row is looked up by `url`; if none is found a new row is
    ///   inserted using `name` as the display name.
    /// * Each challenge is keyed on `(ctf_id, name)`.  Rows that already exist
    ///   are skipped so that any notes or flags the user has added in ctf-man
    ///   are never overwritten.
    ///
    /// Returns `(ctf_id, new_challenge_count)`.
    pub fn import_ctf(
        &self,
        name: &str,
        url: &str,
        challenges: &[ChallengeImport],
    ) -> Result<(i64, usize)> {
        let ctf_id = self.find_or_create_ctf_by_url(name, url)?;

        let now = chrono::Utc::now().to_rfc3339();
        let mut inserted = 0usize;

        for ch in challenges {
            // Skip if a challenge with this name already exists for this CTF.
            let exists: bool = self
                .conn
                .query_row(
                    "SELECT 1 FROM challenges WHERE ctf_id = ?1 AND name = ?2 LIMIT 1",
                    params![ctf_id, &ch.name],
                    |_| Ok(true),
                )
                .optional()
                .map_err(anyhow::Error::from)?
                .unwrap_or(false);

            if exists {
                continue;
            }

            self.conn.execute(
                "INSERT INTO challenges (ctf_id, name, category, points, solved, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![ctf_id, &ch.name, &ch.category, ch.points, ch.solved, now],
            )?;

            inserted += 1;
        }

        Ok((ctf_id, inserted))
    }

    /// Look up a CTF by its URL; insert a new row if not found.
    ///
    /// Match order:
    /// 1. Exact URL match — fast path, handles re-runs of ctf-dl.
    /// 2. Hostname match — handles scheme/trailing-slash/path differences
    ///    between the URL ctf-dl uses and the one CTFtime stored (e.g.
    ///    `http://ctf.example.com/` vs `https://ctf.example.com`).
    ///    When matched this way the stored URL is updated to `url` so
    ///    subsequent runs hit the fast path.
    /// 3. Insert a new row if neither match succeeds.
    fn find_or_create_ctf_by_url(&self, name: &str, url: &str) -> Result<i64> {
        // 1. Exact match.
        let existing: Option<i64> = self
            .conn
            .query_row(
                "SELECT id FROM ctfs WHERE url = ? LIMIT 1",
                params![url],
                |row| row.get(0),
            )
            .optional()
            .map_err(anyhow::Error::from)?;

        if let Some(id) = existing {
            return Ok(id);
        }

        // 2. Hostname fallback: load all CTFs that have a URL and compare hosts.
        let target_host = reqwest::Url::parse(url)
            .ok()
            .and_then(|u| u.host_str().map(str::to_owned));

        if let Some(target_host) = target_host {
            let mut stmt = self
                .conn
                .prepare("SELECT id, url FROM ctfs WHERE url IS NOT NULL AND url != ''")?;

            let rows: Vec<(i64, String)> = stmt
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
                .filter_map(|r| r.ok())
                .collect();

            for (id, stored_url) in rows {
                let stored_host = reqwest::Url::parse(&stored_url)
                    .ok()
                    .and_then(|u| u.host_str().map(str::to_owned));

                if stored_host.as_deref() == Some(target_host.as_str()) {
                    // Update the stored URL to the canonical platform URL so
                    // future exact-match lookups work.
                    self.conn
                        .execute("UPDATE ctfs SET url = ?1 WHERE id = ?2", params![url, id])?;
                    return Ok(id);
                }
            }
        }

        // 3. No match – insert a new row.
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO ctfs (name, url, created_at) VALUES (?1, ?2, ?3)",
            params![name, url, now],
        )?;

        Ok(self.conn.last_insert_rowid())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_get_ctf() {
        // Use in-memory database for testing
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(crate::db::schema::SCHEMA).unwrap();
        let db = Database::new(conn);

        let ctf = Ctf {
            id: None,
            name: "Test CTF".to_string(),
            url: Some("https://test.com".to_string()),
            start_date: Some("2024-01-01".to_string()),
            end_date: Some("2024-01-03".to_string()),
            team_name: Some("TestTeam".to_string()),
            notes: None,
            created_at: String::new(), // Will be set by database
        };

        let id = db.insert_ctf(&ctf).unwrap();
        assert!(id > 0);

        let retrieved = db.get_ctf_by_id(id).unwrap();
        assert_eq!(retrieved.name, "Test CTF");
        assert_eq!(retrieved.url, Some("https://test.com".to_string()));
    }

    // TODO: Add more tests for other database operations
}
