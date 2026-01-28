use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

/// Render the main CTF list view
/// This function demonstrates how to:
/// - Use ratatui widgets (List, Paragraph, Block)
/// - Apply styles and colors
/// - Handle selection state
/// - Create a bordered layout
pub fn render_ctf_list(frame: &mut Frame, app: &App, active_ctf_id: Option<i64>, area: Rect) {
    // Determine if we need a status bar (loading or error)
    let needs_status_bar = app.is_fetching_api || app.api_error_message.is_some();

    // Create the outer layout with title, optional status, list, settings button, and help
    let mut constraints = vec![Constraint::Length(3)]; // Title area

    if needs_status_bar {
        constraints.push(Constraint::Length(3)); // Status area (loading/error)
    }

    constraints.extend_from_slice(&[
        Constraint::Min(0),    // List area
        Constraint::Length(3), // Settings button area
        Constraint::Length(3), // Help text area
    ]);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    let mut chunk_index = 0;

    // Render title
    render_title(frame, chunks[chunk_index]);
    chunk_index += 1;

    // Render status (loading or error) if needed
    if needs_status_bar {
        render_status(frame, app, chunks[chunk_index]);
        chunk_index += 1;
    }

    // Render the CTF list
    render_list(frame, app, active_ctf_id, chunks[chunk_index]);
    chunk_index += 1;

    // Render settings button
    render_settings_button(frame, chunks[chunk_index]);
    chunk_index += 1;

    // Render help text
    render_help(frame, chunks[chunk_index]);
}

/// Render the title section
fn render_title(frame: &mut Frame, area: Rect) {
    let title = Paragraph::new("CTF Manager")
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

/// Render the status section (loading or error messages)
fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    if app.is_fetching_api {
        // Show loading indicator
        let loading_text = Line::from(vec![
            Span::styled("⏳ ", Style::default().fg(Color::Yellow)),
            Span::styled(
                "Fetching from CTFtime API... Please wait.",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]);

        let loading = Paragraph::new(loading_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow)),
            )
            .style(Style::default());

        frame.render_widget(loading, area);
    } else if let Some(ref error_msg) = app.api_error_message {
        // Show error message
        let error_text = Line::from(vec![
            Span::styled("❌ ", Style::default().fg(Color::Red)),
            Span::styled(
                error_msg,
                Style::default()
                    .fg(Color::Red)
                    .add_modifier(Modifier::BOLD),
            ),
        ]);

        let error = Paragraph::new(error_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Red)),
            )
            .style(Style::default());

        frame.render_widget(error, area);
    }
}

/// Render the list of CTFs
fn render_list(frame: &mut Frame, app: &App, active_ctf_id: Option<i64>, area: Rect) {
    let mut items: Vec<ListItem> = Vec::new();
    let mut display_index = 0;

    // Add active/upcoming CTFs
    for ctf in app.ctfs.iter() {
        let is_selected = display_index == app.selected_index;
        let item = create_ctf_list_item(ctf, is_selected, active_ctf_id);
        items.push(item);
        display_index += 1;
    }

    // Add archive header and archived CTFs if any exist
    if !app.archived_ctfs.is_empty() {
        let is_selected = display_index == app.selected_index;
        let archive_header = create_archive_header(app.archived_ctfs.len(), app.archive_expanded, is_selected);
        items.push(archive_header);
        display_index += 1;

        // Add archived CTFs if expanded
        if app.archive_expanded {
            for ctf in app.archived_ctfs.iter() {
                let is_selected = display_index == app.selected_index;
                let item = create_ctf_list_item(ctf, is_selected, active_ctf_id);
                items.push(item);
                display_index += 1;
            }
        }
    }

    // Create the list widget
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" CTFs (↑↓/kj to navigate, Enter to set active, →/l to view challenges, ←/h to go back, 'a' to toggle archive) ")
                .border_style(Style::default().fg(Color::White)),
        )
        .style(Style::default().fg(Color::White));

    frame.render_widget(list, area);
}

/// Create a list item for a CTF
fn create_ctf_list_item(ctf: &crate::db::models::Ctf, is_selected: bool, active_ctf_id: Option<i64>) -> ListItem<'static> {
    // Check if this is the active CTF
    let is_active = ctf.id == active_ctf_id;

    // Build the display text for this CTF
    let name = ctf.name.clone();
    let date_info = match (&ctf.start_date, &ctf.end_date) {
        (Some(start), Some(end)) => format!(" [{} - {}]", start, end),
        (Some(start), None) => format!(" [{}]", start),
        _ => String::new(),
    };

    let team = ctf
        .team_name
        .as_ref()
        .map(|t| format!(" ({})", t))
        .unwrap_or_default();

    let active_indicator = if is_active { " [ACTIVE]" } else { "" };
    let line_text = format!("{}{}{}{}", name, date_info, team, active_indicator);

    // Create styled content
    let content = if is_selected {
        // Highlighted/selected style
        let mut spans = vec![Span::raw("▶ ")];
        spans.push(Span::styled(
            line_text,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
        Line::from(spans)
    } else {
        // Normal style with green color for active CTF
        let style = if is_active {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::White)
        };
        Line::from(vec![Span::raw("  "), Span::styled(line_text, style)])
    };

    ListItem::new(content)
}

/// Create the archive header item
fn create_archive_header(count: usize, is_expanded: bool, is_selected: bool) -> ListItem<'static> {
    let arrow = if is_expanded { "▼" } else { "▶" };
    let text = format!("{} Archived CTFs ({})", arrow, count);

    let content = if is_selected {
        Line::from(vec![
            Span::raw("▶ "),
            Span::styled(
                text,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ])
    } else {
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                text,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ])
    };

    ListItem::new(content)
}

/// Render the settings button section
fn render_settings_button(frame: &mut Frame, area: Rect) {
    let button_text = vec![
        Span::styled("Press ", Style::default()),
        Span::styled("s", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(" to open ", Style::default()),
        Span::styled("Settings", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
    ];

    let settings = Paragraph::new(Line::from(button_text))
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::White));

    frame.render_widget(settings, area);
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
        Span::raw(" set active | "),
        Span::styled("→/l", Style::default().fg(Color::Green)),
        Span::raw(" challenges | "),
        Span::styled("←/h", Style::default().fg(Color::Yellow)),
        Span::raw(" back | "),
        Span::styled("a", Style::default().fg(Color::Cyan)),
        Span::raw(" archive | "),
        Span::styled("r", Style::default().fg(Color::Blue)),
        Span::raw(" refresh | "),
        Span::styled("q", Style::default().fg(Color::Red)),
        Span::raw(" quit"),
    ];

    let help = Paragraph::new(Line::from(help_text))
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::White));

    frame.render_widget(help, area);
}

// TODO: Enhancement ideas for this component:
// - Add scrolling when list is longer than screen
// - Show more details in a side panel when a CTF is selected
// - Add filtering/search functionality
// - Color-code CTFs by status (upcoming, ongoing, completed)
// - Show challenge count and solved percentage per CTF
// - Add icons or indicators for different CTF types
