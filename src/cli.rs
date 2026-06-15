use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

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
    /// JSON payload for the event. Use `-` in later milestones to read stdin.
    #[arg(long)]
    pub payload: Option<String>,
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

    use super::{Cli, Commands, ConfigCommand, HermesCommands, ReleaseCommands};

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
                assert_eq!(args.event, "hermes.agent.started");
                assert_eq!(args.payload.as_deref(), Some(r#"{"session_id":"demo"}"#));
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
            let argv = std::iter::once("hermeship".to_string()).chain(args.into_iter());
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
