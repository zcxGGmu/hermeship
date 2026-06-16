use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::events::IncomingEvent;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HermesHookEnvelope {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(
        default,
        rename = "event",
        alias = "event_type",
        alias = "kind",
        alias = "type",
        skip_serializing_if = "Option::is_none"
    )]
    pub event: Option<String>,
    #[serde(default = "empty_context")]
    pub context: Value,
}

impl HermesHookEnvelope {
    pub fn with_source_default(mut self, source: impl Into<String>) -> Self {
        if normalize_text(self.source.as_deref()).is_none() {
            let source = source.into();
            self.source = normalize_text(Some(source.as_str()));
        }
        self
    }

    pub fn into_incoming_event(self) -> Result<IncomingEvent> {
        let event = normalize_text(self.event.as_deref())
            .ok_or_else(|| anyhow::anyhow!("Hermes hook event must not be empty"))?;
        let provider = normalize_text(self.provider.as_deref()).unwrap_or_else(|| "hermes".into());
        let source = normalize_text(self.source.as_deref()).unwrap_or_else(|| "gateway".into());

        if !self.context.is_object() {
            anyhow::bail!("Hermes hook context must be a JSON object");
        }

        Ok(IncomingEvent::new(
            event.clone(),
            json!({
                "provider": provider,
                "source": source,
                "event": event,
                "context": self.context,
            }),
        ))
    }
}

fn empty_context() -> Value {
    json!({})
}

fn normalize_text(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::HermesHookEnvelope;
    use crate::event::{EventBody, compat::from_incoming_event};

    #[test]
    fn hermes_hook_envelope_defaults_and_normalizes_to_incoming_event() {
        let hook: HermesHookEnvelope = serde_json::from_value(json!({
            "event": "agent:start",
            "context": {
                "platform": "telegram",
                "session_id": "synthetic-session-001",
                "agent_name": "demo-agent"
            }
        }))
        .unwrap();

        let event = hook.into_incoming_event().unwrap();

        assert_eq!(event.kind, "agent:start");
        assert_eq!(event.payload["provider"], json!("hermes"));
        assert_eq!(event.payload["source"], json!("gateway"));
        assert_eq!(event.payload["event"], json!("agent:start"));
        assert_eq!(
            event.payload["context"]["session_id"],
            json!("synthetic-session-001")
        );

        let envelope = from_incoming_event(&event).unwrap();
        assert_eq!(envelope.canonical_kind(), "hermes.agent.started");
        assert_eq!(envelope.metadata.provider.as_deref(), Some("hermes"));
        assert_eq!(envelope.metadata.source.as_deref(), Some("gateway"));
        assert_eq!(
            envelope.metadata.session_id.as_deref(),
            Some("synthetic-session-001")
        );
        assert!(matches!(envelope.body, EventBody::HermesAgentStarted(_)));
    }

    #[test]
    fn hermes_hook_envelope_accepts_event_type_alias_and_source_override() {
        let hook: HermesHookEnvelope = serde_json::from_value(json!({
            "provider": "hermes",
            "source": "gateway-test",
            "event_type": "session:end",
            "context": {
                "session_id": "synthetic-session-002",
                "success": true
            }
        }))
        .unwrap();

        let event = hook.into_incoming_event().unwrap();

        assert_eq!(event.kind, "session:end");
        assert_eq!(event.payload["provider"], json!("hermes"));
        assert_eq!(event.payload["source"], json!("gateway-test"));
        assert_eq!(event.payload["event"], json!("session:end"));
        assert_eq!(event.payload["context"]["success"], json!(true));
    }

    #[test]
    fn hermes_hook_normalization_maps_gateway_events_through_existing_compat() {
        for (raw, canonical) in [
            ("gateway:startup", "hermes.gateway.started"),
            ("session:start", "hermes.session.started"),
            ("session:end", "hermes.session.finished"),
            ("session:reset", "hermes.session.reset"),
            ("agent:start", "hermes.agent.started"),
            ("agent:end", "hermes.agent.finished"),
        ] {
            let hook: HermesHookEnvelope = serde_json::from_value(json!({
                "event": raw,
                "context": {
                    "session_id": "synthetic-session-003",
                    "agent_name": "demo-agent"
                }
            }))
            .unwrap();

            let event = hook.into_incoming_event().unwrap();
            let envelope = from_incoming_event(&event).unwrap();

            assert_eq!(
                envelope.canonical_kind(),
                canonical,
                "canonical kind mismatch for {raw}"
            );
        }
    }

    #[test]
    fn hermes_hook_agent_end_failure_maps_to_failed_event() {
        let hook: HermesHookEnvelope = serde_json::from_value(json!({
            "event": "agent:end",
            "context": {
                "session_id": "synthetic-session-004",
                "agent_name": "demo-agent",
                "success": false,
                "error_message": "synthetic failure"
            }
        }))
        .unwrap();

        let event = hook.into_incoming_event().unwrap();
        let envelope = from_incoming_event(&event).unwrap();

        assert_eq!(envelope.canonical_kind(), "hermes.agent.failed");
        assert!(matches!(envelope.body, EventBody::HermesAgentFailed(_)));
    }

    #[test]
    fn hermes_hook_rejects_missing_event() {
        let hook: HermesHookEnvelope = serde_json::from_value(json!({
            "context": {
                "session_id": "synthetic-session-005"
            }
        }))
        .unwrap();

        let error = hook.into_incoming_event().unwrap_err().to_string();

        assert!(
            error.contains("Hermes hook event must not be empty"),
            "{error}"
        );
    }
}
