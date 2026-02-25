use anyhow::{Context, Result};
use chrono::{Datelike, Duration, NaiveDate, Utc, Weekday};
use reqwest::blocking::Client;
use rusqlite::Connection;
use serde::Deserialize;
use std::thread;
use std::time::Duration as StdDuration;

/// CTFtime API response structure
#[derive(Debug, Deserialize)]
struct CtfTimeEvent {
    title: String,
    url: String,
    start: String,  // ISO 8601 format: "2024-01-15T00:00:00+00:00"
    finish: String,
    description: String,
    organizers: Vec<Organizer>,
    format: Option<String>,
    onsite: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct Organizer {
    name: String,
}

/// Calculate the previous Thursday from today
fn get_previous_thursday() -> NaiveDate {
    let today = Utc::now().naive_utc().date();
    let weekday = today.weekday();

    // Calculate days to subtract to get to previous Thursday
    let days_back = match weekday {
        Weekday::Thu => 7, // If today is Thursday, go back to last Thursday
        Weekday::Fri => 1,
        Weekday::Sat => 2,
        Weekday::Sun => 3,
        Weekday::Mon => 4,
        Weekday::Tue => 5,
        Weekday::Wed => 6,
    };

    today - Duration::days(days_back)
}

/// Fetch CTF events from CTFtime.org
pub fn fetch_ctftime_events(conn: &Connection) -> Result<usize> {
    // Calculate timestamps
    let start_date = get_previous_thursday();
    let end_date = start_date + Duration::weeks(2);

    // Convert to Unix timestamps
    let start_timestamp = start_date
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc()
        .timestamp();
    let end_timestamp = end_date.and_hms_opt(23, 59, 59).unwrap().and_utc().timestamp();

    // Build API URL
    let url = format!(
        "https://ctftime.org/api/v1/events/?limit=100&start={}&finish={}",
        start_timestamp, end_timestamp
    );

    // Make HTTP request
    let client = Client::builder()
        .user_agent("Mozilla/5.0")  // CTFtime requires a proper user agent
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    let response = client
        .get(&url)
        .send()
        .context("Failed to fetch CTFtime events")?;

    if !response.status().is_success() {
        anyhow::bail!("CTFtime API returned error: {}", response.status());
    }

    let events: Vec<CtfTimeEvent> = response
        .json()
        .context("Failed to parse CTFtime response")?;

    // Insert events into database
    let mut inserted = 0;
    for event in events {
        // Extract organizer name if available
        let organizer = event.organizers.first().map(|org| org.name.clone());

        // Parse dates (convert from ISO 8601 to simple date format)
        let start_date = event.start.split('T').next().unwrap_or("");
        let end_date = event.finish.split('T').next().unwrap_or("");

        // Build notes with description and organizer
        let mut notes = String::new();

        // Add format if available
        if let Some(format) = &event.format {
            notes.push_str(&format!("Format: {}\n", format));
        }

        // Add organizer
        if let Some(org) = organizer {
            notes.push_str(&format!("Organized by: {}\n", org));
        }

        // Add onsite info
        if let Some(onsite) = event.onsite {
            if onsite {
                notes.push_str("Location: Onsite\n");
            }
        }

        notes.push_str("\n");

        // Add description (truncate to 500 chars)
        let short_desc = if event.description.chars().count() > 500 {
            let truncated: String = event.description.chars().take(500).collect();
            format!("{}...", truncated)
        } else {
            event.description.clone()
        };
        notes.push_str(&short_desc);

        // Insert or update existing event (using REPLACE to handle duplicates)
        match conn.execute(
            "INSERT OR REPLACE INTO ctfs (name, url, start_date, end_date, notes)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            [
                event.title.as_str(),
                event.url.as_str(),
                start_date,
                end_date,
                notes.as_str(),
            ],
        ) {
            Ok(_) => {
                inserted += 1;
            }
            Err(_e) => {
                // Silently continue on errors (shouldn't happen with INSERT OR REPLACE)
            }
        }
    }

    Ok(inserted)
}

/// Fetch CTFtime events with retry logic and cache metadata updates
/// This is the main entry point for fetching CTFtime data with error handling
pub fn fetch_and_cache_ctftime_events(db: &crate::db::queries::Database, conn: &Connection) -> Result<usize> {
    const MAX_RETRIES: u32 = 3;
    const INITIAL_BACKOFF_MS: u64 = 1000;

    let mut last_error = None;

    // Try up to MAX_RETRIES times with exponential backoff
    for attempt in 0..MAX_RETRIES {
        if attempt > 0 {
            let backoff = StdDuration::from_millis(INITIAL_BACKOFF_MS * 2_u64.pow(attempt - 1));
            thread::sleep(backoff);
        }

        match fetch_ctftime_events(conn) {
            Ok(count) => {
                // Success! Update cache metadata
                let timestamp = Utc::now().to_rfc3339();
                let status = format!("success: {} events", count);

                if let Err(_e) = db.update_ctftime_fetch_metadata(&timestamp, &status) {
                    // Don't fail the whole operation just because metadata update failed
                }

                return Ok(count);
            }
            Err(e) => {
                last_error = Some(e);
            }
        }
    }

    // All retries failed, update metadata with failure status
    let timestamp = Utc::now().to_rfc3339();
    let error_msg = last_error.as_ref()
        .map(|e| format!("error: {}", e))
        .unwrap_or_else(|| "error: unknown".to_string());

    if let Err(_e) = db.update_ctftime_fetch_metadata(&timestamp, &error_msg) {
        // Silently continue
    }

    Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Failed to fetch CTFtime events after {} retries", MAX_RETRIES)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_previous_thursday() {
        let thursday = get_previous_thursday();
        assert_eq!(thursday.weekday(), Weekday::Thu);
    }
}
