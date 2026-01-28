use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Represents a CTF competition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ctf {
    pub id: Option<i64>,
    pub name: String,
    pub url: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub team_name: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
}

impl Ctf {
    pub fn get_directory(&self, config: &crate::config::Config) -> PathBuf {
        let base = config.default_ctf_directory
            .as_ref()
            .cloned()
            .unwrap_or_else(|| PathBuf::from("."));

        let dir_name = self.name
            .replace(" ", "_")
            .replace("/", "-")
            .replace("\\", "-")
            .to_lowercase();

        base.join(dir_name)
    }
}

/// Represents a single challenge within a CTF
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Challenge {
    pub id: Option<i64>,
    pub ctf_id: i64,
    pub name: String,
    pub category: Option<String>,
    pub points: Option<i32>,
    pub flag: Option<String>,
    pub solved: bool,
    pub notes: Option<String>,
    pub created_at: String,
}

