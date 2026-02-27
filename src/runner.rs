use anyhow::{Context, Result};
use std::fs as std_fs;
use std::io::IsTerminal;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crate::app::{Action, App, OnboardingState, ViewMode};
use crate::config::{Config, expand_tilde, get_database_path, setup_initial_directory};
use crate::db::{Database, init_database};
use crate::fs::sync_filesystem_to_database;
use crate::templates;
use crate::ui::{
    components::{
        render_challenge_list, render_ctf_list, render_onboarding_dialog, render_settings_dialog,
    },
    event::poll_event,
    restore_terminal, setup_terminal,
};

/// Run the TUI to select active CTF
pub fn run_tui(config: &mut Config) -> Result<()> {
    // Check if stdout is a terminal (TTY)
    // This prevents the TUI from trying to render when output is redirected/captured
    if !std::io::stdout().is_terminal() {
        eprintln!("Error: Cannot launch TUI - stdout is not a terminal.");
        eprintln!("This usually happens when output is being captured or redirected.");
        eprintln!("Please run the binary directly without output redirection.");
        std::process::exit(1);
    }

    // Make config mutable for the entire function
    // Initialize database
    let db_path = get_database_path()?;
    let conn = init_database(&db_path)?;
    let db = Database::new(conn);

    // Sync filesystem to database if enabled
    if config.sync.auto_sync_on_startup {
        let _ = perform_sync(&db, config);
    }

    // Check if onboarding is needed (first-time setup)
    let needs_onboarding = config.default_ctf_directory.is_none();

    // Create app state
    let mut app = App::new()?;

    // If onboarding is needed, initialize onboarding state and skip normal setup
    if needs_onboarding {
        app.onboarding_state = Some(OnboardingState::new());
    } else {
        // Check if we should auto-refresh CTFtime data (12+ hours since last fetch)
        if let Ok(should_refresh) = db.should_refresh_ctftime() {
            if should_refresh {
                let _ = db.fetch_ctftime_with_cache();
            }
        }

        app.load_ctfs(&db)?;

        // If database is empty, force a fetch from CTFtime
        if app.ctfs.is_empty() && app.archived_ctfs.is_empty() {
            if db.fetch_ctftime_with_cache().is_ok() {
                app.load_ctfs(&db)?;
            }
        }

        // If still empty after fetch attempt, exit
        if app.ctfs.is_empty() && app.archived_ctfs.is_empty() {
            return Ok(());
        }
    }

    // Setup terminal
    let mut terminal = setup_terminal()?;

    // Main event loop
    let tick_duration = Duration::from_millis(config.ui.tick_rate_ms);
    loop {
        // Draw the UI
        terminal.draw(|frame| {
            let area = frame.area();

            // If onboarding is active, only render onboarding dialog
            if let Some(ref onboarding_state) = app.onboarding_state {
                render_onboarding_dialog(frame, onboarding_state, area);
            } else {
                // Render the appropriate screen based on view mode
                match app.current_view {
                    ViewMode::CtfList => render_ctf_list(frame, &app, config.active_ctf_id, area),
                    ViewMode::ChallengeList => render_challenge_list(frame, &app, area),
                }

                // Render settings dialog on top if open
                if app.show_settings {
                    render_settings_dialog(frame, config, &app.settings_state, area);
                }
            }
        })?;

        // Handle events
        let in_edit_mode =
            app.show_settings && app.settings_state.editing || app.onboarding_state.is_some();
        if let Some(action) = poll_event(tick_duration, in_edit_mode)? {
            // If onboarding is active, handle onboarding events
            if app.onboarding_state.is_some() {
                match action {
                    Action::Quit => {
                        // Allow Ctrl+C to exit during onboarding
                        app.should_quit = true;
                    }
                    Action::InputChar(c) => {
                        if let Some(ref mut state) = app.onboarding_state {
                            state.insert_char(c);
                        }
                    }
                    Action::DeleteChar => {
                        if let Some(ref mut state) = app.onboarding_state {
                            state.delete_char();
                        }
                    }
                    Action::CursorLeft => {
                        if let Some(ref mut state) = app.onboarding_state {
                            state.move_cursor_left();
                        }
                    }
                    Action::CursorRight => {
                        if let Some(ref mut state) = app.onboarding_state {
                            state.move_cursor_right();
                        }
                    }
                    Action::Home => {
                        if let Some(ref mut state) = app.onboarding_state {
                            state.move_cursor_home();
                        }
                    }
                    Action::End => {
                        if let Some(ref mut state) = app.onboarding_state {
                            state.move_cursor_end();
                        }
                    }
                    Action::Enter | Action::SaveSettings => {
                        // Validate and save the directory path
                        if let Some(ref mut state) = app.onboarding_state {
                            if state.is_empty() {
                                state.set_error("Path cannot be empty".to_string());
                            } else {
                                state.start_validation();

                                // Expand tilde and validate path
                                let input = state.get_input();
                                match expand_tilde(input) {
                                    Ok(path) => {
                                        match setup_initial_directory(&path) {
                                            Ok(_) => {
                                                // Success! Save to config
                                                config.default_ctf_directory = Some(path);
                                                if let Err(e) = config.save() {
                                                    state.set_error(format!(
                                                        "Failed to save config: {}",
                                                        e
                                                    ));
                                                } else {
                                                    // Onboarding complete!
                                                    app.onboarding_state = None;

                                                    // Load CTFs now that setup is complete
                                                    if let Err(e) = app.load_ctfs(&db) {
                                                        eprintln!("Failed to load CTFs: {}", e);
                                                    }

                                                    // If database is empty, fetch from CTFtime
                                                    if app.ctfs.is_empty()
                                                        && app.archived_ctfs.is_empty()
                                                    {
                                                        if db.fetch_ctftime_with_cache().is_ok() {
                                                            let _ = app.load_ctfs(&db);
                                                        }
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                state.set_error(format!(
                                                    "Failed to create directory: {}",
                                                    e
                                                ));
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        state.set_error(format!("Invalid path: {}", e));
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            } else if app.show_settings {
                // If settings dialog is open, handle differently
                match action {
                    Action::Quit | Action::Back => {
                        if app.settings_state.editing {
                            // Cancel editing
                            app.settings_state.cancel_editing();
                        } else {
                            // Close settings dialog
                            app.show_settings = false;
                        }
                    }
                    Action::OpenSettings => {
                        app.show_settings = false;
                    }
                    Action::SelectPrevious => {
                        app.settings_state.previous_field();
                    }
                    Action::SelectNext => {
                        app.settings_state.next_field();
                    }
                    Action::Enter => {
                        // Start editing the selected field
                        use crate::app::SettingField;
                        let initial_value = match app.settings_state.selected_field {
                            SettingField::DefaultCtfDirectory => config
                                .default_ctf_directory
                                .as_ref()
                                .map(|p| p.display().to_string())
                                .unwrap_or_default(),
                            SettingField::UiTickRate => config.ui.tick_rate_ms.to_string(),
                        };
                        app.settings_state.start_editing(initial_value);
                    }
                    Action::InputChar(c) => {
                        app.settings_state.push_char(c);
                    }
                    Action::DeleteChar => {
                        app.settings_state.pop_char();
                    }
                    Action::SaveSettings => {
                        // Save the edited value to config
                        use crate::app::SettingField;
                        let value = app.settings_state.finish_editing();

                        match app.settings_state.selected_field {
                            SettingField::DefaultCtfDirectory => {
                                if value.is_empty() {
                                    config.default_ctf_directory = None;
                                } else {
                                    config.default_ctf_directory =
                                        Some(std::path::PathBuf::from(value));
                                }
                            }
                            SettingField::UiTickRate => {
                                if let Ok(rate) = value.parse::<u64>() {
                                    config.ui.tick_rate_ms = rate;
                                }
                            }
                        }

                        // Save to file
                        config.save().context("Failed to save configuration")?;
                    }
                    _ => {}
                }
            } else {
                // Handle actions based on current view
                match app.current_view {
                    ViewMode::CtfList => {
                        match action {
                            Action::Quit | Action::Back => {
                                app.should_quit = true;
                            }
                            Action::SelectPrevious => app.select_previous(),
                            Action::SelectNext => app.select_next(),
                            Action::Enter => {
                                // Check if we're on the archive header
                                if app.is_on_archive_header() {
                                    app.toggle_archive();
                                } else if let Some(ctf) = app.get_selected_ctf() {
                                    // Always set as active and exit
                                    if let Some(id) = ctf.id {
                                        let path = ctf.get_directory(config);

                                        // Create the CTF directory if it doesn't exist
                                        std_fs::create_dir_all(&path)
                                            .context("Failed to create CTF directory")?;

                                        config.set_active_ctf(id, path.clone())?;

                                        // Show confirmation and exit
                                        restore_terminal()?;
                                        println!("Active CTF set to: {}", ctf.name);
                                        println!("Directory: {:?}", path);
                                        return Ok(());
                                    }
                                }
                            }
                            Action::OpenChallengeView => {
                                // Open challenge view for the selected CTF (without setting it as active)
                                if !app.is_on_archive_header() {
                                    if let Some(_ctf) = app.get_selected_ctf() {
                                        app.enter_challenge_view(&db)?;
                                    }
                                }
                            }
                            Action::RefreshFromAPI => {
                                // Spawn background thread for API fetch
                                let (tx, rx) = mpsc::channel();
                                let db_path_clone = db_path.clone();

                                thread::spawn(move || {
                                    // Create a new database connection in this thread
                                    let result = (|| -> Result<usize> {
                                        let conn =
                                            crate::db::schema::init_database(&db_path_clone)?;
                                        let db_thread = Database::new(conn);
                                        db_thread.fetch_ctftime_with_cache()
                                    })();

                                    // Send result back to main thread
                                    let _ = tx.send(result);
                                });

                                // Start loading state
                                app.start_api_refresh(rx);
                            }
                            Action::OpenSettings => {
                                app.show_settings = true;
                            }
                            Action::ToggleArchive => {
                                app.toggle_archive();
                            }
                            _ => {}
                        }
                    }
                    ViewMode::ChallengeList => {
                        match action {
                            Action::Quit | Action::Back => {
                                // Go back to CTF list
                                app.back_to_ctf_list();
                            }
                            Action::SelectPrevious => app.select_previous(),
                            Action::SelectNext => app.select_next(),
                            Action::Enter => {
                                // Navigate to challenge directory
                                if let Some(challenge) = app.get_selected_challenge() {
                                    // Use get_current_ctf() which relies on the tracked CTF ID
                                    // instead of get_selected_ctf() which uses selected_index
                                    if let Some(ctf) = app.get_current_ctf() {
                                        let ctf_path = ctf.get_directory(config);
                                        let category = challenge
                                            .category
                                            .as_ref()
                                            .expect("Challenge must have a category");
                                        let safe_category =
                                            crate::fs::sanitize_challenge_name(category);
                                        let safe_name =
                                            crate::fs::sanitize_challenge_name(&challenge.name);
                                        let challenge_path =
                                            ctf_path.join(&safe_category).join(&safe_name);

                                        // Store path for navigation and exit
                                        app.navigation_path = Some(challenge_path);
                                        app.should_quit = true;
                                    } else {
                                        // This shouldn't happen, but log an error if it does
                                        eprintln!("Error: No CTF found for current challenge view");
                                    }
                                }
                            }
                            Action::ToggleSolved => {
                                if let Some(challenge) = app.get_selected_challenge() {
                                    if let Some(id) = challenge.id {
                                        if db.toggle_solved(id).is_ok() {
                                            if let Some(ctf_id) = app.current_ctf_id {
                                                let _ = app.load_challenges(&db, ctf_id);
                                            }
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        // Check for API fetch completion (non-blocking)
        if let Some(result) = app.check_api_fetch_complete() {
            match result {
                Ok(_count) => {
                    app.complete_api_refresh_success();
                    if let Err(e) = app.load_ctfs(&db) {
                        let error_msg = format!("Failed to reload CTFs: {}", e);
                        app.complete_api_refresh_error(error_msg);
                    }
                    // Don't print to stdout as it interferes with TUI
                    // Just update the app state
                }
                Err(e) => {
                    let error_msg = format!("Failed to fetch from CTFtime: {}", e);
                    app.complete_api_refresh_error(error_msg);
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    restore_terminal()?;

    // If navigation was requested, write to temp file for shell integration
    if let Some(path) = app.navigation_path {
        let temp_file = std::env::temp_dir().join("ctf-man.path");
        if let Err(e) = std_fs::write(&temp_file, path.to_string_lossy().as_bytes()) {
            eprintln!("Warning: Failed to write navigation file: {}", e);
        }
    }

    Ok(())
}

/// Perform filesystem-to-database sync with optional confirmation
pub fn perform_sync(db: &Database, config: &Config) -> Result<crate::fs::SyncResult> {
    // If confirmation is disabled, just do the sync
    if !config.sync.confirm_before_sync_delete {
        return sync_filesystem_to_database(db, config, true);
    }

    // First, do a dry run to see what would be deleted
    let dry_run_result = sync_filesystem_to_database(db, config, false)?;

    // If nothing to delete, return early
    if dry_run_result.is_empty() {
        return Ok(dry_run_result);
    }

    // Show what would be deleted
    println!("\n⚠️  The following challenges will be removed from the database:");
    println!("(Their directories were not found on the filesystem)\n");

    for (ctf_name, challenge_name) in &dry_run_result.challenges_removed {
        println!("  - {} / {}", ctf_name, challenge_name);
    }
    println!();

    // Prompt for confirmation
    print!("Proceed with sync? [y/N]: ");
    use std::io::{self, Write};
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim().to_lowercase();

    if input == "y" || input == "yes" {
        // User confirmed, perform the actual sync
        sync_filesystem_to_database(db, config, true)
    } else {
        println!("Sync cancelled.");
        // Return empty result
        Ok(crate::fs::SyncResult::default())
    }
}

/// Create a challenge folder in the active CTF
pub fn create_challenge(config: &Config, category: &str, challenge_name: &str) -> Result<()> {
    // Get active CTF ID and path
    let ctf_id = config
        .active_ctf_id
        .context("No active CTF. Run 'ctf' to select one")?;

    let ctf_path = config
        .get_active_ctf_path()
        .context("No active CTF. Run 'ctf' to select one")?;

    // Create: ctf-folder/category/challenge-name/
    let safe_category = crate::fs::sanitize_challenge_name(category);
    let safe_name = crate::fs::sanitize_challenge_name(challenge_name);
    let challenge_path = ctf_path.join(&safe_category).join(&safe_name);

    std_fs::create_dir_all(&challenge_path)
        .context(format!("Failed to create directory: {:?}", challenge_path))?;

    // Copy template files if they exist
    if let Some(ref base_ctf_dir) = config.default_ctf_directory {
        let template_path = templates::get_template_path(base_ctf_dir, category);
        templates::copy_template_if_exists(&template_path, &challenge_path)?;
    }

    // Save to database
    let db_path = get_database_path()?;
    let conn = init_database(&db_path)?;
    let db = Database::new(conn);

    let challenge = crate::db::models::Challenge {
        id: None,
        ctf_id,
        name: challenge_name.to_string(),
        category: Some(category.to_string()),
        points: None,
        flag: None,
        solved: false,
        notes: None,
        created_at: String::new(), // Will be set by database
    };

    db.insert_challenge(&challenge)?;

    // Write to temp file for shell integration to cd
    let temp_file = std::env::temp_dir().join("ctf-man.path");
    let _ = std_fs::write(&temp_file, challenge_path.to_string_lossy().as_bytes());

    Ok(())
}
