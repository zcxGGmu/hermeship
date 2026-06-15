use std::time::Duration;

use anyhow::{Context, Result};

use crate::config::DaemonConfig;
use crate::daemon::{EventAcceptedResponse, HealthResponse};
use crate::events::IncomingEvent;

#[derive(Debug, Clone)]
pub struct DaemonClient {
    base_url: String,
    http: reqwest::Client,
}

impl DaemonClient {
    pub fn from_config(config: &DaemonConfig) -> Self {
        Self::from_base_url(config.base_url())
    }

    pub fn from_base_url(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(2))
                .build()
                .expect("reqwest client with a fixed timeout should build"),
        }
    }

    pub fn base_url(&self) -> &str {
        self.base_url.as_str()
    }

    pub fn health_url(&self) -> String {
        format!("{}/health", self.base_url)
    }

    pub fn event_url(&self) -> String {
        format!("{}/event", self.base_url)
    }

    pub async fn health(&self) -> Result<HealthResponse> {
        let url = self.health_url();
        let response = self
            .http
            .get(&url)
            .send()
            .await
            .with_context(|| format!("daemon is not reachable at {url}"))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("daemon health check failed at {url}: HTTP {status}: {body}");
        }

        response
            .json::<HealthResponse>()
            .await
            .with_context(|| format!("daemon returned invalid health response at {url}"))
    }

    pub async fn post_event(&self, event: &IncomingEvent) -> Result<EventAcceptedResponse> {
        let url = self.event_url();
        let response = self
            .http
            .post(&url)
            .json(event)
            .send()
            .await
            .with_context(|| format!("daemon is not reachable at {url}"))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("daemon event enqueue failed at {url}: HTTP {status}: {body}");
        }

        response
            .json::<EventAcceptedResponse>()
            .await
            .with_context(|| format!("daemon returned invalid event response at {url}"))
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::config::DaemonConfig;

    use super::*;

    #[test]
    fn client_uses_daemon_config_base_url() {
        let config = DaemonConfig {
            host: "0.0.0.0".to_string(),
            port: 25296,
            base_url: None,
        };

        let client = DaemonClient::from_config(&config);

        assert_eq!(client.base_url(), "http://127.0.0.1:25296");
        assert_eq!(client.health_url(), "http://127.0.0.1:25296/health");
        assert_eq!(client.event_url(), "http://127.0.0.1:25296/event");
    }

    #[tokio::test]
    async fn health_query_returns_clear_error_when_daemon_is_unavailable() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);

        let client = DaemonClient::from_base_url(format!("http://127.0.0.1:{port}"));

        let error = client.health().await.unwrap_err().to_string();

        assert!(error.contains("daemon is not reachable"), "{error}");
        assert!(error.contains("/health"), "{error}");
    }

    #[tokio::test]
    async fn event_post_returns_clear_error_when_daemon_is_unavailable() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);

        let client = DaemonClient::from_base_url(format!("http://127.0.0.1:{port}"));

        let error = client
            .post_event(&IncomingEvent::new(
                "hermes.agent.started",
                json!({ "session_id": "demo" }),
            ))
            .await
            .unwrap_err()
            .to_string();

        assert!(error.contains("daemon is not reachable"), "{error}");
        assert!(error.contains("/event"), "{error}");
    }
}
