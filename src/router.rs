use std::collections::BTreeMap;
use std::fmt;

use serde::{Serialize, Serializer};

use crate::config::{AppConfig, MessageFormat, RouteRule};
use crate::event::{EventEnvelope, EventMetadata};

#[derive(Debug, Clone)]
pub struct Router {
    config: AppConfig,
}

impl Router {
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }

    pub fn resolve(&self, event: &EventEnvelope) -> Vec<ResolvedDelivery> {
        self.explain(event).deliveries
    }

    pub fn explain(&self, event: &EventEnvelope) -> DeliveryExplanation {
        let canonical_kind = event.canonical_kind().to_string();
        let route_candidates = route_candidates(&canonical_kind);
        let metadata = metadata_context(event);
        let mut routes = Vec::with_capacity(self.config.routes.len());
        let mut deliveries = Vec::new();

        for (index, route) in self.config.routes.iter().enumerate() {
            let pattern_matched = route_candidates
                .iter()
                .any(|candidate| glob_match(&route.event, candidate));
            let filter_results = filter_results(route, &metadata);
            let filters_matched = filter_results.iter().all(|filter| filter.matched);

            let mut matched = false;
            let mut skipped_reason = None;
            let mut delivery = None;

            if !route.enabled {
                skipped_reason = Some("route disabled".to_string());
            } else if !pattern_matched {
                skipped_reason = Some("event pattern mismatch".to_string());
            } else if !filters_matched {
                skipped_reason = Some("filter mismatch".to_string());
            } else {
                matched = true;
                match self.resolve_delivery(event, route, index) {
                    Ok(resolved) => {
                        deliveries.push(resolved.clone());
                        delivery = Some(resolved);
                    }
                    Err(reason) => skipped_reason = Some(reason),
                }
            }

            routes.push(RouteExplanation {
                route_index: index,
                event_pattern: route.event.clone(),
                enabled: route.enabled,
                matched,
                pattern_matched,
                filter_results,
                skipped_reason,
                delivery,
            });
        }

        DeliveryExplanation {
            event_kind: canonical_kind.clone(),
            canonical_kind,
            route_candidates,
            routes,
            deliveries,
        }
    }

    fn resolve_delivery(
        &self,
        event: &EventEnvelope,
        route: &RouteRule,
        route_index: usize,
    ) -> Result<ResolvedDelivery, String> {
        let sink = normalize_route_sink(route);
        if sink != "discord" {
            return Err(format!("unsupported sink {sink:?}"));
        }

        let target = if let Some(webhook) = normalize_text(route.webhook.as_deref()) {
            SinkTarget::DiscordWebhook(webhook)
        } else {
            let channel = normalize_text(route.channel.as_deref())
                .or_else(|| normalize_text(event.metadata.channel_hint.as_deref()))
                .or_else(|| normalize_text(self.config.defaults.channel.as_deref()))
                .ok_or_else(|| "missing delivery target".to_string())?;
            SinkTarget::DiscordChannel(channel)
        };

        let format = event
            .metadata
            .format
            .or(route.format)
            .unwrap_or(self.config.defaults.format);
        let mention = event
            .metadata
            .mention
            .clone()
            .or_else(|| route.mention.clone());
        let template = event
            .metadata
            .template
            .clone()
            .or_else(|| route.template.clone());

        Ok(ResolvedDelivery {
            sink,
            target,
            format,
            mention,
            template,
            matched_route_index: Some(route_index),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResolvedDelivery {
    pub sink: String,
    pub target: SinkTarget,
    pub format: MessageFormat,
    pub mention: Option<String>,
    pub template: Option<String>,
    pub matched_route_index: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SinkTarget {
    DiscordChannel(String),
    DiscordWebhook(String),
}

impl SinkTarget {
    fn diagnostic_label(&self) -> String {
        match self {
            Self::DiscordChannel(channel) => format!("DiscordChannel({channel:?})"),
            Self::DiscordWebhook(_) => "DiscordWebhook(\"[REDACTED]\")".to_string(),
        }
    }
}

impl fmt::Display for SinkTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.diagnostic_label())
    }
}

impl Serialize for SinkTarget {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.diagnostic_label())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DeliveryExplanation {
    pub event_kind: String,
    pub canonical_kind: String,
    pub route_candidates: Vec<String>,
    pub routes: Vec<RouteExplanation>,
    pub deliveries: Vec<ResolvedDelivery>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RouteExplanation {
    pub route_index: usize,
    pub event_pattern: String,
    pub enabled: bool,
    pub matched: bool,
    pub pattern_matched: bool,
    pub filter_results: Vec<FilterExplanation>,
    pub skipped_reason: Option<String>,
    pub delivery: Option<ResolvedDelivery>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FilterExplanation {
    pub key: String,
    pub expected: String,
    pub actual: Option<String>,
    pub matched: bool,
}

impl fmt::Display for DeliveryExplanation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "event: {}", self.event_kind)?;
        if self.canonical_kind != self.event_kind {
            writeln!(f, "canonical: {}", self.canonical_kind)?;
        }
        writeln!(f, "route candidates: {}", self.route_candidates.join(", "))?;

        if self.routes.is_empty() {
            writeln!(f, "routes: (none configured)")?;
        } else {
            writeln!(f, "routes:")?;
            for route in &self.routes {
                writeln!(f, "  {route}")?;
            }
        }

        if self.deliveries.is_empty() {
            writeln!(f, "deliveries: (none)")?;
        } else {
            writeln!(f, "deliveries:")?;
            for delivery in &self.deliveries {
                writeln!(f, "  {}", DeliveryDisplay(delivery))?;
            }
        }

        Ok(())
    }
}

impl fmt::Display for RouteExplanation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = if self.delivery.is_some() {
            "MATCH"
        } else {
            "skip"
        };
        write!(
            f,
            "[{status}] #{idx} event={pattern:?}",
            idx = self.route_index,
            pattern = self.event_pattern,
        )?;

        if let Some(reason) = self.skipped_reason.as_deref() {
            write!(f, " ({reason})")?;
        }
        for filter in &self.filter_results {
            write!(f, " {filter}")?;
        }
        if let Some(delivery) = self.delivery.as_ref() {
            write!(f, " -> {}", DeliveryDisplay(delivery))?;
        }

        Ok(())
    }
}

impl fmt::Display for FilterExplanation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let actual = self.actual.as_deref().unwrap_or("<missing>");
        let result = if self.matched { "yes" } else { "no" };
        write!(
            f,
            "filter {key} expected {expected:?} actual {actual:?} => {result}",
            key = self.key,
            expected = self.expected,
        )
    }
}

struct DeliveryDisplay<'a>(&'a ResolvedDelivery);

impl fmt::Display for DeliveryDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let delivery = self.0;
        let route = delivery
            .matched_route_index
            .map(|index| format!("route #{index}"))
            .unwrap_or_else(|| "default".to_string());
        write!(
            f,
            "[{route}] {sink} -> {target} (format={format}",
            sink = delivery.sink,
            target = delivery.target,
            format = delivery.format.as_str(),
        )?;
        if let Some(mention) = delivery.mention.as_deref() {
            write!(f, ", mention={mention:?}")?;
        }
        if delivery.template.is_some() {
            write!(f, ", template=custom")?;
        }
        write!(f, ")")
    }
}

fn filter_results(
    route: &RouteRule,
    metadata: &BTreeMap<String, String>,
) -> Vec<FilterExplanation> {
    route
        .filter
        .iter()
        .map(|(key, expected)| {
            let actual = metadata.get(key).cloned();
            let matched = actual
                .as_deref()
                .map(|actual| glob_match(expected, actual))
                .unwrap_or(false);
            FilterExplanation {
                key: key.clone(),
                expected: expected.clone(),
                actual,
                matched,
            }
        })
        .collect()
}

fn metadata_context(event: &EventEnvelope) -> BTreeMap<String, String> {
    let metadata = &event.metadata;
    let mut context = BTreeMap::new();

    insert_context(&mut context, "event", Some(event.canonical_kind()));
    insert_context(&mut context, "canonical_kind", Some(event.canonical_kind()));
    insert_context(
        &mut context,
        "source",
        metadata.source.as_deref().or(Some(event.source.as_str())),
    );
    insert_metadata_fields(&mut context, metadata);

    context
}

fn insert_metadata_fields(context: &mut BTreeMap<String, String>, metadata: &EventMetadata) {
    for (key, value) in [
        ("tool", metadata.tool.as_deref()),
        ("provider", metadata.provider.as_deref()),
        ("platform", metadata.platform.as_deref()),
        ("user_id", metadata.user_id.as_deref()),
        ("chat_id", metadata.chat_id.as_deref()),
        ("thread_id", metadata.thread_id.as_deref()),
        ("chat_type", metadata.chat_type.as_deref()),
        ("session_id", metadata.session_id.as_deref()),
        ("agent_name", metadata.agent_name.as_deref()),
        ("project", metadata.project.as_deref()),
        ("repo_name", metadata.repo_name.as_deref()),
        ("repo_path", metadata.repo_path.as_deref()),
        ("worktree_path", metadata.worktree_path.as_deref()),
        ("branch", metadata.branch.as_deref()),
        ("channel", metadata.channel_hint.as_deref()),
    ] {
        insert_context(context, key, value);
    }
}

fn insert_context(context: &mut BTreeMap<String, String>, key: &str, value: Option<&str>) {
    if let Some(value) = normalize_text(value) {
        context.insert(key.to_string(), value);
    }
}

fn route_candidates(kind: &str) -> Vec<String> {
    let mut candidates = vec![kind.to_string()];
    let parts: Vec<&str> = kind.split('.').collect();
    if parts.len() > 1 {
        for prefix_len in (1..parts.len()).rev() {
            candidates.push(format!("{}.*", parts[..prefix_len].join(".")));
        }
    }
    candidates
}

fn normalize_route_sink(route: &RouteRule) -> String {
    normalize_text(Some(route.sink.as_str())).unwrap_or_else(|| "discord".to_string())
}

fn normalize_text(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

pub(crate) fn glob_match(pattern: &str, value: &str) -> bool {
    if pattern == value {
        return true;
    }
    if !pattern.contains('*') {
        return false;
    }

    let parts: Vec<&str> = pattern.split('*').collect();
    let starts_with_wildcard = pattern.starts_with('*');
    let ends_with_wildcard = pattern.ends_with('*');
    let mut remainder = value;

    for (index, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }

        if index == 0 && !starts_with_wildcard {
            if !remainder.starts_with(part) {
                return false;
            }
            remainder = &remainder[part.len()..];
            continue;
        }

        if index == parts.len() - 1 && !ends_with_wildcard {
            return remainder.ends_with(part);
        }

        if let Some(position) = remainder.find(part) {
            remainder = &remainder[position + part.len()..];
        } else {
            return false;
        }
    }

    ends_with_wildcard || remainder.is_empty()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde_json::json;

    use super::*;
    use crate::config::{AppConfig, DefaultsConfig, MessageFormat, RouteRule};
    use crate::event::compat::from_incoming_event;
    use crate::events::IncomingEvent;

    #[test]
    fn resolve_returns_multiple_deliveries_and_inherits_route_fields() {
        let config = AppConfig {
            defaults: DefaultsConfig {
                channel: Some("fallback".to_string()),
                format: MessageFormat::Compact,
                ..DefaultsConfig::default()
            },
            routes: vec![
                RouteRule {
                    event: "hermes.agent.*".to_string(),
                    filter: BTreeMap::from([("platform".to_string(), "telegram".to_string())]),
                    channel: Some("ops".to_string()),
                    mention: Some("@ops".to_string()),
                    format: Some(MessageFormat::Alert),
                    template: Some("agent {session_id}".to_string()),
                    ..RouteRule::default()
                },
                RouteRule {
                    event: "hermes.*".to_string(),
                    filter: BTreeMap::from([("session_id".to_string(), "demo".to_string())]),
                    channel: Some("audit".to_string()),
                    ..RouteRule::default()
                },
            ],
            ..AppConfig::default()
        };
        let envelope = envelope(
            "hermes.agent.started",
            json!({
                "provider": "hermes",
                "source": "gateway",
                "platform": "telegram",
                "session_id": "demo"
            }),
        );

        let deliveries = Router::new(config).resolve(&envelope);

        assert_eq!(deliveries.len(), 2);
        assert_eq!(
            deliveries[0].target,
            SinkTarget::DiscordChannel("ops".to_string())
        );
        assert_eq!(deliveries[0].format, MessageFormat::Alert);
        assert_eq!(deliveries[0].mention.as_deref(), Some("@ops"));
        assert_eq!(
            deliveries[0].template.as_deref(),
            Some("agent {session_id}")
        );
        assert_eq!(deliveries[0].matched_route_index, Some(0));
        assert_eq!(
            deliveries[1].target,
            SinkTarget::DiscordChannel("audit".to_string())
        );
        assert_eq!(deliveries[1].format, MessageFormat::Compact);
        assert_eq!(deliveries[1].matched_route_index, Some(1));
    }

    #[test]
    fn explain_records_failed_filters_disabled_routes_and_missing_targets() {
        let config = AppConfig {
            routes: vec![
                RouteRule {
                    event: "hermes.agent.*".to_string(),
                    filter: BTreeMap::from([("platform".to_string(), "discord".to_string())]),
                    channel: Some("discord-only".to_string()),
                    ..RouteRule::default()
                },
                RouteRule {
                    event: "hermes.agent.*".to_string(),
                    filter: BTreeMap::from([("platform".to_string(), "telegram".to_string())]),
                    channel: Some("disabled".to_string()),
                    enabled: false,
                    ..RouteRule::default()
                },
                RouteRule {
                    event: "hermes.agent.*".to_string(),
                    filter: BTreeMap::from([("platform".to_string(), "telegram".to_string())]),
                    ..RouteRule::default()
                },
            ],
            ..AppConfig::default()
        };
        let envelope = envelope(
            "hermes.agent.started",
            json!({
                "platform": "telegram",
                "session_id": "demo"
            }),
        );

        let explanation = Router::new(config).explain(&envelope);

        assert_eq!(explanation.canonical_kind, "hermes.agent.started");
        assert_eq!(
            explanation.route_candidates,
            vec!["hermes.agent.started", "hermes.agent.*", "hermes.*"]
        );
        assert!(explanation.deliveries.is_empty());
        assert_eq!(explanation.routes.len(), 3);
        assert!(!explanation.routes[0].matched);
        assert_eq!(
            explanation.routes[0].skipped_reason.as_deref(),
            Some("filter mismatch")
        );
        assert_eq!(
            explanation.routes[0].filter_results[0].actual.as_deref(),
            Some("telegram")
        );
        assert_eq!(
            explanation.routes[1].skipped_reason.as_deref(),
            Some("route disabled")
        );
        assert!(explanation.routes[2].matched);
        assert_eq!(
            explanation.routes[2].skipped_reason.as_deref(),
            Some("missing delivery target")
        );
    }

    #[test]
    fn explain_reports_no_deliveries_when_no_route_matches() {
        let config = AppConfig {
            routes: vec![RouteRule {
                event: "hermes.session.*".to_string(),
                channel: Some("sessions".to_string()),
                ..RouteRule::default()
            }],
            ..AppConfig::default()
        };
        let envelope = envelope(
            "hermes.agent.started",
            json!({
                "platform": "telegram",
                "session_id": "demo"
            }),
        );

        let explanation = Router::new(config).explain(&envelope);
        let output = explanation.to_string();

        assert!(explanation.deliveries.is_empty());
        assert_eq!(
            explanation.routes[0].skipped_reason.as_deref(),
            Some("event pattern mismatch")
        );
        assert!(output.contains("deliveries: (none)"));
    }

    #[test]
    fn git_route_filters_match_repo_and_branch_metadata() {
        let config = AppConfig {
            routes: vec![
                RouteRule {
                    event: "git.*".to_string(),
                    filter: BTreeMap::from([
                        ("repo_name".to_string(), "hermeship".to_string()),
                        ("branch".to_string(), "codex/*".to_string()),
                    ]),
                    channel: Some("git-alerts".to_string()),
                    ..RouteRule::default()
                },
                RouteRule {
                    event: "git.*".to_string(),
                    filter: BTreeMap::from([("repo_name".to_string(), "other".to_string())]),
                    channel: Some("wrong-repo".to_string()),
                    ..RouteRule::default()
                },
            ],
            ..AppConfig::default()
        };
        let envelope = envelope(
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

        let explanation = Router::new(config).explain(&envelope);

        assert_eq!(explanation.canonical_kind, "git.branch-changed");
        assert_eq!(
            explanation.route_candidates,
            vec!["git.branch-changed", "git.*"]
        );
        assert_eq!(explanation.deliveries.len(), 1);
        assert_eq!(
            explanation.deliveries[0].target,
            SinkTarget::DiscordChannel("git-alerts".to_string())
        );
        assert!(explanation.routes[0].matched);
        assert_eq!(
            explanation.routes[0].filter_results[0].actual.as_deref(),
            Some("codex/milestone-8-git")
        );
        assert_eq!(
            explanation.routes[0].filter_results[1].actual.as_deref(),
            Some("hermeship")
        );
        assert_eq!(
            explanation.routes[1].skipped_reason.as_deref(),
            Some("filter mismatch")
        );
    }

    #[test]
    fn explain_redacts_webhook_targets_in_diagnostics() {
        let raw_webhook = "https://discord.com/api/webhooks/synthetic-id/synthetic-secret-token";
        let config = AppConfig {
            routes: vec![RouteRule {
                event: "hermes.agent.*".to_string(),
                webhook: Some(raw_webhook.to_string()),
                ..RouteRule::default()
            }],
            ..AppConfig::default()
        };
        let envelope = envelope(
            "hermes.agent.started",
            json!({
                "platform": "telegram",
                "session_id": "demo"
            }),
        );

        let explanation = Router::new(config).explain(&envelope);
        let output = explanation.to_string();
        let serialized = serde_json::to_string(&explanation).unwrap();

        assert!(matches!(
            &explanation.deliveries[0].target,
            SinkTarget::DiscordWebhook(value) if value == raw_webhook
        ));
        assert!(output.contains("DiscordWebhook(\"[REDACTED]\")"));
        assert!(serialized.contains("DiscordWebhook"));
        assert!(serialized.contains("[REDACTED]"));
        assert!(!output.contains(raw_webhook));
        assert!(!serialized.contains(raw_webhook));
    }

    #[test]
    fn resolve_uses_event_hints_and_default_channel_when_route_target_is_absent() {
        let config = AppConfig {
            defaults: DefaultsConfig {
                channel: Some("fallback".to_string()),
                format: MessageFormat::Inline,
                ..DefaultsConfig::default()
            },
            routes: vec![RouteRule {
                event: "hermes.session.*".to_string(),
                ..RouteRule::default()
            }],
            ..AppConfig::default()
        };
        let envelope = envelope_with_hints(IncomingEvent {
            kind: "hermes.session.finished".to_string(),
            channel: Some("event-channel".to_string()),
            mention: Some("@session".to_string()),
            format: Some(MessageFormat::Raw),
            template: Some("session {session_id}".to_string()),
            payload: json!({ "session_id": "demo" }),
        });

        let deliveries = Router::new(config).resolve(&envelope);

        assert_eq!(deliveries.len(), 1);
        assert_eq!(
            deliveries[0].target,
            SinkTarget::DiscordChannel("event-channel".to_string())
        );
        assert_eq!(deliveries[0].format, MessageFormat::Raw);
        assert_eq!(deliveries[0].mention.as_deref(), Some("@session"));
        assert_eq!(
            deliveries[0].template.as_deref(),
            Some("session {session_id}")
        );
    }

    #[test]
    fn glob_match_supports_exact_prefix_suffix_and_middle_wildcards() {
        assert!(glob_match("hermes.agent.*", "hermes.agent.started"));
        assert!(glob_match("*.agent.started", "hermes.agent.started"));
        assert!(glob_match("hermes.*.started", "hermes.agent.started"));
        assert!(glob_match("*agent*", "hermes.agent.started"));
        assert!(!glob_match("hermes.session.*", "hermes.agent.started"));
    }

    fn envelope(kind: &str, payload: serde_json::Value) -> crate::event::EventEnvelope {
        envelope_with_hints(IncomingEvent::new(kind, payload))
    }

    fn envelope_with_hints(event: IncomingEvent) -> crate::event::EventEnvelope {
        from_incoming_event(&event).unwrap()
    }
}
