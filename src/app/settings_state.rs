/// Which setting field is currently selected/being edited
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingField {
    DefaultCtfDirectory,
    UiTickRate,
}

/// State for the settings dialog
#[derive(Debug, Clone)]
pub struct SettingsState {
    /// Currently selected field
    pub selected_field: SettingField,

    /// Whether we're currently editing a field
    pub editing: bool,

    /// Input buffer for editing
    pub input_buffer: String,
}

impl SettingsState {
    pub fn new() -> Self {
        Self {
            selected_field: SettingField::DefaultCtfDirectory,
            editing: false,
            input_buffer: String::new(),
        }
    }

    /// Move to the next setting field
    pub fn next_field(&mut self) {
        if !self.editing {
            self.selected_field = match self.selected_field {
                SettingField::DefaultCtfDirectory => SettingField::UiTickRate,
                SettingField::UiTickRate => SettingField::DefaultCtfDirectory,
            };
        }
    }

    /// Move to the previous setting field
    pub fn previous_field(&mut self) {
        if !self.editing {
            self.selected_field = match self.selected_field {
                SettingField::DefaultCtfDirectory => SettingField::UiTickRate,
                SettingField::UiTickRate => SettingField::DefaultCtfDirectory,
            };
        }
    }

    /// Start editing the current field with initial value
    pub fn start_editing(&mut self, initial_value: String) {
        self.editing = true;
        self.input_buffer = initial_value;
    }

    /// Stop editing and return the current value
    pub fn finish_editing(&mut self) -> String {
        self.editing = false;
        self.input_buffer.clone()
    }

    /// Cancel editing
    pub fn cancel_editing(&mut self) {
        self.editing = false;
        self.input_buffer.clear();
    }

    /// Add a character to the input buffer
    pub fn push_char(&mut self, c: char) {
        if self.editing {
            self.input_buffer.push(c);
        }
    }

    /// Remove the last character from the input buffer
    pub fn pop_char(&mut self) {
        if self.editing {
            self.input_buffer.pop();
        }
    }
}

impl Default for SettingsState {
    fn default() -> Self {
        Self::new()
    }
}
