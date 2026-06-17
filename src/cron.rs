use anyhow::Result;
use serde_json::{Map, Value, json};

use crate::config::{AppConfig, CronJob};
use crate::events::IncomingEvent;
use crate::source::git::{MAX_DISPLAY_FIELD_CHARS, MAX_SUMMARY_CHARS, validate_single_line};

pub fn configured_run_event(config: &AppConfig, id: &str) -> Result<IncomingEvent> {
    let id = id.trim();
    let job = config
        .cron
        .jobs
        .iter()
        .find(|job| job.id == id)
        .ok_or_else(|| anyhow::anyhow!("cron job '{id}' was not found"))?;

    if !job.enabled {
        anyhow::bail!("cron job '{id}' is disabled");
    }

    validate_job(job)?;
    Ok(build_run_event(job))
}

pub fn validate_job(job: &CronJob) -> Result<()> {
    let id = validate_single_line("cron_job_id", &job.id, MAX_DISPLAY_FIELD_CHARS)
        .map_err(|_| anyhow::anyhow!("cron jobs must set id"))?;
    let schedule = validate_single_line("cron_schedule", &job.schedule, MAX_DISPLAY_FIELD_CHARS)
        .map_err(|_| anyhow::anyhow!("cron job '{id}' must set schedule"))?;
    if schedule.split_whitespace().count() != 5 {
        anyhow::bail!("cron job '{id}' schedule must have 5 fields");
    }
    validate_single_line("summary", &job.message, MAX_SUMMARY_CHARS)
        .map_err(|error| anyhow::anyhow!("cron job '{id}': {error}"))?;
    Ok(())
}

fn build_run_event(job: &CronJob) -> IncomingEvent {
    let summary = job.message.trim();
    let mut payload = Map::new();
    insert_string(&mut payload, "cron_job_id", Some(job.id.as_str()));
    insert_string(&mut payload, "cron_schedule", Some(job.schedule.as_str()));
    insert_string(&mut payload, "summary", Some(summary));
    payload.insert("summary_chars".to_string(), json!(summary.chars().count()));

    IncomingEvent {
        kind: "cron.run".to_string(),
        channel: normalize_optional_text(job.channel.clone()),
        mention: None,
        format: None,
        template: None,
        payload: Value::Object(payload),
    }
}

fn insert_string(payload: &mut Map<String, Value>, key: &str, value: Option<&str>) {
    if let Some(value) = normalize_text(value) {
        payload.insert(key.to_string(), json!(value));
    }
}

fn normalize_optional_text(value: Option<String>) -> Option<String> {
    value.and_then(|value| normalize_text(Some(value.as_str())))
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

    use crate::config::{AppConfig, CronConfig, CronJob};
    use crate::event::{EventBody, compat::from_incoming_event};
    use crate::privacy::sanitize_payload;

    use super::{configured_run_event, validate_job};

    #[test]
    fn cron_configured_run_builds_safe_event_from_job() {
        let config = AppConfig {
            cron: CronConfig {
                jobs: vec![CronJob {
                    id: "dev-followup".to_string(),
                    schedule: "*/30 * * * *".to_string(),
                    message: "check open PRs and blockers".to_string(),
                    channel: Some("ops".to_string()),
                    enabled: true,
                }],
            },
            ..AppConfig::default()
        };

        let event = configured_run_event(&config, "dev-followup").unwrap();

        assert_eq!(event.kind, "cron.run");
        assert_eq!(event.channel.as_deref(), Some("ops"));
        assert_eq!(event.payload["cron_job_id"], json!("dev-followup"));
        assert_eq!(event.payload["cron_schedule"], json!("*/30 * * * *"));
        assert_eq!(
            event.payload["summary"],
            json!("check open PRs and blockers")
        );
        assert_eq!(event.payload["summary_chars"], json!(27));

        let sanitized = sanitize_payload(&event.payload, &Default::default());
        assert!(sanitized.get("message").is_none());
        assert!(sanitized.get("token").is_none());
        assert!(sanitized.get("secret").is_none());

        let envelope = from_incoming_event(&event).unwrap();
        assert_eq!(envelope.source, "cron");
        assert_eq!(envelope.canonical_kind(), "cron.run");
        match envelope.body {
            EventBody::CronRun(body) => {
                assert_eq!(body.job_id, "dev-followup");
                assert_eq!(body.schedule, "*/30 * * * *");
                assert_eq!(body.summary, "check open PRs and blockers");
                assert_eq!(body.summary_chars, 27);
            }
            other => panic!("expected CronRun, got {other:?}"),
        }
    }

    #[test]
    fn cron_run_rejects_missing_disabled_and_invalid_jobs() {
        let config = AppConfig {
            cron: CronConfig {
                jobs: vec![CronJob {
                    id: "disabled".to_string(),
                    schedule: "0 * * * *".to_string(),
                    message: "disabled job".to_string(),
                    channel: None,
                    enabled: false,
                }],
            },
            ..AppConfig::default()
        };

        let missing = configured_run_event(&config, "missing")
            .unwrap_err()
            .to_string();
        assert!(
            missing.contains("cron job 'missing' was not found"),
            "{missing}"
        );

        let disabled = configured_run_event(&config, "disabled")
            .unwrap_err()
            .to_string();
        assert!(
            disabled.contains("cron job 'disabled' is disabled"),
            "{disabled}"
        );

        let invalid = validate_job(&CronJob {
            id: "dev-followup".to_string(),
            schedule: " ".to_string(),
            message: "follow\nfull body should not render".to_string(),
            ..CronJob::default()
        })
        .unwrap_err()
        .to_string();
        assert!(invalid.contains("must set schedule"), "{invalid}");
    }
}
