use crate::config::Config;
use crate::db::Database;
use crate::db::models::{Challenge, Ctf};
use anyhow::{Context, Result};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// Result of a filesystem-to-database sync operation
#[derive(Debug, Default)]
pub struct SyncResult {
    pub challenges_removed: Vec<(String, String)>, // (ctf_name, challenge_name)
}

impl SyncResult {
    pub fn is_empty(&self) -> bool {
        self.challenges_removed.is_empty()
    }
}

/// Sanitize a challenge name for directory matching
pub fn sanitize_challenge_name(name: &str) -> String {
    let mut result = String::with_capacity(name.len());
    let mut last_was_underscore = false;

    for c in name.chars() {
        if c == '/'
            || c == '\\'
            || c == ':'
            || c == '*'
            || c == '?'
            || c == '"'
            || c == '<'
            || c == '>'
            || c == '|'
            || c == '\0'
            || c == ' '
            || !c.is_ascii()
        {
            if !last_was_underscore {
                result.push('_');
                last_was_underscore = true;
            }
        } else {
            result.push(c);
            last_was_underscore = false;
        }
    }

    result.trim_matches('_').to_lowercase()
}

/// Scan challenge directories within a CTF directory
/// Returns a map of category -> set of challenge names
fn scan_challenge_directories(ctf_path: &Path) -> Result<HashSet<(String, String)>> {
    let mut challenges = HashSet::new();

    if !ctf_path.exists() {
        return Ok(challenges);
    }

    // Read category directories
    let category_entries =
        fs::read_dir(ctf_path).context(format!("Failed to read CTF directory: {:?}", ctf_path))?;

    for category_entry in category_entries {
        let category_entry = category_entry.context("Failed to read category directory")?;
        let category_path = category_entry.path();

        if category_path.is_dir() {
            if let Some(category_name) = category_path.file_name().and_then(|n| n.to_str()) {
                // Read challenge directories within this category
                let challenge_entries = fs::read_dir(&category_path);

                if let Ok(challenge_entries) = challenge_entries {
                    for challenge_entry in challenge_entries {
                        let challenge_entry =
                            challenge_entry.context("Failed to read challenge directory")?;
                        let challenge_path = challenge_entry.path();

                        if challenge_path.is_dir() {
                            if let Some(challenge_name) =
                                challenge_path.file_name().and_then(|n| n.to_str())
                            {
                                challenges.insert((
                                    sanitize_challenge_name(category_name),
                                    sanitize_challenge_name(challenge_name),
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(challenges)
}

/// Find challenges in the database that don't have corresponding directories
fn find_orphaned_challenges(db: &Database, ctf: &Ctf, ctf_path: &Path) -> Result<Vec<Challenge>> {
    let all_challenges = db.get_challenges_for_ctf(ctf.id.unwrap())?;
    let existing_challenges = scan_challenge_directories(ctf_path)?;

    let mut orphaned = Vec::new();

    for challenge in all_challenges {
        let category =
            sanitize_challenge_name(challenge.category.as_deref().unwrap_or("uncategorized"));
        let challenge_name = sanitize_challenge_name(&challenge.name);

        let expected_key = (category.to_string(), challenge_name);

        if !existing_challenges.contains(&expected_key) {
            orphaned.push(challenge);
        }
    }

    Ok(orphaned)
}

/// Main sync function: removes challenge database entries for deleted filesystem directories
/// Note: This only syncs challenges, NOT CTFs. CTFs can exist in the database without
/// directories (e.g., from CTFtime API), but challenges are always user-created.
pub fn sync_filesystem_to_database(
    db: &Database,
    config: &Config,
    confirm: bool,
) -> Result<SyncResult> {
    let mut result = SyncResult::default();

    // Get the base CTF directory
    if config.default_ctf_directory.is_none() {
        // No CTF directory configured, nothing to sync
        return Ok(result);
    }

    // Get all CTFs from the database
    let all_ctfs = db.get_all_ctfs()?;

    // For each CTF, check if its directory exists and sync challenges
    for ctf in all_ctfs {
        let ctf_path = ctf.get_directory(config);

        // Only sync challenges for CTFs that have directories
        // CTFs without directories (e.g., from CTFtime) are skipped
        if ctf_path.exists() {
            let orphaned_challenges = find_orphaned_challenges(db, &ctf, &ctf_path)?;

            for challenge in orphaned_challenges {
                let challenge_name = challenge.name.clone();

                if confirm {
                    // Delete the orphaned challenge from the database
                    db.delete_challenge(challenge.id.unwrap())?;
                }

                result
                    .challenges_removed
                    .push((ctf.name.clone(), challenge_name));
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_challenge_name() {
        assert_eq!(sanitize_challenge_name("Easy Pwn"), "easy_pwn");
        assert_eq!(sanitize_challenge_name("Web/XSS"), "web_xss");
        assert_eq!(
            sanitize_challenge_name("⭐ Step 1: Read the rules"),
            "step_1_read_the_rules"
        );
        assert_eq!(
            sanitize_challenge_name("Super<Crazy>|Name?"),
            "super_crazy_name"
        );
    }
}
