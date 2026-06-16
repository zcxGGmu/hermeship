use anyhow::Result;
use serde_json::{Map, Value, json};

use crate::events::IncomingEvent;
use crate::source::git::{
    MAX_DISPLAY_FIELD_CHARS, MAX_SUMMARY_CHARS, validate_commit_sha, validate_optional_single_line,
    validate_single_line,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GithubIssueInput {
    pub owner: String,
    pub repo: String,
    pub number: u64,
    pub title: String,
    pub author: Option<String>,
    pub url: Option<String>,
    pub channel: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GithubPullRequestInput {
    pub owner: String,
    pub repo: String,
    pub number: u64,
    pub title: String,
    pub branch: String,
    pub base_branch: Option<String>,
    pub commit: Option<String>,
    pub author: Option<String>,
    pub url: Option<String>,
    pub channel: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GithubCheckInput {
    pub owner: String,
    pub repo: String,
    pub workflow: String,
    pub status: String,
    pub branch: String,
    pub commit: Option<String>,
    pub title: Option<String>,
    pub url: Option<String>,
    pub channel: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GithubReleaseInput {
    pub owner: String,
    pub repo: String,
    pub tag: String,
    pub title: Option<String>,
    pub author: Option<String>,
    pub url: Option<String>,
    pub channel: Option<String>,
}

pub fn issue_opened_event(input: GithubIssueInput) -> Result<IncomingEvent> {
    let owner = validate_display_field("owner", &input.owner)?;
    let repo = validate_display_field("repo", &input.repo)?;
    let number = validate_positive_number(input.number)?;
    let title = validate_summary_field("title", &input.title)?;
    let author = validate_optional_display_field("author", input.author.as_deref())?;
    let url = validate_optional_url(input.url.as_deref())?;

    let mut payload = base_payload(&owner, &repo);
    insert_number(&mut payload, "number", number);
    insert_string(&mut payload, "title", Some(title.as_str()));
    insert_string(&mut payload, "author", author.as_deref());
    insert_string(&mut payload, "url", url.as_deref());

    Ok(incoming("github.issue-opened", input.channel, payload))
}

pub fn pull_request_opened_event(input: GithubPullRequestInput) -> Result<IncomingEvent> {
    let owner = validate_display_field("owner", &input.owner)?;
    let repo = validate_display_field("repo", &input.repo)?;
    let number = validate_positive_number(input.number)?;
    let title = validate_summary_field("title", &input.title)?;
    let branch = validate_display_field("branch", &input.branch)?;
    let base_branch = validate_optional_display_field("base_branch", input.base_branch.as_deref())?;
    let commit = input
        .commit
        .as_deref()
        .map(validate_commit_sha)
        .transpose()?;
    let author = validate_optional_display_field("author", input.author.as_deref())?;
    let url = validate_optional_url(input.url.as_deref())?;

    let mut payload = base_payload(&owner, &repo);
    insert_number(&mut payload, "number", number);
    insert_string(&mut payload, "title", Some(title.as_str()));
    insert_string(&mut payload, "branch", Some(branch.as_str()));
    insert_string(&mut payload, "base_branch", base_branch.as_deref());
    insert_string(&mut payload, "commit", commit.as_deref());
    if let Some(commit) = &commit {
        insert_string(
            &mut payload,
            "short_commit",
            Some(short_sha(commit).as_str()),
        );
    }
    insert_string(&mut payload, "author", author.as_deref());
    insert_string(&mut payload, "url", url.as_deref());

    Ok(incoming("github.pr-opened", input.channel, payload))
}

pub fn check_failed_event(input: GithubCheckInput) -> Result<IncomingEvent> {
    let owner = validate_display_field("owner", &input.owner)?;
    let repo = validate_display_field("repo", &input.repo)?;
    let workflow = validate_display_field("workflow", &input.workflow)?;
    let status = validate_status(&input.status)?;
    let branch = validate_display_field("branch", &input.branch)?;
    let commit = input
        .commit
        .as_deref()
        .map(validate_commit_sha)
        .transpose()?;
    let title = validate_optional_summary_field("title", input.title.as_deref())?;
    let url = validate_optional_url(input.url.as_deref())?;

    let mut payload = base_payload(&owner, &repo);
    insert_string(&mut payload, "workflow", Some(workflow.as_str()));
    insert_string(&mut payload, "status", Some(status.as_str()));
    insert_string(&mut payload, "branch", Some(branch.as_str()));
    insert_string(&mut payload, "commit", commit.as_deref());
    if let Some(commit) = &commit {
        insert_string(
            &mut payload,
            "short_commit",
            Some(short_sha(commit).as_str()),
        );
    }
    insert_string(&mut payload, "title", title.as_deref());
    insert_string(&mut payload, "url", url.as_deref());

    Ok(incoming("github.check-failed", input.channel, payload))
}

pub fn release_published_event(input: GithubReleaseInput) -> Result<IncomingEvent> {
    let owner = validate_display_field("owner", &input.owner)?;
    let repo = validate_display_field("repo", &input.repo)?;
    let tag = validate_display_field("tag", &input.tag)?;
    let title = validate_optional_summary_field("title", input.title.as_deref())?;
    let author = validate_optional_display_field("author", input.author.as_deref())?;
    let url = validate_optional_url(input.url.as_deref())?;

    let mut payload = base_payload(&owner, &repo);
    insert_string(&mut payload, "tag", Some(tag.as_str()));
    insert_string(&mut payload, "title", title.as_deref());
    insert_string(&mut payload, "author", author.as_deref());
    insert_string(&mut payload, "url", url.as_deref());

    Ok(incoming("github.release-published", input.channel, payload))
}

fn base_payload(owner: &str, repo: &str) -> Map<String, Value> {
    let mut payload = Map::new();
    insert_string(&mut payload, "owner", Some(owner));
    insert_string(&mut payload, "repo", Some(repo));
    insert_string(&mut payload, "repo_name", Some(repo));
    payload
}

fn incoming(kind: &str, channel: Option<String>, payload: Map<String, Value>) -> IncomingEvent {
    IncomingEvent {
        kind: kind.to_string(),
        channel: normalize_optional_text(channel),
        mention: None,
        format: None,
        template: None,
        payload: Value::Object(payload),
    }
}

fn insert_string(payload: &mut Map<String, Value>, key: &str, value: Option<&str>) {
    if let Some(value) = normalize_text(value) {
        payload.insert(key.to_string(), json!(value));
    }
}

fn insert_number(payload: &mut Map<String, Value>, key: &str, value: u64) {
    payload.insert(key.to_string(), json!(value));
}

fn normalize_optional_text(value: Option<String>) -> Option<String> {
    value.and_then(|value| normalize_text(Some(value.as_str())))
}

fn normalize_text(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

pub(crate) fn validate_display_field(name: &str, raw: &str) -> Result<String> {
    validate_single_line(name, raw, MAX_DISPLAY_FIELD_CHARS)
}

pub(crate) fn validate_summary_field(name: &str, raw: &str) -> Result<String> {
    validate_single_line(name, raw, MAX_SUMMARY_CHARS)
}

pub(crate) fn validate_optional_display_field(
    name: &str,
    value: Option<&str>,
) -> Result<Option<String>> {
    validate_optional_single_line(name, value, MAX_DISPLAY_FIELD_CHARS)
}

pub(crate) fn validate_optional_summary_field(
    name: &str,
    value: Option<&str>,
) -> Result<Option<String>> {
    validate_optional_single_line(name, value, MAX_SUMMARY_CHARS)
}

pub(crate) fn validate_optional_url(value: Option<&str>) -> Result<Option<String>> {
    validate_optional_single_line("url", value, MAX_SUMMARY_CHARS)
}

pub(crate) fn validate_positive_number(number: u64) -> Result<u64> {
    if number == 0 {
        anyhow::bail!("number must be greater than 0");
    }
    Ok(number)
}

pub(crate) fn validate_status(raw: &str) -> Result<String> {
    let status = validate_display_field("status", raw)?.to_ascii_lowercase();
    if matches!(
        status.as_str(),
        "success"
            | "failure"
            | "failed"
            | "cancelled"
            | "skipped"
            | "neutral"
            | "timed_out"
            | "action_required"
    ) {
        Ok(status)
    } else {
        anyhow::bail!(
            "status must be one of success, failure, failed, cancelled, skipped, neutral, timed_out, action_required"
        );
    }
}

pub(crate) fn short_sha(commit: &str) -> String {
    commit.chars().take(7).collect()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::privacy::sanitize_payload;
    use crate::source::github::{
        GithubCheckInput, GithubIssueInput, GithubPullRequestInput, GithubReleaseInput,
        check_failed_event, issue_opened_event, pull_request_opened_event, release_published_event,
    };

    #[test]
    fn github_issue_source_builds_sanitized_metadata_event() {
        let event = issue_opened_event(GithubIssueInput {
            owner: "posp".to_string(),
            repo: "hermeship".to_string(),
            number: 42,
            title: "Add deterministic GitHub source".to_string(),
            author: Some("synthetic-user".to_string()),
            url: Some("https://github.example.invalid/posp/hermeship/issues/42".to_string()),
            channel: Some("ops".to_string()),
        })
        .unwrap();

        assert_eq!(event.kind, "github.issue-opened");
        assert_eq!(event.channel.as_deref(), Some("ops"));
        assert_eq!(event.payload["owner"], json!("posp"));
        assert_eq!(event.payload["repo"], json!("hermeship"));
        assert_eq!(event.payload["repo_name"], json!("hermeship"));
        assert_eq!(event.payload["number"], json!(42));
        assert_eq!(
            event.payload["title"],
            json!("Add deterministic GitHub source")
        );
        assert_eq!(event.payload["author"], json!("synthetic-user"));
        assert_eq!(
            event.payload["url"],
            json!("https://github.example.invalid/posp/hermeship/issues/42")
        );

        let sanitized = sanitize_payload(&event.payload, &Default::default());
        assert!(sanitized.get("body").is_none());
        assert!(sanitized.get("token").is_none());
        assert!(sanitized.get("secret").is_none());
        assert!(sanitized.get("provider_response").is_none());
    }

    #[test]
    fn github_pr_check_and_release_sources_build_route_metadata_events() {
        let pr = pull_request_opened_event(GithubPullRequestInput {
            owner: "posp".to_string(),
            repo: "hermeship".to_string(),
            number: 17,
            title: "Ship GitHub source".to_string(),
            branch: "codex/milestone-8-github".to_string(),
            base_branch: Some("main".to_string()),
            commit: Some("1234567890abcdef1234567890abcdef12345678".to_string()),
            author: Some("synthetic-user".to_string()),
            url: None,
            channel: None,
        })
        .unwrap();
        assert_eq!(pr.kind, "github.pr-opened");
        assert_eq!(pr.payload["owner"], json!("posp"));
        assert_eq!(pr.payload["repo_name"], json!("hermeship"));
        assert_eq!(pr.payload["number"], json!(17));
        assert_eq!(pr.payload["branch"], json!("codex/milestone-8-github"));
        assert_eq!(pr.payload["base_branch"], json!("main"));
        assert_eq!(pr.payload["short_commit"], json!("1234567"));

        let check = check_failed_event(GithubCheckInput {
            owner: "posp".to_string(),
            repo: "hermeship".to_string(),
            workflow: "ci".to_string(),
            status: "failure".to_string(),
            branch: "main".to_string(),
            commit: Some("abcdef1234567890abcdef1234567890abcdef12".to_string()),
            title: Some("cargo test failed".to_string()),
            url: None,
            channel: None,
        })
        .unwrap();
        assert_eq!(check.kind, "github.check-failed");
        assert_eq!(check.payload["workflow"], json!("ci"));
        assert_eq!(check.payload["status"], json!("failure"));
        assert_eq!(check.payload["branch"], json!("main"));
        assert_eq!(check.payload["short_commit"], json!("abcdef1"));

        let release = release_published_event(GithubReleaseInput {
            owner: "posp".to_string(),
            repo: "hermeship".to_string(),
            tag: "v0.1.0".to_string(),
            title: Some("Hermeship v0.1.0".to_string()),
            author: Some("synthetic-user".to_string()),
            url: None,
            channel: None,
        })
        .unwrap();
        assert_eq!(release.kind, "github.release-published");
        assert_eq!(release.payload["tag"], json!("v0.1.0"));
        assert_eq!(release.payload["title"], json!("Hermeship v0.1.0"));
    }

    #[test]
    fn github_source_rejects_invalid_number_sha_status_and_multiline_title() {
        let invalid_number = issue_opened_event(GithubIssueInput {
            owner: "posp".to_string(),
            repo: "hermeship".to_string(),
            number: 0,
            title: "Add deterministic GitHub source".to_string(),
            author: None,
            url: None,
            channel: None,
        })
        .unwrap_err()
        .to_string();
        assert!(invalid_number.contains("number must be greater than 0"));

        let multiline_title = issue_opened_event(GithubIssueInput {
            owner: "posp".to_string(),
            repo: "hermeship".to_string(),
            number: 42,
            title: "safe title\nfull body should not render".to_string(),
            author: None,
            url: None,
            channel: None,
        })
        .unwrap_err()
        .to_string();
        assert!(multiline_title.contains("title must be a single line"));

        let invalid_sha = pull_request_opened_event(GithubPullRequestInput {
            owner: "posp".to_string(),
            repo: "hermeship".to_string(),
            number: 17,
            title: "Ship GitHub source".to_string(),
            branch: "codex/milestone-8-github".to_string(),
            base_branch: None,
            commit: Some("not-a-sha".to_string()),
            author: None,
            url: None,
            channel: None,
        })
        .unwrap_err()
        .to_string();
        assert!(invalid_sha.contains("commit must be 7-64 hex characters"));

        let invalid_status = check_failed_event(GithubCheckInput {
            owner: "posp".to_string(),
            repo: "hermeship".to_string(),
            workflow: "ci".to_string(),
            status: "maybe".to_string(),
            branch: "main".to_string(),
            commit: None,
            title: None,
            url: None,
            channel: None,
        })
        .unwrap_err()
        .to_string();
        assert!(invalid_status.contains("status must be one of"));
    }
}
