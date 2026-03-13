use regex::Regex;

/// Extracts the repository name from a GitHub API URL.
pub fn extract_repo_name_from_url(url: &str) -> Option<String> {
    let re = Regex::new(r"https://api\.github\.com/repos/[^/]+/([^/]+)/?").unwrap();
    re.captures(url).and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_repo_name_is_correctly_extracted() {
        let cases = [
            ("https://api.github.com/repos/org/repo", Some("repo".to_string())),
            ("https://api.github.com/repos/org/repo/", Some("repo".to_string())),
            ("https://api.github.com/repos/org/repo/issues", Some("repo".to_string())),
            ("https://api.github.com/repos/org", None),
            ("https://api.github.com/repos/", None),
            ("https://notgithub.com/org/repo", None),
        ];
        for (url, expected) in cases {
            assert_eq!(extract_repo_name_from_url(url), expected);
        }
    }
}