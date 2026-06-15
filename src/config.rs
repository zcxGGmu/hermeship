use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

pub const CONFIG_ENV_VAR: &str = "HERMESHIP_CONFIG";
const DAEMON_URL_ENV_VAR: &str = "HERMESHIP_DAEMON_URL";
const DISCORD_TOKEN_ENV_VAR: &str = "HERMESHIP_DISCORD_TOKEN";
const DEFAULT_CHANNEL_ENV_VAR: &str = "HERMESHIP_DEFAULT_CHANNEL";
const DRY_RUN_ENV_VAR: &str = "HERMESHIP_DRY_RUN";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub daemon: DaemonConfig,
    pub providers: ProvidersConfig,
    pub defaults: DefaultsConfig,
    pub privacy: PrivacyConfig,
    pub hermes: HermesConfig,
    pub routes: Vec<RouteRule>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            daemon: DaemonConfig::default(),
            providers: ProvidersConfig::default(),
            defaults: DefaultsConfig::default(),
            privacy: PrivacyConfig::default(),
            hermes: HermesConfig::default(),
            routes: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DaemonConfig {
    #[serde(alias = "bind_host")]
    pub host: String,
    pub port: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            host: default_daemon_host(),
            port: default_daemon_port(),
            base_url: None,
        }
    }
}

impl DaemonConfig {
    pub fn base_url(&self) -> String {
        self.base_url.clone().unwrap_or_else(|| {
            let host = if self.host == "0.0.0.0" {
                "127.0.0.1"
            } else {
                self.host.as_str()
            };
            format!("http://{host}:{}", self.port)
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ProvidersConfig {
    pub discord: DiscordConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct DiscordConfig {
    #[serde(alias = "bot_token", skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_channel: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DefaultsConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
    pub format: MessageFormat,
    pub project: String,
    pub dry_run: bool,
}

impl Default for DefaultsConfig {
    fn default() -> Self {
        Self {
            channel: None,
            format: MessageFormat::Compact,
            project: default_project(),
            dry_run: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PrivacyConfig {
    pub include_message_excerpt: bool,
    pub include_response_excerpt: bool,
    pub max_excerpt_chars: usize,
    pub dedupe_window_secs: u64,
    pub redact_keys: Vec<String>,
}

impl Default for PrivacyConfig {
    fn default() -> Self {
        Self {
            include_message_excerpt: false,
            include_response_excerpt: false,
            max_excerpt_chars: 240,
            dedupe_window_secs: 30,
            redact_keys: default_redact_keys(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct HermesConfig {
    pub hook_timeout_secs: f64,
    pub enable_agent_step: bool,
    pub enable_command_events: bool,
}

impl Default for HermesConfig {
    fn default() -> Self {
        Self {
            hook_timeout_secs: 2.0,
            enable_agent_step: false,
            enable_command_events: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RouteRule {
    pub event: String,
    pub filter: BTreeMap<String, String>,
    pub sink: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mention: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<MessageFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
    pub enabled: bool,
}

impl Default for RouteRule {
    fn default() -> Self {
        Self {
            event: String::new(),
            filter: BTreeMap::new(),
            sink: default_sink(),
            channel: None,
            webhook: None,
            mention: None,
            format: None,
            template: None,
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MessageFormat {
    Compact,
    Inline,
    Alert,
    Raw,
}

impl Default for MessageFormat {
    fn default() -> Self {
        Self::Compact
    }
}

impl MessageFormat {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Compact => "compact",
            Self::Inline => "inline",
            Self::Alert => "alert",
            Self::Raw => "raw",
        }
    }
}

pub fn default_config_path() -> PathBuf {
    config_path_from_env(|name| env::var(name).ok())
}

fn config_path_from_env<F>(mut get_env: F) -> PathBuf
where
    F: FnMut(&str) -> Option<String>,
{
    if let Some(path) = normalize_text(get_env(CONFIG_ENV_VAR)) {
        return PathBuf::from(path);
    }

    let home = normalize_text(get_env("HOME"))
        .or_else(|| normalize_text(get_env("USERPROFILE")))
        .unwrap_or_else(|| ".".to_string());

    PathBuf::from(home).join(".hermeship").join("config.toml")
}

impl AppConfig {
    pub fn load_or_default(path: &Path) -> Result<Self> {
        Self::load_or_default_with_env(path, |name| env::var(name).ok())
    }

    pub fn load_or_default_with_env<F>(path: &Path, mut get_env: F) -> Result<Self>
    where
        F: FnMut(&str) -> Option<String>,
    {
        let mut config = if path.exists() {
            let raw = fs::read_to_string(path).with_context(|| {
                format!("failed to read hermeship config at {}", path.display())
            })?;
            toml::from_str(&raw)
                .with_context(|| format!("invalid hermeship config TOML at {}", path.display()))?
        } else {
            Self::default()
        };

        config.normalize();
        config.apply_env_overrides(&mut get_env)?;
        config.normalize();
        Ok(config)
    }

    pub fn to_pretty_toml(&self) -> Result<String> {
        Ok(toml::to_string_pretty(self)?)
    }

    pub fn validate(&self) -> Result<()> {
        if self.daemon.host.trim().is_empty() {
            anyhow::bail!("daemon.host must not be empty");
        }
        if self.daemon.port == 0 {
            anyhow::bail!("daemon.port must be greater than 0");
        }
        if self.privacy.max_excerpt_chars == 0 {
            anyhow::bail!("privacy.max_excerpt_chars must be greater than 0");
        }
        if self.privacy.dedupe_window_secs == 0 {
            anyhow::bail!("privacy.dedupe_window_secs must be greater than 0");
        }
        if self.hermes.hook_timeout_secs <= 0.0 {
            anyhow::bail!("hermes.hook_timeout_secs must be greater than 0");
        }

        for (index, route) in self.routes.iter().enumerate() {
            let number = index + 1;
            if route.event.trim().is_empty() {
                anyhow::bail!("route #{number} must set event");
            }
            if !matches!(route.sink.as_str(), "discord") {
                anyhow::bail!(
                    "route #{number} ({}) uses unsupported sink '{}'",
                    route.event,
                    route.sink
                );
            }
            if route.channel.is_some() && route.webhook.is_some() {
                anyhow::bail!(
                    "route #{number} ({}) cannot set both channel and webhook",
                    route.event
                );
            }
        }

        Ok(())
    }

    fn apply_env_overrides<F>(&mut self, get_env: &mut F) -> Result<()>
    where
        F: FnMut(&str) -> Option<String>,
    {
        if let Some(base_url) = normalize_text(get_env(DAEMON_URL_ENV_VAR)) {
            self.daemon.base_url = Some(base_url);
        }
        if let Some(token) = normalize_secret(get_env(DISCORD_TOKEN_ENV_VAR)) {
            self.providers.discord.token = Some(token);
        }
        if let Some(channel) = normalize_text(get_env(DEFAULT_CHANNEL_ENV_VAR)) {
            self.defaults.channel = Some(channel);
        }
        if let Some(raw) = normalize_text(get_env(DRY_RUN_ENV_VAR)) {
            self.defaults.dry_run = parse_env_bool(DRY_RUN_ENV_VAR, &raw)?;
        }

        Ok(())
    }

    fn normalize(&mut self) {
        self.daemon.host =
            normalize_text(Some(self.daemon.host.clone())).unwrap_or_else(default_daemon_host);
        self.daemon.base_url = normalize_text(self.daemon.base_url.clone());
        self.providers.discord.token = normalize_secret(self.providers.discord.token.clone());
        self.providers.discord.default_channel =
            normalize_text(self.providers.discord.default_channel.clone());
        self.defaults.channel = normalize_text(self.defaults.channel.clone())
            .or_else(|| normalize_text(self.providers.discord.default_channel.clone()));
        self.defaults.project =
            normalize_text(Some(self.defaults.project.clone())).unwrap_or_else(default_project);
        self.privacy.redact_keys = normalize_list(self.privacy.redact_keys.clone());
        if self.privacy.redact_keys.is_empty() {
            self.privacy.redact_keys = default_redact_keys();
        }

        for route in &mut self.routes {
            route.event = normalize_text(Some(route.event.clone())).unwrap_or_default();
            route.sink = normalize_text(Some(route.sink.clone())).unwrap_or_else(default_sink);
            route.channel = normalize_text(route.channel.clone());
            route.webhook = normalize_text(route.webhook.clone());
            route.mention = normalize_text(route.mention.clone());
            route.template = normalize_text(route.template.clone());
            route.filter = route
                .filter
                .iter()
                .filter_map(|(key, value)| {
                    let key = normalize_text(Some(key.clone()))?;
                    let value = normalize_text(Some(value.clone()))?;
                    Some((key, value))
                })
                .collect();
        }
    }
}

fn default_daemon_host() -> String {
    "127.0.0.1".to_string()
}

fn default_daemon_port() -> u16 {
    25295
}

fn default_project() -> String {
    "hermes".to_string()
}

fn default_sink() -> String {
    "discord".to_string()
}

fn default_redact_keys() -> Vec<String> {
    [
        "token",
        "api_key",
        "authorization",
        "password",
        "secret",
        "cookie",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn normalize_text(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_string())
    })
}

fn normalize_secret(value: Option<String>) -> Option<String> {
    normalize_text(value)
}

fn normalize_list(values: Vec<String>) -> Vec<String> {
    values
        .into_iter()
        .filter_map(|value| normalize_text(Some(value)))
        .collect()
}

fn parse_env_bool(name: &str, value: &str) -> Result<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Ok(true),
        "0" | "false" | "no" | "off" => Ok(false),
        _ => anyhow::bail!("{name} must be a boolean value"),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn config_default_path_uses_home_and_hermeship_config_override() {
        let overridden = config_path_from_env(|name| match name {
            "HERMESHIP_CONFIG" => Some("/tmp/hermeship/custom.toml".to_string()),
            "HOME" => Some("/home/tester".to_string()),
            _ => None,
        });

        assert_eq!(overridden, PathBuf::from("/tmp/hermeship/custom.toml"));

        let default = config_path_from_env(|name| match name {
            "HOME" => Some("/home/tester".to_string()),
            _ => None,
        });

        assert_eq!(
            default,
            PathBuf::from("/home/tester/.hermeship/config.toml")
        );
    }

    #[test]
    fn config_missing_file_returns_default_config() {
        let path = temp_config_path("missing-defaults");
        let config = AppConfig::load_or_default_with_env(&path, |_| None).unwrap();

        assert_eq!(config.daemon.host, "127.0.0.1");
        assert_eq!(config.daemon.port, 25295);
        assert_eq!(config.daemon.base_url(), "http://127.0.0.1:25295");
        assert_eq!(config.defaults.channel, None);
        assert_eq!(config.defaults.format, MessageFormat::Compact);
        assert_eq!(config.defaults.project, "hermes");
        assert!(!config.defaults.dry_run);
        assert_eq!(config.privacy.max_excerpt_chars, 240);
        assert_eq!(config.privacy.dedupe_window_secs, 30);
        assert!(!config.privacy.include_message_excerpt);
        assert!(!config.privacy.include_response_excerpt);
        assert_eq!(
            config.privacy.redact_keys,
            [
                "token",
                "api_key",
                "authorization",
                "password",
                "secret",
                "cookie"
            ]
        );
        assert_eq!(config.hermes.hook_timeout_secs, 2.0);
        assert!(!config.hermes.enable_agent_step);
        assert!(!config.hermes.enable_command_events);
        assert!(config.routes.is_empty());
    }

    #[test]
    fn config_loads_toml_and_normalizes_empty_values() {
        let path = temp_config_path("normalizes-empty-values");
        fs::write(
            &path,
            r#"
[daemon]
host = " 127.0.0.1 "
port = 25296
base_url = " "

[providers.discord]
token = "   "
default_channel = "  "

[defaults]
channel = "  alerts "
format = "alert"
project = " hermes-gateway "
dry_run = true

[privacy]
include_message_excerpt = true
include_response_excerpt = true
max_excerpt_chars = 128
dedupe_window_secs = 45
redact_keys = [" token ", "", "authorization"]

[hermes]
hook_timeout_secs = 3.5
enable_agent_step = true
enable_command_events = true

[[routes]]
event = " hermes.agent.* "
filter = { project = " hermes ", empty = " " }
sink = " "
channel = " "
mention = " <@123> "
format = "compact"
template = " {{event}} "
"#,
        )
        .unwrap();

        let config = AppConfig::load_or_default_with_env(&path, |_| None).unwrap();

        assert_eq!(config.daemon.host, "127.0.0.1");
        assert_eq!(config.daemon.port, 25296);
        assert_eq!(config.daemon.base_url(), "http://127.0.0.1:25296");
        assert_eq!(config.providers.discord.token, None);
        assert_eq!(config.providers.discord.default_channel, None);
        assert_eq!(config.defaults.channel.as_deref(), Some("alerts"));
        assert_eq!(config.defaults.format, MessageFormat::Alert);
        assert_eq!(config.defaults.project, "hermes-gateway");
        assert!(config.defaults.dry_run);
        assert_eq!(config.privacy.max_excerpt_chars, 128);
        assert_eq!(config.privacy.dedupe_window_secs, 45);
        assert_eq!(config.privacy.redact_keys, ["token", "authorization"]);
        assert_eq!(config.hermes.hook_timeout_secs, 3.5);
        assert!(config.hermes.enable_agent_step);
        assert!(config.hermes.enable_command_events);

        let route = &config.routes[0];
        assert_eq!(route.event, "hermes.agent.*");
        assert_eq!(
            route.filter,
            BTreeMap::from([("project".into(), "hermes".into())])
        );
        assert_eq!(route.sink, "discord");
        assert_eq!(route.channel, None);
        assert_eq!(route.mention.as_deref(), Some("<@123>"));
        assert_eq!(route.format, Some(MessageFormat::Compact));
        assert_eq!(route.template.as_deref(), Some("{{event}}"));
    }

    #[test]
    fn config_env_overrides_file_values() {
        let path = temp_config_path("env-overrides");
        fs::write(
            &path,
            r#"
[daemon]
base_url = "http://127.0.0.1:25295"

[providers.discord]
token = "file-token"

[defaults]
channel = "file-channel"
dry_run = false
"#,
        )
        .unwrap();

        let config = AppConfig::load_or_default_with_env(&path, |name| match name {
            "HERMESHIP_DAEMON_URL" => Some(" http://127.0.0.1:3000 ".to_string()),
            "HERMESHIP_DISCORD_TOKEN" => Some(" env-token ".to_string()),
            "HERMESHIP_DEFAULT_CHANNEL" => Some(" env-channel ".to_string()),
            "HERMESHIP_DRY_RUN" => Some("true".to_string()),
            _ => None,
        })
        .unwrap();

        assert_eq!(config.daemon.base_url(), "http://127.0.0.1:3000");
        assert_eq!(config.providers.discord.token.as_deref(), Some("env-token"));
        assert_eq!(config.defaults.channel.as_deref(), Some("env-channel"));
        assert!(config.defaults.dry_run);
    }

    #[test]
    fn config_invalid_toml_returns_error() {
        let path = temp_config_path("invalid-toml");
        fs::write(&path, "[daemon\nport = 25295").unwrap();

        let error = AppConfig::load_or_default_with_env(&path, |_| None)
            .unwrap_err()
            .to_string();

        assert!(
            error.contains("invalid hermeship config"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn config_unknown_keys_are_ignored_for_forward_compatibility() {
        let path = temp_config_path("unknown-keys");
        fs::write(
            &path,
            r#"
unknown_root = true

[daemon]
host = "127.0.0.1"
future_field = "ignored"

[providers.discord]
token = "token"
future_field = "ignored"

[future]
new_field = "ignored"

[[routes]]
event = "hermes.agent.*"
future_field = "ignored"
"#,
        )
        .unwrap();

        let config = AppConfig::load_or_default_with_env(&path, |_| None).unwrap();

        assert_eq!(config.providers.discord.token.as_deref(), Some("token"));
        assert_eq!(config.routes[0].event, "hermes.agent.*");
    }

    #[test]
    fn config_verify_rejects_invalid_route_shape() {
        let missing_event = AppConfig {
            routes: vec![RouteRule::default()],
            ..AppConfig::default()
        };

        let error = missing_event.validate().unwrap_err().to_string();
        assert!(error.contains("route #1 must set event"));

        let unsupported_sink = AppConfig {
            routes: vec![RouteRule {
                event: "hermes.agent.*".into(),
                sink: "pagerduty".into(),
                ..RouteRule::default()
            }],
            ..AppConfig::default()
        };

        let error = unsupported_sink.validate().unwrap_err().to_string();
        assert!(error.contains("unsupported sink 'pagerduty'"));
    }

    #[test]
    fn config_show_serializes_default_config_as_toml() {
        let rendered = AppConfig::default().to_pretty_toml().unwrap();

        assert!(rendered.contains("[daemon]"));
        assert!(rendered.contains("port = 25295"));
        assert!(rendered.contains("[privacy]"));
        assert!(rendered.contains("format = \"compact\""));
    }

    fn temp_config_path(label: &str) -> PathBuf {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "hermeship-config-test-{}-{label}-{now}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir.join("config.toml")
    }
}
