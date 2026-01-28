/// State for the onboarding dialog
#[derive(Debug, Clone)]
pub struct OnboardingState {
    /// Input buffer for the CTF directory path
    pub input_buffer: String,

    /// Cursor position in the input buffer
    pub cursor_position: usize,

    /// Validation error message (if any)
    pub error_message: Option<String>,

    /// Whether we're currently validating (loading state)
    pub validating: bool,
}

impl OnboardingState {
    /// Create a new onboarding state with the default path
    pub fn new() -> Self {
        let default_path = "~/ctf".to_string();
        let cursor_pos = default_path.len();

        Self {
            input_buffer: default_path,
            cursor_position: cursor_pos,
            error_message: None,
            validating: false,
        }
    }

    /// Add a character at the cursor position
    pub fn insert_char(&mut self, c: char) {
        self.input_buffer.insert(self.cursor_position, c);
        self.cursor_position += 1;
        // Clear error when user starts typing
        self.error_message = None;
    }

    /// Remove the character before the cursor (backspace)
    pub fn delete_char(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.input_buffer.remove(self.cursor_position);
            // Clear error when user starts typing
            self.error_message = None;
        }
    }

    /// Move cursor left
    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    /// Move cursor right
    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.input_buffer.len() {
            self.cursor_position += 1;
        }
    }

    /// Move cursor to the beginning
    pub fn move_cursor_home(&mut self) {
        self.cursor_position = 0;
    }

    /// Move cursor to the end
    pub fn move_cursor_end(&mut self) {
        self.cursor_position = self.input_buffer.len();
    }

    /// Get the current input value
    pub fn get_input(&self) -> &str {
        &self.input_buffer
    }

    /// Set an error message
    pub fn set_error(&mut self, message: String) {
        self.error_message = Some(message);
        self.validating = false;
    }

    /// Start validation (show loading state)
    pub fn start_validation(&mut self) {
        self.validating = true;
        self.error_message = None;
    }

    /// Check if the path is empty
    pub fn is_empty(&self) -> bool {
        self.input_buffer.trim().is_empty()
    }
}

impl Default for OnboardingState {
    fn default() -> Self {
        Self::new()
    }
}
