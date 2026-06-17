use anyhow::Result;
use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::event::{
    CronRunEvent, CustomEvent, EventBody, EventEnvelope, EventMetadata, EventPriority,
    GitBranchChangedEvent, GitCommitEvent, GithubCheckEvent, GithubIssueEvent,
    GithubPullRequestEvent, GithubReleaseEvent, HermesAgentEvent, HermesGatewayEvent,
    HermesSessionEvent, TmuxKeywordEvent, TmuxStaleEvent,
};
use crate::events::IncomingEvent;
use crate::source::git::{
    MAX_DISPLAY_FIELD_CHARS, MAX_SUMMARY_CHARS, validate_commit_sha, validate_optional_single_line,
    validate_single_line,
};
use crate::source::github::{
    short_sha as github_short_sha, validate_display_field, validate_optional_display_field,
    validate_optional_summary_field, validate_optional_url, validate_positive_number,
    validate_status, validate_summary_field,
};

pub fn from_incoming_event(event: &IncomingEvent) -> Result<EventEnvelope> {
    EventEnvelope::try_from(event)
}

impl TryFrom<&IncomingEvent> for EventEnvelope {
    type Error = anyhow::Error;

    fn try_from(event: &IncomingEvent) -> Result<Self> {
        let canonical_kind = canonical_kind(event.kind.as_str(), &event.payload);
        let metadata = metadata_for(event, &canonical_kind);

        Ok(Self {
            id: event_id_for(&event.payload).unwrap_or_else(Uuid::new_v4),
            timestamp: OffsetDateTime::now_utc(),
            source: source_for_kind(&canonical_kind),
            body: body_for(&canonical_kind, &event.payload, &metadata)?,
            metadata,
        })
    }
}

fn canonical_kind(kind: &str, payload: &Value) -> String {
    match kind.trim() {
        "gateway:startup" | "gateway.startup" | "hermes.gateway.started" => {
            "hermes.gateway.started"
        }
        "session:start" | "session.started" | "hermes.session.started" => "hermes.session.started",
        "session:end" | "session.finished" | "hermes.session.finished" => "hermes.session.finished",
        "session:reset" | "session.reset" | "hermes.session.reset" => "hermes.session.reset",
        "agent:start" | "agent.started" | "hermes.agent.started" => "hermes.agent.started",
        "agent:step" | "agent.step" | "hermes.agent.step" => "hermes.agent.step",
        "agent:end" | "agent.finished" | "hermes.agent.finished" if is_agent_failure(payload) => {
            "hermes.agent.failed"
        }
        "agent:end" | "agent.finished" | "hermes.agent.finished" => "hermes.agent.finished",
        "agent:failed" | "agent.failed" | "hermes.agent.failed" => "hermes.agent.failed",
        other => other,
    }
    .to_string()
}

fn metadata_for(event: &IncomingEvent, canonical_kind: &str) -> EventMetadata {
    EventMetadata {
        channel_hint: event.channel.clone(),
        mention: event
            .mention
            .clone()
            .or_else(|| string_field(&event.payload, "mention")),
        format: event.format,
        template: event.template.clone(),
        priority: priority_for(canonical_kind),
        tool: string_field(&event.payload, "tool"),
        provider: string_field(&event.payload, "provider"),
        source: string_field(&event.payload, "source"),
        platform: string_field(&event.payload, "platform"),
        user_id: string_field(&event.payload, "user_id"),
        chat_id: string_field(&event.payload, "chat_id"),
        thread_id: string_field(&event.payload, "thread_id"),
        chat_type: string_field(&event.payload, "chat_type"),
        session_id: string_field(&event.payload, "session_id"),
        agent_name: string_field(&event.payload, "agent_name"),
        project: string_field(&event.payload, "project"),
        repo_name: string_field(&event.payload, "repo_name"),
        repo_path: string_field(&event.payload, "repo_path"),
        worktree_path: string_field(&event.payload, "worktree_path"),
        branch: string_field(&event.payload, "branch"),
    }
}

fn body_for(kind: &str, payload: &Value, metadata: &EventMetadata) -> Result<EventBody> {
    Ok(match kind {
        "git.commit" => EventBody::GitCommit(git_commit_event(payload, metadata)?),
        "git.branch-changed" => {
            EventBody::GitBranchChanged(git_branch_changed_event(payload, metadata))
        }
        "github.issue-opened" => EventBody::GithubIssue(github_issue_event(payload, metadata)?),
        "github.pr-opened" => {
            EventBody::GithubPullRequest(github_pull_request_event(payload, metadata)?)
        }
        "github.check-failed" => EventBody::GithubCheck(github_check_event(payload, metadata)?),
        "github.release-published" => {
            EventBody::GithubRelease(github_release_event(payload, metadata)?)
        }
        "tmux.keyword" => EventBody::TmuxKeyword(tmux_keyword_event(payload)?),
        "tmux.stale" => EventBody::TmuxStale(tmux_stale_event(payload)?),
        "cron.run" => EventBody::CronRun(cron_run_event(payload)?),
        "hermes.gateway.started" => EventBody::HermesGatewayStarted(HermesGatewayEvent {
            provider: metadata.provider.clone(),
            source: metadata.source.clone(),
            platform: metadata.platform.clone(),
            project: metadata.project.clone(),
        }),
        "hermes.session.started" => {
            EventBody::HermesSessionStarted(session_event("started", payload, metadata))
        }
        "hermes.session.finished" => {
            EventBody::HermesSessionFinished(session_event("finished", payload, metadata))
        }
        "hermes.session.reset" => {
            EventBody::HermesSessionReset(session_event("reset", payload, metadata))
        }
        "hermes.agent.started" => {
            EventBody::HermesAgentStarted(agent_event("started", payload, metadata))
        }
        "hermes.agent.step" => EventBody::HermesAgentStep(agent_event("step", payload, metadata)),
        "hermes.agent.finished" => {
            EventBody::HermesAgentFinished(agent_event("finished", payload, metadata))
        }
        "hermes.agent.failed" => {
            EventBody::HermesAgentFailed(agent_event("failed", payload, metadata))
        }
        _ => EventBody::Custom(CustomEvent {
            kind: kind.to_string(),
            message: string_field(payload, "message")
                .or_else(|| string_field(payload, "summary"))
                .unwrap_or_else(|| kind.to_string()),
            payload: if payload.is_null() {
                None
            } else {
                Some(payload.clone())
            },
        }),
    })
}

fn git_commit_event(payload: &Value, metadata: &EventMetadata) -> Result<GitCommitEvent> {
    let sha = string_field(payload, "commit")
        .or_else(|| string_field(payload, "sha"))
        .unwrap_or_default();
    let sha = validate_commit_sha(&sha)?;
    let short_sha = string_field(payload, "short_commit")
        .or_else(|| string_field(payload, "short_sha"))
        .map(|value| validate_commit_sha(&value))
        .transpose()?
        .map(|value| short_sha(&value))
        .unwrap_or_else(|| short_sha(&sha));
    let summary = validate_single_line(
        "summary",
        string_field(payload, "summary")
            .as_deref()
            .unwrap_or_default(),
        MAX_SUMMARY_CHARS,
    )?;
    let author_name = validate_optional_single_line(
        "author_name",
        string_field(payload, "author_name").as_deref(),
        MAX_DISPLAY_FIELD_CHARS,
    )?;
    let author_email = validate_optional_single_line(
        "author_email",
        string_field(payload, "author_email").as_deref(),
        MAX_DISPLAY_FIELD_CHARS,
    )?;

    Ok(GitCommitEvent {
        repo: string_field(payload, "repo")
            .or_else(|| metadata.repo_name.clone())
            .unwrap_or_default(),
        branch: metadata.branch.clone().unwrap_or_default(),
        sha,
        short_sha,
        summary,
        repo_path: metadata.repo_path.clone(),
        worktree_path: metadata.worktree_path.clone(),
        author_name,
        author_email,
    })
}

fn git_branch_changed_event(payload: &Value, metadata: &EventMetadata) -> GitBranchChangedEvent {
    GitBranchChangedEvent {
        repo: string_field(payload, "repo")
            .or_else(|| metadata.repo_name.clone())
            .unwrap_or_default(),
        old_branch: string_field(payload, "old_branch").unwrap_or_default(),
        new_branch: string_field(payload, "new_branch")
            .or_else(|| metadata.branch.clone())
            .unwrap_or_default(),
        repo_path: metadata.repo_path.clone(),
        worktree_path: metadata.worktree_path.clone(),
    }
}

fn github_issue_event(payload: &Value, metadata: &EventMetadata) -> Result<GithubIssueEvent> {
    Ok(GithubIssueEvent {
        owner: required_display_field(payload, "owner")?,
        repo: required_repo(payload, metadata)?,
        number: github_number(payload)?,
        title: required_summary_field(payload, "title")?,
        author: validate_optional_display_field(
            "author",
            string_field(payload, "author").as_deref(),
        )?,
        url: validate_optional_url(string_field(payload, "url").as_deref())?,
    })
}

fn github_pull_request_event(
    payload: &Value,
    metadata: &EventMetadata,
) -> Result<GithubPullRequestEvent> {
    let sha = optional_github_sha(payload)?;
    let short_sha = optional_github_short_sha(payload, sha.as_deref())?;

    Ok(GithubPullRequestEvent {
        owner: required_display_field(payload, "owner")?,
        repo: required_repo(payload, metadata)?,
        number: github_number(payload)?,
        title: required_summary_field(payload, "title")?,
        branch: required_branch(payload, metadata)?,
        base_branch: validate_optional_display_field(
            "base_branch",
            string_field(payload, "base_branch").as_deref(),
        )?,
        sha,
        short_sha,
        author: validate_optional_display_field(
            "author",
            string_field(payload, "author").as_deref(),
        )?,
        url: validate_optional_url(string_field(payload, "url").as_deref())?,
    })
}

fn github_check_event(payload: &Value, metadata: &EventMetadata) -> Result<GithubCheckEvent> {
    let sha = optional_github_sha(payload)?;
    let short_sha = optional_github_short_sha(payload, sha.as_deref())?;

    Ok(GithubCheckEvent {
        owner: required_display_field(payload, "owner")?,
        repo: required_repo(payload, metadata)?,
        workflow: required_display_field(payload, "workflow")?,
        status: validate_status(
            string_field(payload, "status")
                .as_deref()
                .unwrap_or_default(),
        )?,
        branch: required_branch(payload, metadata)?,
        sha,
        short_sha,
        title: validate_optional_summary_field("title", string_field(payload, "title").as_deref())?,
        url: validate_optional_url(string_field(payload, "url").as_deref())?,
    })
}

fn github_release_event(payload: &Value, metadata: &EventMetadata) -> Result<GithubReleaseEvent> {
    Ok(GithubReleaseEvent {
        owner: required_display_field(payload, "owner")?,
        repo: required_repo(payload, metadata)?,
        tag: required_display_field(payload, "tag")?,
        title: validate_optional_summary_field("title", string_field(payload, "title").as_deref())?,
        author: validate_optional_display_field(
            "author",
            string_field(payload, "author").as_deref(),
        )?,
        url: validate_optional_url(string_field(payload, "url").as_deref())?,
    })
}

fn tmux_keyword_event(payload: &Value) -> Result<TmuxKeywordEvent> {
    let line = required_summary_field(payload, "line")?;
    let line_chars = u64_field(payload, "line_chars")
        .map(|value| value as usize)
        .unwrap_or_else(|| line.chars().count());

    Ok(TmuxKeywordEvent {
        session: required_display_field(payload, "session")?,
        window: validate_optional_display_field(
            "window",
            string_field(payload, "window").as_deref(),
        )?,
        pane: validate_optional_display_field("pane", string_field(payload, "pane").as_deref())?,
        keyword: required_display_field(payload, "keyword")?,
        line,
        line_chars,
    })
}

fn tmux_stale_event(payload: &Value) -> Result<TmuxStaleEvent> {
    let last_line = required_summary_field(payload, "last_line")?;
    let last_line_chars = u64_field(payload, "last_line_chars")
        .map(|value| value as usize)
        .unwrap_or_else(|| last_line.chars().count());

    Ok(TmuxStaleEvent {
        session: required_display_field(payload, "session")?,
        window: validate_optional_display_field(
            "window",
            string_field(payload, "window").as_deref(),
        )?,
        pane: required_display_field(payload, "pane")?,
        minutes: validate_positive_minutes(u64_field(payload, "minutes").unwrap_or_default())?,
        last_line,
        last_line_chars,
    })
}

fn cron_run_event(payload: &Value) -> Result<CronRunEvent> {
    let summary = required_summary_field(payload, "summary")?;
    let summary_chars = u64_field(payload, "summary_chars")
        .map(|value| value as usize)
        .unwrap_or_else(|| summary.chars().count());

    Ok(CronRunEvent {
        job_id: required_display_field(payload, "cron_job_id")?,
        schedule: required_display_field(payload, "cron_schedule")?,
        summary,
        summary_chars,
    })
}

fn required_repo(payload: &Value, metadata: &EventMetadata) -> Result<String> {
    let repo = string_field(payload, "repo").or_else(|| metadata.repo_name.clone());
    validate_display_field("repo", repo.as_deref().unwrap_or_default())
}

fn required_branch(payload: &Value, metadata: &EventMetadata) -> Result<String> {
    let branch = string_field(payload, "branch").or_else(|| metadata.branch.clone());
    validate_display_field("branch", branch.as_deref().unwrap_or_default())
}

fn required_display_field(payload: &Value, key: &str) -> Result<String> {
    validate_display_field(
        key,
        string_field(payload, key).as_deref().unwrap_or_default(),
    )
}

fn required_summary_field(payload: &Value, key: &str) -> Result<String> {
    validate_summary_field(
        key,
        string_field(payload, key).as_deref().unwrap_or_default(),
    )
}

fn github_number(payload: &Value) -> Result<u64> {
    validate_positive_number(u64_field(payload, "number").unwrap_or_default())
}

fn optional_github_sha(payload: &Value) -> Result<Option<String>> {
    string_field(payload, "commit")
        .or_else(|| string_field(payload, "sha"))
        .map(|value| validate_commit_sha(&value))
        .transpose()
}

fn optional_github_short_sha(payload: &Value, sha: Option<&str>) -> Result<Option<String>> {
    string_field(payload, "short_commit")
        .or_else(|| string_field(payload, "short_sha"))
        .map(|value| validate_commit_sha(&value).map(|value| github_short_sha(&value)))
        .transpose()
        .map(|value| value.or_else(|| sha.map(github_short_sha)))
}

fn validate_positive_minutes(minutes: u64) -> Result<u64> {
    if minutes == 0 {
        anyhow::bail!("minutes must be greater than 0");
    }
    Ok(minutes)
}

fn session_event(status: &str, payload: &Value, metadata: &EventMetadata) -> HermesSessionEvent {
    HermesSessionEvent {
        status: status.to_string(),
        session_id: metadata.session_id.clone(),
        platform: metadata.platform.clone(),
        project: metadata.project.clone(),
        message_chars: u64_field(payload, "message_chars"),
        response_chars: u64_field(payload, "response_chars"),
        has_message: bool_field(payload, "has_message"),
        has_response: bool_field(payload, "has_response"),
        success: bool_field(payload, "success"),
    }
}

fn agent_event(status: &str, payload: &Value, metadata: &EventMetadata) -> HermesAgentEvent {
    HermesAgentEvent {
        status: status.to_string(),
        agent_name: metadata.agent_name.clone(),
        session_id: metadata.session_id.clone(),
        platform: metadata.platform.clone(),
        project: metadata.project.clone(),
        step_name: string_field(payload, "step_name")
            .or_else(|| string_field(payload, "step"))
            .or_else(|| string_field(payload, "tool_name")),
        message_chars: u64_field(payload, "message_chars"),
        response_chars: u64_field(payload, "response_chars"),
        has_message: bool_field(payload, "has_message"),
        has_response: bool_field(payload, "has_response"),
        elapsed_secs: u64_field(payload, "elapsed_secs"),
        success: bool_field(payload, "success"),
        error_message: string_field(payload, "error_message")
            .or_else(|| string_field(payload, "error_summary"))
            .or_else(|| string_field(payload, "error")),
    }
}

fn priority_for(kind: &str) -> EventPriority {
    match kind {
        "hermes.agent.failed" => EventPriority::Critical,
        "tmux.stale" => EventPriority::High,
        "custom" => EventPriority::Low,
        _ if !kind.starts_with("hermes.") => EventPriority::Low,
        _ => EventPriority::Normal,
    }
}

fn source_for_kind(kind: &str) -> String {
    kind.split(['.', ':'])
        .next()
        .filter(|value| !value.is_empty())
        .unwrap_or("custom")
        .to_string()
}

fn is_agent_failure(payload: &Value) -> bool {
    if bool_field(payload, "success") == Some(false) {
        return true;
    }

    string_field(payload, "error_message")
        .or_else(|| string_field(payload, "error_summary"))
        .or_else(|| string_field(payload, "error"))
        .is_some()
        || string_field(payload, "status")
            .map(|status| {
                matches!(
                    status.to_ascii_lowercase().as_str(),
                    "failed" | "failure" | "error"
                )
            })
            .unwrap_or(false)
}

fn string_field(payload: &Value, key: &str) -> Option<String> {
    value_field(payload, key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn u64_field(payload: &Value, key: &str) -> Option<u64> {
    value_field(payload, key).and_then(Value::as_u64)
}

fn bool_field(payload: &Value, key: &str) -> Option<bool> {
    value_field(payload, key).and_then(Value::as_bool)
}

fn value_field<'a>(payload: &'a Value, key: &str) -> Option<&'a Value> {
    payload
        .get(key)
        .or_else(|| payload.get("context").and_then(|context| context.get(key)))
}

fn event_id_for(payload: &Value) -> Option<Uuid> {
    string_field(payload, "event_id").and_then(|value| Uuid::parse_str(&value).ok())
}

fn short_sha(commit: &str) -> String {
    commit.chars().take(7).collect()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::event::{EventBody, EventPriority};
    use crate::events::{IncomingEvent, MessageFormat};

    #[test]
    fn maps_all_hermes_gateway_hooks_to_canonical_bodies() {
        let cases = [
            (
                "gateway:startup",
                "hermes.gateway.started",
                "HermesGatewayStarted",
            ),
            (
                "session:start",
                "hermes.session.started",
                "HermesSessionStarted",
            ),
            (
                "session:end",
                "hermes.session.finished",
                "HermesSessionFinished",
            ),
            (
                "session:reset",
                "hermes.session.reset",
                "HermesSessionReset",
            ),
            ("agent:start", "hermes.agent.started", "HermesAgentStarted"),
            ("agent:step", "hermes.agent.step", "HermesAgentStep"),
            ("agent:end", "hermes.agent.finished", "HermesAgentFinished"),
        ];

        for (raw, canonical, expected_body) in cases {
            let envelope = from_incoming_event(&IncomingEvent::new(
                raw,
                json!({
                    "provider": "hermes",
                    "source": "gateway",
                    "context": {
                        "session_id": "synthetic-session-001",
                        "agent_name": "demo-agent"
                    }
                }),
            ))
            .unwrap();

            assert_eq!(envelope.source, "hermes", "source mismatch for {raw}");
            assert_eq!(
                envelope.canonical_kind(),
                canonical,
                "canonical kind mismatch for {raw}"
            );
            assert_eq!(
                body_variant_name(&envelope.body),
                expected_body,
                "typed body mismatch for {raw}"
            );
        }
    }

    #[test]
    fn converts_agent_start_fixture_with_metadata_and_route_hints() {
        let mut event: IncomingEvent =
            serde_json::from_str(include_str!("../../tests/fixtures/hermes/agent_start.json"))
                .unwrap();
        event.channel = Some("ops".to_string());
        event.mention = Some("@here".to_string());
        event.format = Some(MessageFormat::Alert);
        event.template = Some("agent {session_id}".to_string());

        let envelope = from_incoming_event(&event).unwrap();

        assert_ne!(envelope.id.to_string(), uuid::Uuid::nil().to_string());
        assert_eq!(envelope.source, "hermes");
        assert_eq!(envelope.canonical_kind(), "hermes.agent.started");
        assert_eq!(envelope.metadata.channel_hint.as_deref(), Some("ops"));
        assert_eq!(envelope.metadata.mention.as_deref(), Some("@here"));
        assert_eq!(envelope.metadata.format, Some(MessageFormat::Alert));
        assert_eq!(
            envelope.metadata.template.as_deref(),
            Some("agent {session_id}")
        );
        assert_eq!(envelope.metadata.provider.as_deref(), Some("hermes"));
        assert_eq!(envelope.metadata.source.as_deref(), Some("gateway"));
        assert_eq!(envelope.metadata.platform.as_deref(), Some("telegram"));
        assert_eq!(
            envelope.metadata.session_id.as_deref(),
            Some("synthetic-session-001")
        );
        assert_eq!(envelope.metadata.agent_name.as_deref(), Some("demo-agent"));
        assert_eq!(envelope.metadata.project.as_deref(), Some("hermes"));
        assert_eq!(envelope.metadata.priority, EventPriority::Normal);

        match envelope.body {
            EventBody::HermesAgentStarted(body) => {
                assert_eq!(body.status, "started");
                assert_eq!(body.agent_name.as_deref(), Some("demo-agent"));
                assert_eq!(body.session_id.as_deref(), Some("synthetic-session-001"));
                assert_eq!(body.platform.as_deref(), Some("telegram"));
                assert_eq!(body.project.as_deref(), Some("hermes"));
                assert_eq!(body.message_chars, Some(42));
                assert_eq!(body.has_message, Some(true));
            }
            other => panic!("expected HermesAgentStarted, got {other:?}"),
        }
    }

    #[test]
    fn converts_session_end_fixture_without_requiring_agent_fields() {
        let event: IncomingEvent =
            serde_json::from_str(include_str!("../../tests/fixtures/hermes/session_end.json"))
                .unwrap();

        let envelope = from_incoming_event(&event).unwrap();

        assert_eq!(envelope.canonical_kind(), "hermes.session.finished");
        assert_eq!(
            envelope.metadata.session_id.as_deref(),
            Some("synthetic-session-002")
        );

        match envelope.body {
            EventBody::HermesSessionFinished(body) => {
                assert_eq!(body.status, "finished");
                assert_eq!(body.session_id.as_deref(), Some("synthetic-session-002"));
                assert_eq!(body.response_chars, Some(128));
                assert_eq!(body.has_response, Some(true));
                assert_eq!(body.success, Some(true));
            }
            other => panic!("expected HermesSessionFinished, got {other:?}"),
        }
    }

    #[test]
    fn explicit_agent_end_failure_maps_to_failed_priority() {
        let envelope = from_incoming_event(&IncomingEvent::new(
            "agent:end",
            json!({
                "provider": "hermes",
                "source": "gateway",
                "context": {
                    "session_id": "synthetic-session-003",
                    "agent_name": "demo-agent",
                    "success": false,
                    "error_message": "synthetic failure"
                }
            }),
        ))
        .unwrap();

        assert_eq!(envelope.canonical_kind(), "hermes.agent.failed");
        assert_eq!(envelope.metadata.priority, EventPriority::Critical);
        match envelope.body {
            EventBody::HermesAgentFailed(body) => {
                assert_eq!(body.status, "failed");
                assert_eq!(body.success, Some(false));
                assert_eq!(body.error_message.as_deref(), Some("synthetic failure"));
            }
            other => panic!("expected HermesAgentFailed, got {other:?}"),
        }
    }

    #[test]
    fn agent_end_failure_status_is_case_insensitive() {
        let envelope = from_incoming_event(&IncomingEvent::new(
            "agent:end",
            json!({
                "provider": "hermes",
                "source": "gateway",
                "context": {
                    "session_id": "synthetic-session-004",
                    "agent_name": "demo-agent",
                    "status": "Failed"
                }
            }),
        ))
        .unwrap();

        assert_eq!(envelope.canonical_kind(), "hermes.agent.failed");
        assert_eq!(envelope.metadata.priority, EventPriority::Critical);
        assert!(matches!(envelope.body, EventBody::HermesAgentFailed(_)));
    }

    #[test]
    fn git_commit_event_converts_to_typed_body_with_route_metadata() {
        let event = IncomingEvent::new(
            "git.commit",
            json!({
                "repo": "hermeship",
                "repo_name": "hermeship",
                "repo_path": "/tmp/hermeship",
                "worktree_path": "/tmp/hermeship-worktree",
                "branch": "main",
                "commit": "1234567890abcdef1234567890abcdef12345678",
                "short_commit": "1234567",
                "summary": "ship git source",
                "author_name": "Synthetic Author",
                "author_email": "synthetic@example.invalid"
            }),
        );

        let envelope = from_incoming_event(&event).unwrap();

        assert_eq!(envelope.source, "git");
        assert_eq!(envelope.canonical_kind(), "git.commit");
        assert_eq!(envelope.metadata.priority, EventPriority::Low);
        assert_eq!(envelope.metadata.repo_name.as_deref(), Some("hermeship"));
        assert_eq!(
            envelope.metadata.repo_path.as_deref(),
            Some("/tmp/hermeship")
        );
        assert_eq!(
            envelope.metadata.worktree_path.as_deref(),
            Some("/tmp/hermeship-worktree")
        );
        assert_eq!(envelope.metadata.branch.as_deref(), Some("main"));

        match envelope.body {
            EventBody::GitCommit(body) => {
                assert_eq!(body.repo, "hermeship");
                assert_eq!(body.branch, "main");
                assert_eq!(body.sha, "1234567890abcdef1234567890abcdef12345678");
                assert_eq!(body.short_sha, "1234567");
                assert_eq!(body.summary, "ship git source");
                assert_eq!(body.repo_path.as_deref(), Some("/tmp/hermeship"));
                assert_eq!(
                    body.worktree_path.as_deref(),
                    Some("/tmp/hermeship-worktree")
                );
                assert_eq!(body.author_name.as_deref(), Some("Synthetic Author"));
                assert_eq!(
                    body.author_email.as_deref(),
                    Some("synthetic@example.invalid")
                );
            }
            other => panic!("expected GitCommit, got {other:?}"),
        }
    }

    #[test]
    fn git_branch_changed_event_converts_to_typed_body_with_new_branch_metadata() {
        let event = IncomingEvent::new(
            "git.branch-changed",
            json!({
                "repo": "hermeship",
                "repo_name": "hermeship",
                "repo_path": "/tmp/hermeship",
                "worktree_path": "/tmp/hermeship-worktree",
                "old_branch": "main",
                "new_branch": "codex/milestone-8-git",
                "branch": "codex/milestone-8-git"
            }),
        );

        let envelope = from_incoming_event(&event).unwrap();

        assert_eq!(envelope.source, "git");
        assert_eq!(envelope.canonical_kind(), "git.branch-changed");
        assert_eq!(envelope.metadata.priority, EventPriority::Low);
        assert_eq!(envelope.metadata.repo_name.as_deref(), Some("hermeship"));
        assert_eq!(
            envelope.metadata.branch.as_deref(),
            Some("codex/milestone-8-git")
        );

        match envelope.body {
            EventBody::GitBranchChanged(body) => {
                assert_eq!(body.repo, "hermeship");
                assert_eq!(body.old_branch, "main");
                assert_eq!(body.new_branch, "codex/milestone-8-git");
                assert_eq!(body.repo_path.as_deref(), Some("/tmp/hermeship"));
                assert_eq!(
                    body.worktree_path.as_deref(),
                    Some("/tmp/hermeship-worktree")
                );
            }
            other => panic!("expected GitBranchChanged, got {other:?}"),
        }
    }

    #[test]
    fn git_commit_event_rejects_invalid_sha_and_multiline_summary() {
        let invalid_sha = from_incoming_event(&IncomingEvent::new(
            "git.commit",
            json!({
                "repo": "hermeship",
                "branch": "main",
                "commit": "not-a-sha",
                "summary": "ship git source"
            }),
        ))
        .unwrap_err()
        .to_string();
        assert!(invalid_sha.contains("commit must be 7-64 hex characters"));

        let multiline = from_incoming_event(&IncomingEvent::new(
            "git.commit",
            json!({
                "repo": "hermeship",
                "branch": "main",
                "commit": "1234567890abcdef1234567890abcdef12345678",
                "summary": "ship git source\nsynthetic diff should not render"
            }),
        ))
        .unwrap_err()
        .to_string();
        assert!(multiline.contains("summary must be a single line"));
    }

    #[test]
    fn github_issue_pr_check_and_release_events_convert_to_typed_bodies() {
        let issue = from_incoming_event(&IncomingEvent::new(
            "github.issue-opened",
            json!({
                "owner": "posp",
                "repo": "hermeship",
                "repo_name": "hermeship",
                "number": 42,
                "title": "Add deterministic GitHub source",
                "author": "synthetic-user",
                "url": "https://github.example.invalid/posp/hermeship/issues/42"
            }),
        ))
        .unwrap();
        assert_eq!(issue.source, "github");
        assert_eq!(issue.canonical_kind(), "github.issue-opened");
        assert_eq!(issue.metadata.priority, EventPriority::Low);
        assert_eq!(issue.metadata.repo_name.as_deref(), Some("hermeship"));
        match issue.body {
            EventBody::GithubIssue(body) => {
                assert_eq!(body.owner, "posp");
                assert_eq!(body.repo, "hermeship");
                assert_eq!(body.number, 42);
                assert_eq!(body.title, "Add deterministic GitHub source");
                assert_eq!(body.author.as_deref(), Some("synthetic-user"));
            }
            other => panic!("expected GithubIssue, got {other:?}"),
        }

        let pr = from_incoming_event(&IncomingEvent::new(
            "github.pr-opened",
            json!({
                "owner": "posp",
                "repo": "hermeship",
                "repo_name": "hermeship",
                "number": 17,
                "title": "Ship GitHub source",
                "branch": "codex/milestone-8-github",
                "base_branch": "main",
                "commit": "1234567890abcdef1234567890abcdef12345678",
                "short_commit": "1234567"
            }),
        ))
        .unwrap();
        assert_eq!(pr.canonical_kind(), "github.pr-opened");
        assert_eq!(
            pr.metadata.branch.as_deref(),
            Some("codex/milestone-8-github")
        );
        match pr.body {
            EventBody::GithubPullRequest(body) => {
                assert_eq!(body.number, 17);
                assert_eq!(body.branch, "codex/milestone-8-github");
                assert_eq!(body.base_branch.as_deref(), Some("main"));
                assert_eq!(body.short_sha.as_deref(), Some("1234567"));
            }
            other => panic!("expected GithubPullRequest, got {other:?}"),
        }

        let check = from_incoming_event(&IncomingEvent::new(
            "github.check-failed",
            json!({
                "owner": "posp",
                "repo": "hermeship",
                "repo_name": "hermeship",
                "workflow": "ci",
                "status": "failure",
                "branch": "main",
                "commit": "abcdef1234567890abcdef1234567890abcdef12",
                "short_commit": "abcdef1",
                "title": "cargo test failed"
            }),
        ))
        .unwrap();
        assert_eq!(check.canonical_kind(), "github.check-failed");
        assert_eq!(check.metadata.branch.as_deref(), Some("main"));
        match check.body {
            EventBody::GithubCheck(body) => {
                assert_eq!(body.workflow, "ci");
                assert_eq!(body.status, "failure");
                assert_eq!(body.title.as_deref(), Some("cargo test failed"));
                assert_eq!(body.short_sha.as_deref(), Some("abcdef1"));
            }
            other => panic!("expected GithubCheck, got {other:?}"),
        }

        let release = from_incoming_event(&IncomingEvent::new(
            "github.release-published",
            json!({
                "owner": "posp",
                "repo": "hermeship",
                "repo_name": "hermeship",
                "tag": "v0.1.0",
                "title": "Hermeship v0.1.0",
                "author": "synthetic-user"
            }),
        ))
        .unwrap();
        assert_eq!(release.canonical_kind(), "github.release-published");
        match release.body {
            EventBody::GithubRelease(body) => {
                assert_eq!(body.tag, "v0.1.0");
                assert_eq!(body.title.as_deref(), Some("Hermeship v0.1.0"));
                assert_eq!(body.author.as_deref(), Some("synthetic-user"));
            }
            other => panic!("expected GithubRelease, got {other:?}"),
        }
    }

    #[test]
    fn github_events_reject_invalid_numbers_statuses_and_multiline_titles() {
        let invalid_number = from_incoming_event(&IncomingEvent::new(
            "github.issue-opened",
            json!({
                "owner": "posp",
                "repo": "hermeship",
                "number": 0,
                "title": "Add deterministic GitHub source"
            }),
        ))
        .unwrap_err()
        .to_string();
        assert!(invalid_number.contains("number must be greater than 0"));

        let multiline = from_incoming_event(&IncomingEvent::new(
            "github.pr-opened",
            json!({
                "owner": "posp",
                "repo": "hermeship",
                "number": 17,
                "title": "Ship GitHub source\nfull PR body should not render",
                "branch": "codex/milestone-8-github"
            }),
        ))
        .unwrap_err()
        .to_string();
        assert!(multiline.contains("title must be a single line"));

        let invalid_status = from_incoming_event(&IncomingEvent::new(
            "github.check-failed",
            json!({
                "owner": "posp",
                "repo": "hermeship",
                "workflow": "ci",
                "status": "maybe",
                "branch": "main"
            }),
        ))
        .unwrap_err()
        .to_string();
        assert!(invalid_status.contains("status must be one of"));
    }

    #[test]
    fn tmux_keyword_and_stale_events_convert_to_typed_bodies() {
        let keyword = from_incoming_event(&IncomingEvent::new(
            "tmux.keyword",
            json!({
                "session": "hermes-agent",
                "window": "main",
                "pane": "%1",
                "keyword": "FAILED",
                "line": "build FAILED at deterministic fixture",
                "line_chars": 37
            }),
        ))
        .unwrap();

        assert_eq!(keyword.source, "tmux");
        assert_eq!(keyword.canonical_kind(), "tmux.keyword");
        assert_eq!(keyword.metadata.priority, EventPriority::Low);
        match keyword.body {
            EventBody::TmuxKeyword(body) => {
                assert_eq!(body.session, "hermes-agent");
                assert_eq!(body.window.as_deref(), Some("main"));
                assert_eq!(body.pane.as_deref(), Some("%1"));
                assert_eq!(body.keyword, "FAILED");
                assert_eq!(body.line, "build FAILED at deterministic fixture");
                assert_eq!(body.line_chars, 37);
            }
            other => panic!("expected TmuxKeyword, got {other:?}"),
        }

        let stale = from_incoming_event(&IncomingEvent::new(
            "tmux.stale",
            json!({
                "session": "hermes-agent",
                "window": "main",
                "pane": "%2",
                "minutes": 15,
                "last_line": "waiting for agent output",
                "last_line_chars": 24
            }),
        ))
        .unwrap();

        assert_eq!(stale.source, "tmux");
        assert_eq!(stale.canonical_kind(), "tmux.stale");
        assert_eq!(stale.metadata.priority, EventPriority::High);
        match stale.body {
            EventBody::TmuxStale(body) => {
                assert_eq!(body.session, "hermes-agent");
                assert_eq!(body.window.as_deref(), Some("main"));
                assert_eq!(body.pane, "%2");
                assert_eq!(body.minutes, 15);
                assert_eq!(body.last_line, "waiting for agent output");
                assert_eq!(body.last_line_chars, 24);
            }
            other => panic!("expected TmuxStale, got {other:?}"),
        }
    }

    #[test]
    fn cron_run_event_converts_to_typed_body_with_route_metadata() {
        let event = IncomingEvent::new(
            "cron.run",
            json!({
                "cron_job_id": "dev-followup",
                "cron_schedule": "*/30 * * * *",
                "summary": "check open PRs and blockers",
                "summary_chars": 27
            }),
        );

        let envelope = from_incoming_event(&event).unwrap();

        assert_eq!(envelope.source, "cron");
        assert_eq!(envelope.canonical_kind(), "cron.run");
        assert_eq!(envelope.metadata.priority, EventPriority::Low);
        match envelope.body {
            EventBody::CronRun(body) => {
                assert_eq!(body.job_id, "dev-followup");
                assert_eq!(body.schedule, "*/30 * * * *");
                assert_eq!(body.summary, "check open PRs and blockers");
                assert_eq!(body.summary_chars, 27);
            }
            other => panic!("expected CronRun, got {other:?}"),
        }
    }

    #[test]
    fn cron_run_event_rejects_empty_or_multiline_fields() {
        let missing_job = from_incoming_event(&IncomingEvent::new(
            "cron.run",
            json!({
                "cron_schedule": "*/30 * * * *",
                "summary": "check open PRs"
            }),
        ))
        .unwrap_err()
        .to_string();
        assert!(missing_job.contains("cron_job_id must not be empty"));

        let multiline_summary = from_incoming_event(&IncomingEvent::new(
            "cron.run",
            json!({
                "cron_job_id": "dev-followup",
                "cron_schedule": "*/30 * * * *",
                "summary": "check open PRs\nfull cron body should not render"
            }),
        ))
        .unwrap_err()
        .to_string();
        assert!(multiline_summary.contains("summary must be a single line"));
    }

    #[test]
    fn tmux_events_reject_empty_multiline_and_zero_minute_fields() {
        let empty_session = from_incoming_event(&IncomingEvent::new(
            "tmux.keyword",
            json!({
                "session": "",
                "keyword": "FAILED",
                "line": "build failed"
            }),
        ))
        .unwrap_err()
        .to_string();
        assert!(empty_session.contains("session must not be empty"));

        let multiline = from_incoming_event(&IncomingEvent::new(
            "tmux.keyword",
            json!({
                "session": "hermes-agent",
                "keyword": "FAILED",
                "line": "build failed\nfull pane capture should not render"
            }),
        ))
        .unwrap_err()
        .to_string();
        assert!(multiline.contains("line must be a single line"));

        let zero_minutes = from_incoming_event(&IncomingEvent::new(
            "tmux.stale",
            json!({
                "session": "hermes-agent",
                "pane": "%2",
                "minutes": 0,
                "last_line": "waiting"
            }),
        ))
        .unwrap_err()
        .to_string();
        assert!(zero_minutes.contains("minutes must be greater than 0"));
    }

    #[test]
    fn unknown_event_becomes_custom_without_losing_payload() {
        let event = IncomingEvent::new(
            "plugin.custom",
            json!({
                "message": "synthetic custom event",
                "extra": true
            }),
        );

        let envelope = from_incoming_event(&event).unwrap();

        assert_eq!(envelope.source, "plugin");
        assert_eq!(envelope.canonical_kind(), "plugin.custom");
        assert_eq!(envelope.metadata.priority, EventPriority::Low);
        match envelope.body {
            EventBody::Custom(body) => {
                assert_eq!(body.kind, "plugin.custom");
                assert_eq!(body.message, "synthetic custom event");
                assert_eq!(body.payload.unwrap()["extra"], json!(true));
            }
            other => panic!("expected Custom, got {other:?}"),
        }
    }

    #[test]
    fn missing_session_id_degrades_without_error() {
        let envelope = from_incoming_event(&IncomingEvent::new(
            "agent:start",
            json!({
                "provider": "hermes",
                "source": "gateway",
                "context": {
                    "agent_name": "demo-agent",
                    "platform": "discord"
                }
            }),
        ))
        .unwrap();

        assert_eq!(envelope.canonical_kind(), "hermes.agent.started");
        assert_eq!(envelope.metadata.session_id, None);
        match envelope.body {
            EventBody::HermesAgentStarted(body) => {
                assert_eq!(body.session_id, None);
                assert_eq!(body.agent_name.as_deref(), Some("demo-agent"));
            }
            other => panic!("expected HermesAgentStarted, got {other:?}"),
        }
    }

    fn body_variant_name(body: &EventBody) -> &'static str {
        match body {
            EventBody::GitCommit(_) => "GitCommit",
            EventBody::GitBranchChanged(_) => "GitBranchChanged",
            EventBody::GithubIssue(_) => "GithubIssue",
            EventBody::GithubPullRequest(_) => "GithubPullRequest",
            EventBody::GithubCheck(_) => "GithubCheck",
            EventBody::GithubRelease(_) => "GithubRelease",
            EventBody::TmuxKeyword(_) => "TmuxKeyword",
            EventBody::TmuxStale(_) => "TmuxStale",
            EventBody::CronRun(_) => "CronRun",
            EventBody::HermesGatewayStarted(_) => "HermesGatewayStarted",
            EventBody::HermesSessionStarted(_) => "HermesSessionStarted",
            EventBody::HermesSessionFinished(_) => "HermesSessionFinished",
            EventBody::HermesSessionReset(_) => "HermesSessionReset",
            EventBody::HermesAgentStarted(_) => "HermesAgentStarted",
            EventBody::HermesAgentStep(_) => "HermesAgentStep",
            EventBody::HermesAgentFinished(_) => "HermesAgentFinished",
            EventBody::HermesAgentFailed(_) => "HermesAgentFailed",
            EventBody::Custom(_) => "Custom",
        }
    }
}
