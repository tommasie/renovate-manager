/// Application state and event loop for the TUI.
use crate::models::{IssueItem, RenovatePr};

// ---------------------------------------------------------------------------
// Input events
// ---------------------------------------------------------------------------

/// High-level events produced by the input handler.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppEvent {
    Quit,
    NavigateUp,
    NavigateDown,
    Select,
    Refresh,
}

// ---------------------------------------------------------------------------
// Screen / view
// ---------------------------------------------------------------------------

/// Which view is currently rendered.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Screen {
    /// Main list of Renovate pull requests.
    #[default]
    List,
    /// Detailed view for a single pull request.
    Detail(usize),
}

// ---------------------------------------------------------------------------
// App state
// ---------------------------------------------------------------------------

/// Central state object that the TUI renderer reads and the event loop
/// mutates.
pub struct App {
    /// All Issues fetched from GitHub.
    pub issues: Vec<IssueItem>,
    /// Github username of the authenticated user, used for personalized messages.
    pub gh_username: String,
    /// Index of the currently highlighted row in the list.
    pub selected: usize,
    /// Currently active screen.
    pub screen: Screen,
    /// Whether the application should terminate.
    pub should_quit: bool,
    /// Optional status message displayed in the footer.
    pub status_message: Option<String>,
}

impl App {
    /// Creates a new [`App`] with the given Issues and GitHub username.
    pub fn new(issues: Vec<IssueItem>, gh_username: String) -> Self {
        Self {
            issues,
            gh_username,
            selected: 0,
            screen: Screen::default(),
            should_quit: false,
            status_message: None,
        }
    }

    /// Handles an [`AppEvent`] and mutates state accordingly.
    pub fn handle_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::Quit => self.should_quit = true,
            AppEvent::NavigateUp => self.move_selection_up(),
            AppEvent::NavigateDown => self.move_selection_down(),
            AppEvent::Select => self.open_selected(),
            AppEvent::Refresh => {
                self.status_message = Some("Refreshing…".to_owned());
            }
        }
    }

    // -----------------------------------------------------------------------
    // Navigation helpers
    // -----------------------------------------------------------------------

    fn move_selection_up(&mut self) {
        if matches!(self.screen, Screen::Detail(_)) {
            self.screen = Screen::List;
            return;
        }
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    fn move_selection_down(&mut self) {
        if matches!(self.screen, Screen::List) && !self.issues.is_empty() {
            self.selected = (self.selected + 1).min(self.issues.len() - 1);
        }
    }

    fn open_selected(&mut self) {
        if !self.issues.is_empty() {
            self.screen = Screen::Detail(self.selected);
        }
    }

    /// Returns the issue currently highlighted in the list, if any.
    pub fn selected_issue(&self) -> Option<&IssueItem> {
        self.issues.get(self.selected)
    }

    /// Replaces the current issue list and resets selection.
    pub fn update_issues(&mut self, issues: Vec<IssueItem>) {
        self.issues = issues;
        self.selected = 0;
        self.screen = Screen::List;
        self.status_message = None;
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_issues(n: usize) -> Vec<IssueItem> {
        (0..n)
            .map(|i| {
                IssueItem::new(
                    format!("owner/repo-{i}"),
                    format!("Update dep {i}"),
                    format!("https://github.com/owner/repo-{i}/pull/{i}"),
                )
            })
            .collect()
    }

    // ------------------------------------------------------------------
    // Initial state
    // ------------------------------------------------------------------

    #[test]
    fn new_app_starts_at_first_issue() {
        let app = App::new(sample_issues(5), "test_user".to_string());
        assert_eq!(app.selected, 0);
        assert_eq!(app.screen, Screen::List);
        assert!(!app.should_quit);
    }

    #[test]
    fn new_app_with_no_issues() {
        let app = App::new(vec![], "test_user".to_string());
        assert_eq!(app.selected, 0);
        assert!(app.selected_issue().is_none());
    }

    // ------------------------------------------------------------------
    // Quit
    // ------------------------------------------------------------------

    #[test]
    fn quit_event_sets_should_quit() {
        let mut app = App::new(sample_issues(3), "test_user".to_string());
        app.handle_event(AppEvent::Quit);
        assert!(app.should_quit);
    }

    // ------------------------------------------------------------------
    // Navigation – down
    // ------------------------------------------------------------------

    #[test]
    fn navigate_down_increments_selection() {
        let mut app = App::new(sample_issues(5), "test_user".to_string());
        app.handle_event(AppEvent::NavigateDown);
        assert_eq!(app.selected, 1);
    }

    #[test]
    fn navigate_down_does_not_exceed_last_item() {
        let mut app = App::new(sample_issues(3), "test_user".to_string());
        for _ in 0..10 {
            app.handle_event(AppEvent::NavigateDown);
        }
        assert_eq!(app.selected, 2); // last index
    }

    #[test]
    fn navigate_down_on_empty_list_is_noop() {
        let mut app = App::new(vec![], "test_user".to_string());
        app.handle_event(AppEvent::NavigateDown);
        assert_eq!(app.selected, 0);
    }

    // ------------------------------------------------------------------
    // Navigation – up
    // ------------------------------------------------------------------

    #[test]
    fn navigate_up_decrements_selection() {
        let mut app = App::new(sample_issues(5), "test_user".to_string());
        app.handle_event(AppEvent::NavigateDown);
        app.handle_event(AppEvent::NavigateDown);
        app.handle_event(AppEvent::NavigateUp);
        assert_eq!(app.selected, 1);
    }

    #[test]
    fn navigate_up_does_not_go_below_zero() {
        let mut app = App::new(sample_issues(3), "test_user".to_string());
        for _ in 0..5 {
            app.handle_event(AppEvent::NavigateUp);
        }
        assert_eq!(app.selected, 0);
    }

    // ------------------------------------------------------------------
    // Select / detail screen
    // ------------------------------------------------------------------

    #[test]
    fn select_opens_detail_screen() {
        let mut app = App::new(sample_issues(5), "test_user".to_string());
        app.handle_event(AppEvent::NavigateDown);
        app.handle_event(AppEvent::Select);
        assert_eq!(app.screen, Screen::Detail(1));
    }

    #[test]
    fn navigate_up_from_detail_returns_to_list() {
        let mut app = App::new(sample_issues(5), "test_user".to_string());
        app.handle_event(AppEvent::Select);
        assert_eq!(app.screen, Screen::Detail(0));
        app.handle_event(AppEvent::NavigateUp);
        assert_eq!(app.screen, Screen::List);
    }

    #[test]
    fn select_on_empty_list_stays_on_list_screen() {
        let mut app = App::new(vec![], "test_user".to_string());
        app.handle_event(AppEvent::Select);
        assert_eq!(app.screen, Screen::List);
    }

    // ------------------------------------------------------------------
    // selected_issue
    // ------------------------------------------------------------------

    #[test]
    fn selected_issue_returns_correct_item() {
        let mut app = App::new(sample_issues(5), "test_user".to_string());
        app.handle_event(AppEvent::NavigateDown);
        app.handle_event(AppEvent::NavigateDown);
        let issue = app.selected_issue().unwrap();
        assert_eq!(issue.title, "Update dep 2");
    }

    // ------------------------------------------------------------------
    // update_issues
    // ------------------------------------------------------------------

    #[test]
    fn update_issues_resets_selection_and_screen() {
        let mut app = App::new(sample_issues(5), "test_user".to_string());
        app.handle_event(AppEvent::NavigateDown);
        app.handle_event(AppEvent::Select);
        app.update_issues(sample_issues(3));
        assert_eq!(app.selected, 0);
        assert_eq!(app.screen, Screen::List);
        assert_eq!(app.issues.len(), 3);
    }
}
