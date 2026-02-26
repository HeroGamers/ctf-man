use crate::app::App;
use crate::db::models::Challenge;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};
use std::collections::HashMap;

/// Render the challenge list view for the active CTF
/// Challenges are grouped by category
pub fn render_challenge_list(frame: &mut Frame, app: &App, area: Rect) {
    // Create the outer layout with title, list, and help
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title area
            Constraint::Min(0),    // List area
            Constraint::Length(3), // Help text area
        ])
        .split(area);

    // Render title
    render_title(frame, app, chunks[0]);

    // Render the challenge list
    render_list(frame, app, chunks[1]);

    // Render help text
    render_help(frame, chunks[2]);
}

/// Render the title section with CTF name
fn render_title(frame: &mut Frame, app: &App, area: Rect) {
    let ctf_name = app
        .get_selected_ctf()
        .map(|ctf| ctf.name.as_str())
        .unwrap_or("Unknown CTF");

    let title = Paragraph::new(format!("Challenges - {}", ctf_name))
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );

    frame.render_widget(title, area);
}

/// Group challenges by category and maintain their original indices
fn group_challenges_by_category(
    challenges: &[Challenge],
) -> Vec<(String, Vec<(usize, &Challenge)>)> {
    let mut category_map: HashMap<String, Vec<(usize, &Challenge)>> = HashMap::new();

    for (index, challenge) in challenges.iter().enumerate() {
        let category = challenge
            .category
            .clone()
            .unwrap_or_else(|| "Uncategorized".to_string());

        category_map
            .entry(category)
            .or_insert_with(Vec::new)
            .push((index, challenge));
    }

    // Convert to sorted vector for consistent ordering
    let mut categories: Vec<_> = category_map.into_iter().collect();
    categories.sort_by(|a, b| a.0.cmp(&b.0));

    categories
}

/// Calculate the visual index (including headers and spacing) from flat challenge index
fn calculate_visual_index(
    grouped_challenges: &[(String, Vec<(usize, &Challenge)>)],
    flat_index: usize,
) -> Option<usize> {
    let mut visual_index = 0;

    for (_, challenges) in grouped_challenges {
        // Skip category header
        visual_index += 1;

        for (original_index, _) in challenges {
            if *original_index == flat_index {
                return Some(visual_index);
            }
            visual_index += 1;
        }

        // Skip spacing between categories
        visual_index += 1;
    }

    None
}

/// Render the list of challenges grouped by category
fn render_list(frame: &mut Frame, app: &App, area: Rect) {
    let grouped_challenges = group_challenges_by_category(&app.challenges);

    let mut items: Vec<ListItem> = Vec::new();

    for (category, challenges) in &grouped_challenges {
        // Add category header
        items.push(ListItem::new(Line::from(vec![Span::styled(
            format!("━━ {} ━━", category),
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )])));

        // Add challenges in this category
        for (original_index, challenge) in challenges {
            let is_selected = *original_index == app.challenge_selected_index;

            // Format: [✓] Challenge Name (100 pts)
            let solved_indicator = if challenge.solved { "✓" } else { " " };
            let points_text = challenge
                .points
                .map(|p| format!(" ({}pts)", p))
                .unwrap_or_default();

            let line_text = format!("[{}] {}{}", solved_indicator, challenge.name, points_text);

            // Create styled content
            let content = if is_selected {
                // Highlighted/selected style
                Line::from(vec![
                    Span::raw("  ▶ "),
                    Span::styled(
                        line_text,
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                ])
            } else {
                // Normal style with color based on solved status
                let style = if challenge.solved {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::White)
                };

                Line::from(vec![Span::raw("    "), Span::styled(line_text, style)])
            };

            items.push(ListItem::new(content));
        }

        // Add spacing between categories
        items.push(ListItem::new(Line::from("")));
    }

    // Show message if no challenges
    if app.challenges.is_empty() {
        items.push(ListItem::new(Line::from(Span::styled(
            "No challenges yet. Use 'ctf <category> <challenge>' to create one.",
            Style::default()
                .fg(Color::Gray)
                .add_modifier(Modifier::ITALIC),
        ))));
    }

    // Create the list widget
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Challenges (↑↓/kj to navigate, Enter to open, Esc to go back) ")
                .border_style(Style::default().fg(Color::White)),
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    // Calculate visual index for the selected challenge and create ListState
    let mut list_state = ListState::default();
    if !app.challenges.is_empty() {
        if let Some(visual_idx) =
            calculate_visual_index(&grouped_challenges, app.challenge_selected_index)
        {
            list_state.select(Some(visual_idx));
        }
    }

    frame.render_stateful_widget(list, area, &mut list_state);
}

/// Render the help text at the bottom
fn render_help(frame: &mut Frame, area: Rect) {
    let help_text = vec![
        Span::styled("Keys: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("↑/k "),
        Span::styled("up", Style::default().fg(Color::Gray)),
        Span::raw(" | ↓/j "),
        Span::styled("down", Style::default().fg(Color::Gray)),
        Span::raw(" | "),
        Span::styled("Enter", Style::default().fg(Color::Green)),
        Span::raw(" open | "),
        Span::styled("Space", Style::default().fg(Color::Cyan)),
        Span::raw(" toggle solved | "),
        Span::styled("Esc", Style::default().fg(Color::Red)),
        Span::raw(" back to CTF list | "),
        Span::styled("q", Style::default().fg(Color::Red)),
        Span::raw(" quit"),
    ];

    let help = Paragraph::new(Line::from(help_text))
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::White));

    frame.render_widget(help, area);
}
