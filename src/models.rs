/// Domain models for the renovate-manager application.
use serde::{Deserialize, Serialize};

/// The overall status of a pull request's CI check suite.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ChecksStatus {
    /// All checks passed.
    Success,
    /// One or more checks are still running.
    Pending,
    /// One or more checks failed.
    Failure,
    /// The status could not be determined (e.g. no checks configured).
    #[default]
    Unknown,
}

impl ChecksStatus {
    /// Returns a compact human-readable symbol for the status.
    pub fn symbol(&self) -> &'static str {
        match self {
            ChecksStatus::Success => "✓",
            ChecksStatus::Pending => "⏳",
            ChecksStatus::Failure => "✗",
            ChecksStatus::Unknown => "?",
        }
    }

    /// Returns a descriptive label used in the UI.
    pub fn label(&self) -> &'static str {
        match self {
            ChecksStatus::Success => "success",
            ChecksStatus::Pending => "pending",
            ChecksStatus::Failure => "failure",
            ChecksStatus::Unknown => "unknown",
        }
    }
}

impl std::fmt::Display for ChecksStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.symbol(), self.label())
    }
}

/// A Renovate pull request as displayed in the TUI list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenovatePr {
    /// GitHub repository full name, e.g. `"owner/repo"`.
    pub repo: String,
    /// Pull request number.
    pub number: u64,
    /// Pull request title.
    pub title: String,
    /// URL of the pull request on GitHub.
    pub url: String,
    /// Aggregated CI checks status.
    pub checks_status: ChecksStatus,
}

impl RenovatePr {
    /// Creates a new [`RenovatePr`].
    pub fn new(
        repo: impl Into<String>,
        number: u64,
        title: impl Into<String>,
        url: impl Into<String>,
        checks_status: ChecksStatus,
    ) -> Self {
        Self {
            repo: repo.into(),
            number,
            title: title.into(),
            url: url.into(),
            checks_status,
        }
    }
}

/// An IssueItem represents a GitHub Issue (or PR) returned by the search API, with only the fields we care about for display.
/// TODO find a way to include the PR checks status
pub struct IssueItem {
    pub repo: String,
    pub title: String,
    pub pull_request_url: String,
}

impl IssueItem {
    pub fn new(
        repo: impl Into<String>,
        title: impl Into<String>,
        pull_request_url: impl Into<String>,
    ) -> Self {
        Self {
            repo: repo.into(),
            title: title.into(),
            pull_request_url: pull_request_url.into(),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checks_status_symbol_and_label_are_consistent() {
        let cases = [
            (ChecksStatus::Success, "✓", "success"),
            (ChecksStatus::Pending, "⏳", "pending"),
            (ChecksStatus::Failure, "✗", "failure"),
            (ChecksStatus::Unknown, "?", "unknown"),
        ];
        for (status, expected_symbol, expected_label) in cases {
            assert_eq!(status.symbol(), expected_symbol);
            assert_eq!(status.label(), expected_label);
        }
    }

    #[test]
    fn checks_status_display_combines_symbol_and_label() {
        assert_eq!(ChecksStatus::Success.to_string(), "✓ success");
        assert_eq!(ChecksStatus::Failure.to_string(), "✗ failure");
    }

    #[test]
    fn renovate_pr_fields_stored_correctly() {
        let pr = RenovatePr::new(
            "owner/repo",
            42,
            "Update dependency foo to v2",
            "https://github.com/owner/repo/pull/42",
            ChecksStatus::Success,
        );
        assert_eq!(pr.repo, "owner/repo");
        assert_eq!(pr.number, 42);
        assert_eq!(pr.title, "Update dependency foo to v2");
        assert_eq!(pr.checks_status, ChecksStatus::Success);
    }

    #[test]
    fn checks_status_default_is_unknown() {
        let status: ChecksStatus = Default::default();
        assert_eq!(status, ChecksStatus::Unknown);
    }
}
