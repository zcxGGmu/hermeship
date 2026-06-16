use std::collections::HashMap;
use std::sync::Arc;

use crate::event::EventEnvelope;
use crate::render::Renderer;
use crate::router::{ResolvedDelivery, Router, SinkTarget};
use crate::sink::{Sink, SinkMessage};

#[derive(Clone)]
pub struct Dispatcher {
    router: Router,
    renderer: Arc<dyn Renderer>,
    sinks: HashMap<String, Arc<dyn Sink>>,
}

impl Dispatcher {
    pub fn new(
        router: Router,
        renderer: Arc<dyn Renderer>,
        sinks: HashMap<String, Arc<dyn Sink>>,
    ) -> Self {
        Self {
            router,
            renderer,
            sinks,
        }
    }

    pub async fn run(&self, mut receiver: tokio::sync::mpsc::Receiver<EventEnvelope>) {
        while let Some(event) = receiver.recv().await {
            let report = self.dispatch_event(&event).await;
            report.log_failures();
        }
    }

    pub async fn run_queue(
        &self,
        mut receiver: tokio::sync::mpsc::Receiver<EventEnvelope>,
    ) -> Vec<DispatchReport> {
        let mut reports = Vec::new();
        while let Some(event) = receiver.recv().await {
            reports.push(self.dispatch_event(&event).await);
        }
        reports
    }

    pub async fn dispatch_event(&self, event: &EventEnvelope) -> DispatchReport {
        let deliveries = self.router.resolve(event);
        let mut report = DispatchReport::new(event.canonical_kind(), deliveries.len());

        for delivery in deliveries {
            report
                .outcomes
                .push(self.dispatch_delivery(event, &delivery).await);
        }

        report.delivered = report
            .outcomes
            .iter()
            .filter(|outcome| outcome.status == DeliveryStatus::Delivered)
            .count();
        report.failed = report.outcomes.len().saturating_sub(report.delivered);
        report
    }

    async fn dispatch_delivery(
        &self,
        event: &EventEnvelope,
        delivery: &ResolvedDelivery,
    ) -> DeliveryOutcome {
        let Some(sink) = self.sinks.get(&delivery.sink) else {
            return DeliveryOutcome::failed(
                delivery,
                DeliveryStatus::MissingSink,
                format!("missing sink {:?}", delivery.sink),
            );
        };

        let rendered = match self.renderer.render(event, delivery) {
            Ok(rendered) => rendered,
            Err(error) => {
                return DeliveryOutcome::failed(
                    delivery,
                    DeliveryStatus::RenderFailed,
                    error.to_string(),
                );
            }
        };

        let message = SinkMessage {
            event_kind: event.canonical_kind().to_string(),
            format: rendered.format,
            content: rendered.content,
            mention: delivery.mention.clone(),
            matched_route_index: delivery.matched_route_index,
        };

        match sink.send(&delivery.target, &message).await {
            Ok(()) => DeliveryOutcome::delivered(delivery),
            Err(error) => {
                DeliveryOutcome::failed(delivery, DeliveryStatus::SinkFailed, error.to_string())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DispatchReport {
    pub event_kind: String,
    pub resolved_deliveries: usize,
    pub delivered: usize,
    pub failed: usize,
    pub outcomes: Vec<DeliveryOutcome>,
}

impl DispatchReport {
    fn new(event_kind: &str, resolved_deliveries: usize) -> Self {
        Self {
            event_kind: event_kind.to_string(),
            resolved_deliveries,
            delivered: 0,
            failed: 0,
            outcomes: Vec::new(),
        }
    }

    fn log_failures(&self) {
        for outcome in &self.outcomes {
            if outcome.status == DeliveryStatus::Delivered {
                continue;
            }
            let error = outcome.error.as_deref().unwrap_or("unknown dispatch error");
            eprintln!(
                "hermeship dispatcher failed event={} sink={} target={} route={:?}: {}",
                self.event_kind, outcome.sink, outcome.target, outcome.matched_route_index, error
            );
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeliveryOutcome {
    pub sink: String,
    pub target: SinkTarget,
    pub matched_route_index: Option<usize>,
    pub status: DeliveryStatus,
    pub error: Option<String>,
}

impl DeliveryOutcome {
    fn delivered(delivery: &ResolvedDelivery) -> Self {
        Self {
            sink: delivery.sink.clone(),
            target: delivery.target.clone(),
            matched_route_index: delivery.matched_route_index,
            status: DeliveryStatus::Delivered,
            error: None,
        }
    }

    fn failed(
        delivery: &ResolvedDelivery,
        status: DeliveryStatus,
        error: impl Into<String>,
    ) -> Self {
        Self {
            sink: delivery.sink.clone(),
            target: delivery.target.clone(),
            matched_route_index: delivery.matched_route_index,
            status,
            error: Some(error.into()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryStatus {
    Delivered,
    MissingSink,
    RenderFailed,
    SinkFailed,
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::Duration;

    use anyhow::Result;
    use serde_json::{Value, json};
    use tokio::sync::mpsc;

    use crate::config::{AppConfig, MessageFormat, RouteRule};
    use crate::daemon::daemon_router_with_queue;
    use crate::event::{EventEnvelope, compat::from_incoming_event};
    use crate::events::IncomingEvent;
    use crate::render::{DefaultRenderer, RenderedMessage, Renderer};
    use crate::router::{ResolvedDelivery, Router, SinkTarget};
    use crate::sink::Sink;
    use crate::sink::fake::FakeSink;

    use super::{DeliveryStatus, Dispatcher};

    #[tokio::test]
    async fn dispatch_event_routes_renders_and_sends_multiple_deliveries() {
        let config = config_with_routes(vec![
            route("hermes.agent.*", "ops", MessageFormat::Compact),
            route("hermes.*", "audit", MessageFormat::Inline),
        ]);
        let event = envelope("hermes.agent.started", agent_payload());
        let fake = FakeSink::default();
        let dispatcher = dispatcher(config, Arc::new(DefaultRenderer), sinks(&fake));

        let report = dispatcher.dispatch_event(&event).await;

        assert_eq!(report.event_kind, "hermes.agent.started");
        assert_eq!(report.resolved_deliveries, 2);
        assert_eq!(report.delivered, 2);
        assert_eq!(report.failed, 0);
        assert_eq!(
            report
                .outcomes
                .iter()
                .map(|outcome| outcome.status)
                .collect::<Vec<_>>(),
            vec![DeliveryStatus::Delivered, DeliveryStatus::Delivered]
        );

        let deliveries = fake.deliveries();
        assert_eq!(deliveries.len(), 2);
        assert_eq!(
            deliveries[0].target,
            SinkTarget::DiscordChannel("ops".to_string())
        );
        assert_eq!(deliveries[0].message.event_kind, "hermes.agent.started");
        assert_eq!(deliveries[0].message.format, MessageFormat::Compact);
        assert_eq!(deliveries[0].message.matched_route_index, Some(0));
        assert!(
            deliveries[0]
                .message
                .content
                .contains("hermes agent started")
        );
        assert_eq!(
            deliveries[1].target,
            SinkTarget::DiscordChannel("audit".to_string())
        );
        assert_eq!(deliveries[1].message.format, MessageFormat::Inline);
        assert_eq!(deliveries[1].message.matched_route_index, Some(1));
    }

    #[tokio::test]
    async fn dispatch_event_keeps_delivering_after_one_sink_failure() {
        let config = config_with_routes(vec![
            route("hermes.agent.*", "ops", MessageFormat::Compact),
            route("hermes.*", "audit", MessageFormat::Compact),
        ]);
        let event = envelope("hermes.agent.started", agent_payload());
        let fake = FakeSink::default();
        fake.fail_route_index(0);
        let dispatcher = dispatcher(config, Arc::new(DefaultRenderer), sinks(&fake));

        let report = dispatcher.dispatch_event(&event).await;

        assert_eq!(report.resolved_deliveries, 2);
        assert_eq!(report.delivered, 1);
        assert_eq!(report.failed, 1);
        assert_eq!(report.outcomes[0].status, DeliveryStatus::SinkFailed);
        assert!(
            report.outcomes[0]
                .error
                .as_deref()
                .unwrap_or_default()
                .contains("synthetic fake sink failure")
        );
        assert_eq!(report.outcomes[1].status, DeliveryStatus::Delivered);

        let deliveries = fake.deliveries();
        assert_eq!(deliveries.len(), 1);
        assert_eq!(deliveries[0].message.matched_route_index, Some(1));
        assert_eq!(
            deliveries[0].target,
            SinkTarget::DiscordChannel("audit".to_string())
        );
    }

    #[tokio::test]
    async fn dispatch_event_passes_delivery_mention_to_sink_message() {
        let config = config_with_routes(vec![RouteRule {
            event: "hermes.agent.*".to_string(),
            channel: Some("ops".to_string()),
            mention: Some("<@123>".to_string()),
            format: Some(MessageFormat::Compact),
            ..RouteRule::default()
        }]);
        let event = envelope("hermes.agent.started", agent_payload());
        let fake = FakeSink::default();
        let dispatcher = dispatcher(config, Arc::new(DefaultRenderer), sinks(&fake));

        let report = dispatcher.dispatch_event(&event).await;

        assert_eq!(report.delivered, 1);
        let deliveries = fake.deliveries();
        assert_eq!(deliveries.len(), 1);
        assert_eq!(deliveries[0].message.mention.as_deref(), Some("<@123>"));
        assert!(deliveries[0].message.content.starts_with("<@123> "));
    }

    #[tokio::test]
    async fn dispatch_event_reports_no_route_without_sink_calls() {
        let config = config_with_routes(vec![route(
            "hermes.session.*",
            "sessions",
            MessageFormat::Compact,
        )]);
        let event = envelope("hermes.agent.started", agent_payload());
        let fake = FakeSink::default();
        let dispatcher = dispatcher(config, Arc::new(DefaultRenderer), sinks(&fake));

        let report = dispatcher.dispatch_event(&event).await;

        assert_eq!(report.event_kind, "hermes.agent.started");
        assert_eq!(report.resolved_deliveries, 0);
        assert_eq!(report.delivered, 0);
        assert_eq!(report.failed, 0);
        assert!(report.outcomes.is_empty());
        assert!(fake.deliveries().is_empty());
    }

    #[tokio::test]
    async fn dispatch_event_reports_renderer_failure_without_calling_sink() {
        let config =
            config_with_routes(vec![route("hermes.agent.*", "ops", MessageFormat::Compact)]);
        let event = envelope("hermes.agent.started", agent_payload());
        let fake = FakeSink::default();
        let dispatcher = dispatcher(config, Arc::new(FailingRenderer), sinks(&fake));

        let report = dispatcher.dispatch_event(&event).await;

        assert_eq!(report.resolved_deliveries, 1);
        assert_eq!(report.delivered, 0);
        assert_eq!(report.failed, 1);
        assert_eq!(report.outcomes[0].status, DeliveryStatus::RenderFailed);
        assert!(
            report.outcomes[0]
                .error
                .as_deref()
                .unwrap_or_default()
                .contains("synthetic render failure")
        );
        assert!(fake.deliveries().is_empty());
    }

    #[tokio::test]
    async fn dispatch_event_reports_missing_sink_without_panicking() {
        let config =
            config_with_routes(vec![route("hermes.agent.*", "ops", MessageFormat::Compact)]);
        let event = envelope("hermes.agent.started", agent_payload());
        let dispatcher = dispatcher(config, Arc::new(DefaultRenderer), HashMap::new());

        let report = dispatcher.dispatch_event(&event).await;

        assert_eq!(report.resolved_deliveries, 1);
        assert_eq!(report.delivered, 0);
        assert_eq!(report.failed, 1);
        assert_eq!(report.outcomes[0].status, DeliveryStatus::MissingSink);
        assert!(
            report.outcomes[0]
                .error
                .as_deref()
                .unwrap_or_default()
                .contains("missing sink")
        );
    }

    #[tokio::test]
    async fn dispatch_queue_consumes_events_until_sender_is_closed() {
        let config =
            config_with_routes(vec![route("hermes.agent.*", "ops", MessageFormat::Compact)]);
        let event = envelope("hermes.agent.started", agent_payload());
        let fake = FakeSink::default();
        let dispatcher = dispatcher(config, Arc::new(DefaultRenderer), sinks(&fake));
        let (tx, rx) = mpsc::channel(2);

        tx.send(event).await.unwrap();
        drop(tx);

        let reports = dispatcher.run_queue(rx).await;

        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].delivered, 1);
        assert_eq!(fake.deliveries().len(), 1);
    }

    #[tokio::test]
    async fn daemon_dispatch_e2e_uses_fixture_and_fake_sink_without_leaking_sensitive_fields() {
        let mut config =
            config_with_routes(vec![route("hermes.agent.*", "ops", MessageFormat::Compact)]);
        config.daemon.port = 0;
        let (queue_tx, queue_rx) = mpsc::channel(8);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(
                listener,
                daemon_router_with_queue(config.clone(), "test", queue_tx),
            )
            .await
        });

        let fake = FakeSink::default();
        let dispatcher = dispatcher(
            config_with_routes(vec![route("hermes.agent.*", "ops", MessageFormat::Compact)]),
            Arc::new(DefaultRenderer),
            sinks(&fake),
        );
        let dispatcher_task = tokio::spawn(async move { dispatcher.run_queue(queue_rx).await });

        let hook = hermes_hook_fixture_with_sensitive_fields();
        let accepted = crate::client::DaemonClient::from_base_url(format!("http://{address}"))
            .post_hermes_hook(&hook)
            .await
            .unwrap();

        let deliveries =
            tokio::time::timeout(Duration::from_secs(2), fake.wait_for_delivery_count(1))
                .await
                .unwrap();
        server.abort();
        let reports = dispatcher_task.await.unwrap();

        assert_eq!(accepted.canonical_kind, "hermes.agent.started");
        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].delivered, 1);
        assert_eq!(deliveries.len(), 1);
        let content = &deliveries[0].message.content;
        assert!(content.contains("hermes agent started"));
        assert!(content.contains("session=synthetic-session-001"));
        assert!(content.contains("message_chars="));

        for forbidden in [
            "synthetic full prompt should not leak",
            "synthetic full response should not leak",
            "synthetic-token-should-not-leak",
            "synthetic-cookie-should-not-leak",
            "synthetic-secret-should-not-leak",
        ] {
            assert!(!content.contains(forbidden), "leaked `{forbidden}`");
        }
    }

    #[derive(Debug)]
    struct FailingRenderer;

    impl Renderer for FailingRenderer {
        fn render(
            &self,
            _event: &EventEnvelope,
            _delivery: &ResolvedDelivery,
        ) -> Result<RenderedMessage> {
            anyhow::bail!("synthetic render failure")
        }
    }

    fn dispatcher(
        config: AppConfig,
        renderer: Arc<dyn Renderer>,
        sinks: HashMap<String, Arc<dyn Sink>>,
    ) -> Dispatcher {
        Dispatcher::new(Router::new(config), renderer, sinks)
    }

    fn sinks(fake: &FakeSink) -> HashMap<String, Arc<dyn Sink>> {
        let mut sinks: HashMap<String, Arc<dyn Sink>> = HashMap::new();
        sinks.insert("discord".to_string(), Arc::new(fake.clone()));
        sinks
    }

    fn config_with_routes(routes: Vec<RouteRule>) -> AppConfig {
        AppConfig {
            routes,
            ..AppConfig::default()
        }
    }

    fn route(event: &str, channel: &str, format: MessageFormat) -> RouteRule {
        RouteRule {
            event: event.to_string(),
            channel: Some(channel.to_string()),
            format: Some(format),
            ..RouteRule::default()
        }
    }

    fn envelope(kind: &str, payload: Value) -> EventEnvelope {
        from_incoming_event(&IncomingEvent::new(kind, payload)).unwrap()
    }

    fn agent_payload() -> Value {
        json!({
            "provider": "hermes",
            "source": "gateway",
            "platform": "telegram",
            "session_id": "synthetic-session-001",
            "agent_name": "demo-agent",
            "project": "hermes",
            "message_chars": 42,
            "has_message": true
        })
    }

    fn hermes_hook_fixture_with_sensitive_fields() -> crate::hermes::HermesHookEnvelope {
        let mut value: Value =
            serde_json::from_str(include_str!("../tests/fixtures/hermes/agent_start.json"))
                .unwrap();
        let context = value["context"].as_object_mut().unwrap();
        context.insert(
            "message".to_string(),
            json!("synthetic full prompt should not leak"),
        );
        context.insert(
            "response".to_string(),
            json!("synthetic full response should not leak"),
        );
        context.insert(
            "token".to_string(),
            json!("synthetic-token-should-not-leak"),
        );
        context.insert(
            "nested".to_string(),
            json!({
                "cookie": "synthetic-cookie-should-not-leak",
                "secret": "synthetic-secret-should-not-leak"
            }),
        );

        serde_json::from_value(value).unwrap()
    }
}
