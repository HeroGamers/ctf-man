// Public library modules (always available, no TUI dependency)
pub mod config;
pub mod db;
pub mod fs;
pub mod templates;

// CLI modules
pub mod cli;

// TUI modules – only compiled when the "tui" feature is enabled.
// Downstream crates that use ctf-man purely as a library (e.g. ctf-dl) should
// depend with `default-features = false` to avoid pulling in ratatui/crossterm.
#[cfg(feature = "tui")]
pub mod ui;

#[cfg(feature = "tui")]
pub mod app;

// High-level runner orchestration (depends on ui + app, so also TUI-only)
#[cfg(feature = "tui")]
pub mod runner;
