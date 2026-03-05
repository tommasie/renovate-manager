/// Authentication module.
///
/// Retrieves a GitHub personal access token from the `gh` CLI so the user does
/// not have to manage tokens manually. The `gh` CLI stores the token in its
/// secure credential store and exposes it via:
///
/// ```sh
/// gh auth token
/// ```
use anyhow::{Context, Result, anyhow};
use std::process::Command;

/// Returns the GitHub token managed by the `gh` CLI.
///
/// # Errors
/// Returns an error if the `gh` binary is not found, exits with a non-zero
/// status, or produces non-UTF-8 output.
pub fn get_gh_token() -> Result<String> {
    let output = Command::new("gh")
        .args(["auth", "token"])
        .output()
        .context("Failed to run `gh auth token`. Is the GitHub CLI installed and authenticated?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!(
            "`gh auth token` failed (exit {}): {}",
            output.status,
            stderr.trim()
        ));
    }

    let token = String::from_utf8(output.stdout)
        .context("`gh auth token` returned non-UTF-8 output")?
        .trim()
        .to_owned();

    if token.is_empty() {
        return Err(anyhow!(
            "`gh auth token` returned an empty token. Run `gh auth login` first."
        ));
    }

    Ok(token)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Smoke-test: the function returns *some* non-empty string when the `gh`
    /// CLI is present and the user is logged in.  We only assert the structural
    /// properties (non-empty, no surrounding whitespace) so that the test does
    /// not require a specific token value.
    ///
    /// This test is intentionally marked `#[ignore]` so it does not run in CI
    /// environments without a valid `gh` session; run it locally with
    /// `cargo test -- --ignored`.
    #[test]
    #[ignore]
    fn smoke_returns_non_empty_token() {
        let token = get_gh_token().expect("should retrieve a token");
        assert!(!token.is_empty(), "token must not be empty");
        assert_eq!(token, token.trim(), "token must have no leading/trailing whitespace");
    }

    /// When the gh binary does not exist the function should return an error
    /// that mentions the binary name.  We cannot easily mock `Command` without
    /// extra scaffolding, so this unit test verifies the error message
    /// construction logic using a fake binary path helper.
    #[test]
    fn missing_binary_produces_descriptive_error() {
        let result = call_gh_with_binary("__nonexistent_binary_xyz__");
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("gh auth token") || msg.contains("__nonexistent_binary_xyz__"),
            "error should mention the command; got: {msg}"
        );
    }

    // Helper that behaves like `get_gh_token` but uses a custom binary name,
    // useful for injecting failure cases in tests.
    fn call_gh_with_binary(binary: &str) -> Result<String> {
        let output = Command::new(binary)
            .args(["auth", "token"])
            .output()
            .with_context(|| {
                format!("Failed to run `{binary} auth token`. Is the GitHub CLI installed and authenticated?")
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!(
                "`{binary} auth token` failed (exit {}): {}",
                output.status,
                stderr.trim()
            ));
        }

        let token = String::from_utf8(output.stdout)
            .context("non-UTF-8 output")?
            .trim()
            .to_owned();

        if token.is_empty() {
            return Err(anyhow!("empty token"));
        }

        Ok(token)
    }
}
