pub mod challenge_list;
pub mod ctf_list;
pub mod onboarding_dialog;
pub mod settings_dialog;

pub use challenge_list::render_challenge_list;
pub use ctf_list::render_ctf_list;
pub use onboarding_dialog::render_onboarding_dialog;
pub use settings_dialog::render_settings_dialog;

// TODO: Add more components as you build the UI:
// - help_popup.rs - Show keyboard shortcuts
// - input_dialog.rs - Get text input from user
// - details_panel.rs - Show detailed information
// - status_bar.rs - Status bar at the bottom
