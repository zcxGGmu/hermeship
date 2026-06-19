use anyhow::Result;
use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::event::{
    CronRunEvent, CustomEvent, EventBody, EventEnvelope, EventMetadata, EventPriority,
    GitBranchChangedEvent, GitCommitEvent, GithubCheckEvent, GithubIssueEvent,
    GithubPullRequestEvent, GithubReleaseEvent, HermesAgentEvent, HermesGatewayEvent,
    HermesObserverEvent, HermesSessionEvent, ObserverFieldValue, TmuxKeywordEvent, TmuxStaleEvent,
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
    let provider = if canonical_kind.starts_with("hermes.observer.") {
        Some("hermes".to_string())
    } else {
        string_field(&event.payload, "provider")
    };
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
        provider,
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
        observer if observer.starts_with("hermes.observer.") => {
            EventBody::HermesObserver(observer_event(observer, payload))
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

fn observer_event(kind: &str, payload: &Value) -> HermesObserverEvent {
    let (category, action) = observer_category_action(kind);
    let mut fields = std::collections::BTreeMap::new();

    for key in OBSERVER_STRING_FIELDS {
        if let Some(value) = observer_string_field(payload, key) {
            fields.insert((*key).to_string(), ObserverFieldValue::String(value));
        }
    }
    for key in OBSERVER_CODE_FIELDS {
        insert_observer_code_or_summary(&mut fields, payload, key);
    }
    for key in OBSERVER_STATUS_FIELDS {
        insert_observer_status_or_summary(&mut fields, payload, key);
    }
    for key in OBSERVER_SUMMARY_TEXT_FIELDS {
        insert_observer_text_summary(&mut fields, payload, key);
    }
    for key in OBSERVER_NUMBER_FIELDS {
        if let Some(value) = u64_field(payload, key) {
            fields.insert((*key).to_string(), ObserverFieldValue::Number(value));
        }
    }
    for key in OBSERVER_BOOL_FIELDS {
        if let Some(value) = bool_field(payload, key) {
            fields.insert((*key).to_string(), ObserverFieldValue::Bool(value));
        }
    }
    for key in OBSERVER_STRING_LIST_FIELDS {
        if let Some(values) = string_list_field(payload, key) {
            fields.insert((*key).to_string(), ObserverFieldValue::StringList(values));
        }
    }

    if let Some(error_summary) = string_field(payload, "error_summary")
        .or_else(|| string_field(payload, "error_message"))
        .or_else(|| string_field(payload, "error"))
    {
        fields.insert(
            "error_message_chars".to_string(),
            ObserverFieldValue::Number(error_summary.chars().count() as u64),
        );
        fields.insert(
            "has_error_message".to_string(),
            ObserverFieldValue::Bool(!error_summary.trim().is_empty()),
        );
    }

    HermesObserverEvent {
        kind: kind.to_string(),
        category,
        action,
        schema_version: u64_field(payload, "observer_schema_version"),
        fields,
    }
}

const OBSERVER_STRING_FIELDS: &[&str] = &[
    "session_id",
    "task_id",
    "turn_id",
    "api_request_id",
    "platform",
    "model",
    "provider",
    "api_mode",
    "response_model",
    "finish_reason",
    "tool_call_id",
    "tool_name",
    "surface",
    "pattern_key",
    "choice",
    "parent_session_id",
    "parent_turn_id",
    "parent_subagent_id",
    "child_session_id",
    "child_subagent_id",
    "child_role",
];

const OBSERVER_CODE_FIELDS: &[&str] = &["error_type"];
const OBSERVER_STATUS_FIELDS: &[&str] = &["status", "child_status"];
const OBSERVER_SUMMARY_TEXT_FIELDS: &[&str] = &["session_key", "reason"];
const OBSERVER_NUMBER_FIELDS: &[&str] = &[
    "api_call_count",
    "message_count",
    "tool_count",
    "approx_input_tokens",
    "request_char_count",
    "max_tokens",
    "duration_ms",
    "api_duration",
    "assistant_content_chars",
    "assistant_tool_call_count",
    "response_chars",
    "message_chars",
    "history_count",
    "arg_key_count",
    "arg_chars",
    "result_chars",
    "session_key_chars",
    "reason_chars",
    "status_chars",
    "child_status_chars",
    "error_type_chars",
    "pattern_key_count",
    "description_chars",
    "command_chars",
    "child_goal_chars",
    "child_summary_chars",
    "input_tokens",
    "output_tokens",
    "total_tokens",
    "prompt_tokens",
    "completion_tokens",
];

const OBSERVER_BOOL_FIELDS: &[&str] = &[
    "completed",
    "interrupted",
    "is_first_turn",
    "has_session_key",
    "has_reason",
    "has_status",
    "has_child_status",
    "has_error_type",
];

const OBSERVER_STRING_LIST_FIELDS: &[&str] = &["arg_keys", "pattern_keys"];
const MAX_OBSERVER_TEXT_CHARS: usize = 240;
const MAX_OBSERVER_CODE_CHARS: usize = 64;
const MAX_OBSERVER_LIST_VALUES: usize = 16;
const MAX_OBSERVER_LIST_VALUE_CHARS: usize = 64;

fn observer_category_action(kind: &str) -> (String, String) {
    let mut parts = kind
        .strip_prefix("hermes.observer.")
        .unwrap_or(kind)
        .split('.')
        .filter(|part| !part.is_empty());
    let category = parts.next().unwrap_or("unknown").to_string();
    let action = parts.collect::<Vec<_>>().join(".");
    let action = if action.is_empty() {
        "unknown".to_string()
    } else {
        action
    };
    (category, action)
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

fn observer_string_field(payload: &Value, key: &str) -> Option<String> {
    string_field(payload, key).map(|value| observer_bounded_text(&value, MAX_OBSERVER_TEXT_CHARS))
}

fn insert_observer_code_or_summary(
    fields: &mut std::collections::BTreeMap<String, ObserverFieldValue>,
    payload: &Value,
    key: &str,
) {
    let Some(value) = string_field(payload, key) else {
        return;
    };
    if let Some(code) = observer_code_value(&value) {
        fields.insert(key.to_string(), ObserverFieldValue::String(code));
    } else {
        insert_observer_text_summary_value(fields, key, &value);
    }
}

fn insert_observer_status_or_summary(
    fields: &mut std::collections::BTreeMap<String, ObserverFieldValue>,
    payload: &Value,
    key: &str,
) {
    let Some(value) = string_field(payload, key) else {
        return;
    };
    if let Some(status) = observer_status_value(&value) {
        fields.insert(key.to_string(), ObserverFieldValue::String(status));
    } else {
        insert_observer_text_summary_value(fields, key, &value);
    }
}

fn insert_observer_text_summary(
    fields: &mut std::collections::BTreeMap<String, ObserverFieldValue>,
    payload: &Value,
    key: &str,
) {
    if let Some(value) = string_field(payload, key) {
        insert_observer_text_summary_value(fields, key, &value);
    }
}

fn insert_observer_text_summary_value(
    fields: &mut std::collections::BTreeMap<String, ObserverFieldValue>,
    key: &str,
    value: &str,
) {
    fields.insert(
        format!("{key}_chars"),
        ObserverFieldValue::Number(value.chars().count() as u64),
    );
    fields.insert(
        format!("has_{key}"),
        ObserverFieldValue::Bool(!value.trim().is_empty()),
    );
}

fn observer_code_value(value: &str) -> Option<String> {
    let bounded = observer_bounded_text(value, MAX_OBSERVER_CODE_CHARS);
    let valid = bounded == value
        && bounded.chars().any(|ch| ch.is_ascii_alphanumeric())
        && bounded.ends_with("Error")
        && bounded
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
        && bounded
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_alphabetic() || ch == '_');
    valid.then_some(bounded)
}

fn observer_status_value(value: &str) -> Option<String> {
    let normalized = value.trim().to_ascii_lowercase().replace('_', "-");
    let canonical = match normalized.as_str() {
        "ok" | "success" | "succeeded" => "ok",
        "done" | "completed" | "complete" => "done",
        "failed" | "failure" | "error" => "failed",
        "interrupted" | "cancelled" | "canceled" => "interrupted",
        "timeout" | "timed-out" => "timeout",
        "running" | "started" | "pending" | "skipped" => normalized.as_str(),
        _ => return None,
    };
    Some(canonical.to_string())
}

fn observer_bounded_text(value: &str, limit: usize) -> String {
    value
        .replace(['\n', '\r'], " ")
        .chars()
        .take(limit)
        .collect()
}

fn string_list_field(payload: &Value, key: &str) -> Option<Vec<String>> {
    let values = value_field(payload, key)?.as_array()?;
    let values = values
        .iter()
        .filter_map(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .take(MAX_OBSERVER_LIST_VALUES)
        .map(|value| observer_bounded_text(value, MAX_OBSERVER_LIST_VALUE_CHARS))
        .collect::<Vec<_>>();
    (!values.is_empty()).then_some(values)
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
    use crate::event::{EventBody, EventPriority, ObserverFieldValue};
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
    fn observer_events_convert_to_typed_body_with_safe_fields_only() {
        let envelope = from_incoming_event(&IncomingEvent::new(
            "hermes.observer.tool.started",
            json!({
                "provider": "hermes",
                "source": "plugin",
                "observer_schema_version": 1,
                "session_id": "session-1",
                "task_id": "task-1",
                "turn_id": "turn-1",
                "api_request_id": "api-1",
                "tool_call_id": "tool-1",
                "tool_name": "terminal",
                "arg_keys": ["mode", "path"],
                "arg_key_count": 2,
                "arg_chars": 37,
                "command": "RAW_COMMAND_DO_NOT_FORWARD",
                "tool_result": "RAW_TOOL_RESULT_DO_NOT_FORWARD",
                "request": {"body": "RAW_REQUEST_DO_NOT_FORWARD"}
            }),
        ))
        .unwrap();

        assert_eq!(envelope.source, "hermes");
        assert_eq!(envelope.canonical_kind(), "hermes.observer.tool.started");
        assert_eq!(envelope.metadata.provider.as_deref(), Some("hermes"));
        assert_eq!(envelope.metadata.source.as_deref(), Some("plugin"));
        assert_eq!(envelope.metadata.session_id.as_deref(), Some("session-1"));
        assert_eq!(envelope.metadata.priority, EventPriority::Normal);

        match envelope.body {
            EventBody::HermesObserver(body) => {
                assert_eq!(body.kind, "hermes.observer.tool.started");
                assert_eq!(body.category, "tool");
                assert_eq!(body.action, "started");
                assert_eq!(
                    body.fields.get("tool_name"),
                    Some(&ObserverFieldValue::String("terminal".to_string()))
                );
                assert_eq!(
                    body.fields.get("arg_keys"),
                    Some(&ObserverFieldValue::StringList(vec![
                        "mode".to_string(),
                        "path".to_string()
                    ]))
                );
                assert_eq!(
                    body.fields.get("arg_chars"),
                    Some(&ObserverFieldValue::Number(37))
                );
                assert!(!body.fields.contains_key("command"));
                assert!(!body.fields.contains_key("tool_result"));
                assert!(!body.fields.contains_key("request"));
            }
            other => panic!("expected HermesObserver, got {other:?}"),
        }
    }

    #[test]
    fn observer_error_and_subagent_events_store_summaries_not_raw_text() {
        let api_failed = from_incoming_event(&IncomingEvent::new(
            "hermes.observer.api.request.failed",
            json!({
                "provider": "hermes",
                "source": "plugin",
                "observer_schema_version": 1,
                "session_id": "session-1",
                "api_request_id": "api-1",
                "model": "synthetic-model",
                "api_mode": "chat",
                "api_call_count": 3,
                "error_type": "RuntimeError",
                "error_message": "RAW_ERROR_MESSAGE_DO_NOT_FORWARD",
                "error_summary": "bounded error summary",
                "duration_ms": 7,
                "response": "RAW_RESPONSE_DO_NOT_FORWARD"
            }),
        ))
        .unwrap();

        match api_failed.body {
            EventBody::HermesObserver(body) => {
                assert_eq!(body.category, "api");
                assert_eq!(body.action, "request.failed");
                assert_eq!(
                    body.fields.get("error_message_chars"),
                    Some(&ObserverFieldValue::Number(21))
                );
                assert_eq!(
                    body.fields.get("has_error_message"),
                    Some(&ObserverFieldValue::Bool(true))
                );
                assert!(!body.fields.values().any(|value| {
                    match value {
                        ObserverFieldValue::String(value) => {
                            value.contains("RAW_ERROR") || value.contains("bounded error summary")
                        }
                        ObserverFieldValue::StringList(values) => values
                            .iter()
                            .any(|value| value.contains("RAW_ERROR") || value.contains("bounded")),
                        _ => false,
                    }
                }));
                assert!(!body.fields.contains_key("response"));
            }
            other => panic!("expected HermesObserver, got {other:?}"),
        }

        let subagent_finished = from_incoming_event(&IncomingEvent::new(
            "hermes.observer.subagent.finished",
            json!({
                "provider": "hermes",
                "source": "plugin",
                "parent_session_id": "parent-session",
                "child_session_id": "child-session",
                "child_role": "reviewer",
                "child_status": "done",
                "child_summary": "RAW_CHILD_SUMMARY_DO_NOT_FORWARD",
                "child_summary_chars": 33,
                "duration_ms": 9
            }),
        ))
        .unwrap();

        match subagent_finished.body {
            EventBody::HermesObserver(body) => {
                assert_eq!(body.category, "subagent");
                assert_eq!(body.action, "finished");
                assert_eq!(
                    body.fields.get("child_role"),
                    Some(&ObserverFieldValue::String("reviewer".to_string()))
                );
                assert_eq!(
                    body.fields.get("child_summary_chars"),
                    Some(&ObserverFieldValue::Number(33))
                );
                assert!(!body.fields.contains_key("child_summary"));
            }
            other => panic!("expected HermesObserver, got {other:?}"),
        }
    }

    #[test]
    fn observer_sensitive_and_free_text_fields_store_only_summaries() {
        let error_type = format!("RuntimeError\n{}", "raw exception text ".repeat(20));
        let reason = format!(
            "operator requested reset\n{}",
            "raw reason text ".repeat(20)
        );
        let status = format!("failed\n{}", "raw status text ".repeat(20));
        let session_key = "sk-secret-session-key-should-not-forward";
        let envelope = from_incoming_event(&IncomingEvent::new(
            "hermes.observer.approval.requested",
            json!({
                "provider": "hermes",
                "source": "plugin",
                "observer_schema_version": 1,
                "session_key": session_key,
                "surface": "terminal",
                "pattern_key": "shell-command",
                "turn_id": "turn-approval",
                "tool_call_id": "tool-approval",
                "error_type": error_type,
                "reason": reason,
                "status": status
            }),
        ))
        .unwrap();

        match envelope.body {
            EventBody::HermesObserver(body) => {
                assert!(!body.fields.contains_key("session_key"));
                assert!(!body.fields.contains_key("error_type"));
                assert!(!body.fields.contains_key("reason"));
                assert!(!body.fields.contains_key("status"));
                assert_eq!(
                    body.fields.get("session_key_chars"),
                    Some(&ObserverFieldValue::Number(
                        session_key.chars().count() as u64
                    ))
                );
                assert_eq!(
                    body.fields.get("has_session_key"),
                    Some(&ObserverFieldValue::Bool(true))
                );
                assert_eq!(
                    body.fields.get("error_type_chars"),
                    Some(&ObserverFieldValue::Number(
                        error_type.trim().chars().count() as u64
                    ))
                );
                assert_eq!(
                    body.fields.get("reason_chars"),
                    Some(&ObserverFieldValue::Number(
                        reason.trim().chars().count() as u64
                    ))
                );
                assert_eq!(
                    body.fields.get("status_chars"),
                    Some(&ObserverFieldValue::Number(
                        status.trim().chars().count() as u64
                    ))
                );
                assert!(!format!("{:?}", body.fields).contains("raw exception text"));
                assert!(!format!("{:?}", body.fields).contains("raw reason text"));
                assert!(!format!("{:?}", body.fields).contains("raw status text"));
                assert!(!format!("{:?}", body.fields).contains(session_key));
            }
            other => panic!("expected HermesObserver, got {other:?}"),
        }
    }

    #[test]
    fn observer_secret_shaped_error_type_is_summarized() {
        let secret_error_type = "sk-secret-session-key";
        let envelope = from_incoming_event(&IncomingEvent::new(
            "hermes.observer.api.request.failed",
            json!({
                "provider": "hermes",
                "source": "plugin",
                "session_id": "session-1",
                "model": "synthetic-model",
                "error_type": secret_error_type
            }),
        ))
        .unwrap();

        match envelope.body {
            EventBody::HermesObserver(body) => {
                assert!(!body.fields.contains_key("error_type"));
                assert_eq!(
                    body.fields.get("error_type_chars"),
                    Some(&ObserverFieldValue::Number(secret_error_type.len() as u64))
                );
                assert_eq!(
                    body.fields.get("has_error_type"),
                    Some(&ObserverFieldValue::Bool(true))
                );
                assert!(!format!("{:?}", body.fields).contains(secret_error_type));
            }
            other => panic!("expected HermesObserver, got {other:?}"),
        }
    }

    #[test]
    fn observer_provider_metadata_is_hermes_even_when_body_provider_is_api_provider() {
        let envelope = from_incoming_event(&IncomingEvent::new(
            "hermes.observer.api.request.started",
            json!({
                "provider": "synthetic-api-provider",
                "source": "plugin",
                "session_id": "session-1",
                "model": "synthetic-model",
                "api_mode": "chat"
            }),
        ))
        .unwrap();

        assert_eq!(envelope.metadata.provider.as_deref(), Some("hermes"));
        match envelope.body {
            EventBody::HermesObserver(body) => {
                assert_eq!(
                    body.fields.get("provider"),
                    Some(&ObserverFieldValue::String(
                        "synthetic-api-provider".to_string()
                    ))
                );
            }
            other => panic!("expected HermesObserver, got {other:?}"),
        }
    }

    #[test]
    fn observer_preserves_plugin_generated_sensitive_field_summaries() {
        let envelope = from_incoming_event(&IncomingEvent::new(
            "hermes.observer.approval.requested",
            json!({
                "provider": "hermes",
                "source": "plugin",
                "session_key_chars": 39,
                "has_session_key": true,
                "reason_chars": 21,
                "has_reason": true,
                "status_chars": 17,
                "has_status": true,
                "error_type_chars": 29,
                "has_error_type": true,
                "surface": "terminal"
            }),
        ))
        .unwrap();

        match envelope.body {
            EventBody::HermesObserver(body) => {
                assert_eq!(
                    body.fields.get("session_key_chars"),
                    Some(&ObserverFieldValue::Number(39))
                );
                assert_eq!(
                    body.fields.get("has_session_key"),
                    Some(&ObserverFieldValue::Bool(true))
                );
                assert_eq!(
                    body.fields.get("reason_chars"),
                    Some(&ObserverFieldValue::Number(21))
                );
                assert_eq!(
                    body.fields.get("has_reason"),
                    Some(&ObserverFieldValue::Bool(true))
                );
                assert_eq!(
                    body.fields.get("status_chars"),
                    Some(&ObserverFieldValue::Number(17))
                );
                assert_eq!(
                    body.fields.get("has_status"),
                    Some(&ObserverFieldValue::Bool(true))
                );
                assert_eq!(
                    body.fields.get("error_type_chars"),
                    Some(&ObserverFieldValue::Number(29))
                );
                assert_eq!(
                    body.fields.get("has_error_type"),
                    Some(&ObserverFieldValue::Bool(true))
                );
                assert!(!body.fields.contains_key("session_key"));
                assert!(!body.fields.contains_key("reason"));
                assert!(!body.fields.contains_key("status"));
                assert!(!body.fields.contains_key("error_type"));
            }
            other => panic!("expected HermesObserver, got {other:?}"),
        }
    }

    #[test]
    fn observer_string_fields_are_single_line_and_bounded() {
        let long_tool_name = format!("terminal\n{}", "x".repeat(400));
        let envelope = from_incoming_event(&IncomingEvent::new(
            "hermes.observer.tool.started",
            json!({
                "provider": "hermes",
                "source": "plugin",
                "session_id": "session-1",
                "tool_call_id": "tool-1",
                "tool_name": long_tool_name,
                "arg_keys": [
                    "pattern-0-RAW_PATTERN_KEY_SHOULD_BE_BOUNDEDxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
                    "pattern-1-RAW_PATTERN_KEY_SHOULD_BE_BOUNDEDxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
                    "pattern-2-RAW_PATTERN_KEY_SHOULD_BE_BOUNDEDxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
                    "pattern-3-RAW_PATTERN_KEY_SHOULD_BE_BOUNDEDxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
                    "pattern-4-RAW_PATTERN_KEY_SHOULD_BE_BOUNDEDxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
                    "pattern-5-RAW_PATTERN_KEY_SHOULD_BE_BOUNDEDxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
                    "pattern-6-RAW_PATTERN_KEY_SHOULD_BE_BOUNDEDxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
                    "pattern-7-RAW_PATTERN_KEY_SHOULD_BE_BOUNDEDxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
                    "pattern-8-RAW_PATTERN_KEY_SHOULD_BE_BOUNDEDxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
                    "pattern-9-RAW_PATTERN_KEY_SHOULD_BE_BOUNDEDxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
                    "pattern-10-RAW_PATTERN_KEY_SHOULD_BE_BOUNDEDxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
                    "pattern-11-RAW_PATTERN_KEY_SHOULD_BE_BOUNDEDxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
                    "pattern-12-RAW_PATTERN_KEY_SHOULD_BE_BOUNDEDxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
                    "pattern-13-RAW_PATTERN_KEY_SHOULD_BE_BOUNDEDxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
                    "pattern-14-RAW_PATTERN_KEY_SHOULD_BE_BOUNDEDxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
                    "pattern-15-RAW_PATTERN_KEY_SHOULD_BE_BOUNDEDxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
                    "pattern-16-RAW_PATTERN_KEY_SHOULD_BE_BOUNDEDxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
                ]
            }),
        ))
        .unwrap();

        match envelope.body {
            EventBody::HermesObserver(body) => {
                let Some(ObserverFieldValue::String(tool_name)) = body.fields.get("tool_name")
                else {
                    panic!("missing bounded tool_name");
                };
                assert_eq!(tool_name.chars().count(), 240);
                assert!(!tool_name.contains('\n'));

                let Some(ObserverFieldValue::StringList(arg_keys)) = body.fields.get("arg_keys")
                else {
                    panic!("missing bounded arg_keys");
                };
                assert_eq!(arg_keys.len(), 16);
                assert!(arg_keys.iter().all(|value| value.chars().count() <= 64));
            }
            other => panic!("expected HermesObserver, got {other:?}"),
        }
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
            EventBody::HermesObserver(_) => "HermesObserver",
            EventBody::Custom(_) => "Custom",
        }
    }
}
