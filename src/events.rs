use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

pub use crate::config::MessageFormat;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct IncomingEvent {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mention: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<MessageFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
    pub payload: Value,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct RoutingMetadata {
    pub tool: Option<String>,
    pub provider: Option<String>,
    pub source: Option<String>,
    pub platform: Option<String>,
    pub user_id: Option<String>,
    pub chat_id: Option<String>,
    pub thread_id: Option<String>,
    pub chat_type: Option<String>,
    pub session_id: Option<String>,
    pub agent_name: Option<String>,
    pub project: Option<String>,
    pub repo_name: Option<String>,
    pub repo_path: Option<String>,
    pub worktree_path: Option<String>,
    pub branch: Option<String>,
}

#[derive(Debug, Deserialize)]
struct IncomingEventWire {
    #[serde(rename = "type", alias = "kind", alias = "event")]
    kind: String,
    #[serde(default)]
    channel: Option<String>,
    #[serde(default)]
    mention: Option<String>,
    #[serde(default)]
    format: Option<MessageFormat>,
    #[serde(default)]
    template: Option<String>,
    #[serde(default)]
    payload: Option<Value>,
    #[serde(flatten)]
    extra: BTreeMap<String, Value>,
}

impl<'de> Deserialize<'de> for IncomingEvent {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = IncomingEventWire::deserialize(deserializer)?;
        let payload = match wire.payload {
            Some(Value::Null) | None => {
                Value::Object(wire.extra.into_iter().collect::<Map<String, Value>>())
            }
            Some(payload) => payload,
        };

        Ok(Self {
            kind: wire.kind,
            channel: normalize_optional_text(wire.channel),
            mention: normalize_optional_text(wire.mention),
            format: wire.format,
            template: normalize_optional_text(wire.template),
            payload,
        })
    }
}

impl IncomingEvent {
    pub fn new(kind: impl Into<String>, payload: Value) -> Self {
        Self {
            kind: kind.into(),
            channel: None,
            mention: None,
            format: None,
            template: None,
            payload,
        }
    }

    pub fn custom(channel: Option<String>, message: String) -> Self {
        Self {
            kind: "custom".to_string(),
            channel: normalize_optional_text(channel),
            mention: None,
            format: None,
            template: None,
            payload: json!({ "message": message }),
        }
    }

    pub fn with_routing_metadata(mut self, routing: &RoutingMetadata) -> Self {
        if !self.payload.is_object() {
            self.payload = json!({ "value": self.payload });
        }

        let Some(payload) = self.payload.as_object_mut() else {
            return self;
        };

        for (key, value) in [
            ("tool", routing.tool.as_deref()),
            ("provider", routing.provider.as_deref()),
            ("source", routing.source.as_deref()),
            ("platform", routing.platform.as_deref()),
            ("user_id", routing.user_id.as_deref()),
            ("chat_id", routing.chat_id.as_deref()),
            ("thread_id", routing.thread_id.as_deref()),
            ("chat_type", routing.chat_type.as_deref()),
            ("session_id", routing.session_id.as_deref()),
            ("agent_name", routing.agent_name.as_deref()),
            ("project", routing.project.as_deref()),
            ("repo_name", routing.repo_name.as_deref()),
            ("repo_path", routing.repo_path.as_deref()),
            ("worktree_path", routing.worktree_path.as_deref()),
            ("branch", routing.branch.as_deref()),
        ] {
            if let Some(value) = normalize_text(value) {
                payload.insert(key.to_string(), json!(value));
            }
        }

        self
    }
}

fn normalize_optional_text(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_string())
    })
}

fn normalize_text(value: Option<&str>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_string())
    })
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;

    #[test]
    fn reuses_config_message_format_and_parses_labels() {
        let _: crate::config::MessageFormat = MessageFormat::Compact;

        assert_eq!(
            MessageFormat::from_label("compact").unwrap(),
            MessageFormat::Compact
        );
        assert_eq!(
            MessageFormat::from_label("inline").unwrap(),
            MessageFormat::Inline
        );
        assert_eq!(
            MessageFormat::from_label("alert").unwrap(),
            MessageFormat::Alert
        );
        assert_eq!(
            MessageFormat::from_label("raw").unwrap(),
            MessageFormat::Raw
        );

        let error = MessageFormat::from_label("verbose")
            .unwrap_err()
            .to_string();
        assert!(error.contains("unsupported message format"));
    }

    #[test]
    fn deserializes_incoming_event_field_aliases() {
        let event: IncomingEvent = serde_json::from_value(json!({
            "event": "hermes.agent.started",
            "channel": "ops",
            "mention": "@here",
            "format": "alert",
            "template": "agent {session_id}",
            "payload": {
                "session_id": "synthetic-session-001"
            }
        }))
        .unwrap();

        assert_eq!(event.kind, "hermes.agent.started");
        assert_eq!(event.channel.as_deref(), Some("ops"));
        assert_eq!(event.mention.as_deref(), Some("@here"));
        assert_eq!(event.format, Some(MessageFormat::Alert));
        assert_eq!(event.template.as_deref(), Some("agent {session_id}"));
        assert_eq!(event.payload["session_id"], json!("synthetic-session-001"));
    }

    #[test]
    fn deserializes_extra_fields_as_payload_when_payload_is_absent() {
        let event: IncomingEvent = serde_json::from_value(json!({
            "type": "hermes.session.finished",
            "session_id": "synthetic-session-002",
            "response_chars": 128,
            "success": true
        }))
        .unwrap();

        assert_eq!(event.kind, "hermes.session.finished");
        assert_eq!(
            event.payload,
            json!({
                "session_id": "synthetic-session-002",
                "response_chars": 128,
                "success": true
            })
        );
    }

    #[test]
    fn null_payload_deserializes_as_empty_object() {
        let event: IncomingEvent = serde_json::from_value(json!({
            "kind": "hermes.agent.started",
            "payload": null
        }))
        .unwrap();

        assert_eq!(event.payload, json!({}));
    }

    #[test]
    fn routing_metadata_is_inserted_into_object_payload() {
        let event = IncomingEvent {
            kind: "hermes.agent.started".to_string(),
            channel: None,
            mention: None,
            format: None,
            template: None,
            payload: json!({"existing": "value"}),
        }
        .with_routing_metadata(&RoutingMetadata {
            provider: Some("hermes".to_string()),
            source: Some("gateway".to_string()),
            platform: Some("telegram".to_string()),
            chat_id: Some("synthetic-chat-001".to_string()),
            session_id: Some("synthetic-session-001".to_string()),
            project: Some("hermes".to_string()),
            branch: Some("main".to_string()),
            ..RoutingMetadata::default()
        });

        assert_eq!(event.payload["existing"], json!("value"));
        assert_eq!(event.payload["provider"], json!("hermes"));
        assert_eq!(event.payload["source"], json!("gateway"));
        assert_eq!(event.payload["platform"], json!("telegram"));
        assert_eq!(event.payload["chat_id"], json!("synthetic-chat-001"));
        assert_eq!(event.payload["session_id"], json!("synthetic-session-001"));
        assert_eq!(event.payload["project"], json!("hermes"));
        assert_eq!(event.payload["branch"], json!("main"));
    }

    #[test]
    fn hermes_fixtures_are_synthetic_and_parseable_except_invalid_payload() {
        let agent_start: Value =
            serde_json::from_str(include_str!("../tests/fixtures/hermes/agent_start.json"))
                .unwrap();
        let session_end: Value =
            serde_json::from_str(include_str!("../tests/fixtures/hermes/session_end.json"))
                .unwrap();

        assert_eq!(agent_start["provider"], json!("hermes"));
        assert_eq!(agent_start["event"], json!("agent:start"));
        assert_eq!(session_end["event"], json!("session:end"));

        let fixture_text = format!("{agent_start}{session_end}");
        for forbidden in ["token", "cookie", "secret", "authorization", "api_key"] {
            assert!(
                !fixture_text.to_ascii_lowercase().contains(forbidden),
                "fixture contains forbidden marker `{forbidden}`"
            );
        }

        let invalid = include_str!("../tests/fixtures/hermes/invalid_payload.json");
        assert!(serde_json::from_str::<Value>(invalid).is_err());
    }
}
