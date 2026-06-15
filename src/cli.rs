use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use serde_json::Value;

use crate::events::{IncomingEvent, MessageFormat};

#[derive(Debug, Parser)]
#[command(
    name = "hermeship",
    version,
    about = "Hermes-native daemon-first event gateway for notifications"
)]
pub struct Cli {
    /// Override the config file path.
    #[arg(long, global = true, env = "HERMESHIP_CONFIG")]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

impl Cli {
    pub fn config_path(&self) -> PathBuf {
        self.config
            .clone()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or_else(crate::config::default_config_path)
    }
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Start the daemon.
    #[command(alias = "serve")]
    Start {
        /// Override the daemon port.
        #[arg(long)]
        port: Option<u16>,
    },
    /// Check daemon health/status.
    Status,
    /// Send a custom message event to the daemon.
    Send {
        /// Override the delivery channel.
        #[arg(long)]
        channel: Option<String>,
        /// Message body for the custom event.
        #[arg(long)]
        message: String,
    },
    /// Emit a structured event to the daemon.
    Emit(EventArgs),
    /// Explain how a structured event would be routed.
    Explain(EventArgs),
    /// Manage configuration.
    Config {
        #[command(subcommand)]
        command: Option<ConfigCommand>,
    },
    /// Receive or install Hermes gateway hook integration.
    Hermes {
        #[command(subcommand)]
        command: HermesCommands,
    },
    /// Install hermeship local files and service scaffolding.
    Install,
    /// Uninstall hermeship local files and service scaffolding.
    Uninstall,
    /// Release consistency checks.
    Release {
        #[command(subcommand)]
        command: ReleaseCommands,
    },
}

#[derive(Debug, Clone, Args)]
pub struct EventArgs {
    /// Canonical event name, such as hermes.agent.started.
    pub event: String,
    /// Event fields as `--key value` pairs. Includes --payload, --channel, --mention, --format, and --template.
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub fields: Vec<String>,
}

impl EventArgs {
    pub fn into_event(self) -> Result<IncomingEvent> {
        let mut channel = None;
        let mut mention = None;
        let mut format = None;
        let mut template = None;
        let mut payload = None;
        let mut payload_map = serde_json::Map::new();

        if !self.fields.len().is_multiple_of(2) {
            anyhow::bail!("emit fields must be provided as --key value pairs");
        }

        for pair in self.fields.chunks_exact(2) {
            let key = pair[0]
                .strip_prefix("--")
                .ok_or_else(|| anyhow::anyhow!("emit field names must start with --"))?;
            let key = normalize_emit_key(key);
            let raw_value = pair[1].clone();

            match key {
                "channel" => channel = normalize_cli_text(raw_value),
                "mention" => mention = normalize_cli_text(raw_value),
                "format" => format = Some(MessageFormat::from_label(&raw_value)?),
                "template" => template = normalize_cli_text(raw_value),
                "payload" => {
                    payload = Some(
                        serde_json::from_str::<Value>(&raw_value)
                            .with_context(|| "emit --payload must be valid JSON")?,
                    );
                }
                _ => {
                    payload_map.insert(key.to_string(), parse_emit_value(&raw_value));
                }
            }
        }

        let payload = match payload {
            Some(Value::Object(mut object)) => {
                object.extend(payload_map);
                Value::Object(object)
            }
            Some(other) if payload_map.is_empty() => other,
            Some(_) => anyhow::bail!(
                "emit --payload must be a JSON object when additional --key value fields are provided"
            ),
            None => Value::Object(payload_map),
        };

        Ok(IncomingEvent {
            kind: self.event,
            channel,
            mention,
            format,
            template,
            payload,
        })
    }
}

fn normalize_emit_key(key: &str) -> &str {
    match key {
        "agent" => "agent_name",
        "session" => "session_id",
        "elapsed" => "elapsed_secs",
        "error" => "error_message",
        other => other,
    }
}

fn parse_emit_value(raw: &str) -> Value {
    serde_json::from_str(raw).unwrap_or_else(|_| Value::String(raw.to_string()))
}

fn normalize_cli_text(value: String) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

#[derive(Debug, Clone, Subcommand)]
pub enum ConfigCommand {
    /// Print the active config.
    Show,
    /// Print the active config path.
    Path,
    /// Verify the active config.
    Verify,
}

#[derive(Debug, Clone, Subcommand)]
pub enum HermesCommands {
    /// Forward a Hermes gateway hook payload.
    Hook(HermesHookArgs),
    /// Install Hermes gateway hook files.
    InstallHooks(InstallHooksArgs),
}

#[derive(Debug, Clone, Args)]
pub struct HermesHookArgs {
    /// Source provider for the hook payload.
    #[arg(long, default_value = "gateway")]
    pub provider: String,
    /// Hook payload JSON. Use `-` in later milestones to read stdin.
    #[arg(long)]
    pub payload: String,
}

#[derive(Debug, Clone, Args)]
pub struct InstallHooksArgs {
    /// Install scope for Hermes hooks.
    #[arg(long, default_value = "global")]
    pub scope: String,
    /// Override existing hook files.
    #[arg(long, default_value_t = false)]
    pub force: bool,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ReleaseCommands {
    /// Run release consistency checks.
    Preflight {
        /// Version being prepared for release.
        version: String,
    },
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use serde_json::json;

    use super::{Cli, Commands, ConfigCommand, HermesCommands, ReleaseCommands};
    use crate::events::MessageFormat;

    #[test]
    fn parses_send_command() {
        let cli = Cli::parse_from([
            "hermeship",
            "send",
            "--channel",
            "ops",
            "--message",
            "agent finished",
        ]);

        match cli.command {
            Some(Commands::Send { channel, message }) => {
                assert_eq!(channel.as_deref(), Some("ops"));
                assert_eq!(message, "agent finished");
            }
            other => panic!("expected send command, got {other:?}"),
        }
    }

    #[test]
    fn parses_emit_payload_command() {
        let cli = Cli::parse_from([
            "hermeship",
            "emit",
            "hermes.agent.started",
            "--payload",
            r#"{"session_id":"demo"}"#,
        ]);

        match cli.command {
            Some(Commands::Emit(args)) => {
                let event = args.into_event().unwrap();
                assert_eq!(event.kind, "hermes.agent.started");
                assert_eq!(event.payload["session_id"], json!("demo"));
            }
            other => panic!("expected emit command, got {other:?}"),
        }
    }

    #[test]
    fn emit_args_construct_incoming_event_from_payload_and_flags() {
        let cli = Cli::parse_from([
            "hermeship",
            "emit",
            "hermes.agent.started",
            "--payload",
            r#"{"session_id":"demo","attempt":1}"#,
            "--channel",
            "ops",
            "--mention",
            "@here",
            "--format",
            "alert",
            "--template",
            "agent {session_id}",
            "--platform",
            "telegram",
            "--retry",
            "false",
        ]);

        match cli.command {
            Some(Commands::Emit(args)) => {
                let event = args.into_event().unwrap();
                assert_eq!(event.kind, "hermes.agent.started");
                assert_eq!(event.channel.as_deref(), Some("ops"));
                assert_eq!(event.mention.as_deref(), Some("@here"));
                assert_eq!(event.format, Some(MessageFormat::Alert));
                assert_eq!(event.template.as_deref(), Some("agent {session_id}"));
                assert_eq!(event.payload["session_id"], json!("demo"));
                assert_eq!(event.payload["attempt"], json!(1));
                assert_eq!(event.payload["platform"], json!("telegram"));
                assert_eq!(event.payload["retry"], json!(false));
            }
            other => panic!("expected emit command, got {other:?}"),
        }
    }

    #[test]
    fn emit_args_reject_invalid_key_value_pairs() {
        let cli = Cli::parse_from([
            "hermeship",
            "emit",
            "hermes.agent.started",
            "--payload",
            r#"{"session_id":"demo"}"#,
            "--platform",
        ]);

        match cli.command {
            Some(Commands::Emit(args)) => {
                let error = args.into_event().unwrap_err().to_string();
                assert!(error.contains("emit fields must be provided as --key value pairs"));
            }
            other => panic!("expected emit command, got {other:?}"),
        }
    }

    #[test]
    fn emit_args_reject_invalid_format() {
        let cli = Cli::parse_from([
            "hermeship",
            "emit",
            "hermes.agent.started",
            "--format",
            "verbose",
        ]);

        match cli.command {
            Some(Commands::Emit(args)) => {
                let error = args.into_event().unwrap_err().to_string();
                assert!(error.contains("unsupported message format"));
            }
            other => panic!("expected emit command, got {other:?}"),
        }
    }

    #[test]
    fn emit_args_reject_fields_without_flag_prefix() {
        let cli = Cli::parse_from([
            "hermeship",
            "emit",
            "hermes.agent.started",
            "platform",
            "telegram",
        ]);

        match cli.command {
            Some(Commands::Emit(args)) => {
                let error = args.into_event().unwrap_err().to_string();
                assert!(error.contains("emit field names must start with --"));
            }
            other => panic!("expected emit command, got {other:?}"),
        }
    }

    #[test]
    fn parses_hermes_hook_payload_command() {
        let cli = Cli::parse_from([
            "hermeship",
            "hermes",
            "hook",
            "--provider",
            "gateway",
            "--payload",
            r#"{"event":"agent:start"}"#,
        ]);

        match cli.command {
            Some(Commands::Hermes {
                command: HermesCommands::Hook(args),
            }) => {
                assert_eq!(args.provider, "gateway");
                assert_eq!(args.payload, r#"{"event":"agent:start"}"#);
            }
            other => panic!("expected hermes hook command, got {other:?}"),
        }
    }

    #[test]
    fn parses_hermes_install_hooks_command() {
        let cli = Cli::parse_from([
            "hermeship",
            "hermes",
            "install-hooks",
            "--scope",
            "global",
            "--force",
        ]);

        match cli.command {
            Some(Commands::Hermes {
                command: HermesCommands::InstallHooks(args),
            }) => {
                assert_eq!(args.scope, "global");
                assert!(args.force);
            }
            other => panic!("expected hermes install-hooks command, got {other:?}"),
        }
    }

    #[test]
    fn parses_public_command_fixture() {
        let mut commands = Vec::new();
        for raw in include_str!("../tests/fixtures/cli/public_commands.txt").lines() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            commands.push(line.to_string());
            let args = split_fixture_command(line).unwrap_or_else(|error| {
                panic!("public command fixture is invalid `{line}`: {error}")
            });
            let argv = std::iter::once("hermeship".to_string()).chain(args);
            Cli::try_parse_from(argv).unwrap_or_else(|error| {
                panic!("public command fixture failed to parse `{line}`: {error}")
            });
        }

        for expected in [
            "start",
            "status",
            "send ",
            "emit ",
            "explain ",
            "config show",
            "config path",
            "config verify",
            "hermes hook",
            "hermes install-hooks",
            "install",
            "uninstall",
            "release preflight",
        ] {
            assert!(
                commands.iter().any(|command| command.starts_with(expected)),
                "public command fixture is missing required command prefix `{expected}`"
            );
        }
    }

    #[test]
    fn parses_config_and_release_subcommands() {
        let config = Cli::parse_from(["hermeship", "config", "verify"]);
        assert!(matches!(
            config.command,
            Some(Commands::Config {
                command: Some(ConfigCommand::Verify)
            })
        ));

        let release = Cli::parse_from(["hermeship", "release", "preflight", "0.1.0"]);
        assert!(matches!(
            release.command,
            Some(Commands::Release {
                command: ReleaseCommands::Preflight { version }
            }) if version == "0.1.0"
        ));
    }

    fn split_fixture_command(line: &str) -> Result<Vec<String>, String> {
        let mut args = Vec::new();
        let mut current = String::new();
        let mut quote = None;

        for char in line.chars() {
            match (quote, char) {
                (None, '\'') | (None, '"') => quote = Some(char),
                (Some(active), value) if active == value => quote = None,
                (None, value) if value.is_whitespace() => {
                    if !current.is_empty() {
                        args.push(std::mem::take(&mut current));
                    }
                }
                (_, value) => current.push(value),
            }
        }

        if let Some(active) = quote {
            return Err(format!("unterminated {active} quote"));
        }

        if !current.is_empty() {
            args.push(current);
        }

        Ok(args)
    }
}
