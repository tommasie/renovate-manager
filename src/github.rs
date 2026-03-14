/// GitHub API client built on top of [`octocrab`].
///
/// Responsibilities:
/// * Build an authenticated [`octocrab::Octocrab`] instance from a raw token.
/// * Discover all repositories belonging to the authenticated user's GitHub
///   teams.
/// * Fetch open pull requests labelled `renovate` from those repositories and
///   return them as [`RenovatePr`] domain objects.
use anyhow::{Context, Result};
use octocrab::{
    Octocrab,
    models::pulls::{MergeableState, PullRequest},
    params::{Direction, State, pulls::Sort},
};
use serde::Deserialize;

use crate::octocrab_ext::RenovatePrFetcher;
use crate::{
    models::{ChecksStatus, IssueItem, RenovatePr},
    utils::extract_repo_name_from_url,
};

/// Label that all Renovate-created pull requests carry.
const RENOVATE_LABEL: &str = "renovate";

// ---------------------------------------------------------------------------
// Client wrapper
// ---------------------------------------------------------------------------

/// Wraps an [`Octocrab`] instance and provides high-level methods for
/// fetching Renovate pull requests.
pub struct GithubClient {
    octocrab: Octocrab,
}

impl GithubClient {
    /// Creates a new [`GithubClient`] from a personal access token.
    pub fn new(token: &str) -> Result<Self> {
        let octocrab = Octocrab::builder()
            .personal_token(token.to_owned())
            .build()
            .context("Failed to build Octocrab client")?;
        Ok(Self { octocrab })
    }

    // -----------------------------------------------------------------------
    // Public API
    // -----------------------------------------------------------------------

    /// Returns the login name of the authenticated user.
    pub async fn current_user_login(&self) -> Result<String> {
        let user = self
            .octocrab
            .current()
            .user()
            .await
            .context("Failed to fetch authenticated user")?;
        Ok(user.login)
    }

    pub async fn renovate_prs_for_user(&self) -> Result<Vec<IssueItem>> {
        let gh_user = self.current_user_login().await?;
        let pulls = self.octocrab.list_renovate_prs_for_user(gh_user).await?;
        let renovate_prs: Vec<IssueItem> = pulls
            .items
            .into_iter()
            .map(|issue: octocrab::models::issues::Issue| {
                IssueItem::new(
                    extract_repo_name_from_url(issue.repository_url.as_str()).unwrap_or_default(),
                    issue.title,
                    issue.url.to_string(),
                )
            })
            .collect();
        Ok(renovate_prs)
    }

    pub async fn get_pr_from_issue(&self, issue: &IssueItem) -> Result<PullRequest> {
        let (owner, repo) = split_repo(&issue.repo)?;
        let pr_number = issue
            .pull_request_url
            .rsplit('/')
            .next()
            .and_then(|n| n.parse::<u64>().ok())
            .context("Failed to extract PR number from issue URL")?;
        let pr = self
            .octocrab
            .pulls(&owner, &repo)
            .get(pr_number)
            .await
            .with_context(|| format!("Failed to fetch pull request for issue '{}'", issue.title))?;
        Ok(pr)
    }

    /// Fetches all open Renovate pull requests for a single repository.
    ///
    /// `full_repo` must be in the form `"owner/repo"`.
    pub async fn renovate_prs(&self, full_repo: &str) -> Result<Vec<RenovatePr>> {
        let (owner, repo) = split_repo(full_repo)?;

        let prs = self
            .octocrab
            .pulls(&owner, &repo)
            .list()
            .state(State::Open)
            .sort(Sort::Created)
            .direction(Direction::Descending)
            .per_page(100)
            .send()
            .await
            .with_context(|| format!("Failed to list pull requests for '{full_repo}'"))?;

        let renovate_prs: Vec<RenovatePr> = prs
            .items
            .into_iter()
            .filter(|pr| is_renovate_pr(pr))
            .map(|pr| {
                let checks_status = derive_checks_status(&pr);
                RenovatePr::new(
                    full_repo,
                    pr.number,
                    pr.title.unwrap_or_default(),
                    pr.html_url.map(|u| u.to_string()).unwrap_or_default(),
                    checks_status,
                )
            })
            .collect();

        Ok(renovate_prs)
    }

    /// Aggregates Renovate PRs across all provided repository names.
    pub async fn all_renovate_prs(&self, repos: &[String]) -> Result<Vec<RenovatePr>> {
        let mut all = Vec::new();
        for repo in repos {
            let prs = self.renovate_prs(repo).await.unwrap_or_else(|err| {
                tracing_warn(repo, &err);
                Vec::new()
            });
            all.extend(prs);
        }
        Ok(all)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Splits `"owner/repo"` into `("owner", "repo")`.
fn split_repo(full_repo: &str) -> Result<(String, String)> {
    let mut parts = full_repo.splitn(2, '/');
    let owner = parts
        .next()
        .filter(|s| !s.is_empty())
        .with_context(|| format!("Invalid repo name '{full_repo}'"))?
        .to_owned();
    let repo = parts
        .next()
        .filter(|s| !s.is_empty())
        .with_context(|| format!("Invalid repo name '{full_repo}'"))?
        .to_owned();
    Ok((owner, repo))
}

/// Returns `true` when any label name in `names` equals `"renovate"`
/// (case-insensitive).  This is the pure, testable core of [`is_renovate_pr`].
fn has_renovate_label<'a>(mut names: impl Iterator<Item = &'a str>) -> bool {
    names.any(|n| n.to_lowercase() == RENOVATE_LABEL)
}

/// Returns `true` when the pull request carries the `renovate` label.
fn is_renovate_pr(pr: &PullRequest) -> bool {
    pr.labels
        .as_ref()
        .map(|labels| has_renovate_label(labels.iter().map(|l| l.name.as_str())))
        .unwrap_or(false)
}

/// Maps an octocrab [`MergeableState`] to a [`ChecksStatus`].
/// Extracted as a pure function so it can be unit-tested without constructing
/// a full `PullRequest`.
fn checks_status_from_state(state: Option<&MergeableState>) -> ChecksStatus {
    match state {
        Some(MergeableState::Clean) => ChecksStatus::Success,
        Some(MergeableState::Blocked)
        | Some(MergeableState::Unstable)
        | Some(MergeableState::Dirty) => ChecksStatus::Failure,
        Some(MergeableState::Behind) => ChecksStatus::Pending,
        _ => ChecksStatus::Unknown,
    }
}

/// Derives a [`ChecksStatus`] from whatever status information Octocrab
/// provides on the PR. We use the `mergeable_state` enum returned by the
/// GitHub API as a lightweight proxy while avoiding extra API calls.
///
/// A production implementation would fetch the check-runs via the Checks API:
/// `GET /repos/{owner}/{repo}/commits/{ref}/check-runs`.
fn derive_checks_status(pr: &PullRequest) -> ChecksStatus {
    checks_status_from_state(pr.mergeable_state.as_ref())
}

/// Poor-man's tracing: write a warning to stderr without pulling in a full
/// tracing subscriber just for this.
fn tracing_warn(repo: &str, err: &anyhow::Error) {
    eprintln!("[WARN] Could not fetch PRs for '{repo}': {err}");
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------
    // split_repo
    // ------------------------------------------------------------------

    #[test]
    fn split_repo_valid() {
        let (owner, repo) = split_repo("octocat/hello-world").unwrap();
        assert_eq!(owner, "octocat");
        assert_eq!(repo, "hello-world");
    }

    #[test]
    fn split_repo_multiple_slashes_keeps_right_part() {
        // splitn(2) means only the first slash is the delimiter.
        let (owner, repo) = split_repo("org/repo/extra").unwrap();
        assert_eq!(owner, "org");
        assert_eq!(repo, "repo/extra");
    }

    #[test]
    fn split_repo_missing_slash_errors() {
        assert!(split_repo("noslash").is_err());
    }

    #[test]
    fn split_repo_empty_owner_errors() {
        assert!(split_repo("/repo").is_err());
    }

    #[test]
    fn split_repo_empty_repo_errors() {
        assert!(split_repo("owner/").is_err());
    }

    // ------------------------------------------------------------------
    // has_renovate_label  (pure function, no octocrab types needed)
    // ------------------------------------------------------------------

    #[test]
    fn has_renovate_label_with_renovate() {
        assert!(has_renovate_label(
            ["renovate", "dependencies"].iter().copied()
        ));
    }

    #[test]
    fn has_renovate_label_without_renovate() {
        assert!(!has_renovate_label(["bug", "enhancement"].iter().copied()));
    }

    #[test]
    fn has_renovate_label_empty_iterator() {
        assert!(!has_renovate_label(std::iter::empty()));
    }

    #[test]
    fn has_renovate_label_case_insensitive() {
        assert!(has_renovate_label(["Renovate"].iter().copied()));
        assert!(has_renovate_label(["RENOVATE"].iter().copied()));
    }

    // ------------------------------------------------------------------
    // checks_status_from_state  (pure function, no octocrab types needed)
    // ------------------------------------------------------------------

    #[test]
    fn checks_status_clean_is_success() {
        assert_eq!(
            checks_status_from_state(Some(&MergeableState::Clean)),
            ChecksStatus::Success
        );
    }

    #[test]
    fn checks_status_blocked_is_failure() {
        assert_eq!(
            checks_status_from_state(Some(&MergeableState::Blocked)),
            ChecksStatus::Failure
        );
    }

    #[test]
    fn checks_status_unstable_is_failure() {
        assert_eq!(
            checks_status_from_state(Some(&MergeableState::Unstable)),
            ChecksStatus::Failure
        );
    }

    #[test]
    fn checks_status_behind_is_pending() {
        assert_eq!(
            checks_status_from_state(Some(&MergeableState::Behind)),
            ChecksStatus::Pending
        );
    }

    #[test]
    fn checks_status_none_is_unknown() {
        assert_eq!(checks_status_from_state(None), ChecksStatus::Unknown);
    }
}
