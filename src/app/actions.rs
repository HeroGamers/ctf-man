/// Actions represent events that can occur in the application.
/// This pattern decouples event handling from state updates.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// Quit the application
    Quit,

    /// Move selection up in the current list
    SelectPrevious,

    /// Move selection down in the current list
    SelectNext,

    /// Confirm/enter the current selection
    Enter,

    /// Go back to previous screen
    Back,

    /// Open challenge view for the selected CTF
    OpenChallengeView,

    /// Refresh data from CTFtime API (force fetch)
    RefreshFromAPI,

    /// Open settings dialog
    OpenSettings,

    /// Input character for editing
    InputChar(char),

    /// Delete last character in input
    DeleteChar,

    /// Save settings changes
    SaveSettings,

    /// Toggle archive section expanded/collapsed
    ToggleArchive,

    /// Move cursor left (for text input)
    CursorLeft,

    /// Move cursor right (for text input)
    CursorRight,

    /// Move cursor to start (for text input)
    Home,

    /// Move cursor to end (for text input)
    End,

    /// Toggle the solved status of the currently selected challenge
    ToggleSolved,

    // TODO: Add more actions as you build features:
    // - SwitchScreen(Screen) - Navigate between different views
    // - OpenChallenge(usize) - Open a specific challenge
    // - CreateCTF(String) - Create a new CTF
    // - DeleteCTF(usize) - Delete a CTF
    // - FilterByCategory(String) - Filter challenges
}
