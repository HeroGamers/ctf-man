use crate::app::{SettingField, SettingsState};
use crate::config::Config;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

/// Render the settings dialog as a centered popup
pub fn render_settings_dialog(
    frame: &mut Frame,
    config: &Config,
    settings_state: &SettingsState,
    area: Rect,
) {
    // Create a centered popup area
    let popup_area = centered_rect(60, 60, area);

    // Clear the area behind the popup
    frame.render_widget(Clear, popup_area);

    // Build the settings content
    let mut lines = vec![
        Line::from(vec![Span::styled(
            "Settings (Use ↑/↓ to navigate, Enter to edit, Esc to save)",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
    ];

    // Helper to render a field with selection highlight
    let render_field = |field: SettingField, label: String, value: String| -> Vec<Line> {
        let is_selected = settings_state.selected_field == field;
        let is_editing = settings_state.editing && is_selected;

        // Display value - use input buffer if editing, otherwise use the actual value
        let display_value = if is_editing {
            format!("{}_", &settings_state.input_buffer) // Add cursor
        } else {
            value
        };

        let label_style = if is_selected {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().add_modifier(Modifier::BOLD)
        };

        let value_style = if is_editing {
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD)
        } else if is_selected {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let indicator = if is_selected { "▶ " } else { "  " };

        vec![
            Line::from(vec![
                Span::raw(indicator),
                Span::styled(label, label_style),
            ]),
            Line::from(vec![
                Span::raw("    "),
                Span::styled(display_value, value_style),
            ]),
            Line::from(""),
        ]
    };

    // Default CTF Directory
    let default_dir = config
        .default_ctf_directory
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| String::new());
    lines.extend(render_field(
        SettingField::DefaultCtfDirectory,
        "Default CTF Directory:".to_string(),
        default_dir,
    ));

    // UI Tick Rate
    let tick_rate = config.ui.tick_rate_ms.to_string();
    lines.extend(render_field(
        SettingField::UiTickRate,
        "UI Tick Rate (ms):".to_string(),
        tick_rate,
    ));

    // Divider
    lines.push(Line::from("─".repeat(50)));
    lines.push(Line::from(""));

    // Active CTF Info (read-only)
    let active_id = config
        .active_ctf_id
        .map(|id| id.to_string())
        .unwrap_or_else(|| "None".to_string());
    lines.push(Line::from(vec![
        Span::styled(
            "Active CTF ID: ",
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::styled(active_id, Style::default().fg(Color::Gray)),
    ]));

    let active_path = config
        .active_ctf_path
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "None".to_string());
    lines.push(Line::from(vec![
        Span::styled(
            "Active CTF Path: ",
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::styled(active_path, Style::default().fg(Color::Gray)),
    ]));

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" Settings ")
                .title_alignment(Alignment::Center),
        )
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(Color::White));

    frame.render_widget(paragraph, popup_area);
}

/// Helper function to create a centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
