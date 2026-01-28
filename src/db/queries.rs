use super::models::{Challenge, Ctf};
use anyhow::Result;
use rusqlite::{params, Connection};

/// Database wrapper providing high-level query methods
pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }

    // ==================== CTF Queries ====================

    /// Get all CTFs from the database
    pub fn get_all_ctfs(&self) -> Result<Vec<Ctf>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, url, start_date, end_date, team_name, notes, created_at
             FROM ctfs
             ORDER BY created_at DESC"
        )?;

        let ctfs = stmt.query_map([], |row| {
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
             ORDER BY CASE WHEN start_date IS NOT NULL THEN start_date ELSE created_at END ASC"
        )?;

        let ctfs = stmt.query_map(params![today], |row| {
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
             ORDER BY end_date DESC"
        )?;

        let ctfs = stmt.query_map(params![today], |row| {
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
    #[cfg_attr(not(test), allow(dead_code))]
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
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn get_ctf_by_id(&self, id: i64) -> Result<Ctf> {
        self.conn.query_row(
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
        ).map_err(|e| e.into())
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
             ORDER BY category, name"
        )?;

        let challenges = stmt.query_map(params![ctf_id], |row| {
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
        self.conn.execute(
            "DELETE FROM challenges WHERE id = ?",
            params![id],
        )?;
        Ok(())
    }

    // TODO: Add more challenge operations:
    // - update_challenge(&self, challenge: &Challenge) -> Result<()>
    // - toggle_solved(&self, id: i64) -> Result<()>
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
