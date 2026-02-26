use crate::app::Action;
use anyhow::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyEvent, KeyEventKind};
use std::time::Duration;

/// Convert crossterm keyboard events to application actions
/// This is where you map keys to actions in your application
///
/// The `in_edit_mode` parameter changes behavior for text input
pub fn handle_key_event(key: KeyEvent, in_edit_mode: bool) -> Option<Action> {
    // Only handle key press events, ignore key release and repeat
    if key.kind != KeyEventKind::Press {
        return None;
    }

    // If in edit mode, handle text input
    if in_edit_mode {
        return match key.code {
            KeyCode::Char(c) => Some(Action::InputChar(c)),
            KeyCode::Backspace => Some(Action::DeleteChar),
            KeyCode::Enter => Some(Action::SaveSettings),
            KeyCode::Esc => Some(Action::Back),
            KeyCode::Left => Some(Action::CursorLeft),
            KeyCode::Right => Some(Action::CursorRight),
            KeyCode::Home => Some(Action::Home),
            KeyCode::End => Some(Action::End),
            _ => None,
        };
    }

    // Normal mode key handling
    match key.code {
        // Quit the application
        KeyCode::Char('q') | KeyCode::Char('Q') => Some(Action::Quit),

        // Navigation
        KeyCode::Up | KeyCode::Char('k') => Some(Action::SelectPrevious),
        KeyCode::Down | KeyCode::Char('j') => Some(Action::SelectNext),
        KeyCode::Right | KeyCode::Char('l') => Some(Action::OpenChallengeView),
        KeyCode::Left | KeyCode::Char('h') => Some(Action::Back),
        KeyCode::Enter => Some(Action::Enter),
        KeyCode::Char(' ') => Some(Action::ToggleSolved),
        KeyCode::Esc | KeyCode::Backspace => Some(Action::Back),

        // Refresh from API (force fetch from CTFtime)
        KeyCode::Char('r') | KeyCode::F(5) => Some(Action::RefreshFromAPI),

        // Settings
        KeyCode::Char('s') | KeyCode::Char('S') => Some(Action::OpenSettings),

        // Toggle archive
        KeyCode::Char('a') | KeyCode::Char('A') => Some(Action::ToggleArchive),

        // TODO: Add more key mappings:
        // - Char('n') -> Action::NewCTF
        // - Char('d') -> Action::Delete
        // - Char('/') -> Action::StartSearch
        // - Char('?') -> Action::ShowHelp
        // - Tab -> Action::NextTab
        // - BackTab -> Action::PreviousTab
        _ => None,
    }
}

/// Poll for terminal events with a timeout
/// Returns Some(action) if an event was received and mapped to an action
/// Returns None if timeout elapsed or event didn't map to an action
pub fn poll_event(timeout: Duration, in_edit_mode: bool) -> Result<Option<Action>> {
    if event::poll(timeout)? {
        match event::read()? {
            CrosstermEvent::Key(key) => Ok(handle_key_event(key, in_edit_mode)),
            // TODO: Handle other event types:
            // CrosstermEvent::Mouse(mouse) => { ... }
            // CrosstermEvent::Resize(width, height) => { ... }
            _ => Ok(None),
        }
    } else {
        Ok(None)
    }
}

// Alternative async event loop pattern using tokio
// Uncomment this if you want to use an async approach instead

// use tokio::sync::mpsc;
// use futures::StreamExt;
// use crossterm::event::EventStream;
//
// pub async fn event_loop(action_tx: mpsc::UnboundedSender<Action>) -> Result<()> {
//     let mut event_stream = EventStream::new();
//     let mut tick_interval = tokio::time::interval(Duration::from_millis(250));
//
//     loop {
//         tokio::select! {
//             maybe_event = event_stream.next() => {
//                 match maybe_event {
//                     Some(Ok(CrosstermEvent::Key(key))) => {
//                         if let Some(action) = handle_key_event(key) {
//                             action_tx.send(action)?;
//                         }
//                     }
//                     Some(Err(e)) => return Err(e.into()),
//                     _ => {}
//                 }
//             }
//             _ = tick_interval.tick() => {
//                 action_tx.send(Action::Tick)?;
//             }
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn test_quit_keys() {
        let q_event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        assert_eq!(handle_key_event(q_event, false), Some(Action::Quit));
    }

    #[test]
    fn test_back_keys() {
        let esc_event = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        assert_eq!(handle_key_event(esc_event, false), Some(Action::Back));

        let backspace_event = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
        assert_eq!(handle_key_event(backspace_event, false), Some(Action::Back));
    }

    #[test]
    fn test_navigation_keys() {
        let up_event = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        assert_eq!(
            handle_key_event(up_event, false),
            Some(Action::SelectPrevious)
        );

        let down_event = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        assert_eq!(
            handle_key_event(down_event, false),
            Some(Action::SelectNext)
        );

        let enter_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert_eq!(handle_key_event(enter_event, false), Some(Action::Enter));
    }

    #[test]
    fn test_edit_mode_input() {
        let char_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        assert_eq!(
            handle_key_event(char_event, true),
            Some(Action::InputChar('a'))
        );

        let backspace_event = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
        assert_eq!(
            handle_key_event(backspace_event, true),
            Some(Action::DeleteChar)
        );
    }

    // TODO: Add more tests for other key combinations
}
