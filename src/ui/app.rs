/// Application state and event loop for the TUI.
use crate::models::RenovatePr;

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
    /// All Renovate PRs fetched from GitHub.
    pub prs: Vec<RenovatePr>,
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
    /// Creates a new [`App`] with the given pull requests.
    pub fn new(prs: Vec<RenovatePr>) -> Self {
        Self {
            prs,
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
        if matches!(self.screen, Screen::List) && !self.prs.is_empty() {
            self.selected = (self.selected + 1).min(self.prs.len() - 1);
        }
    }

    fn open_selected(&mut self) {
        if !self.prs.is_empty() {
            self.screen = Screen::Detail(self.selected);
        }
    }

    /// Returns the pull request currently highlighted in the list, if any.
    pub fn selected_pr(&self) -> Option<&RenovatePr> {
        self.prs.get(self.selected)
    }

    /// Replaces the current PR list and resets selection.
    pub fn update_prs(&mut self, prs: Vec<RenovatePr>) {
        self.prs = prs;
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

    fn sample_prs(n: usize) -> Vec<RenovatePr> {
        use crate::models::ChecksStatus;
        (0..n)
            .map(|i| RenovatePr::new(
                format!("owner/repo-{i}"),
                i as u64,
                format!("Update dep {i}"),
                format!("https://github.com/owner/repo-{i}/pull/{i}"),
                ChecksStatus::Success,
            ))
            .collect()
    }

    // ------------------------------------------------------------------
    // Initial state
    // ------------------------------------------------------------------

    #[test]
    fn new_app_starts_at_first_pr() {
        let app = App::new(sample_prs(5));
        assert_eq!(app.selected, 0);
        assert_eq!(app.screen, Screen::List);
        assert!(!app.should_quit);
    }

    #[test]
    fn new_app_with_no_prs() {
        let app = App::new(vec![]);
        assert_eq!(app.selected, 0);
        assert!(app.selected_pr().is_none());
    }

    // ------------------------------------------------------------------
    // Quit
    // ------------------------------------------------------------------

    #[test]
    fn quit_event_sets_should_quit() {
        let mut app = App::new(sample_prs(3));
        app.handle_event(AppEvent::Quit);
        assert!(app.should_quit);
    }

    // ------------------------------------------------------------------
    // Navigation – down
    // ------------------------------------------------------------------

    #[test]
    fn navigate_down_increments_selection() {
        let mut app = App::new(sample_prs(5));
        app.handle_event(AppEvent::NavigateDown);
        assert_eq!(app.selected, 1);
    }

    #[test]
    fn navigate_down_does_not_exceed_last_item() {
        let mut app = App::new(sample_prs(3));
        for _ in 0..10 {
            app.handle_event(AppEvent::NavigateDown);
        }
        assert_eq!(app.selected, 2); // last index
    }

    #[test]
    fn navigate_down_on_empty_list_is_noop() {
        let mut app = App::new(vec![]);
        app.handle_event(AppEvent::NavigateDown);
        assert_eq!(app.selected, 0);
    }

    // ------------------------------------------------------------------
    // Navigation – up
    // ------------------------------------------------------------------

    #[test]
    fn navigate_up_decrements_selection() {
        let mut app = App::new(sample_prs(5));
        app.handle_event(AppEvent::NavigateDown);
        app.handle_event(AppEvent::NavigateDown);
        app.handle_event(AppEvent::NavigateUp);
        assert_eq!(app.selected, 1);
    }

    #[test]
    fn navigate_up_does_not_go_below_zero() {
        let mut app = App::new(sample_prs(3));
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
        let mut app = App::new(sample_prs(5));
        app.handle_event(AppEvent::NavigateDown);
        app.handle_event(AppEvent::Select);
        assert_eq!(app.screen, Screen::Detail(1));
    }

    #[test]
    fn navigate_up_from_detail_returns_to_list() {
        let mut app = App::new(sample_prs(5));
        app.handle_event(AppEvent::Select);
        assert_eq!(app.screen, Screen::Detail(0));
        app.handle_event(AppEvent::NavigateUp);
        assert_eq!(app.screen, Screen::List);
    }

    #[test]
    fn select_on_empty_list_stays_on_list_screen() {
        let mut app = App::new(vec![]);
        app.handle_event(AppEvent::Select);
        assert_eq!(app.screen, Screen::List);
    }

    // ------------------------------------------------------------------
    // selected_pr
    // ------------------------------------------------------------------

    #[test]
    fn selected_pr_returns_correct_item() {
        let mut app = App::new(sample_prs(5));
        app.handle_event(AppEvent::NavigateDown);
        app.handle_event(AppEvent::NavigateDown);
        let pr = app.selected_pr().unwrap();
        assert_eq!(pr.number, 2);
    }

    // ------------------------------------------------------------------
    // update_prs
    // ------------------------------------------------------------------

    #[test]
    fn update_prs_resets_selection_and_screen() {
        let mut app = App::new(sample_prs(5));
        app.handle_event(AppEvent::NavigateDown);
        app.handle_event(AppEvent::Select);
        app.update_prs(sample_prs(3));
        assert_eq!(app.selected, 0);
        assert_eq!(app.screen, Screen::List);
        assert_eq!(app.prs.len(), 3);
    }
}
