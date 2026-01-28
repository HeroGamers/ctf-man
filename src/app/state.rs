use crate::app::{OnboardingState, SettingsState};
use crate::db::{models::{Ctf, Challenge}, Database};
use anyhow::Result;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;

/// Represents the different views/screens in the TUI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    /// Viewing the list of CTFs
    CtfList,
    /// Viewing the challenges for the active CTF
    ChallengeList,
}

/// Main application state.
/// This struct holds all the data needed to render the UI and respond to events.
pub struct App {
    /// Whether the application should quit
    pub should_quit: bool,

    /// Current view mode
    pub current_view: ViewMode,

    /// List of active/upcoming CTFs loaded from the database
    pub ctfs: Vec<Ctf>,

    /// List of archived CTFs (past CTFs)
    pub archived_ctfs: Vec<Ctf>,

    /// Whether the archive section is expanded
    pub archive_expanded: bool,

    /// Currently selected index in the CTF list (across both active and archived)
    pub selected_index: usize,

    /// List of challenges for the current CTF
    pub challenges: Vec<Challenge>,

    /// Currently selected index in the challenge list
    pub challenge_selected_index: usize,

    /// ID of the CTF whose challenges are currently being viewed (when in ChallengeList view)
    pub current_ctf_id: Option<i64>,

    /// Whether the settings dialog is open
    pub show_settings: bool,

    /// State for the settings dialog (which field is selected, editing, etc.)
    pub settings_state: SettingsState,

    /// State for the onboarding dialog (only present during first-time setup)
    pub onboarding_state: Option<OnboardingState>,

    /// Path to navigate to when exiting TUI (for shell integration to cd)
    pub navigation_path: Option<PathBuf>,

    /// Whether the app is currently fetching from CTFtime API
    pub is_fetching_api: bool,

    /// Error message from last API fetch (if any)
    pub api_error_message: Option<String>,

    /// Receiver for API fetch results (None if no fetch in progress)
    pub api_fetch_receiver: Option<Receiver<Result<usize>>>,

    // TODO: Add more state as you build features:
    // - filter_text: String (for search/filter)
    // - popup_state: Option<Popup> (for dialogs)
}

impl App {
    /// Create a new App with initial state
    pub fn new() -> Result<Self> {
        Ok(Self {
            should_quit: false,
            current_view: ViewMode::CtfList,
            ctfs: Vec::new(),
            archived_ctfs: Vec::new(),
            archive_expanded: false,
            selected_index: 0,
            challenges: Vec::new(),
            challenge_selected_index: 0,
            current_ctf_id: None,
            show_settings: false,
            settings_state: SettingsState::new(),
            onboarding_state: None,
            navigation_path: None,
            is_fetching_api: false,
            api_error_message: None,
            api_fetch_receiver: None,
        })
    }

    /// Load CTFs from the database (both active and archived)
    pub fn load_ctfs(&mut self, db: &Database) -> Result<()> {
        self.ctfs = db.get_active_ctfs()?;
        self.archived_ctfs = db.get_archived_ctfs()?;
        self.validate_selection_index();
        Ok(())
    }

    /// Load challenges for a specific CTF
    pub fn load_challenges(&mut self, db: &Database, ctf_id: i64) -> Result<()> {
        self.challenges = db.get_challenges_for_ctf(ctf_id)?;
        self.challenge_selected_index = 0;
        Ok(())
    }

    /// Switch to challenge view for the selected CTF
    pub fn enter_challenge_view(&mut self, db: &Database) -> Result<()> {
        if let Some(ctf) = self.get_selected_ctf() {
            if let Some(id) = ctf.id {
                self.load_challenges(db, id)?;
                self.current_ctf_id = Some(id); // Track which CTF we're viewing
                self.current_view = ViewMode::ChallengeList;
            }
        }
        Ok(())
    }

    /// Switch back to CTF list view
    pub fn back_to_ctf_list(&mut self) {
        self.current_view = ViewMode::CtfList;
        self.challenges.clear();
        self.challenge_selected_index = 0;
        self.current_ctf_id = None; // Clear tracked CTF when leaving challenge view
    }

    /// Move selection up in the current list
    pub fn select_previous(&mut self) {
        match self.current_view {
            ViewMode::CtfList => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                    // Skip the archive header if we land on it
                    if self.is_archive_header_index(self.selected_index) && self.selected_index > 0 {
                        self.selected_index -= 1;
                    }
                }
            }
            ViewMode::ChallengeList => {
                if self.challenge_selected_index > 0 {
                    self.challenge_selected_index -= 1;
                }
            }
        }
    }

    /// Move selection down in the current list
    pub fn select_next(&mut self) {
        match self.current_view {
            ViewMode::CtfList => {
                let max_index = self.get_max_ctf_index();
                if self.selected_index < max_index {
                    self.selected_index += 1;
                    // If we land on the archive header
                    if self.is_archive_header_index(self.selected_index) {
                        if !self.archive_expanded {
                            // Auto-expand the archive when navigating to it
                            self.archive_expanded = true;
                        }
                        // Skip the header and move to first archived CTF
                        // Recalculate max_index after potential expansion
                        let new_max_index = self.get_max_ctf_index();
                        if self.selected_index < new_max_index {
                            self.selected_index += 1;
                        }
                    }
                }
            }
            ViewMode::ChallengeList => {
                if self.challenge_selected_index < self.challenges.len().saturating_sub(1) {
                    self.challenge_selected_index += 1;
                }
            }
        }
    }

    /// Get the currently selected CTF (handles both active and archived CTFs)
    pub fn get_selected_ctf(&self) -> Option<&Ctf> {
        if self.selected_index < self.ctfs.len() {
            // Selection is in active CTFs
            self.ctfs.get(self.selected_index)
        } else if !self.archived_ctfs.is_empty() && self.archive_expanded {
            // Selection is in archived CTFs (only accessible when expanded)
            // Account for archive header (1 item)
            let archive_offset = self.ctfs.len() + 1;
            let archived_index = self.selected_index.saturating_sub(archive_offset);
            self.archived_ctfs.get(archived_index)
        } else {
            None
        }
    }

    /// Get the currently selected challenge
    pub fn get_selected_challenge(&self) -> Option<&Challenge> {
        self.challenges.get(self.challenge_selected_index)
    }

    /// Get the CTF whose challenges are currently being viewed
    /// This is reliable when in ChallengeList view, as it uses the tracked CTF ID
    pub fn get_current_ctf(&self) -> Option<&Ctf> {
        let ctf_id = self.current_ctf_id?;

        // Search in active CTFs first
        if let Some(ctf) = self.ctfs.iter().find(|c| c.id == Some(ctf_id)) {
            return Some(ctf);
        }

        // Search in archived CTFs
        self.archived_ctfs.iter().find(|c| c.id == Some(ctf_id))
    }

    /// Start API refresh (set loading state and receiver)
    pub fn start_api_refresh(&mut self, receiver: Receiver<Result<usize>>) {
        self.is_fetching_api = true;
        self.api_error_message = None;
        self.api_fetch_receiver = Some(receiver);
    }

    /// Check for API fetch completion (non-blocking)
    /// Returns Some(result) if fetch is complete, None if still in progress
    pub fn check_api_fetch_complete(&mut self) -> Option<Result<usize>> {
        if let Some(ref receiver) = self.api_fetch_receiver {
            // Try to receive without blocking
            match receiver.try_recv() {
                Ok(result) => {
                    // Fetch completed, clear receiver
                    self.api_fetch_receiver = None;
                    self.is_fetching_api = false;
                    Some(result)
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    // Still waiting for result
                    None
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    // Thread panicked or channel closed unexpectedly
                    self.api_fetch_receiver = None;
                    self.is_fetching_api = false;
                    Some(Err(anyhow::anyhow!("API fetch thread disconnected unexpectedly")))
                }
            }
        } else {
            None
        }
    }

    /// Complete API refresh with success
    pub fn complete_api_refresh_success(&mut self) {
        self.is_fetching_api = false;
        self.api_error_message = None;
        self.api_fetch_receiver = None;
    }

    /// Complete API refresh with error
    pub fn complete_api_refresh_error(&mut self, error: String) {
        self.is_fetching_api = false;
        self.api_error_message = Some(error);
        self.api_fetch_receiver = None;
    }

    /// Toggle the archive section expanded/collapsed
    pub fn toggle_archive(&mut self) {
        self.archive_expanded = !self.archive_expanded;
        // Validate selection after toggling to ensure it's still valid
        self.validate_selection_index();
    }

    /// Validate and fix the selection index to ensure it's within bounds
    /// This should be called after loading data or changing view state
    fn validate_selection_index(&mut self) {
        let max_index = self.get_max_ctf_index();
        if self.selected_index > max_index {
            self.selected_index = if max_index == 0 && self.ctfs.is_empty() {
                0
            } else {
                max_index
            };
        }
        // Ensure we're not on the archive header
        if self.is_archive_header_index(self.selected_index) {
            // Move to previous item if possible, otherwise next
            if self.selected_index > 0 {
                self.selected_index -= 1;
            } else if self.selected_index < max_index {
                self.selected_index += 1;
            }
        }
    }

    /// Check if the given index is the archive header
    fn is_archive_header_index(&self, index: usize) -> bool {
        !self.archived_ctfs.is_empty() && index == self.ctfs.len()
    }

    /// Get the maximum valid index for CTF selection
    fn get_max_ctf_index(&self) -> usize {
        let active_count = self.ctfs.len();
        if self.archived_ctfs.is_empty() {
            // Only active CTFs, no archive section
            active_count.saturating_sub(1)
        } else if self.archive_expanded {
            // Active CTFs + archive header (not selectable) + archived CTFs
            // Max index is: active_count + 1 (header) + archived_count - 1
            active_count + self.archived_ctfs.len()
        } else {
            // Archive collapsed but header is reachable for auto-expand
            // Allow navigation TO the header position so select_next can expand it
            active_count
        }
    }

    /// Check if currently selected item is the archive header
    pub fn is_on_archive_header(&self) -> bool {
        self.is_archive_header_index(self.selected_index)
    }

    // TODO: Add more methods as you build features:
    // - create_ctf(&mut self, name: String) -> Result<()>
    // - delete_ctf(&mut self, id: i64) -> Result<()>
    // - toggle_solved(&mut self, challenge_id: i64) -> Result<()>
}

impl Default for App {
    fn default() -> Self {
        Self::new().expect("Failed to create default App")
    }
}
