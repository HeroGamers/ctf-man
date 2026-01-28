use anyhow::{Context, Result};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;

/// Type alias for the terminal backend we're using
pub type Tui = Terminal<CrosstermBackend<io::Stdout>>;

/// Setup the terminal for TUI mode
/// This enables raw mode and switches to the alternate screen
pub fn setup_terminal() -> Result<Tui> {
    // Enable raw mode (disables line buffering, echo, etc.)
    enable_raw_mode()
        .context("Failed to enable raw mode")?;

    // Switch to alternate screen (preserves user's terminal state)
    execute!(io::stdout(), EnterAlternateScreen)
        .context("Failed to enter alternate screen")?;

    // Create the terminal backend
    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)
        .context("Failed to create terminal")?;

    Ok(terminal)
}

/// Restore the terminal to its original state
/// Call this before exiting the application or when switching out of TUI mode
pub fn restore_terminal() -> Result<()> {
    // Disable raw mode
    disable_raw_mode()
        .context("Failed to disable raw mode")?;

    // Leave alternate screen
    execute!(io::stdout(), LeaveAlternateScreen)
        .context("Failed to leave alternate screen")?;

    Ok(())
}

// The terminal setup/restore pattern is important:
// - Always call restore_terminal() before exiting
// - Use a panic hook to ensure restore_terminal() is called even on panic
// - See main.rs for an example of setting up the panic hook
