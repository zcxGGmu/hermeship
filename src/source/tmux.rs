use anyhow::Result;
use serde_json::{Map, Value, json};

use crate::events::IncomingEvent;
use crate::source::git::{
    MAX_DISPLAY_FIELD_CHARS, MAX_SUMMARY_CHARS, validate_optional_single_line, validate_single_line,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TmuxKeywordInput {
    pub session: String,
    pub window: Option<String>,
    pub pane: Option<String>,
    pub keyword: String,
    pub line: String,
    pub channel: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TmuxStaleInput {
    pub session: String,
    pub window: Option<String>,
    pub pane: String,
    pub minutes: u64,
    pub last_line: String,
    pub channel: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TmuxWatchInput {
    pub session: String,
    pub keywords: Vec<String>,
    pub stale_minutes: u64,
    pub channel: Option<String>,
    pub tmux_output: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TmuxPane {
    pub session: String,
    pub window: Option<String>,
    pub pane: String,
    pub dead: bool,
    pub current_command: Option<String>,
    pub last_line: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TmuxWatchPlan {
    pub session: String,
    pub keywords: Vec<String>,
    pub stale_minutes: u64,
    pub channel: Option<String>,
    pub panes: Vec<TmuxPane>,
}

pub fn keyword_event(input: TmuxKeywordInput) -> Result<IncomingEvent> {
    let session = validate_tmux_field("session", &input.session)?;
    let window = validate_optional_tmux_field("window", input.window.as_deref())?;
    let pane = validate_optional_tmux_field("pane", input.pane.as_deref())?;
    let keyword = validate_tmux_field("keyword", &input.keyword)?;
    let line = validate_single_line("line", &input.line, MAX_SUMMARY_CHARS)?;

    let mut payload = Map::new();
    insert_string(&mut payload, "session", Some(session.as_str()));
    insert_string(&mut payload, "session_name", Some(session.as_str()));
    insert_string(&mut payload, "window", window.as_deref());
    insert_string(&mut payload, "pane", pane.as_deref());
    insert_string(&mut payload, "keyword", Some(keyword.as_str()));
    insert_string(&mut payload, "line", Some(line.as_str()));
    payload.insert("line_chars".to_string(), json!(line.chars().count()));

    Ok(incoming("tmux.keyword", input.channel, payload))
}

pub fn stale_event(input: TmuxStaleInput) -> Result<IncomingEvent> {
    let session = validate_tmux_field("session", &input.session)?;
    let window = validate_optional_tmux_field("window", input.window.as_deref())?;
    let pane = validate_tmux_field("pane", &input.pane)?;
    let minutes = validate_positive_minutes(input.minutes)?;
    let last_line = validate_single_line("last_line", &input.last_line, MAX_SUMMARY_CHARS)?;

    let mut payload = Map::new();
    insert_string(&mut payload, "session", Some(session.as_str()));
    insert_string(&mut payload, "session_name", Some(session.as_str()));
    insert_string(&mut payload, "window", window.as_deref());
    insert_string(&mut payload, "pane", Some(pane.as_str()));
    payload.insert("minutes".to_string(), json!(minutes));
    insert_string(&mut payload, "last_line", Some(last_line.as_str()));
    payload.insert(
        "last_line_chars".to_string(),
        json!(last_line.chars().count()),
    );

    Ok(incoming("tmux.stale", input.channel, payload))
}

pub fn parse_tmux_panes_output(raw: &str) -> Result<Vec<TmuxPane>> {
    raw.lines()
        .map(str::trim_end)
        .filter(|line| !line.trim().is_empty())
        .enumerate()
        .map(|(index, line)| parse_tmux_pane_line(index + 1, line))
        .collect()
}

pub fn watch_plan_from_output(input: TmuxWatchInput) -> Result<TmuxWatchPlan> {
    let session = validate_tmux_field("session", &input.session)?;
    let keywords = validate_keywords(input.keywords)?;
    let stale_minutes = validate_positive_minutes(input.stale_minutes)?;
    let channel = normalize_optional_text(input.channel);
    let panes = parse_tmux_panes_output(&input.tmux_output)?
        .into_iter()
        .filter(|pane| pane.session == session && !pane.dead)
        .collect();

    Ok(TmuxWatchPlan {
        session,
        keywords,
        stale_minutes,
        channel,
        panes,
    })
}

pub fn format_pane_list(panes: &[TmuxPane]) -> String {
    if panes.is_empty() {
        return "No tmux panes found\n".to_string();
    }

    let mut output = "SESSION\tWINDOW\tPANE\tDEAD\tHAS_COMMAND\tLAST_LINE_CHARS\n".to_string();
    for pane in panes {
        output.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\n",
            pane.session,
            pane.window.as_deref().unwrap_or("-"),
            pane.pane,
            pane.dead,
            pane.current_command.is_some(),
            optional_chars(pane.last_line.as_deref()),
        ));
    }
    output
}

pub fn format_watch_plan(plan: &TmuxWatchPlan) -> String {
    let keywords = if plan.keywords.is_empty() {
        "-".to_string()
    } else {
        plan.keywords.join(",")
    };
    let channel = plan.channel.as_deref().unwrap_or("-");
    let mut output = format!(
        "tmux watch planned: {} (channel={}, keywords={}, stale_minutes={})\n",
        plan.session, channel, keywords, plan.stale_minutes
    );
    if plan.panes.is_empty() {
        output.push_str("  panes: (none)\n");
    } else {
        for pane in &plan.panes {
            output.push_str(&format!(
                "  pane={} window={} dead={} command_seen={} last_line_chars={}\n",
                pane.pane,
                pane.window.as_deref().unwrap_or("-"),
                pane.dead,
                pane.current_command.is_some(),
                optional_chars(pane.last_line.as_deref()),
            ));
        }
    }
    output
}

fn optional_chars(value: Option<&str>) -> usize {
    value.map(|value| value.chars().count()).unwrap_or_default()
}

fn parse_tmux_pane_line(line_number: usize, line: &str) -> Result<TmuxPane> {
    let columns = line.split('\t').collect::<Vec<_>>();
    if columns.len() != 6 {
        anyhow::bail!("tmux output line {line_number} must have 6 tab-separated columns");
    }

    Ok(TmuxPane {
        session: validate_tmux_field("session", columns[0])?,
        window: validate_optional_tmux_field("window", Some(columns[1]))?,
        pane: validate_tmux_field("pane", columns[2])?,
        dead: parse_dead_flag(columns[3])?,
        current_command: validate_optional_tmux_field("current_command", Some(columns[4]))?,
        last_line: validate_optional_summary("last_line", Some(columns[5]))?,
    })
}

fn validate_keywords(values: Vec<String>) -> Result<Vec<String>> {
    let keywords = values
        .into_iter()
        .map(|value| validate_tmux_field("keyword", &value))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .fold(Vec::new(), |mut keywords, keyword| {
            if !keywords.contains(&keyword) {
                keywords.push(keyword);
            }
            keywords
        });

    if keywords.is_empty() {
        anyhow::bail!("keywords must not be empty");
    }

    Ok(keywords)
}

fn parse_dead_flag(raw: &str) -> Result<bool> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "0" | "false" | "no" | "alive" => Ok(false),
        "1" | "true" | "yes" | "dead" => Ok(true),
        other => anyhow::bail!("dead flag must be 0/1 or true/false, got {other:?}"),
    }
}

fn validate_positive_minutes(minutes: u64) -> Result<u64> {
    if minutes == 0 {
        anyhow::bail!("minutes must be greater than 0");
    }
    Ok(minutes)
}

fn validate_tmux_field(name: &str, raw: &str) -> Result<String> {
    validate_single_line(name, raw, MAX_DISPLAY_FIELD_CHARS)
}

fn validate_optional_tmux_field(name: &str, value: Option<&str>) -> Result<Option<String>> {
    validate_optional_single_line(name, value, MAX_DISPLAY_FIELD_CHARS)
}

fn validate_optional_summary(name: &str, value: Option<&str>) -> Result<Option<String>> {
    validate_optional_single_line(name, value, MAX_SUMMARY_CHARS)
}

fn incoming(kind: &str, channel: Option<String>, payload: Map<String, Value>) -> IncomingEvent {
    IncomingEvent {
        kind: kind.to_string(),
        channel: normalize_optional_text(channel),
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

    use crate::privacy::sanitize_payload;
    use crate::source::tmux::{
        TmuxKeywordInput, TmuxStaleInput, TmuxWatchInput, format_pane_list, format_watch_plan,
        keyword_event, parse_tmux_panes_output, stale_event, watch_plan_from_output,
    };

    #[test]
    fn tmux_keyword_source_builds_sanitized_metadata_event() {
        let event = keyword_event(TmuxKeywordInput {
            session: "hermes-agent".to_string(),
            window: Some("main".to_string()),
            pane: Some("%1".to_string()),
            keyword: "FAILED".to_string(),
            line: "build FAILED at deterministic fixture".to_string(),
            channel: Some("ops".to_string()),
        })
        .unwrap();

        assert_eq!(event.kind, "tmux.keyword");
        assert_eq!(event.channel.as_deref(), Some("ops"));
        assert_eq!(event.payload["session"], json!("hermes-agent"));
        assert_eq!(event.payload["window"], json!("main"));
        assert_eq!(event.payload["pane"], json!("%1"));
        assert_eq!(event.payload["keyword"], json!("FAILED"));
        assert_eq!(
            event.payload["line"],
            json!("build FAILED at deterministic fixture")
        );
        assert_eq!(event.payload["line_chars"], json!(37));

        let sanitized = sanitize_payload(&event.payload, &Default::default());
        assert!(sanitized.get("pane_capture").is_none());
        assert!(sanitized.get("buffer").is_none());
        assert!(sanitized.get("secret").is_none());
    }

    #[test]
    fn tmux_stale_source_builds_stale_metadata_event() {
        let event = stale_event(TmuxStaleInput {
            session: "hermes-agent".to_string(),
            window: Some("main".to_string()),
            pane: "%2".to_string(),
            minutes: 15,
            last_line: "waiting for agent output".to_string(),
            channel: None,
        })
        .unwrap();

        assert_eq!(event.kind, "tmux.stale");
        assert_eq!(event.payload["session"], json!("hermes-agent"));
        assert_eq!(event.payload["window"], json!("main"));
        assert_eq!(event.payload["pane"], json!("%2"));
        assert_eq!(event.payload["minutes"], json!(15));
        assert_eq!(
            event.payload["last_line"],
            json!("waiting for agent output")
        );
        assert_eq!(event.payload["last_line_chars"], json!(24));
    }

    #[test]
    fn tmux_source_rejects_empty_multiline_and_zero_minute_fields() {
        let empty_keyword = keyword_event(TmuxKeywordInput {
            session: "hermes-agent".to_string(),
            window: None,
            pane: None,
            keyword: " ".to_string(),
            line: "build failed".to_string(),
            channel: None,
        })
        .unwrap_err()
        .to_string();
        assert!(empty_keyword.contains("keyword must not be empty"));

        let multiline_line = keyword_event(TmuxKeywordInput {
            session: "hermes-agent".to_string(),
            window: None,
            pane: None,
            keyword: "FAILED".to_string(),
            line: "build failed\nfull pane capture should not render".to_string(),
            channel: None,
        })
        .unwrap_err()
        .to_string();
        assert!(multiline_line.contains("line must be a single line"));

        let zero_minutes = stale_event(TmuxStaleInput {
            session: "hermes-agent".to_string(),
            window: None,
            pane: "%1".to_string(),
            minutes: 0,
            last_line: "waiting".to_string(),
            channel: None,
        })
        .unwrap_err()
        .to_string();
        assert!(zero_minutes.contains("minutes must be greater than 0"));
    }

    #[test]
    fn tmux_watch_and_list_parse_fake_tmux_output_without_real_session() {
        let raw = "hermes-agent\tmain\t%1\t0\tbash\tready\nother\tlogs\t%3\t1\tsh\tdone\n";
        let panes = parse_tmux_panes_output(raw).unwrap();

        assert_eq!(panes.len(), 2);
        assert_eq!(panes[0].session, "hermes-agent");
        assert_eq!(panes[0].window.as_deref(), Some("main"));
        assert_eq!(panes[0].pane, "%1");
        assert!(!panes[0].dead);
        assert_eq!(panes[0].current_command.as_deref(), Some("bash"));
        assert_eq!(panes[0].last_line.as_deref(), Some("ready"));
        assert!(panes[1].dead);

        let plan = watch_plan_from_output(TmuxWatchInput {
            session: "hermes-agent".to_string(),
            keywords: vec!["FAILED".to_string(), "complete".to_string()],
            stale_minutes: 10,
            channel: Some("ops".to_string()),
            tmux_output: raw.to_string(),
        })
        .unwrap();

        assert_eq!(plan.session, "hermes-agent");
        assert_eq!(plan.keywords, vec!["FAILED", "complete"]);
        assert_eq!(plan.stale_minutes, 10);
        assert_eq!(plan.channel.as_deref(), Some("ops"));
        assert_eq!(plan.panes.len(), 1);
        assert_eq!(plan.panes[0].pane, "%1");
    }

    #[test]
    fn tmux_watch_and_list_reports_do_not_echo_command_or_last_line_content() {
        let raw = "hermes-agent\tmain\t%1\t0\tbash -lc token=synthetic-token /Users/zq/private\tAuthorization: Bearer synthetic-secret in /Users/zq/private\n";
        let panes = parse_tmux_panes_output(raw).unwrap();
        let list = format_pane_list(&panes);

        assert!(list.contains("SESSION\tWINDOW\tPANE\tDEAD\tHAS_COMMAND\tLAST_LINE_CHARS"));
        assert!(list.contains("hermes-agent\tmain\t%1\tfalse\ttrue\t"));

        let plan = watch_plan_from_output(TmuxWatchInput {
            session: "hermes-agent".to_string(),
            keywords: vec!["FAILED".to_string()],
            stale_minutes: 10,
            channel: None,
            tmux_output: raw.to_string(),
        })
        .unwrap();
        let watch = format_watch_plan(&plan);

        assert!(watch.contains("pane=%1 window=main dead=false command_seen=true"));
        assert!(watch.contains("last_line_chars="));

        for rendered in [&list, &watch] {
            for forbidden in [
                "bash -lc",
                "token=synthetic-token",
                "Authorization: Bearer",
                "synthetic-secret",
                "/Users/zq/private",
            ] {
                assert!(
                    !rendered.contains(forbidden),
                    "tmux report leaked `{forbidden}`"
                );
            }
        }
    }

    #[test]
    fn tmux_watch_and_list_reject_invalid_deterministic_inputs() {
        let bad_columns = parse_tmux_panes_output("hermes-agent\tmain\t%1").unwrap_err();
        assert!(
            bad_columns
                .to_string()
                .contains("must have 6 tab-separated columns")
        );

        let bad_dead =
            parse_tmux_panes_output("hermes-agent\tmain\t%1\tmaybe\tbash\tready").unwrap_err();
        assert!(bad_dead.to_string().contains("dead flag must be"));

        let empty_keywords = watch_plan_from_output(TmuxWatchInput {
            session: "hermes-agent".to_string(),
            keywords: vec![],
            stale_minutes: 10,
            channel: None,
            tmux_output: "hermes-agent\tmain\t%1\t0\tbash\tready".to_string(),
        })
        .unwrap_err();
        assert!(
            empty_keywords
                .to_string()
                .contains("keywords must not be empty")
        );

        let zero_stale = watch_plan_from_output(TmuxWatchInput {
            session: "hermes-agent".to_string(),
            keywords: vec!["FAILED".to_string()],
            stale_minutes: 0,
            channel: None,
            tmux_output: "hermes-agent\tmain\t%1\t0\tbash\tready".to_string(),
        })
        .unwrap_err();
        assert!(
            zero_stale
                .to_string()
                .contains("minutes must be greater than 0")
        );
    }
}
