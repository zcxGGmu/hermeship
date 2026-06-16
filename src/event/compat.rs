use anyhow::Result;
use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::event::{
    CustomEvent, EventBody, EventEnvelope, EventMetadata, EventPriority, GitBranchChangedEvent,
    GitCommitEvent, HermesAgentEvent, HermesGatewayEvent, HermesSessionEvent,
};
use crate::events::IncomingEvent;
use crate::source::git::{
    MAX_DISPLAY_FIELD_CHARS, MAX_SUMMARY_CHARS, validate_commit_sha, validate_optional_single_line,
    validate_single_line,
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
