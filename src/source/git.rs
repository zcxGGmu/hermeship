use anyhow::Result;
use serde_json::{Map, Value, json};

use crate::events::IncomingEvent;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitCommitInput {
    pub repo: String,
    pub branch: String,
    pub commit: String,
    pub summary: String,
    pub channel: Option<String>,
    pub repo_path: Option<String>,
    pub worktree_path: Option<String>,
    pub author_name: Option<String>,
    pub author_email: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitBranchChangedInput {
    pub repo: String,
    pub old_branch: String,
    pub new_branch: String,
    pub channel: Option<String>,
    pub repo_path: Option<String>,
    pub worktree_path: Option<String>,
}

pub(crate) const MAX_SUMMARY_CHARS: usize = 240;
pub(crate) const MAX_DISPLAY_FIELD_CHARS: usize = 120;

pub fn commit_event(input: GitCommitInput) -> Result<IncomingEvent> {
    let commit = validate_commit_sha(&input.commit)?;
    let summary = validate_single_line("summary", &input.summary, MAX_SUMMARY_CHARS)?;
    let author_name = validate_optional_single_line(
        "author_name",
        input.author_name.as_deref(),
        MAX_DISPLAY_FIELD_CHARS,
    )?;
    let author_email = validate_optional_single_line(
        "author_email",
        input.author_email.as_deref(),
        MAX_DISPLAY_FIELD_CHARS,
    )?;

    let mut payload = Map::new();
    insert_string(&mut payload, "repo", Some(input.repo.as_str()));
    insert_string(&mut payload, "repo_name", Some(input.repo.as_str()));
    insert_string(&mut payload, "repo_path", input.repo_path.as_deref());
    insert_string(
        &mut payload,
        "worktree_path",
        input.worktree_path.as_deref(),
    );
    insert_string(&mut payload, "branch", Some(input.branch.as_str()));
    insert_string(&mut payload, "commit", Some(commit.as_str()));
    insert_string(
        &mut payload,
        "short_commit",
        Some(short_sha(&commit).as_str()),
    );
    insert_string(&mut payload, "summary", Some(summary.as_str()));
    insert_string(&mut payload, "author_name", author_name.as_deref());
    insert_string(&mut payload, "author_email", author_email.as_deref());

    Ok(IncomingEvent {
        kind: "git.commit".to_string(),
        channel: normalize_optional_text(input.channel),
        mention: None,
        format: None,
        template: None,
        payload: Value::Object(payload),
    })
}

pub fn branch_changed_event(input: GitBranchChangedInput) -> IncomingEvent {
    let mut payload = Map::new();
    insert_string(&mut payload, "repo", Some(input.repo.as_str()));
    insert_string(&mut payload, "repo_name", Some(input.repo.as_str()));
    insert_string(&mut payload, "repo_path", input.repo_path.as_deref());
    insert_string(
        &mut payload,
        "worktree_path",
        input.worktree_path.as_deref(),
    );
    insert_string(&mut payload, "old_branch", Some(input.old_branch.as_str()));
    insert_string(&mut payload, "new_branch", Some(input.new_branch.as_str()));
    insert_string(&mut payload, "branch", Some(input.new_branch.as_str()));

    IncomingEvent {
        kind: "git.branch-changed".to_string(),
        channel: normalize_optional_text(input.channel),
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

fn normalize_optional_text(value: Option<String>) -> Option<String> {
    value.and_then(|value| normalize_text(Some(value.as_str())))
}

fn normalize_text(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn short_sha(commit: &str) -> String {
    commit.chars().take(7).collect()
}

pub(crate) fn validate_commit_sha(raw: &str) -> Result<String> {
    let value = raw.trim();
    if (7..=64).contains(&value.len()) && value.chars().all(|ch| ch.is_ascii_hexdigit()) {
        Ok(value.to_ascii_lowercase())
    } else {
        anyhow::bail!("commit must be 7-64 hex characters");
    }
}

pub(crate) fn validate_optional_single_line(
    name: &str,
    value: Option<&str>,
    max_chars: usize,
) -> Result<Option<String>> {
    value
        .and_then(|value| normalize_text(Some(value)))
        .map(|value| validate_single_line(name, &value, max_chars))
        .transpose()
}

pub(crate) fn validate_single_line(name: &str, raw: &str, max_chars: usize) -> Result<String> {
    let value = raw.trim();
    if value.is_empty() {
        anyhow::bail!("{name} must not be empty");
    }
    if value.contains(['\n', '\r']) {
        anyhow::bail!("{name} must be a single line");
    }
    if value.chars().count() > max_chars {
        anyhow::bail!("{name} must be at most {max_chars} characters");
    }
    Ok(value.to_string())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::privacy::sanitize_payload;
    use crate::source::git::{
        GitBranchChangedInput, GitCommitInput, branch_changed_event, commit_event,
    };

    #[test]
    fn git_commit_source_builds_sanitized_metadata_event() {
        let event = commit_event(GitCommitInput {
            repo: "hermeship".to_string(),
            branch: "main".to_string(),
            commit: "1234567890abcdef1234567890abcdef12345678".to_string(),
            summary: "ship git source".to_string(),
            channel: Some("ops".to_string()),
            repo_path: Some("/tmp/hermeship".to_string()),
            worktree_path: Some("/tmp/hermeship-worktree".to_string()),
            author_name: Some("Synthetic Author".to_string()),
            author_email: Some("synthetic@example.invalid".to_string()),
        })
        .unwrap();

        assert_eq!(event.kind, "git.commit");
        assert_eq!(event.channel.as_deref(), Some("ops"));
        assert_eq!(event.payload["repo"], json!("hermeship"));
        assert_eq!(event.payload["repo_name"], json!("hermeship"));
        assert_eq!(event.payload["repo_path"], json!("/tmp/hermeship"));
        assert_eq!(
            event.payload["worktree_path"],
            json!("/tmp/hermeship-worktree")
        );
        assert_eq!(event.payload["branch"], json!("main"));
        assert_eq!(
            event.payload["commit"],
            json!("1234567890abcdef1234567890abcdef12345678")
        );
        assert_eq!(event.payload["short_commit"], json!("1234567"));
        assert_eq!(event.payload["summary"], json!("ship git source"));
        assert_eq!(event.payload["author_name"], json!("Synthetic Author"));
        assert_eq!(
            event.payload["author_email"],
            json!("synthetic@example.invalid")
        );

        let sanitized = sanitize_payload(&event.payload, &Default::default());
        assert!(sanitized.get("diff").is_none());
        assert!(sanitized.get("commit_body").is_none());
        assert!(sanitized.get("token").is_none());
        assert!(sanitized.get("secret").is_none());
    }

    #[test]
    fn git_branch_changed_source_builds_route_metadata_event() {
        let event = branch_changed_event(GitBranchChangedInput {
            repo: "hermeship".to_string(),
            old_branch: "main".to_string(),
            new_branch: "codex/milestone-8-git".to_string(),
            channel: None,
            repo_path: Some("/tmp/hermeship".to_string()),
            worktree_path: Some("/tmp/hermeship-worktree".to_string()),
        });

        assert_eq!(event.kind, "git.branch-changed");
        assert_eq!(event.payload["repo"], json!("hermeship"));
        assert_eq!(event.payload["repo_name"], json!("hermeship"));
        assert_eq!(event.payload["repo_path"], json!("/tmp/hermeship"));
        assert_eq!(
            event.payload["worktree_path"],
            json!("/tmp/hermeship-worktree")
        );
        assert_eq!(event.payload["old_branch"], json!("main"));
        assert_eq!(event.payload["new_branch"], json!("codex/milestone-8-git"));
        assert_eq!(event.payload["branch"], json!("codex/milestone-8-git"));
    }

    #[test]
    fn git_commit_source_rejects_invalid_sha_and_multiline_summary() {
        let invalid_sha = commit_event(GitCommitInput {
            repo: "hermeship".to_string(),
            branch: "main".to_string(),
            commit: "not-a-sha".to_string(),
            summary: "ship git source".to_string(),
            channel: None,
            repo_path: None,
            worktree_path: None,
            author_name: None,
            author_email: None,
        })
        .unwrap_err()
        .to_string();
        assert!(invalid_sha.contains("commit must be 7-64 hex characters"));

        let multiline = commit_event(GitCommitInput {
            repo: "hermeship".to_string(),
            branch: "main".to_string(),
            commit: "1234567890abcdef1234567890abcdef12345678".to_string(),
            summary: "ship git source\nsynthetic diff should not render".to_string(),
            channel: None,
            repo_path: None,
            worktree_path: None,
            author_name: None,
            author_email: None,
        })
        .unwrap_err()
        .to_string();
        assert!(multiline.contains("summary must be a single line"));
    }
}
