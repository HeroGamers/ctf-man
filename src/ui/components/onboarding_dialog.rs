use crate::app::OnboardingState;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

/// Render the onboarding dialog as a centered popup
pub fn render_onboarding_dialog(frame: &mut Frame, state: &OnboardingState, area: Rect) {
    // Create a centered popup area
    let popup_area = centered_rect(70, 50, area);

    // Clear the area behind the popup
    frame.render_widget(Clear, popup_area);

    // Build the onboarding content
    let mut lines = vec![
        Line::from(vec![Span::styled(
            "Welcome to CTF Manager!",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Let's set up your CTF workspace.",
            Style::default().fg(Color::White),
        )]),
        Line::from(""),
        Line::from(vec![Span::raw(
            "This directory will store all your CTF challenges and templates.",
        )]),
        Line::from(""),
        Line::from(""),
    ];

    // Input field label
    lines.push(Line::from(vec![Span::styled(
        "CTF Workspace Directory:",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )]));

    // Input field with cursor
    let input_display = if state.validating {
        format!("{}  [Validating...]", state.input_buffer)
    } else {
        // Show cursor at the current position
        let mut display = state.input_buffer.clone();
        display.insert(state.cursor_position, '|');
        display
    };

    let input_style = if state.error_message.is_some() {
        Style::default()
            .fg(Color::Red)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD)
    };

    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled(input_display, input_style),
    ]));
    lines.push(Line::from(""));

    // Error message (if any)
    if let Some(ref error) = state.error_message {
        lines.push(Line::from(vec![Span::styled(
            format!("✗ {}", error),
            Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(""));
    }

    // Instructions
    lines.push(Line::from(""));
    lines.push(Line::from("─".repeat(50)));
    lines.push(Line::from(""));

    lines.push(Line::from(vec![
        Span::styled("Enter", Style::default().fg(Color::Green)),
        Span::raw(" to confirm  "),
        Span::styled("←/→", Style::default().fg(Color::Yellow)),
        Span::raw(" to move cursor  "),
        Span::styled("Home/End", Style::default().fg(Color::Yellow)),
        Span::raw(" to jump"),
    ]));

    lines.push(Line::from(vec![
        Span::styled("Ctrl+C", Style::default().fg(Color::Red)),
        Span::raw(" to exit setup"),
    ]));

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" Initial Setup ")
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
