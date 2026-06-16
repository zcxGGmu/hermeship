use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;

use anyhow::{Context, Result};
use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tokio::sync::mpsc;

use crate::config::AppConfig;
use crate::dispatch::Dispatcher;
use crate::event::{EventEnvelope, compat::from_incoming_event};
use crate::events::IncomingEvent;
use crate::hermes::HermesHookEnvelope;
use crate::privacy::sanitize_payload;
use crate::render::DefaultRenderer;
use crate::router::Router as EventRouter;
use crate::sink::{Sink, discord::DiscordSink};

pub const DEFAULT_QUEUE_CAPACITY: usize = 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HealthResponse {
    pub version: String,
    pub status: String,
    pub queue: QueueHealth,
    pub configured_sinks: Vec<String>,
}

impl HealthResponse {
    pub fn from_config(config: &AppConfig, version: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            status: "ok".to_string(),
            queue: QueueHealth::not_configured(),
            configured_sinks: configured_sinks(config),
        }
    }

    fn from_config_and_queue(
        config: &AppConfig,
        version: impl Into<String>,
        queue: &mpsc::Sender<EventEnvelope>,
    ) -> Self {
        Self {
            version: version.into(),
            status: "ok".to_string(),
            queue: QueueHealth::from_sender(queue),
            configured_sinks: configured_sinks(config),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QueueHealth {
    pub status: String,
    pub pending: usize,
    pub capacity: usize,
}

impl QueueHealth {
    fn not_configured() -> Self {
        Self {
            status: "not_configured".to_string(),
            pending: 0,
            capacity: 0,
        }
    }

    fn from_sender(sender: &mpsc::Sender<EventEnvelope>) -> Self {
        let capacity = sender.max_capacity();
        let available = sender.capacity();
        let pending = capacity.saturating_sub(available);
        let status = if sender.is_closed() {
            "closed"
        } else if available == 0 {
            "full"
        } else {
            "ready"
        };

        Self {
            status: status.to_string(),
            pending,
            capacity,
        }
    }
}

#[derive(Clone)]
struct DaemonState {
    config: AppConfig,
    version: String,
    queue: QueueHandle,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventAcceptedResponse {
    pub id: String,
    pub canonical_kind: String,
    pub queued: bool,
    pub queue: QueueHealth,
}

#[derive(Clone)]
struct QueueHandle {
    sender: mpsc::Sender<EventEnvelope>,
}

impl QueueHandle {
    fn from_sender(sender: mpsc::Sender<EventEnvelope>) -> Self {
        Self { sender }
    }
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

struct DaemonApiError {
    status: StatusCode,
    message: String,
}

impl DaemonApiError {
    fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.into(),
        }
    }

    fn queue_unavailable(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::SERVICE_UNAVAILABLE,
            message: message.into(),
        }
    }
}

impl IntoResponse for DaemonApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorResponse {
                error: self.message,
            }),
        )
            .into_response()
    }
}

pub async fn serve(config: AppConfig, version: impl Into<String>) -> Result<()> {
    let listener = bind_listener(&config).await?;
    serve_listener(listener, config, version).await
}

pub async fn bind_listener(config: &AppConfig) -> Result<TcpListener> {
    let address = format!("{}:{}", config.daemon.host, config.daemon.port);
    TcpListener::bind(&address)
        .await
        .with_context(|| format!("failed to bind hermeship daemon to {address}"))
}

pub async fn serve_listener(
    listener: TcpListener,
    config: AppConfig,
    version: impl Into<String>,
) -> Result<()> {
    let router = health_router(config, version);
    axum::serve(listener, router)
        .await
        .context("hermeship daemon server failed")
}

pub fn health_router(config: AppConfig, version: impl Into<String>) -> Router {
    daemon_router(config, version)
}

pub fn daemon_router(config: AppConfig, version: impl Into<String>) -> Router {
    let (queue_tx, queue_rx) = mpsc::channel(DEFAULT_QUEUE_CAPACITY);
    spawn_dispatcher(config.clone(), queue_rx);

    let state = DaemonState {
        config,
        version: version.into(),
        queue: QueueHandle::from_sender(queue_tx),
    };

    Router::new()
        .route("/health", get(health))
        .route("/event", post(event))
        .route("/api/hermes/hook", post(hermes_hook))
        .with_state(state)
}

fn spawn_dispatcher(config: AppConfig, queue_rx: mpsc::Receiver<EventEnvelope>) {
    let dispatcher = Dispatcher::new(
        EventRouter::new(config.clone()),
        Arc::new(DefaultRenderer),
        sink_registry_from_config(&config),
    );
    tokio::spawn(async move {
        dispatcher.run(queue_rx).await;
    });
}

fn sink_registry_from_config(config: &AppConfig) -> HashMap<String, Arc<dyn Sink>> {
    let mut sinks: HashMap<String, Arc<dyn Sink>> = HashMap::new();
    match DiscordSink::from_config(config) {
        Ok(sink) => {
            sinks.insert("discord".to_string(), Arc::new(sink));
        }
        Err(error) => {
            eprintln!("hermeship daemon failed to initialize Discord sink: {error}");
        }
    }
    sinks
}

pub fn daemon_router_with_queue(
    config: AppConfig,
    version: impl Into<String>,
    queue_tx: mpsc::Sender<EventEnvelope>,
) -> Router {
    let state = DaemonState {
        config,
        version: version.into(),
        queue: QueueHandle::from_sender(queue_tx),
    };

    Router::new()
        .route("/health", get(health))
        .route("/event", post(event))
        .route("/api/hermes/hook", post(hermes_hook))
        .with_state(state)
}

async fn health(State(state): State<DaemonState>) -> Json<HealthResponse> {
    Json(HealthResponse::from_config_and_queue(
        &state.config,
        state.version.clone(),
        &state.queue.sender,
    ))
}

async fn event(
    State(state): State<DaemonState>,
    Json(incoming): Json<IncomingEvent>,
) -> std::result::Result<(StatusCode, Json<EventAcceptedResponse>), DaemonApiError> {
    enqueue_incoming_event(&state, incoming)
}

async fn hermes_hook(
    State(state): State<DaemonState>,
    Json(hook): Json<HermesHookEnvelope>,
) -> std::result::Result<(StatusCode, Json<EventAcceptedResponse>), DaemonApiError> {
    let incoming = hook
        .into_incoming_event()
        .map_err(|error| DaemonApiError::bad_request(error.to_string()))?;
    enqueue_incoming_event(&state, incoming)
}

fn enqueue_incoming_event(
    state: &DaemonState,
    mut incoming: IncomingEvent,
) -> std::result::Result<(StatusCode, Json<EventAcceptedResponse>), DaemonApiError> {
    if incoming.kind.trim().is_empty() {
        return Err(DaemonApiError::bad_request(
            "event kind must not be empty; set type, kind, or event",
        ));
    }

    incoming.payload = sanitize_payload(&incoming.payload, &state.config.privacy);
    let envelope = from_incoming_event(&incoming)
        .map_err(|error| DaemonApiError::bad_request(format!("invalid event payload: {error}")))?;

    let id = envelope.id.to_string();
    let canonical_kind = envelope.canonical_kind().to_string();
    state.queue.sender.try_send(envelope).map_err(|error| {
        DaemonApiError::queue_unavailable(format!("event queue unavailable: {error}"))
    })?;

    Ok((
        StatusCode::ACCEPTED,
        Json(EventAcceptedResponse {
            id,
            canonical_kind,
            queued: true,
            queue: QueueHealth::from_sender(&state.queue.sender),
        }),
    ))
}

fn configured_sinks(config: &AppConfig) -> Vec<String> {
    let mut sinks = BTreeSet::new();

    for route in &config.routes {
        if route.enabled && !route.sink.trim().is_empty() {
            sinks.insert(route.sink.clone());
        }
    }

    if config.providers.discord.token.is_some() || config.defaults.channel.is_some() {
        sinks.insert("discord".to_string());
    }

    sinks.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use axum::http::header::CONTENT_TYPE;
    use serde_json::json;
    use tokio::sync::mpsc;

    use crate::config::{AppConfig, RouteRule};
    use crate::event::{EventBody, EventEnvelope};
    use crate::events::IncomingEvent;

    use super::*;

    #[test]
    fn health_response_reports_version_queue_and_configured_sinks() {
        let config = AppConfig {
            routes: vec![
                RouteRule {
                    event: "hermes.agent.*".to_string(),
                    sink: "discord".to_string(),
                    filter: BTreeMap::new(),
                    enabled: true,
                    ..RouteRule::default()
                },
                RouteRule {
                    event: "hermes.session.*".to_string(),
                    sink: "discord".to_string(),
                    filter: BTreeMap::new(),
                    enabled: false,
                    ..RouteRule::default()
                },
            ],
            ..AppConfig::default()
        };

        let health = HealthResponse::from_config(&config, "test-version");

        assert_eq!(health.version, "test-version");
        assert_eq!(health.status, "ok");
        assert_eq!(health.queue.status, "not_configured");
        assert_eq!(health.queue.pending, 0);
        assert_eq!(health.queue.capacity, 0);
        assert_eq!(health.configured_sinks, vec!["discord"]);
    }

    #[test]
    fn daemon_sink_registry_registers_discord_sink() {
        let sinks = sink_registry_from_config(&AppConfig::default());

        assert!(sinks.contains_key("discord"));
    }

    #[tokio::test]
    async fn health_endpoint_returns_schema_over_http() {
        let mut config = AppConfig::default();
        config.daemon.port = 0;
        config.routes.push(RouteRule {
            event: "hermes.agent.*".to_string(),
            sink: "discord".to_string(),
            enabled: true,
            ..RouteRule::default()
        });

        let listener = bind_listener(&config).await.unwrap();
        let address = listener.local_addr().unwrap();
        let server =
            tokio::spawn(async move { serve_listener(listener, config, "test-version").await });

        let client = crate::client::DaemonClient::from_base_url(format!("http://{address}"));
        let health = client.health().await.unwrap();
        server.abort();

        assert_eq!(health.version, "test-version");
        assert_eq!(health.status, "ok");
        assert_eq!(health.queue.status, "ready");
        assert_eq!(health.queue.pending, 0);
        assert!(health.queue.capacity > 0);
        assert_eq!(health.configured_sinks, vec!["discord"]);
    }

    #[tokio::test]
    async fn daemon_router_consumes_internal_queue_with_dispatcher() {
        let mut config = AppConfig::default();
        config.routes.push(RouteRule {
            event: "hermes.agent.*".to_string(),
            sink: "discord".to_string(),
            channel: Some("ops".to_string()),
            enabled: true,
            ..RouteRule::default()
        });
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let router = daemon_router(config, "test-version");
        let server = tokio::spawn(async move { axum::serve(listener, router).await });
        let client = crate::client::DaemonClient::from_base_url(format!("http://{address}"));

        client
            .post_event(&IncomingEvent::new(
                "hermes.agent.started",
                json!({ "session_id": "demo" }),
            ))
            .await
            .unwrap();

        tokio::time::timeout(std::time::Duration::from_secs(2), async {
            loop {
                if client.health().await.unwrap().queue.pending == 0 {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        server.abort();
    }

    #[tokio::test]
    async fn daemon_event_endpoint_accepts_incoming_event_and_enqueues_envelope() {
        let config = AppConfig::default();
        let (queue_tx, mut queue_rx) = mpsc::channel(8);
        let (base_url, server) = spawn_test_daemon(config, queue_tx).await;
        let client = crate::client::DaemonClient::from_base_url(base_url);
        let event: IncomingEvent =
            serde_json::from_str(include_str!("../tests/fixtures/hermes/agent_start.json"))
                .unwrap();

        let accepted = client.post_event(&event).await.unwrap();

        assert!(accepted.queued);
        assert_eq!(accepted.canonical_kind, "hermes.agent.started");
        assert_eq!(accepted.queue.status, "ready");
        assert_eq!(accepted.queue.pending, 1);
        assert_eq!(accepted.queue.capacity, 8);

        let envelope = queue_rx.try_recv().unwrap();
        server.abort();

        assert_eq!(envelope.id.to_string(), accepted.id);
        assert_eq!(envelope.canonical_kind(), "hermes.agent.started");
        assert_eq!(envelope.metadata.provider.as_deref(), Some("hermes"));
        assert_eq!(envelope.metadata.source.as_deref(), Some("gateway"));
        assert_eq!(envelope.metadata.platform.as_deref(), Some("telegram"));
        assert_eq!(
            envelope.metadata.session_id.as_deref(),
            Some("synthetic-session-001")
        );
        assert!(matches!(envelope.body, EventBody::HermesAgentStarted(_)));
    }

    #[tokio::test]
    async fn daemon_event_endpoint_sanitizes_payload_before_enqueuing() {
        let config = AppConfig::default();
        let (queue_tx, mut queue_rx) = mpsc::channel(8);
        let (base_url, server) = spawn_test_daemon(config, queue_tx).await;
        let client = crate::client::DaemonClient::from_base_url(base_url);
        let event = IncomingEvent::new(
            "plugin.custom",
            json!({
                "message": "synthetic full message should not leak",
                "response": "synthetic full response should not leak",
                "token": "synthetic-token-value",
                "cookie": "synthetic-cookie-value",
                "secret": "synthetic-secret-value",
                "provider_response": {
                    "body": "synthetic provider response should not leak"
                },
                "tool_result": {
                    "body": "synthetic tool result should not leak"
                }
            }),
        );

        let accepted = client.post_event(&event).await.unwrap();
        let envelope = queue_rx.try_recv().unwrap();
        server.abort();

        assert_eq!(accepted.canonical_kind, "plugin.custom");
        match &envelope.body {
            EventBody::Custom(body) => {
                assert_eq!(body.kind, "plugin.custom");
                assert_eq!(body.message, "plugin.custom");
                let payload = body.payload.as_ref().unwrap();
                assert_eq!(payload["has_message"], json!(true));
                assert_eq!(payload["has_response"], json!(true));
                assert_eq!(payload["message_chars"], json!(38));
                assert_eq!(payload["response_chars"], json!(39));
                assert_eq!(payload["token"], json!("[REDACTED]"));
                assert_eq!(payload["cookie"], json!("[REDACTED]"));
                assert_eq!(payload["secret"], json!("[REDACTED]"));
                assert!(payload.get("message").is_none());
                assert!(payload.get("response").is_none());
                assert!(payload.get("provider_response").is_none());
                assert!(payload.get("tool_result").is_none());
            }
            other => panic!("expected Custom event, got {other:?}"),
        }

        let rendered = format!("{envelope:?}");
        for forbidden in [
            "synthetic full message should not leak",
            "synthetic full response should not leak",
            "synthetic-token-value",
            "synthetic-cookie-value",
            "synthetic-secret-value",
            "synthetic provider response should not leak",
            "synthetic tool result should not leak",
        ] {
            assert!(!rendered.contains(forbidden), "leaked `{forbidden}`");
        }
    }

    #[tokio::test]
    async fn daemon_event_endpoint_returns_4xx_for_invalid_json_without_enqueuing() {
        let config = AppConfig::default();
        let (queue_tx, mut queue_rx) = mpsc::channel(8);
        let (base_url, server) = spawn_test_daemon(config, queue_tx).await;
        let response = reqwest::Client::new()
            .post(format!("{base_url}/event"))
            .header(CONTENT_TYPE, "application/json")
            .body(include_str!(
                "../tests/fixtures/hermes/invalid_payload.json"
            ))
            .send()
            .await
            .unwrap();

        let status = response.status();
        let body = response.text().await.unwrap();
        server.abort();

        assert!(
            status.is_client_error(),
            "expected 4xx, got {status}: {body}"
        );
        assert!(!body.trim().is_empty());
        assert!(queue_rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn daemon_event_endpoint_returns_4xx_for_missing_event_kind_without_enqueuing() {
        let config = AppConfig::default();
        let (queue_tx, mut queue_rx) = mpsc::channel(8);
        let (base_url, server) = spawn_test_daemon(config, queue_tx).await;
        let response = reqwest::Client::new()
            .post(format!("{base_url}/event"))
            .json(&json!({ "payload": { "session_id": "demo" } }))
            .send()
            .await
            .unwrap();

        let status = response.status();
        let body = response.text().await.unwrap();
        server.abort();

        assert!(
            status.is_client_error(),
            "expected 4xx, got {status}: {body}"
        );
        assert!(
            body.contains("missing field") || body.contains("event") || body.contains("type"),
            "unexpected error body: {body}"
        );
        assert!(queue_rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn daemon_health_reports_queue_pending_after_event_ingress() {
        let config = AppConfig::default();
        let (queue_tx, _queue_rx) = mpsc::channel(8);
        let (base_url, server) = spawn_test_daemon(config, queue_tx).await;
        let client = crate::client::DaemonClient::from_base_url(base_url);

        let initial = client.health().await.unwrap();
        assert_eq!(initial.queue.status, "ready");
        assert_eq!(initial.queue.pending, 0);
        assert_eq!(initial.queue.capacity, 8);

        client
            .post_event(&IncomingEvent::new(
                "hermes.agent.started",
                json!({ "session_id": "demo" }),
            ))
            .await
            .unwrap();

        let after = client.health().await.unwrap();
        server.abort();

        assert_eq!(after.queue.status, "ready");
        assert_eq!(after.queue.pending, 1);
        assert_eq!(after.queue.capacity, 8);
    }

    #[tokio::test]
    async fn daemon_event_endpoint_returns_503_when_queue_is_full() {
        let config = AppConfig::default();
        let (queue_tx, _queue_rx) = mpsc::channel(1);
        let (base_url, server) = spawn_test_daemon(config, queue_tx).await;
        let client = crate::client::DaemonClient::from_base_url(base_url);

        client
            .post_event(&IncomingEvent::new(
                "hermes.agent.started",
                json!({ "session_id": "first" }),
            ))
            .await
            .unwrap();

        let error = client
            .post_event(&IncomingEvent::new(
                "hermes.agent.started",
                json!({ "session_id": "second" }),
            ))
            .await
            .unwrap_err()
            .to_string();
        let health = client.health().await.unwrap();
        server.abort();

        assert!(error.contains("/event"), "{error}");
        assert!(error.contains("HTTP 503"), "{error}");
        assert!(error.contains("event queue unavailable"), "{error}");
        assert_eq!(health.queue.status, "full");
        assert_eq!(health.queue.pending, 1);
        assert_eq!(health.queue.capacity, 1);
    }

    #[tokio::test]
    async fn daemon_hermes_hook_endpoint_accepts_hook_and_enqueues_envelope() {
        let config = AppConfig::default();
        let (queue_tx, mut queue_rx) = mpsc::channel(8);
        let (base_url, server) = spawn_test_daemon(config, queue_tx).await;
        let response = reqwest::Client::new()
            .post(format!("{base_url}/api/hermes/hook"))
            .json(&json!({
                "event": "agent:start",
                "context": {
                    "platform": "telegram",
                    "session_id": "synthetic-session-001",
                    "agent_name": "demo-agent"
                }
            }))
            .send()
            .await
            .unwrap();

        let status = response.status();
        let accepted = response.json::<EventAcceptedResponse>().await.unwrap();

        assert_eq!(status, StatusCode::ACCEPTED);
        assert!(accepted.queued);
        assert_eq!(accepted.canonical_kind, "hermes.agent.started");
        assert_eq!(accepted.queue.pending, 1);

        let envelope = queue_rx.try_recv().unwrap();
        server.abort();

        assert_eq!(envelope.id.to_string(), accepted.id);
        assert_eq!(envelope.canonical_kind(), "hermes.agent.started");
        assert_eq!(envelope.metadata.provider.as_deref(), Some("hermes"));
        assert_eq!(envelope.metadata.source.as_deref(), Some("gateway"));
        assert_eq!(envelope.metadata.platform.as_deref(), Some("telegram"));
        assert_eq!(
            envelope.metadata.session_id.as_deref(),
            Some("synthetic-session-001")
        );
        assert!(matches!(envelope.body, EventBody::HermesAgentStarted(_)));
    }

    #[tokio::test]
    async fn daemon_hermes_hook_endpoint_sanitizes_payload_before_enqueuing() {
        let config = AppConfig::default();
        let (queue_tx, mut queue_rx) = mpsc::channel(8);
        let (base_url, server) = spawn_test_daemon(config, queue_tx).await;

        let accepted = reqwest::Client::new()
            .post(format!("{base_url}/api/hermes/hook"))
            .json(&json!({
                "event": "agent:end",
                "context": {
                    "session_id": "synthetic-session-privacy",
                    "agent_name": "demo-agent",
                    "success": false,
                    "message": "synthetic full message should not leak",
                    "response": "synthetic full response should not leak",
                    "token": "synthetic-token-value",
                    "cookie": "synthetic-cookie-value",
                    "secret": "synthetic-secret-value"
                }
            }))
            .send()
            .await
            .unwrap()
            .json::<EventAcceptedResponse>()
            .await
            .unwrap();

        let envelope = queue_rx.try_recv().unwrap();
        server.abort();

        assert_eq!(accepted.canonical_kind, "hermes.agent.failed");
        match &envelope.body {
            EventBody::HermesAgentFailed(body) => {
                assert_eq!(body.message_chars, Some(38));
                assert_eq!(body.response_chars, Some(39));
                assert_eq!(body.has_message, Some(true));
                assert_eq!(body.has_response, Some(true));
                assert_eq!(body.success, Some(false));
            }
            other => panic!("expected HermesAgentFailed event, got {other:?}"),
        }

        let rendered = format!("{envelope:?}");
        for forbidden in [
            "synthetic full message should not leak",
            "synthetic full response should not leak",
            "synthetic-token-value",
            "synthetic-cookie-value",
            "synthetic-secret-value",
        ] {
            assert!(!rendered.contains(forbidden), "leaked `{forbidden}`");
        }
    }

    #[tokio::test]
    async fn daemon_hermes_hook_endpoint_returns_4xx_for_missing_event_without_enqueuing() {
        let config = AppConfig::default();
        let (queue_tx, mut queue_rx) = mpsc::channel(8);
        let (base_url, server) = spawn_test_daemon(config, queue_tx).await;
        let response = reqwest::Client::new()
            .post(format!("{base_url}/api/hermes/hook"))
            .json(&json!({
                "context": {
                    "session_id": "synthetic-session-missing-event"
                }
            }))
            .send()
            .await
            .unwrap();

        let status = response.status();
        let body = response.text().await.unwrap();
        server.abort();

        assert!(
            status.is_client_error(),
            "expected 4xx, got {status}: {body}"
        );
        assert!(
            body.contains("Hermes hook event must not be empty") || body.contains("event"),
            "unexpected error body: {body}"
        );
        assert!(queue_rx.try_recv().is_err());
    }

    async fn spawn_test_daemon(
        config: AppConfig,
        queue_tx: mpsc::Sender<EventEnvelope>,
    ) -> (String, tokio::task::JoinHandle<anyhow::Result<()>>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let router = daemon_router_with_queue(config, "test-version", queue_tx);
        let server = tokio::spawn(async move {
            axum::serve(listener, router)
                .await
                .context("test daemon failed")
        });

        (format!("http://{address}"), server)
    }
}
