/// Rendering helpers – maps application state to Ratatui widgets.
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
};

use crate::models::{ChecksStatus, IssueItem, RenovatePr};
use crate::ui::app::{App, Screen};

// ---------------------------------------------------------------------------
// Colour palette
// ---------------------------------------------------------------------------

const COLOR_HEADER: Color = Color::Cyan;
const COLOR_SELECTED_BG: Color = Color::DarkGray;
const COLOR_SUCCESS: Color = Color::Green;
const COLOR_FAILURE: Color = Color::Red;
const COLOR_PENDING: Color = Color::Yellow;
const COLOR_UNKNOWN: Color = Color::Gray;
const COLOR_FOOTER: Color = Color::DarkGray;
const COLOR_BORDER: Color = Color::White;

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Renders the complete UI for the current frame.
pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // header
            Constraint::Min(0),    // content
            Constraint::Length(3), // footer / key hints
        ])
        .split(area);

    render_header(frame, chunks[0], app.gh_username.clone());

    match &app.screen {
        Screen::List => render_pr_list(frame, chunks[1], app),
        Screen::Detail(idx) => {
            // if let Some(pr) = app.prs.get(*idx) {
            //     render_pr_detail(frame, chunks[1], pr);
            // }
        }
    }

    render_footer(frame, chunks[2], app);
}

// ---------------------------------------------------------------------------
// Header
// ---------------------------------------------------------------------------

fn render_header(frame: &mut Frame, area: Rect, user: String) {
    let title = Paragraph::new(format!(" 🔧  Renovate Manager - Logged in as: {}", user))
        .style(Style::default().fg(COLOR_HEADER).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(COLOR_BORDER)));
    frame.render_widget(title, area);
}

// ---------------------------------------------------------------------------
// PR list
// ---------------------------------------------------------------------------

fn render_pr_list(frame: &mut Frame, area: Rect, app: &App) {
    if app.issues.is_empty() {
        let empty = Paragraph::new("  No open Renovate pull requests found.")
            .style(Style::default().fg(Color::Gray))
            .block(
                Block::default()
                    .title(" Pull Requests ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(COLOR_BORDER)),
            );
        frame.render_widget(empty, area);
        return;
    }

    let header = Row::new(vec![
        Cell::from("Repository").style(
            Style::default()
                .fg(COLOR_HEADER)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("Title").style(
            Style::default()
                .fg(COLOR_HEADER)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("Checks").style(
            Style::default()
                .fg(COLOR_HEADER)
                .add_modifier(Modifier::BOLD),
        ),
    ]);

    let rows: Vec<Row> = app
        .issues
        .iter()
        .enumerate()
        .map(|(i, issue)| {
            let style = if i == app.selected {
                Style::default()
                    .bg(COLOR_SELECTED_BG)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let checks_cell = checks_cell_issue(issue);

            Row::new(vec![
                Cell::from(shortened_repo_name(&issue.repo)),
                Cell::from(issue.title.clone()),
                checks_cell,
            ])
            .style(style)
        })
        .collect();

    let widths = [
        Constraint::Percentage(25),
        Constraint::Percentage(60),
        Constraint::Percentage(15),
    ];

    let table = Table::new(rows, widths)
        .header(header.style(Style::default()))
        .block(
            Block::default()
                .title(format!(
                    " Renovate PRs ({}) ",
                    app.issues.len()
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(COLOR_BORDER)),
        )
        .row_highlight_style(
            Style::default()
                .bg(COLOR_SELECTED_BG)
                .add_modifier(Modifier::BOLD),
        );

    let mut state = TableState::default().with_selected(Some(app.selected));
    frame.render_stateful_widget(table, area, &mut state);
}

fn shortened_repo_name(repo_name: &str) -> String {
    // Extract the last part of the repo name for brevity, e.g. "owner/repo" -> "repo"
    repo_name.split('/').last().unwrap_or(repo_name).to_string()
}

fn checks_cell(pr: &RenovatePr) -> Cell<'static> {
    let (label, color) = match pr.checks_status {
        ChecksStatus::Success => (format!("{} success", ChecksStatus::Success.symbol()), COLOR_SUCCESS),
        ChecksStatus::Pending => (format!("{} pending", ChecksStatus::Pending.symbol()), COLOR_PENDING),
        ChecksStatus::Failure => (format!("{} failure", ChecksStatus::Failure.symbol()), COLOR_FAILURE),
        ChecksStatus::Unknown => (format!("{} unknown", ChecksStatus::Unknown.symbol()), COLOR_UNKNOWN),
    };
    Cell::from(label).style(Style::default().fg(color))
}
fn checks_cell_issue(_issue: &IssueItem) -> Cell<'static> {
    Cell::from(format!("{} unknown", ChecksStatus::Unknown.symbol())).style(Style::default().fg(COLOR_UNKNOWN))
}

// ---------------------------------------------------------------------------
// PR detail
// ---------------------------------------------------------------------------

fn render_pr_detail(frame: &mut Frame, area: Rect, pr: &RenovatePr, issue: &IssueItem) {
    let (checks_label, checks_color) = match &pr.checks_status {
        ChecksStatus::Success => ("✓ success", COLOR_SUCCESS),
        ChecksStatus::Pending => ("⏳ pending", COLOR_PENDING),
        ChecksStatus::Failure => ("✗ failure", COLOR_FAILURE),
        ChecksStatus::Unknown => ("? unknown", COLOR_UNKNOWN),
    };

    let lines = vec![
        Line::from(vec![
            Span::styled("  Repository:  ", Style::default().fg(COLOR_HEADER).add_modifier(Modifier::BOLD)),
            Span::raw(pr.repo.clone()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  PR #:        ", Style::default().fg(COLOR_HEADER).add_modifier(Modifier::BOLD)),
            Span::raw(pr.number.to_string()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Title:       ", Style::default().fg(COLOR_HEADER).add_modifier(Modifier::BOLD)),
            Span::raw(pr.title.clone()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Checks:      ", Style::default().fg(COLOR_HEADER).add_modifier(Modifier::BOLD)),
            Span::styled(checks_label, Style::default().fg(checks_color)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  URL:         ", Style::default().fg(COLOR_HEADER).add_modifier(Modifier::BOLD)),
            Span::raw(pr.url.clone()),
        ]),
    ];

    let detail = Paragraph::new(lines)
        .block(
            Block::default()
                .title(format!(" PR #{}: {} ", pr.number, pr.title))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(COLOR_BORDER)),
        )
        .alignment(Alignment::Left);

    frame.render_widget(detail, area);
}

// ---------------------------------------------------------------------------
// Footer
// ---------------------------------------------------------------------------

fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
    let base_hints = " [↑/↓] Navigate  [Enter] Details  [r] Refresh  [q] Quit";

    let content = if let Some(msg) = &app.status_message {
        format!("{base_hints}  │  {msg}")
    } else {
        base_hints.to_owned()
    };

    let footer = Paragraph::new(content)
        .style(Style::default().fg(COLOR_FOOTER))
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(COLOR_BORDER)));

    frame.render_widget(footer, area);
}
