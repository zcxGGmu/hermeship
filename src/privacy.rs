use serde_json::{Map, Value, json};

use crate::config::PrivacyConfig;

const REDACTED: &str = "[REDACTED]";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExcerptPolicy {
    pub include_message_excerpt: bool,
    pub include_response_excerpt: bool,
    pub max_excerpt_chars: usize,
}

pub fn excerpt_policy(config: &PrivacyConfig) -> ExcerptPolicy {
    ExcerptPolicy {
        include_message_excerpt: config.include_message_excerpt,
        include_response_excerpt: config.include_response_excerpt,
        max_excerpt_chars: config.max_excerpt_chars.max(1),
    }
}

pub fn sanitize_payload(payload: &Value, config: &PrivacyConfig) -> Value {
    let policy = excerpt_policy(config);
    sanitize_value(payload, config, &policy)
}

pub fn redact_value(value: &Value, config: &PrivacyConfig) -> Value {
    match value {
        Value::Object(map) => Value::Object(redact_object(map, config)),
        Value::Array(items) => Value::Array(
            items
                .iter()
                .map(|item| redact_value(item, config))
                .collect(),
        ),
        Value::String(text) => Value::String(redact_inline_secrets(text, config)),
        _ => value.clone(),
    }
}

fn sanitize_value(value: &Value, config: &PrivacyConfig, policy: &ExcerptPolicy) -> Value {
    match value {
        Value::Object(map) => Value::Object(sanitize_object(map, config, policy)),
        Value::Array(items) => Value::Array(
            items
                .iter()
                .map(|item| sanitize_value(item, config, policy))
                .collect(),
        ),
        Value::String(text) => Value::String(redact_inline_secrets(text, config)),
        _ => value.clone(),
    }
}

fn redact_object(map: &Map<String, Value>, config: &PrivacyConfig) -> Map<String, Value> {
    map.iter()
        .map(|(key, value)| {
            let value = if is_sensitive_key(key, config) {
                Value::String(REDACTED.to_string())
            } else {
                redact_value(value, config)
            };
            (key.clone(), value)
        })
        .collect()
}

fn sanitize_object(
    map: &Map<String, Value>,
    config: &PrivacyConfig,
    policy: &ExcerptPolicy,
) -> Map<String, Value> {
    let mut sanitized = Map::new();

    for (key, value) in map {
        if let Some(body_kind) = body_kind_for_key(key) {
            add_body_summary(&mut sanitized, body_kind, value, config, policy);
            continue;
        }

        if is_summary_key(key) {
            if let Some(summary) = safe_summary_value(key, value) {
                sanitized.entry(key.clone()).or_insert(summary);
            }
            continue;
        }

        let value = if is_sensitive_key(key, config) {
            Value::String(REDACTED.to_string())
        } else {
            sanitize_value(value, config, policy)
        };
        sanitized.insert(key.clone(), value);
    }

    sanitized
}

fn add_body_summary(
    output: &mut Map<String, Value>,
    body_kind: BodyKind,
    value: &Value,
    config: &PrivacyConfig,
    policy: &ExcerptPolicy,
) {
    match body_kind {
        BodyKind::Message => {
            output.insert("has_message".to_string(), json!(!value.is_null()));
            output.insert("message_chars".to_string(), json!(body_char_count(value)));
            if policy.include_message_excerpt {
                output.insert(
                    "message_excerpt".to_string(),
                    Value::String(safe_excerpt(value, config, policy.max_excerpt_chars)),
                );
            }
        }
        BodyKind::Response => {
            output.insert("has_response".to_string(), json!(!value.is_null()));
            output.insert("response_chars".to_string(), json!(body_char_count(value)));
            if policy.include_response_excerpt {
                output.insert(
                    "response_excerpt".to_string(),
                    Value::String(safe_excerpt(value, config, policy.max_excerpt_chars)),
                );
            }
        }
        BodyKind::Drop => {}
    }
}

fn safe_excerpt(value: &Value, config: &PrivacyConfig, max_chars: usize) -> String {
    let policy = ExcerptPolicy {
        include_message_excerpt: false,
        include_response_excerpt: false,
        max_excerpt_chars: max_chars,
    };
    let sanitized = sanitize_value(value, config, &policy);
    let text = match sanitized {
        Value::String(text) => text,
        other => other.to_string(),
    };
    truncate_chars(&text, max_chars)
}

fn body_char_count(value: &Value) -> usize {
    match value {
        Value::String(text) => text.chars().count(),
        Value::Null => 0,
        other => other.to_string().chars().count(),
    }
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    value.chars().take(max_chars).collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BodyKind {
    Message,
    Response,
    Drop,
}

fn body_kind_for_key(key: &str) -> Option<BodyKind> {
    match canonical_key(key).as_str() {
        "message" | "message_body" | "prompt" | "prompt_body" | "user_message"
        | "user_message_body" => Some(BodyKind::Message),
        "response" | "response_body" | "assistant_response" | "assistant_response_body" => {
            Some(BodyKind::Response)
        }
        "conversation_history"
        | "conversation"
        | "messages"
        | "request"
        | "request_body"
        | "provider_request"
        | "provider_request_body"
        | "provider_response"
        | "provider_response_body"
        | "tool_result"
        | "tool_result_body"
        | "tool_results"
        | "tool_results_body" => Some(BodyKind::Drop),
        _ => None,
    }
}

fn is_summary_key(key: &str) -> bool {
    matches!(
        canonical_key(key).as_str(),
        "message_chars" | "response_chars" | "has_message" | "has_response"
    )
}

fn safe_summary_value(key: &str, value: &Value) -> Option<Value> {
    match canonical_key(key).as_str() {
        "message_chars" | "response_chars" => value.as_u64().map(|value| json!(value)),
        "has_message" | "has_response" => value.as_bool().map(|value| json!(value)),
        _ => None,
    }
}

fn is_sensitive_key(key: &str, config: &PrivacyConfig) -> bool {
    let key = canonical_key(key);
    config
        .redact_keys
        .iter()
        .map(|redact_key| canonical_key(redact_key))
        .filter(|redact_key| !redact_key.is_empty())
        .any(|redact_key| key_matches(&key, &redact_key))
}

fn key_matches(candidate: &str, redact_key: &str) -> bool {
    candidate == redact_key
        || candidate.starts_with(&format!("{redact_key}_"))
        || candidate.ends_with(&format!("_{redact_key}"))
        || candidate.contains(&format!("_{redact_key}_"))
}

fn canonical_key(key: &str) -> String {
    let chars = key
        .trim()
        .trim_matches(|ch: char| matches!(ch, '"' | '\'' | '{' | '[' | '(' | ',' | '.'))
        .chars()
        .collect::<Vec<_>>();

    let normalized = chars
        .iter()
        .enumerate()
        .fold(String::new(), |mut output, (index, ch)| {
            let previous = index
                .checked_sub(1)
                .and_then(|previous| chars.get(previous))
                .copied();
            let next = chars.get(index + 1).copied();

            if ch.is_ascii_uppercase() {
                if previous
                    .map(|previous| {
                        previous.is_ascii_lowercase()
                            || previous.is_ascii_digit()
                            || (previous.is_ascii_uppercase()
                                && next.map(|next| next.is_ascii_lowercase()).unwrap_or(false))
                    })
                    .unwrap_or(false)
                {
                    output.push('_');
                }
                output.push(ch.to_ascii_lowercase());
            } else if matches!(ch, '-' | '.' | ' ') {
                output.push('_');
            } else {
                output.push(ch.to_ascii_lowercase());
            }
            output
        });

    normalized
        .split('_')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("_")
}

fn redact_inline_secrets(input: &str, config: &PrivacyConfig) -> String {
    match sensitive_assignment_end(input, config) {
        Some(end) => format!("{}{}", &input[..end], REDACTED),
        None => input.to_string(),
    }
}

fn sensitive_assignment_end(input: &str, config: &PrivacyConfig) -> Option<usize> {
    input.char_indices().find_map(|(separator_index, ch)| {
        if !matches!(ch, '=' | ':') {
            return None;
        }

        let key = key_before_separator(&input[..separator_index])?;
        if !is_sensitive_key(key, config) {
            return None;
        }

        Some(skip_inline_value_prefix(
            input,
            separator_index + ch.len_utf8(),
        ))
    })
}

fn key_before_separator(prefix: &str) -> Option<&str> {
    prefix
        .trim_end()
        .rsplit(|ch: char| {
            ch.is_whitespace()
                || matches!(
                    ch,
                    '"' | '\'' | '{' | '[' | '(' | ',' | '?' | '&' | ';' | '/' | '=' | ':'
                )
        })
        .find(|part| !part.is_empty())
}

fn skip_inline_value_prefix(input: &str, start: usize) -> usize {
    input[start..]
        .char_indices()
        .find(|(_, ch)| !ch.is_whitespace())
        .map(|(index, _)| start + index)
        .unwrap_or(input.len())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn redact_value_recursively_redacts_sensitive_keys() {
        let payload = json!({
            "Token": "synthetic-token-should-not-leak",
            "nested": {
                "api_key": "synthetic-api-key-should-not-leak",
                "apiKey": "synthetic-camel-api-key-should-not-leak",
                "APIKey": "synthetic-acronym-api-key-should-not-leak",
                "XApiKey": "synthetic-prefixed-api-key-should-not-leak",
                "accessToken": "synthetic-camel-token-should-not-leak",
                "APIToken": "synthetic-acronym-token-should-not-leak",
                "Authorization": "Bearer synthetic-authorization",
                "password": 12345,
                "items": [
                    {"cookie": "synthetic-cookie-should-not-leak"},
                    {"secret": {"inner": "synthetic-secret-should-not-leak"}}
                ]
            }
        });

        let redacted = redact_value(&payload, &PrivacyConfig::default());

        assert_eq!(redacted["Token"], json!("[REDACTED]"));
        assert_eq!(redacted["nested"]["api_key"], json!("[REDACTED]"));
        assert_eq!(redacted["nested"]["apiKey"], json!("[REDACTED]"));
        assert_eq!(redacted["nested"]["APIKey"], json!("[REDACTED]"));
        assert_eq!(redacted["nested"]["XApiKey"], json!("[REDACTED]"));
        assert_eq!(redacted["nested"]["accessToken"], json!("[REDACTED]"));
        assert_eq!(redacted["nested"]["APIToken"], json!("[REDACTED]"));
        assert_eq!(redacted["nested"]["Authorization"], json!("[REDACTED]"));
        assert_eq!(redacted["nested"]["password"], json!("[REDACTED]"));
        assert_eq!(
            redacted["nested"]["items"][0]["cookie"],
            json!("[REDACTED]")
        );
        assert_eq!(
            redacted["nested"]["items"][1]["secret"],
            json!("[REDACTED]")
        );

        let rendered = redacted.to_string();
        for forbidden in [
            "synthetic-token-should-not-leak",
            "synthetic-api-key-should-not-leak",
            "synthetic-cookie-should-not-leak",
            "synthetic-secret-should-not-leak",
        ] {
            assert!(!rendered.contains(forbidden), "leaked `{forbidden}`");
        }
    }

    #[test]
    fn sanitize_payload_suppresses_body_fields_and_keeps_safe_summaries() {
        let payload = json!({
            "provider": "hermes",
            "message": "short synthetic message must not leak",
            "response": "synthetic response must not leak",
            "conversation_history": [{"role": "user", "content": "synthetic previous turn"}],
            "messages": [{"role": "user", "content": "synthetic messages array"}],
            "request": {"body": "synthetic provider request"},
            "requestBody": "synthetic request body alias",
            "providerRequest": {"body": "synthetic camel provider request"},
            "providerRequestBody": "synthetic camel provider request body",
            "provider_request": {"body": "synthetic provider request alias"},
            "providerResponse": {"body": "synthetic camel provider response"},
            "providerResponseBody": "synthetic camel provider response body",
            "provider_response": {"body": "synthetic provider response"},
            "toolResult": {"body": "synthetic camel tool result"},
            "toolResults": [{"body": "synthetic plural tool result"}],
            "tool_result": {"body": "synthetic tool result"},
            "tool_result_body": "synthetic tool result body alias",
            "context": {
                "message": "nested synthetic message must not leak",
                "response": "nested synthetic response must not leak",
                "conversationHistory": ["nested synthetic camel history must not leak"],
                "prompt": "nested synthetic prompt must not leak",
                "assistant_response": "nested assistant response must not leak"
            }
        });

        let sanitized = sanitize_payload(&payload, &PrivacyConfig::default());

        assert_eq!(sanitized["provider"], json!("hermes"));
        assert_eq!(sanitized["has_message"], json!(true));
        assert_eq!(sanitized["has_response"], json!(true));
        assert_eq!(sanitized["message_chars"], json!(37));
        assert_eq!(sanitized["response_chars"], json!(32));
        assert_eq!(sanitized["context"]["has_message"], json!(true));
        assert_eq!(sanitized["context"]["has_response"], json!(true));

        assert_absent_keys(
            &sanitized,
            &[
                "message",
                "response",
                "conversation_history",
                "messages",
                "request",
                "requestBody",
                "providerRequest",
                "providerRequestBody",
                "provider_request",
                "providerResponse",
                "providerResponseBody",
                "provider_response",
                "toolResult",
                "toolResults",
                "tool_result",
                "tool_result_body",
                "prompt",
                "user_message",
                "assistant_response",
            ],
        );

        let rendered = sanitized.to_string();
        for forbidden in [
            "short synthetic message must not leak",
            "synthetic response must not leak",
            "synthetic previous turn",
            "synthetic messages array",
            "synthetic provider request",
            "synthetic request body alias",
            "synthetic camel provider request",
            "synthetic camel provider request body",
            "synthetic provider request alias",
            "synthetic camel provider response",
            "synthetic camel provider response body",
            "synthetic provider response",
            "synthetic camel tool result",
            "synthetic plural tool result",
            "synthetic tool result",
            "synthetic tool result body alias",
            "nested synthetic message must not leak",
            "nested synthetic response must not leak",
            "nested synthetic camel history must not leak",
            "nested synthetic prompt must not leak",
            "nested assistant response must not leak",
        ] {
            assert!(!rendered.contains(forbidden), "leaked `{forbidden}`");
        }
    }

    #[test]
    fn sanitize_payload_does_not_mutate_original_payload() {
        let payload = json!({
            "message": "original synthetic message",
            "token": "synthetic-token-should-remain-only-in-original"
        });
        let original = payload.clone();

        let sanitized = sanitize_payload(&payload, &PrivacyConfig::default());

        assert_eq!(payload, original);
        assert!(sanitized.get("message").is_none());
        assert_eq!(sanitized["token"], json!("[REDACTED]"));
    }

    #[test]
    fn sanitize_payload_rejects_unsafe_summary_field_types() {
        let payload = json!({
            "message_chars": "full synthetic message hidden in a summary field",
            "response_chars": "full synthetic response hidden in a summary field",
            "has_message": "true with synthetic message text",
            "has_response": "true with synthetic response text",
            "nested": {
                "message_chars": 42,
                "has_message": true
            }
        });

        let sanitized = sanitize_payload(&payload, &PrivacyConfig::default());

        assert!(sanitized.get("message_chars").is_none());
        assert!(sanitized.get("response_chars").is_none());
        assert!(sanitized.get("has_message").is_none());
        assert!(sanitized.get("has_response").is_none());
        assert_eq!(sanitized["nested"]["message_chars"], json!(42));
        assert_eq!(sanitized["nested"]["has_message"], json!(true));

        let rendered = sanitized.to_string();
        assert!(!rendered.contains("full synthetic message hidden"));
        assert!(!rendered.contains("full synthetic response hidden"));
    }

    #[test]
    fn computed_body_summaries_cannot_be_overwritten_by_payload_summaries() {
        let payload = json!({
            "message": "synthetic message",
            "message_chars": 999,
            "has_message": false,
            "response_chars": "synthetic response leak"
        });

        let sanitized = sanitize_payload(&payload, &PrivacyConfig::default());

        assert_eq!(sanitized["message_chars"], json!(17));
        assert_eq!(sanitized["has_message"], json!(true));
        assert!(sanitized.get("response_chars").is_none());
        assert!(!sanitized.to_string().contains("synthetic response leak"));
    }

    #[test]
    fn opt_in_excerpt_redacts_before_truncating_on_char_boundaries() {
        let config = PrivacyConfig {
            include_message_excerpt: true,
            include_response_excerpt: true,
            max_excerpt_chars: 18,
            ..PrivacyConfig::default()
        };
        let payload = json!({
            "message": "hello token=synthetic-token-should-not-leak after",
            "response": "响应 secret=synthetic-secret-should-not-leak 完成"
        });

        let policy = excerpt_policy(&config);
        assert!(policy.include_message_excerpt);
        assert!(policy.include_response_excerpt);
        assert_eq!(policy.max_excerpt_chars, 18);

        let sanitized = sanitize_payload(&payload, &config);

        assert_eq!(sanitized["message_excerpt"], json!("hello token=[REDAC"));
        assert_eq!(sanitized["response_excerpt"], json!("响应 secret=[REDACTE"));
        assert!(
            sanitized["message_excerpt"]
                .as_str()
                .unwrap()
                .chars()
                .count()
                <= config.max_excerpt_chars
        );
        assert!(
            sanitized["response_excerpt"]
                .as_str()
                .unwrap()
                .chars()
                .count()
                <= config.max_excerpt_chars
        );

        let rendered = sanitized.to_string();
        assert!(!rendered.contains("synthetic-token-should-not-leak"));
        assert!(!rendered.contains("synthetic-secret-should-not-leak"));
    }

    #[test]
    fn opt_in_excerpt_redacts_secret_assignments_with_spaces() {
        let config = PrivacyConfig {
            include_message_excerpt: true,
            include_response_excerpt: true,
            max_excerpt_chars: 80,
            ..PrivacyConfig::default()
        };
        let payload = json!({
            "message": "Authorization: Bearer synthetic-token-should-not-leak trailing text",
            "response": "api_key = synthetic-api-key-should-not-leak trailing text"
        });

        let sanitized = sanitize_payload(&payload, &config);

        assert_eq!(
            sanitized["message_excerpt"],
            json!("Authorization: [REDACTED]")
        );
        assert_eq!(sanitized["response_excerpt"], json!("api_key = [REDACTED]"));

        let rendered = sanitized.to_string();
        assert!(!rendered.contains("synthetic-token-should-not-leak"));
        assert!(!rendered.contains("synthetic-api-key-should-not-leak"));
        assert!(!rendered.contains("trailing text"));
    }

    #[test]
    fn opt_in_excerpt_redacts_url_query_secrets() {
        let config = PrivacyConfig {
            include_message_excerpt: true,
            include_response_excerpt: true,
            max_excerpt_chars: 120,
            ..PrivacyConfig::default()
        };
        let payload = json!({
            "message": "callback=https://example.invalid/cb?token=synthetic-token-should-not-leak&next=ignored",
            "response": "open https://example.invalid/cb?x=1&api_key=synthetic-api-key-should-not-leak"
        });

        let sanitized = sanitize_payload(&payload, &config);

        assert_eq!(
            sanitized["message_excerpt"],
            json!("callback=https://example.invalid/cb?token=[REDACTED]")
        );
        assert_eq!(
            sanitized["response_excerpt"],
            json!("open https://example.invalid/cb?x=1&api_key=[REDACTED]")
        );

        let rendered = sanitized.to_string();
        assert!(!rendered.contains("synthetic-token-should-not-leak"));
        assert!(!rendered.contains("synthetic-api-key-should-not-leak"));
        assert!(!rendered.contains("next=ignored"));
    }

    #[test]
    fn opt_in_structured_excerpt_uses_full_sanitizer_before_truncation() {
        let config = PrivacyConfig {
            include_message_excerpt: true,
            max_excerpt_chars: 240,
            ..PrivacyConfig::default()
        };
        let payload = json!({
            "message": {
                "summary": "safe synthetic summary",
                "conversation_history": ["synthetic nested conversation must not leak"],
                "messages": ["synthetic nested messages must not leak"],
                "provider_request": {"body": "synthetic nested provider request must not leak"},
                "providerRequestBody": "synthetic nested provider request body must not leak",
                "toolResults": ["synthetic nested tool results must not leak"],
                "token": "synthetic-nested-token-should-not-leak"
            }
        });

        let sanitized = sanitize_payload(&payload, &config);
        let excerpt = sanitized["message_excerpt"].as_str().unwrap();

        assert!(excerpt.contains("safe synthetic summary"));
        assert!(excerpt.contains("[REDACTED]"));
        assert!(!excerpt.contains("synthetic nested conversation must not leak"));
        assert!(!excerpt.contains("synthetic nested messages must not leak"));
        assert!(!excerpt.contains("synthetic nested provider request must not leak"));
        assert!(!excerpt.contains("synthetic nested provider request body must not leak"));
        assert!(!excerpt.contains("synthetic nested tool results must not leak"));
        assert!(!excerpt.contains("synthetic-nested-token-should-not-leak"));
    }

    #[test]
    fn fixture_payload_sanitizes_without_leaking_raw_sensitive_values() {
        let payload: Value = serde_json::from_str(include_str!(
            "../tests/fixtures/privacy/sensitive_payload.json"
        ))
        .unwrap();

        let sanitized = sanitize_payload(&payload, &PrivacyConfig::default());

        assert_absent_keys(
            &sanitized,
            &[
                "message",
                "response",
                "conversation_history",
                "messages",
                "request",
                "requestBody",
                "providerRequest",
                "providerRequestBody",
                "provider_request",
                "providerResponse",
                "providerResponseBody",
                "provider_response",
                "toolResult",
                "toolResults",
                "tool_result",
                "tool_result_body",
                "prompt",
                "user_message",
                "assistant_response",
            ],
        );

        let rendered = sanitized.to_string();
        for forbidden in [
            "synthetic-token-value",
            "synthetic-cookie-value",
            "synthetic-secret-value",
            "synthetic message sample",
            "synthetic provider request summary",
            "synthetic provider response summary",
            "synthetic tool result summary",
        ] {
            assert!(!rendered.contains(forbidden), "leaked `{forbidden}`");
        }
    }

    fn assert_absent_keys(value: &Value, forbidden_keys: &[&str]) {
        match value {
            Value::Object(map) => {
                for key in forbidden_keys {
                    assert!(!map.contains_key(*key), "found forbidden key `{key}`");
                }
                for child in map.values() {
                    assert_absent_keys(child, forbidden_keys);
                }
            }
            Value::Array(items) => {
                for child in items {
                    assert_absent_keys(child, forbidden_keys);
                }
            }
            _ => {}
        }
    }
}
