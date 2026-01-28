use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Filesystem sync configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    /// Automatically sync filesystem to database on TUI startup
    pub auto_sync_on_startup: bool,

    /// Confirm before deleting database entries during sync
    pub confirm_before_sync_delete: bool,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            auto_sync_on_startup: true,
            confirm_before_sync_delete: true,
        }
    }
}

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Default directory for CTF workspaces
    pub default_ctf_directory: Option<PathBuf>,

    /// Currently active CTF ID
    pub active_ctf_id: Option<i64>,

    /// Currently active CTF path
    pub active_ctf_path: Option<PathBuf>,

    /// Filesystem sync preferences
    #[serde(default)]
    pub sync: SyncConfig,

    /// UI preferences
    pub ui: UiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Tick rate for UI updates in milliseconds
    pub tick_rate_ms: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_ctf_directory: None,
            active_ctf_id: None,
            active_ctf_path: None,
            sync: SyncConfig::default(),
            ui: UiConfig {
                tick_rate_ms: 250,
            },
        }
    }
}

impl Config {
    /// Load configuration from the config file, creating it with defaults if it doesn't exist
    pub fn load() -> Result<Self> {
        let config_path = get_config_path()?;

        if !config_path.exists() {
            let config = Config::default();
            config.save()?;
            return Ok(config);
        }

        let content = fs::read_to_string(&config_path)
            .context("Failed to read config file")?;

        let config: Config = toml::from_str(&content)
            .context("Failed to parse config file")?;

        Ok(config)
    }

    /// Save configuration to the config file
    pub fn save(&self) -> Result<()> {
        let config_path = get_config_path()?;

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create config directory")?;
        }

        let content = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;

        fs::write(&config_path, content)
            .context("Failed to write config file")?;

        Ok(())
    }

    /// Set the active CTF
    pub fn set_active_ctf(&mut self, ctf_id: i64, path: PathBuf) -> Result<()> {
        self.active_ctf_id = Some(ctf_id);
        self.active_ctf_path = Some(path);
        self.save()
    }

    /// Get the active CTF path, or error if none is set
    pub fn get_active_ctf_path(&self) -> Result<PathBuf> {
        self.active_ctf_path
            .clone()
            .ok_or_else(|| anyhow::anyhow!("No active CTF selected. Run 'ctf' to select one."))
    }
}

/// Get the path to the configuration file
pub fn get_config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .context("Failed to determine config directory")?;

    Ok(config_dir.join("ctf-man").join("config.toml"))
}

/// Get the path to the data directory (for database)
pub fn get_data_dir() -> Result<PathBuf> {
    let data_dir = dirs::data_local_dir()
        .context("Failed to determine data directory")?;

    let dir = data_dir.join("ctf-man");

    fs::create_dir_all(&dir)
        .context("Failed to create data directory")?;

    Ok(dir)
}

/// Get the path to the database file
pub fn get_database_path() -> Result<PathBuf> {
    Ok(get_data_dir()?.join("ctf.db"))
}

/// Expand tilde (~) in a path to the user's home directory
pub fn expand_tilde(path: &str) -> Result<PathBuf> {
    if path.starts_with("~/") {
        let home = dirs::home_dir()
            .context("Failed to determine home directory")?;
        Ok(home.join(&path[2..]))
    } else if path == "~" {
        dirs::home_dir()
            .context("Failed to determine home directory")
    } else {
        Ok(PathBuf::from(path))
    }
}

/// Validate that a directory path exists or can be created
pub fn validate_and_create_directory(path: &Path) -> Result<()> {
    if path.exists() {
        if !path.is_dir() {
            anyhow::bail!("Path exists but is not a directory: {}", path.display());
        }
        // Directory already exists, all good
        Ok(())
    } else {
        // Try to create the directory
        fs::create_dir_all(path)
            .with_context(|| format!("Failed to create directory: {}", path.display()))?;
        Ok(())
    }
}

/// Set up the initial CTF directory with template subdirectories
pub fn setup_initial_directory(path: &Path) -> Result<()> {
    // Ensure the main directory exists
    validate_and_create_directory(path)?;

    // Create template subdirectories
    let templates_dir = path.join("templates");
    fs::create_dir_all(&templates_dir)
        .with_context(|| format!("Failed to create templates directory: {}", templates_dir.display()))?;

    // Create subdirectories for each category
    let categories = ["pwn", "crypto", "web", "rev"];
    for category in &categories {
        let category_dir = templates_dir.join(category);
        fs::create_dir_all(&category_dir)
            .with_context(|| format!("Failed to create category directory: {}", category_dir.display()))?;
    }

    Ok(())
}
