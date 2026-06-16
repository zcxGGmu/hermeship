use anyhow::Result;
use serde_json::{Map, Value, json};

use crate::config::{MessageFormat, PrivacyConfig};
use crate::event::{
    CustomEvent, EventBody, EventEnvelope, EventMetadata, GitBranchChangedEvent, GitCommitEvent,
    HermesAgentEvent, HermesGatewayEvent, HermesSessionEvent,
};
use crate::privacy::sanitize_payload;
use crate::router::{ResolvedDelivery, SinkTarget};

use super::Renderer;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedMessage {
    pub content: String,
    pub format: MessageFormat,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultRenderer;

impl DefaultRenderer {
    pub fn render(
        &self,
        event: &EventEnvelope,
        delivery: &ResolvedDelivery,
    ) -> Result<RenderedMessage> {
        let content = match delivery.format {
            MessageFormat::Raw => render_raw(event, delivery)?,
            _ if delivery.template.is_some() => {
                let template = delivery.template.as_deref().unwrap_or_default();
                apply_mention(
                    render_template(template, event, delivery),
                    delivery.mention.as_deref(),
                )
            }
            MessageFormat::Compact => {
                apply_mention(render_compact(event), delivery.mention.as_deref())
            }
            MessageFormat::Inline => {
                apply_mention(render_inline(event), delivery.mention.as_deref())
            }
            MessageFormat::Alert => apply_mention(
                format!("ALERT: {}", render_compact(event)),
                delivery.mention.as_deref(),
            ),
        };

        Ok(RenderedMessage {
            content,
            format: delivery.format,
        })
    }
}

impl Renderer for DefaultRenderer {
    fn render(
        &self,
        event: &EventEnvelope,
        delivery: &ResolvedDelivery,
    ) -> Result<RenderedMessage> {
        Self::render(self, event, delivery)
    }
}

fn render_compact(event: &EventEnvelope) -> String {
    match &event.body {
        EventBody::Custom(body) => body.message.clone(),
        _ => {
            let label = event_label(event);
            let details = detail_parts(event);
            if details.is_empty() {
                label
            } else {
                format!("{label} ({})", details.join(", "))
            }
        }
    }
}

fn render_inline(event: &EventEnvelope) -> String {
    let parts = detail_parts(event);
    if parts.is_empty() {
        event.canonical_kind().to_string()
    } else {
        format!("{} | {}", event.canonical_kind(), parts.join(" | "))
    }
}

fn render_raw(event: &EventEnvelope, delivery: &ResolvedDelivery) -> Result<String> {
    Ok(serde_json::to_string_pretty(&json!({
        "event": event.canonical_kind(),
        "source": event.source,
        "priority": priority_label(&event.metadata),
        "metadata": metadata_json(event),
        "delivery": delivery_json(delivery),
        "body": body_json(&event.body),
    }))?)
}

fn render_template(template: &str, event: &EventEnvelope, delivery: &ResolvedDelivery) -> String {
    let mut output = String::new();
    let mut remaining = template;

    while let Some(start) = remaining.find('{') {
        output.push_str(&remaining[..start]);
        let token_start = start + 1;
        let Some(end_offset) = remaining[token_start..].find('}') else {
            output.push_str(&remaining[start..]);
            return output;
        };
        let end = token_start + end_offset;
        let token = &remaining[token_start..end];
        if let Some(value) = token_value(token, event, delivery) {
            output.push_str(&value);
        } else {
            output.push('{');
            output.push_str(token);
            output.push('}');
        }
        remaining = &remaining[end + 1..];
    }

    output.push_str(remaining);
    output
}

fn token_value(token: &str, event: &EventEnvelope, delivery: &ResolvedDelivery) -> Option<String> {
    let metadata = &event.metadata;
    match token.trim() {
        "event" | "canonical_kind" => Some(event.canonical_kind().to_string()),
        "source" => metadata
            .source
            .clone()
            .or_else(|| non_empty(event.source.as_str())),
        "provider" => metadata.provider.clone(),
        "platform" => metadata.platform.clone(),
        "session_id" => metadata.session_id.clone(),
        "agent_name" => metadata.agent_name.clone(),
        "project" => metadata.project.clone(),
        "channel" => delivery_channel(delivery),
        _ => None,
    }
}

fn event_label(event: &EventEnvelope) -> String {
    event.canonical_kind().replace(['.', '-'], " ")
}

fn detail_parts(event: &EventEnvelope) -> Vec<String> {
    match &event.body {
        EventBody::GitCommit(body) => git_commit_parts(body),
        EventBody::GitBranchChanged(body) => git_branch_changed_parts(body),
        EventBody::HermesGatewayStarted(body) => gateway_parts(body),
        EventBody::HermesSessionStarted(body)
        | EventBody::HermesSessionFinished(body)
        | EventBody::HermesSessionReset(body) => session_parts(body),
        EventBody::HermesAgentStarted(body)
        | EventBody::HermesAgentStep(body)
        | EventBody::HermesAgentFinished(body)
        | EventBody::HermesAgentFailed(body) => agent_parts(body),
        EventBody::Custom(body) => custom_parts(body),
    }
}

fn git_commit_parts(body: &GitCommitEvent) -> Vec<String> {
    let mut parts = Vec::new();
    push_part(&mut parts, "repo", Some(body.repo.as_str()));
    push_part(&mut parts, "branch", Some(body.branch.as_str()));
    push_part(&mut parts, "commit", Some(body.short_sha.as_str()));
    push_part(&mut parts, "summary", Some(body.summary.as_str()));
    push_part(&mut parts, "author", body.author_name.as_deref());
    parts
}

fn git_branch_changed_parts(body: &GitBranchChangedEvent) -> Vec<String> {
    let mut parts = Vec::new();
    push_part(&mut parts, "repo", Some(body.repo.as_str()));
    push_part(&mut parts, "branch", Some(body.new_branch.as_str()));
    push_part(&mut parts, "old_branch", Some(body.old_branch.as_str()));
    push_part(&mut parts, "new_branch", Some(body.new_branch.as_str()));
    parts
}

fn gateway_parts(body: &HermesGatewayEvent) -> Vec<String> {
    let mut parts = Vec::new();
    push_part(&mut parts, "platform", body.platform.as_deref());
    push_part(&mut parts, "project", body.project.as_deref());
    parts
}

fn session_parts(body: &HermesSessionEvent) -> Vec<String> {
    let mut parts = Vec::new();
    push_part(&mut parts, "platform", body.platform.as_deref());
    push_part(&mut parts, "session", body.session_id.as_deref());
    push_part(&mut parts, "project", body.project.as_deref());
    push_u64_part(&mut parts, "message_chars", body.message_chars);
    push_u64_part(&mut parts, "response_chars", body.response_chars);
    push_bool_part(&mut parts, "success", body.success);
    parts
}

fn agent_parts(body: &HermesAgentEvent) -> Vec<String> {
    let mut parts = Vec::new();
    push_part(&mut parts, "agent", body.agent_name.as_deref());
    push_part(&mut parts, "platform", body.platform.as_deref());
    push_part(&mut parts, "session", body.session_id.as_deref());
    push_part(&mut parts, "project", body.project.as_deref());
    push_part(&mut parts, "step", body.step_name.as_deref());
    push_u64_part(&mut parts, "elapsed", body.elapsed_secs);
    push_u64_part(&mut parts, "message_chars", body.message_chars);
    push_u64_part(&mut parts, "response_chars", body.response_chars);
    push_bool_part(&mut parts, "success", body.success);
    push_part(&mut parts, "error", body.error_message.as_deref());
    parts
}

fn custom_parts(body: &CustomEvent) -> Vec<String> {
    vec![format!("message={}", body.message)]
}

fn push_part(parts: &mut Vec<String>, key: &str, value: Option<&str>) {
    if let Some(value) = value.and_then(non_empty) {
        parts.push(format!("{key}={value}"));
    }
}

fn push_u64_part(parts: &mut Vec<String>, key: &str, value: Option<u64>) {
    if let Some(value) = value {
        parts.push(format!("{key}={value}"));
    }
}

fn push_bool_part(parts: &mut Vec<String>, key: &str, value: Option<bool>) {
    if let Some(value) = value {
        parts.push(format!("{key}={value}"));
    }
}

fn apply_mention(content: String, mention: Option<&str>) -> String {
    match mention.and_then(non_empty) {
        Some(mention) => format!("{mention} {content}"),
        None => content,
    }
}

fn non_empty(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_string())
}

fn delivery_channel(delivery: &ResolvedDelivery) -> Option<String> {
    match &delivery.target {
        SinkTarget::DiscordChannel(channel) => non_empty(channel),
        SinkTarget::DiscordWebhook(_) => Some("webhook".to_string()),
    }
}

fn priority_label(metadata: &EventMetadata) -> String {
    format!("{:?}", metadata.priority).to_ascii_lowercase()
}

fn metadata_json(event: &EventEnvelope) -> Value {
    let metadata = &event.metadata;
    let mut output = Map::new();
    insert_json_string(
        &mut output,
        "source",
        metadata.source.as_deref().or(Some(&event.source)),
    );
    insert_json_string(&mut output, "provider", metadata.provider.as_deref());
    insert_json_string(&mut output, "platform", metadata.platform.as_deref());
    insert_json_string(&mut output, "session_id", metadata.session_id.as_deref());
    insert_json_string(&mut output, "agent_name", metadata.agent_name.as_deref());
    insert_json_string(&mut output, "project", metadata.project.as_deref());
    insert_json_string(&mut output, "repo_name", metadata.repo_name.as_deref());
    insert_json_string(&mut output, "branch", metadata.branch.as_deref());
    Value::Object(output)
}

fn delivery_json(delivery: &ResolvedDelivery) -> Value {
    let mut output = Map::new();
    insert_json_string(&mut output, "sink", Some(&delivery.sink));
    insert_json_string(&mut output, "target", Some(&delivery.target.to_string()));
    insert_json_string(
        &mut output,
        "channel",
        delivery_channel(delivery).as_deref(),
    );
    insert_json_string(&mut output, "format", Some(delivery.format.as_str()));
    insert_json_string(&mut output, "mention", delivery.mention.as_deref());
    if let Some(index) = delivery.matched_route_index {
        output.insert("matched_route_index".to_string(), json!(index));
    }
    Value::Object(output)
}

fn body_json(body: &EventBody) -> Value {
    match body {
        EventBody::GitCommit(body) => git_commit_json(body),
        EventBody::GitBranchChanged(body) => git_branch_changed_json(body),
        EventBody::HermesGatewayStarted(body) => json!({
            "kind": "hermes.gateway.started",
            "provider": body.provider,
            "source": body.source,
            "platform": body.platform,
            "project": body.project,
        }),
        EventBody::HermesSessionStarted(body) => session_json("hermes.session.started", body),
        EventBody::HermesSessionFinished(body) => session_json("hermes.session.finished", body),
        EventBody::HermesSessionReset(body) => session_json("hermes.session.reset", body),
        EventBody::HermesAgentStarted(body) => agent_json("hermes.agent.started", body),
        EventBody::HermesAgentStep(body) => agent_json("hermes.agent.step", body),
        EventBody::HermesAgentFinished(body) => agent_json("hermes.agent.finished", body),
        EventBody::HermesAgentFailed(body) => agent_json("hermes.agent.failed", body),
        EventBody::Custom(body) => {
            let sanitized_payload = body
                .payload
                .as_ref()
                .map(|payload| sanitize_payload(payload, &PrivacyConfig::default()));
            json!({
                "kind": body.kind,
                "message_chars": body.message.chars().count(),
                "has_message": !body.message.trim().is_empty(),
                "payload": sanitized_payload,
            })
        }
    }
}

fn git_commit_json(body: &GitCommitEvent) -> Value {
    json!({
        "kind": "git.commit",
        "repo": body.repo,
        "branch": body.branch,
        "commit": body.sha,
        "short_commit": body.short_sha,
        "summary": body.summary,
        "author_name": body.author_name,
    })
}

fn git_branch_changed_json(body: &GitBranchChangedEvent) -> Value {
    json!({
        "kind": "git.branch-changed",
        "repo": body.repo,
        "old_branch": body.old_branch,
        "new_branch": body.new_branch,
    })
}

fn session_json(kind: &str, body: &HermesSessionEvent) -> Value {
    json!({
        "kind": kind,
        "status": body.status,
        "session_id": body.session_id,
        "platform": body.platform,
        "project": body.project,
        "message_chars": body.message_chars,
        "response_chars": body.response_chars,
        "has_message": body.has_message,
        "has_response": body.has_response,
        "success": body.success,
    })
}

fn agent_json(kind: &str, body: &HermesAgentEvent) -> Value {
    json!({
        "kind": kind,
        "status": body.status,
        "agent_name": body.agent_name,
        "session_id": body.session_id,
        "platform": body.platform,
        "project": body.project,
        "step_name": body.step_name,
        "message_chars": body.message_chars,
        "response_chars": body.response_chars,
        "has_message": body.has_message,
        "has_response": body.has_response,
        "elapsed_secs": body.elapsed_secs,
        "success": body.success,
        "error_message_chars": body.error_message.as_deref().map(|value| value.chars().count()),
        "has_error_message": body.error_message.as_deref().map(|value| !value.trim().is_empty()),
    })
}

fn insert_json_string(output: &mut Map<String, Value>, key: &str, value: Option<&str>) {
    if let Some(value) = value.and_then(non_empty) {
        output.insert(key.to_string(), json!(value));
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde_json::{Value, json};
    use time::OffsetDateTime;
    use uuid::Uuid;

    use super::*;
    use crate::config::{AppConfig, MessageFormat, RouteRule};
    use crate::event::compat::from_incoming_event;
    use crate::event::{CustomEvent, EventMetadata, EventPriority};
    use crate::events::IncomingEvent;
    use crate::privacy::sanitize_payload;
    use crate::router::{ResolvedDelivery, Router, SinkTarget};

    #[test]
    fn render_compact_agent_event_with_delivery_context() {
        let event = sanitized_envelope(
            "hermes.agent.started",
            json!({
                "provider": "hermes",
                "source": "gateway",
                "platform": "telegram",
                "session_id": "synthetic-session-001",
                "agent_name": "demo-agent",
                "project": "hermes",
                "message_chars": 42,
                "has_message": true
            }),
        );
        let delivery = delivery(MessageFormat::Compact);

        let rendered = DefaultRenderer.render(&event, &delivery).unwrap();

        assert_eq!(
            rendered.content,
            "hermes agent started (agent=demo-agent, platform=telegram, session=synthetic-session-001, project=hermes, message_chars=42)"
        );
        assert_eq!(rendered.format, MessageFormat::Compact);
    }

    #[test]
    fn render_inline_session_event_as_single_line_summary() {
        let event = sanitized_envelope(
            "hermes.session.finished",
            json!({
                "provider": "hermes",
                "source": "gateway",
                "platform": "telegram",
                "session_id": "synthetic-session-002",
                "project": "hermes",
                "response_chars": 128,
                "has_response": true,
                "success": true
            }),
        );
        let delivery = delivery(MessageFormat::Inline);

        let rendered = DefaultRenderer.render(&event, &delivery).unwrap();

        assert_eq!(
            rendered.content,
            "hermes.session.finished | platform=telegram | session=synthetic-session-002 | project=hermes | response_chars=128 | success=true"
        );
        assert_eq!(rendered.format, MessageFormat::Inline);
    }

    #[test]
    fn render_alert_agent_failure_includes_mention_and_safe_error_summary() {
        let event = sanitized_envelope(
            "agent:end",
            json!({
                "provider": "hermes",
                "source": "gateway",
                "platform": "telegram",
                "session_id": "synthetic-session-003",
                "agent_name": "demo-agent",
                "project": "hermes",
                "success": false,
                "error_message": "synthetic failure summary"
            }),
        );
        let mut delivery = delivery(MessageFormat::Alert);
        delivery.mention = Some("@ops".to_string());

        let rendered = DefaultRenderer.render(&event, &delivery).unwrap();

        assert_eq!(
            rendered.content,
            "@ops ALERT: hermes agent failed (agent=demo-agent, platform=telegram, session=synthetic-session-003, project=hermes, success=false, error=synthetic failure summary)"
        );
        assert_eq!(rendered.format, MessageFormat::Alert);
    }

    #[test]
    fn render_raw_event_as_safe_json_without_sensitive_body_fields() {
        let payload: Value = serde_json::from_str(include_str!(
            "../../tests/fixtures/privacy/sensitive_payload.json"
        ))
        .unwrap();
        let event = sanitized_envelope("custom", payload);
        let delivery = delivery(MessageFormat::Raw);

        let rendered = DefaultRenderer.render(&event, &delivery).unwrap();
        let raw: Value = serde_json::from_str(&rendered.content).unwrap();

        assert_eq!(raw["event"], json!("custom"));
        assert_eq!(raw["delivery"]["format"], json!("raw"));
        assert_eq!(raw["body"]["message_chars"], json!(6));
        assert_eq!(raw["body"]["has_message"], json!(true));

        for forbidden in [
            "synthetic-token-value",
            "synthetic-cookie-value",
            "synthetic-secret-value",
            "synthetic message sample",
            "synthetic response sample",
            "synthetic provider request summary",
            "synthetic provider response summary",
            "synthetic tool result summary",
        ] {
            assert!(
                !rendered.content.contains(forbidden),
                "raw render leaked `{forbidden}`"
            );
        }
    }

    #[test]
    fn raw_format_ignores_template_and_remains_json() {
        let event = sanitized_envelope(
            "hermes.agent.started",
            json!({
                "provider": "hermes",
                "source": "gateway",
                "platform": "telegram",
                "session_id": "synthetic-session-raw-template",
                "agent_name": "demo-agent",
                "project": "hermes"
            }),
        );
        let mut delivery = delivery(MessageFormat::Raw);
        delivery.mention = Some("@ops".to_string());
        delivery.template = Some("not json {event} {session_id}".to_string());

        let rendered = DefaultRenderer.render(&event, &delivery).unwrap();
        let raw: Value = serde_json::from_str(&rendered.content).unwrap();

        assert_eq!(raw["event"], json!("hermes.agent.started"));
        assert_eq!(raw["delivery"]["format"], json!("raw"));
        assert!(!rendered.content.starts_with("@ops"));
        assert!(!rendered.content.contains("not json"));
    }

    #[test]
    fn raw_format_summarizes_direct_typed_free_text_without_leaking_it() {
        let event = EventEnvelope {
            id: Uuid::new_v4(),
            timestamp: OffsetDateTime::now_utc(),
            source: "custom".to_string(),
            body: EventBody::Custom(CustomEvent {
                kind: "custom".to_string(),
                message: "token=synthetic-token-raw-leak full synthetic prompt body".to_string(),
                payload: Some(json!({
                    "secret": "synthetic-secret-raw-leak",
                    "message": "synthetic nested message raw leak"
                })),
            }),
            metadata: EventMetadata::default(),
        };
        let rendered = DefaultRenderer
            .render(&event, &delivery(MessageFormat::Raw))
            .unwrap();
        let raw: Value = serde_json::from_str(&rendered.content).unwrap();

        assert_eq!(raw["body"]["message_chars"], json!(57));
        assert_eq!(raw["body"]["has_message"], json!(true));
        assert!(raw["body"].get("message").is_none());
        for forbidden in [
            "synthetic-token-raw-leak",
            "full synthetic prompt body",
            "synthetic-secret-raw-leak",
            "synthetic nested message raw leak",
        ] {
            assert!(
                !rendered.content.contains(forbidden),
                "leaked `{forbidden}`"
            );
        }

        let event = EventEnvelope {
            id: Uuid::new_v4(),
            timestamp: OffsetDateTime::now_utc(),
            source: "hermes".to_string(),
            body: EventBody::HermesAgentFailed(HermesAgentEvent {
                status: "failed".to_string(),
                agent_name: Some("demo-agent".to_string()),
                session_id: Some("synthetic-session-raw-error".to_string()),
                platform: Some("telegram".to_string()),
                project: Some("hermes".to_string()),
                step_name: None,
                message_chars: None,
                response_chars: None,
                has_message: None,
                has_response: None,
                elapsed_secs: None,
                success: Some(false),
                error_message: Some(
                    "secret=synthetic-agent-secret full provider failure body".to_string(),
                ),
            }),
            metadata: EventMetadata {
                priority: EventPriority::Critical,
                ..EventMetadata::default()
            },
        };
        let rendered = DefaultRenderer
            .render(&event, &delivery(MessageFormat::Raw))
            .unwrap();
        let raw: Value = serde_json::from_str(&rendered.content).unwrap();

        assert_eq!(raw["body"]["error_message_chars"], json!(56));
        assert_eq!(raw["body"]["has_error_message"], json!(true));
        assert!(raw["body"].get("error_message").is_none());
        assert!(!rendered.content.contains("synthetic-agent-secret"));
        assert!(!rendered.content.contains("full provider failure body"));
    }

    #[test]
    fn render_template_replaces_safe_tokens_and_preserves_unknown_tokens() {
        let event = sanitized_envelope(
            "hermes.agent.started",
            json!({
                "provider": "hermes",
                "source": "gateway",
                "platform": "telegram",
                "session_id": "synthetic-session-004",
                "agent_name": "demo-agent",
                "project": "hermes"
            }),
        );
        let mut delivery = delivery(MessageFormat::Compact);
        delivery.template = Some(
            "{event} {agent_name} {session_id} {platform} {project} {source} {provider} {channel} {missing}"
                .to_string(),
        );

        let rendered = DefaultRenderer.render(&event, &delivery).unwrap();

        assert_eq!(
            rendered.content,
            "hermes.agent.started demo-agent synthetic-session-004 telegram hermes gateway hermes ops {missing}"
        );
    }

    #[test]
    fn render_template_preserves_unapproved_tokens() {
        let event = sanitized_envelope(
            "agent:end",
            json!({
                "provider": "hermes",
                "source": "gateway",
                "platform": "telegram",
                "session_id": "synthetic-session-token-allowlist",
                "agent_name": "demo-agent",
                "project": "hermes",
                "success": false,
                "error_message": "synthetic failure summary"
            }),
        );
        let mut delivery = delivery(MessageFormat::Compact);
        delivery.template =
            Some("{event} {error_message} {repo_path} {chat_id} {status}".to_string());

        let rendered = DefaultRenderer.render(&event, &delivery).unwrap();

        assert_eq!(
            rendered.content,
            "hermes.agent.failed {error_message} {repo_path} {chat_id} {status}"
        );
    }

    #[test]
    fn render_degrades_cleanly_when_optional_fields_are_missing() {
        let event = sanitized_envelope(
            "hermes.gateway.started",
            json!({
                "provider": "hermes",
                "source": "gateway"
            }),
        );
        let delivery = delivery(MessageFormat::Compact);

        let rendered = DefaultRenderer.render(&event, &delivery).unwrap();

        assert_eq!(rendered.content, "hermes gateway started");
    }

    #[test]
    fn render_uses_route_level_format_template_and_mention_from_router() {
        let config = AppConfig {
            routes: vec![RouteRule {
                event: "hermes.agent.*".to_string(),
                filter: BTreeMap::from([("platform".to_string(), "telegram".to_string())]),
                channel: Some("route-channel".to_string()),
                mention: Some("@route".to_string()),
                format: Some(MessageFormat::Alert),
                template: Some("route {event} {session_id} {channel}".to_string()),
                ..RouteRule::default()
            }],
            ..AppConfig::default()
        };
        let event = sanitized_envelope(
            "hermes.agent.started",
            json!({
                "platform": "telegram",
                "session_id": "synthetic-session-005"
            }),
        );
        let delivery = Router::new(config).resolve(&event).remove(0);

        let rendered = DefaultRenderer.render(&event, &delivery).unwrap();

        assert_eq!(
            rendered.content,
            "@route route hermes.agent.started synthetic-session-005 route-channel"
        );
        assert_eq!(rendered.format, MessageFormat::Alert);
    }

    #[test]
    fn render_git_commit_compact_and_raw_without_commit_body_leaks() {
        let event = sanitized_envelope(
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
                "author_email": "synthetic@example.invalid",
                "commit_body": "synthetic full commit body should not render",
                "diff": "synthetic diff should not render",
                "token": "synthetic-token-should-not-render"
            }),
        );

        let rendered = DefaultRenderer
            .render(&event, &delivery(MessageFormat::Compact))
            .unwrap();

        assert_eq!(
            rendered.content,
            "git commit (repo=hermeship, branch=main, commit=1234567, summary=ship git source, author=Synthetic Author)"
        );

        let raw = DefaultRenderer
            .render(&event, &delivery(MessageFormat::Raw))
            .unwrap();
        let raw_json: Value = serde_json::from_str(&raw.content).unwrap();

        assert_eq!(raw_json["event"], json!("git.commit"));
        assert_eq!(raw_json["metadata"]["repo_name"], json!("hermeship"));
        assert_eq!(raw_json["metadata"]["branch"], json!("main"));
        assert_eq!(raw_json["body"]["short_commit"], json!("1234567"));
        assert_eq!(raw_json["body"]["summary"], json!("ship git source"));
        for forbidden in [
            "synthetic full commit body should not render",
            "synthetic diff should not render",
            "synthetic-token-should-not-render",
            "/tmp/hermeship",
            "/tmp/hermeship-worktree",
            "synthetic@example.invalid",
        ] {
            assert!(
                !raw.content.contains(forbidden),
                "git raw render leaked `{forbidden}`"
            );
        }
    }

    #[test]
    fn render_git_branch_changed_compact_summary() {
        let event = sanitized_envelope(
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

        let rendered = DefaultRenderer
            .render(&event, &delivery(MessageFormat::Compact))
            .unwrap();

        assert_eq!(
            rendered.content,
            "git branch changed (repo=hermeship, branch=codex/milestone-8-git, old_branch=main, new_branch=codex/milestone-8-git)"
        );

        let raw = DefaultRenderer
            .render(&event, &delivery(MessageFormat::Raw))
            .unwrap();
        let raw_json: Value = serde_json::from_str(&raw.content).unwrap();

        assert_eq!(raw_json["event"], json!("git.branch-changed"));
        assert_eq!(
            raw_json["body"]["new_branch"],
            json!("codex/milestone-8-git")
        );
        assert!(!raw.content.contains("/tmp/hermeship"));
        assert!(!raw.content.contains("/tmp/hermeship-worktree"));
    }

    fn sanitized_envelope(kind: &str, payload: Value) -> crate::event::EventEnvelope {
        let payload = sanitize_payload(&payload, &crate::config::PrivacyConfig::default());
        from_incoming_event(&IncomingEvent::new(kind, payload)).unwrap()
    }

    fn delivery(format: MessageFormat) -> ResolvedDelivery {
        ResolvedDelivery {
            sink: "discord".to_string(),
            target: SinkTarget::DiscordChannel("ops".to_string()),
            format,
            mention: None,
            template: None,
            matched_route_index: Some(0),
        }
    }
}
