use std::collections::BTreeSet;

use anyhow::{Context, Result};
use axum::{Json, Router, extract::State, routing::get};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;

use crate::config::AppConfig;

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
}

#[derive(Clone)]
struct DaemonState {
    health: HealthResponse,
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
    let state = DaemonState {
        health: HealthResponse::from_config(&config, version),
    };

    Router::new()
        .route("/health", get(health))
        .with_state(state)
}

async fn health(State(state): State<DaemonState>) -> Json<HealthResponse> {
    Json(state.health)
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

    use crate::config::{AppConfig, RouteRule};

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
        assert_eq!(health.queue.status, "not_configured");
        assert_eq!(health.configured_sinks, vec!["discord"]);
    }
}
