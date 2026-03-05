/// Renovate Manager – entry point.
///
/// Start-up sequence:
/// 1. Read the GitHub token from the `gh` CLI.
/// 2. Build an [`octocrab`]-backed GitHub client.
/// 3. Collect all repositories accessible to the user's GitHub teams.
/// 4. Fetch open Renovate PRs from those repositories.
/// 5. Launch the Ratatui TUI event loop.
mod auth;
mod github;
mod models;
mod ui;

use std::io;
use std::time::Duration;

use anyhow::{Context, Result};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use auth::get_gh_token;
use github::GithubClient;
use ui::{App, AppEvent, widgets};

#[tokio::main]
async fn main() -> Result<()> {
    // ------------------------------------------------------------------
    // 1. Authentication
    // ------------------------------------------------------------------
    let token = get_gh_token().context("Could not obtain GitHub token from `gh` CLI")?;

    // ------------------------------------------------------------------
    // 2. GitHub client
    // ------------------------------------------------------------------
    let client = GithubClient::new(&token)
        .context("Could not create GitHub client")?;

    // ------------------------------------------------------------------
    // 3. Discover the authenticated user and their team repos
    // ------------------------------------------------------------------
    let login = client
        .current_user_login()
        .await
        .context("Could not determine authenticated GitHub user")?;

    eprintln!("[INFO] Logged in as: {login}");

    // Collect repos from all orgs/teams the token has access to.
    let prs = fetch_all_renovate_prs(&client).await?;

    // ------------------------------------------------------------------
    // 4. Set up the terminal
    // ------------------------------------------------------------------
    enable_raw_mode().context("Failed to enable terminal raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .context("Failed to enter alternate screen")?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("Failed to create terminal")?;

    // ------------------------------------------------------------------
    // 5. Run the TUI
    // ------------------------------------------------------------------
    let result = run_app(&mut terminal, prs).await;

    // Restore terminal on any exit path
    disable_raw_mode().ok();
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .ok();
    terminal.show_cursor().ok();

    result
}

// ---------------------------------------------------------------------------
// TUI event loop
// ---------------------------------------------------------------------------

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    prs: Vec<models::RenovatePr>,
) -> Result<()> {
    let mut app = App::new(prs);

    loop {
        terminal.draw(|frame| widgets::render(frame, &app))?;

        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                // Only react on key press, not release, to avoid double events.
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                if let Some(ev) = map_key_event(key.code) {
                    app.handle_event(ev);
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Key mapping
// ---------------------------------------------------------------------------

fn map_key_event(code: KeyCode) -> Option<AppEvent> {
    match code {
        KeyCode::Char('q') | KeyCode::Char('Q') => Some(AppEvent::Quit),
        KeyCode::Up | KeyCode::Char('k') => Some(AppEvent::NavigateUp),
        KeyCode::Down | KeyCode::Char('j') => Some(AppEvent::NavigateDown),
        KeyCode::Enter => Some(AppEvent::Select),
        KeyCode::Char('r') | KeyCode::Char('R') => Some(AppEvent::Refresh),
        KeyCode::Esc => Some(AppEvent::NavigateUp), // go back / close detail
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// PR fetching
// ---------------------------------------------------------------------------

/// Discovers repositories via GitHub team membership and returns all open
/// Renovate PRs from those repositories.
///
/// Uses `GET /user/teams` to fetch only the teams the authenticated user is
/// actually a member of, rather than iterating every org and every team.
async fn fetch_all_renovate_prs(
    client: &GithubClient,
) -> Result<Vec<models::RenovatePr>> {
    // Fetch only the teams the authenticated user belongs to in a single
    // API call, replacing the previous two-step approach of listing all
    // organisations then all org teams.
    let teams = match client.teams_for_authenticated_user().await {
        Ok(t) => t,
        Err(e) => {
            eprintln!("[WARN] Could not list user teams: {e}");
            vec![]
        }
    };

    let mut all_repos: Vec<String> = Vec::new();

    for (org, team_slug) in &teams {
        match client.repos_for_team(org, team_slug).await {
            Ok(repos) => all_repos.extend(repos),
            Err(e) => eprintln!("[WARN] Could not list repos for {org}/{team_slug}: {e}"),
        }
    }

    // Deduplicate repos.
    all_repos.sort();
    all_repos.dedup();

    if all_repos.is_empty() {
        eprintln!("[INFO] No team repositories found; showing empty list.");
        return Ok(vec![]);
    }

    eprintln!(
        "[INFO] Fetching Renovate PRs from {} repositories…",
        all_repos.len()
    );
    client
        .all_renovate_prs(&all_repos)
        .await
        .context("Failed to fetch Renovate pull requests")
}
