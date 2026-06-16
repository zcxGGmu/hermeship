use std::collections::BTreeSet;
use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::header::{AUTHORIZATION, HeaderValue};
use serde::Serialize;

use crate::config::AppConfig;
use crate::router::SinkTarget;

use super::{Sink, SinkFuture, SinkMessage};

const DISCORD_API_BASE: &str = "https://discord.com/api/v10";
const DISCORD_CONTENT_LIMIT: usize = 2000;
const DISCORD_TRUNCATION_MARKER: &str = "...";

#[derive(Debug, Clone)]
pub struct DiscordSink {
    bot_token: Option<String>,
    api_base: String,
    http: reqwest::Client,
}

impl DiscordSink {
    pub fn from_config(config: &AppConfig) -> Result<Self> {
        Self::new(config.providers.discord.token.clone(), DISCORD_API_BASE)
    }

    fn new(bot_token: Option<String>, api_base: impl Into<String>) -> Result<Self> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .context("failed to build Discord HTTP client")?;

        Ok(Self {
            bot_token: normalize_text(bot_token),
            api_base: api_base.into().trim_end_matches('/').to_string(),
            http,
        })
    }

    fn prepare_request(
        &self,
        target: &SinkTarget,
        message: &SinkMessage,
    ) -> Result<PreparedDiscordRequest> {
        let payload = DiscordMessagePayload::from_message(message)?;

        match target {
            SinkTarget::DiscordChannel(channel) => {
                let channel = normalize_text(Some(channel.clone())).ok_or_else(|| {
                    anyhow::anyhow!("missing Discord channel for channel delivery")
                })?;
                let token = self.bot_token.as_deref().ok_or_else(|| {
                    anyhow::anyhow!(
                        "missing Discord bot token for channel delivery; configure [providers.discord].token or use a route webhook"
                    )
                })?;

                Ok(PreparedDiscordRequest {
                    url: format!("{}/channels/{channel}/messages", self.api_base),
                    authorization: Some(format!("Bot {token}")),
                    payload,
                })
            }
            SinkTarget::DiscordWebhook(webhook) => {
                let webhook = normalize_text(Some(webhook.clone())).ok_or_else(|| {
                    anyhow::anyhow!("missing Discord webhook for webhook delivery")
                })?;

                Ok(PreparedDiscordRequest {
                    url: webhook_url_with_wait(&webhook)?,
                    authorization: None,
                    payload,
                })
            }
        }
    }

    async fn execute(&self, request: PreparedDiscordRequest) -> Result<()> {
        let mut builder = self.http.post(&request.url).json(&request.payload);
        if let Some(authorization) = request.authorization.as_deref() {
            builder = builder.header(AUTHORIZATION, HeaderValue::from_str(authorization)?);
        }

        let response = builder
            .send()
            .await
            .context("Discord request transport failed")?;
        let status = response.status();
        if status.is_success() {
            return Ok(());
        }

        let body = response.text().await.unwrap_or_default();
        anyhow::bail!(
            "Discord request failed with HTTP {status}: {}",
            body_tail(&body)
        );
    }

    #[cfg(test)]
    fn for_tests(bot_token: Option<String>, api_base: &str) -> Self {
        Self::new(bot_token, api_base).unwrap()
    }
}

impl Sink for DiscordSink {
    fn send<'a>(&'a self, target: &'a SinkTarget, message: &'a SinkMessage) -> SinkFuture<'a> {
        Box::pin(async move {
            let request = self.prepare_request(target, message)?;
            self.execute(request).await
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PreparedDiscordRequest {
    url: String,
    authorization: Option<String>,
    payload: DiscordMessagePayload,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct DiscordMessagePayload {
    content: String,
    allowed_mentions: DiscordAllowedMentions,
}

impl DiscordMessagePayload {
    fn from_message(message: &SinkMessage) -> Result<Self> {
        let content = normalize_text(Some(message.content.clone()))
            .ok_or_else(|| anyhow::anyhow!("Discord message content must not be empty"))?;

        Ok(Self {
            content: truncate_discord_content(&content),
            allowed_mentions: DiscordAllowedMentions::from_mention(message.mention.as_deref()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct DiscordAllowedMentions {
    parse: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    users: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    roles: Vec<String>,
}

impl DiscordAllowedMentions {
    fn from_mention(mention: Option<&str>) -> Self {
        let (users, roles) = mention
            .and_then(|mention| normalize_text(Some(mention.to_string())))
            .map(|mention| parse_allowed_mentions(&mention))
            .unwrap_or_default();

        Self {
            parse: Vec::new(),
            users,
            roles,
        }
    }
}

fn parse_allowed_mentions(mention: &str) -> (Vec<String>, Vec<String>) {
    let mut users = Vec::new();
    let mut roles = Vec::new();
    let mut seen_users = BTreeSet::new();
    let mut seen_roles = BTreeSet::new();
    let bytes = mention.as_bytes();
    let mut index = 0;

    while index + 3 < bytes.len() {
        if bytes[index] != b'<' || bytes[index + 1] != b'@' {
            index += 1;
            continue;
        }

        let mut cursor = index + 2;
        let is_role = bytes.get(cursor) == Some(&b'&');
        if is_role || bytes.get(cursor) == Some(&b'!') {
            cursor += 1;
        }

        let id_start = cursor;
        while cursor < bytes.len() && bytes[cursor].is_ascii_digit() {
            cursor += 1;
        }

        if cursor > id_start && bytes.get(cursor) == Some(&b'>') {
            let id = &mention[id_start..cursor];
            if is_role {
                if seen_roles.insert(id.to_string()) {
                    roles.push(id.to_string());
                }
            } else if seen_users.insert(id.to_string()) {
                users.push(id.to_string());
            }
            index = cursor + 1;
        } else {
            index += 1;
        }
    }

    (users, roles)
}

fn truncate_discord_content(content: &str) -> String {
    if content.chars().count() <= DISCORD_CONTENT_LIMIT {
        return content.to_string();
    }

    let keep = DISCORD_CONTENT_LIMIT.saturating_sub(DISCORD_TRUNCATION_MARKER.chars().count());
    let mut truncated: String = content.chars().take(keep).collect();
    truncated.push_str(DISCORD_TRUNCATION_MARKER);
    truncated
}

fn webhook_url_with_wait(webhook_url: &str) -> Result<String> {
    let mut url =
        reqwest::Url::parse(webhook_url).context("Discord webhook URL must be absolute")?;
    let pairs: Vec<(String, String)> = url
        .query_pairs()
        .filter(|(key, _)| key != "wait")
        .map(|(key, value)| (key.into_owned(), value.into_owned()))
        .collect();

    url.set_query(None);
    {
        let mut query = url.query_pairs_mut();
        for (key, value) in pairs {
            query.append_pair(&key, &value);
        }
        query.append_pair("wait", "true");
    }
    Ok(url.to_string())
}

fn body_tail(body: &str) -> String {
    const MAX_BODY_CHARS: usize = 512;
    let body = body.trim();
    if body.chars().count() <= MAX_BODY_CHARS {
        return body.to_string();
    }

    body.chars().take(MAX_BODY_CHARS).collect()
}

fn normalize_text(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_string())
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    use crate::config::MessageFormat;
    use crate::router::SinkTarget;
    use crate::sink::{Sink, SinkMessage};

    use super::*;

    #[test]
    fn webhook_payload_blocks_unconfigured_mentions() {
        let message = message(
            "<@123> hermes agent started; body mentions <@999> and @everyone",
            Some("<@123>"),
        );

        let payload = DiscordMessagePayload::from_message(&message).unwrap();
        let serialized = serde_json::to_value(&payload).unwrap();

        assert_eq!(
            serialized,
            json!({
                "content": "<@123> hermes agent started; body mentions <@999> and @everyone",
                "allowed_mentions": {
                    "parse": [],
                    "users": ["123"]
                }
            })
        );
    }

    #[test]
    fn allowed_mentions_include_user_and_role_ids_from_explicit_mention() {
        let message = message(
            "<@123> <@!456> <@&789> hermes agent failed",
            Some("<@123> <@!456> <@&789> @everyone"),
        );

        let payload = DiscordMessagePayload::from_message(&message).unwrap();
        let serialized = serde_json::to_value(&payload).unwrap();

        assert_eq!(serialized["allowed_mentions"]["parse"], json!([]));
        assert_eq!(
            serialized["allowed_mentions"]["users"],
            json!(["123", "456"])
        );
        assert_eq!(serialized["allowed_mentions"]["roles"], json!(["789"]));
        assert!(serialized["allowed_mentions"]["everyone"].is_null());
    }

    #[test]
    fn payload_truncates_content_to_discord_limit() {
        let over_limit = "x".repeat(DISCORD_CONTENT_LIMIT + 24);
        let message = message(&over_limit, None);

        let payload = DiscordMessagePayload::from_message(&message).unwrap();

        assert_eq!(payload.content.chars().count(), DISCORD_CONTENT_LIMIT);
        assert!(payload.content.ends_with(DISCORD_TRUNCATION_MARKER));
    }

    #[test]
    fn bot_channel_request_uses_configured_token_and_api_base() {
        let sink = DiscordSink::for_tests(
            Some("synthetic-token".to_string()),
            "http://discord.test/api",
        );
        let message = message("hermes agent started", None);

        let request = sink
            .prepare_request(
                &SinkTarget::DiscordChannel("ops-channel".to_string()),
                &message,
            )
            .unwrap();

        assert_eq!(
            request.url,
            "http://discord.test/api/channels/ops-channel/messages"
        );
        assert_eq!(
            request.authorization.as_deref(),
            Some("Bot synthetic-token")
        );
        assert_eq!(request.payload.content, "hermes agent started");
    }

    #[test]
    fn webhook_request_uses_route_webhook_and_wait_true() {
        let sink = DiscordSink::for_tests(None, "http://discord.test/api");
        let message = message("hermes session finished", None);

        let request = sink
            .prepare_request(
                &SinkTarget::DiscordWebhook(
                    "http://local.test/webhook?thread_id=synthetic-thread".to_string(),
                ),
                &message,
            )
            .unwrap();

        assert_eq!(
            request.url,
            "http://local.test/webhook?thread_id=synthetic-thread&wait=true"
        );
        assert_eq!(request.authorization, None);
        assert_eq!(request.payload.content, "hermes session finished");
    }

    #[test]
    fn webhook_request_forces_wait_true_without_misreading_other_query_keys() {
        let sink = DiscordSink::for_tests(None, "http://discord.test/api");
        let message = message("hermes session finished", None);

        let request = sink
            .prepare_request(
                &SinkTarget::DiscordWebhook(
                    "http://local.test/webhook?await=true&wait=false&thread_id=1".to_string(),
                ),
                &message,
            )
            .unwrap();

        assert_eq!(
            request.url,
            "http://local.test/webhook?await=true&thread_id=1&wait=true"
        );
    }

    #[test]
    fn request_builder_reports_missing_token_channel_and_webhook() {
        let no_token = DiscordSink::for_tests(None, "http://discord.test/api");
        let with_token = DiscordSink::for_tests(
            Some("synthetic-token".to_string()),
            "http://discord.test/api",
        );
        let message = message("hermes agent started", None);

        let token_error = no_token
            .prepare_request(&SinkTarget::DiscordChannel("ops".to_string()), &message)
            .unwrap_err()
            .to_string();
        assert!(
            token_error.contains("missing Discord bot token"),
            "{token_error}"
        );

        let channel_error = with_token
            .prepare_request(&SinkTarget::DiscordChannel(" ".to_string()), &message)
            .unwrap_err()
            .to_string();
        assert!(
            channel_error.contains("missing Discord channel"),
            "{channel_error}"
        );

        let webhook_error = no_token
            .prepare_request(&SinkTarget::DiscordWebhook(" ".to_string()), &message)
            .unwrap_err()
            .to_string();
        assert!(
            webhook_error.contains("missing Discord webhook"),
            "{webhook_error}"
        );
    }

    #[tokio::test]
    async fn sink_posts_webhook_payload_to_fake_http() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut buffer = vec![0_u8; 4096];
            let read = stream.read(&mut buffer).await.unwrap();
            let request = String::from_utf8_lossy(&buffer[..read]).to_string();
            stream
                .write_all(b"HTTP/1.1 204 No Content\r\ncontent-length: 0\r\n\r\n")
                .await
                .unwrap();
            request
        });
        let sink = DiscordSink::for_tests(None, "http://discord.test/api");
        let message = message("<@123> hermes agent started", Some("<@123>"));

        sink.send(
            &SinkTarget::DiscordWebhook(format!("http://{address}/webhook")),
            &message,
        )
        .await
        .unwrap();
        let request = server.await.unwrap();

        assert!(
            request.starts_with("POST /webhook?wait=true HTTP/1.1"),
            "{request}"
        );
        assert!(request.contains(r#""content":"<@123> hermes agent started""#));
        assert!(request.contains(r#""allowed_mentions":{"parse":[],"users":["123"]}"#));
    }

    fn message(content: &str, mention: Option<&str>) -> SinkMessage {
        SinkMessage {
            event_kind: "hermes.agent.started".to_string(),
            format: MessageFormat::Compact,
            content: content.to_string(),
            mention: mention.map(ToString::to_string),
            matched_route_index: Some(0),
        }
    }
}
