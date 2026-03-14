use octocrab::models::issues::Issue;
use octocrab::{Octocrab, Page, Result};

const RENOVATE_LABEL: &str = "renovate";

fn build_query_params(gh_username: &str) -> String {
    format!(
        "is:open is:pr review-requested:{} archived:false label:{}",
        gh_username, RENOVATE_LABEL
    )
}

#[async_trait::async_trait]
/// This trait defines a method to fetch open Renovate pull requests for a given GitHub username.
pub trait RenovatePrFetcher {
    async fn list_renovate_prs_for_user(&self, gh_username: String) -> Result<Page<Issue>>;
    // async fn renovate_prs_for_user(&self, gh_username: String) -> Result<Page<PullRequest>>;
}

#[async_trait::async_trait]
impl RenovatePrFetcher for Octocrab {
    async fn list_renovate_prs_for_user(&self, gh_username: String) -> Result<Page<Issue>> {
        let query_params = &[("q", &build_query_params(&gh_username))];

        let issues: Page<Issue> = self.get("/search/issues", Some(query_params)).await?;
        Ok(issues)
    }
}
