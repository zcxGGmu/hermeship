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
    /// Scaffold common setup values into the Hermeship config.
    Setup(SetupArgs),
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
    /// Emit local git source events.
    Git {
        #[command(subcommand)]
        command: GitCommands,
    },
    /// Emit local GitHub source events.
    Github {
        #[command(subcommand)]
        command: GithubCommands,
    },
    /// Install hermeship local files and service scaffolding.
    Install(InstallArgs),
    /// Uninstall hermeship local files and service scaffolding.
    Uninstall(UninstallArgs),
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

#[derive(Debug, Clone, Subcommand)]
pub enum GitCommands {
    /// Emit a git commit event.
    Commit(GitCommitArgs),
    /// Emit a git branch change event.
    #[command(name = "branch-changed")]
    BranchChanged(GitBranchChangedArgs),
}

#[derive(Debug, Clone, Args)]
pub struct GitCommitArgs {
    /// Repository display name.
    #[arg(long)]
    pub repo: String,
    /// Branch name for the commit.
    #[arg(long)]
    pub branch: String,
    /// Commit SHA, 7 to 64 hex characters.
    #[arg(long)]
    pub commit: String,
    /// One-line commit summary.
    #[arg(long)]
    pub summary: String,
    /// Override the delivery channel.
    #[arg(long)]
    pub channel: Option<String>,
    /// Repository root path for route metadata.
    #[arg(long)]
    pub repo_path: Option<PathBuf>,
    /// Worktree path for route metadata.
    #[arg(long)]
    pub worktree_path: Option<PathBuf>,
    /// Commit author display name.
    #[arg(long)]
    pub author_name: Option<String>,
    /// Commit author email for structured metadata.
    #[arg(long)]
    pub author_email: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub struct GitBranchChangedArgs {
    /// Repository display name.
    #[arg(long)]
    pub repo: String,
    /// Previous branch name.
    #[arg(long)]
    pub old_branch: String,
    /// New branch name.
    #[arg(long)]
    pub new_branch: String,
    /// Override the delivery channel.
    #[arg(long)]
    pub channel: Option<String>,
    /// Repository root path for route metadata.
    #[arg(long)]
    pub repo_path: Option<PathBuf>,
    /// Worktree path for route metadata.
    #[arg(long)]
    pub worktree_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Subcommand)]
pub enum GithubCommands {
    /// Emit a GitHub issue opened event.
    #[command(name = "issue-opened")]
    IssueOpened(GithubIssueArgs),
    /// Emit a GitHub pull request opened event.
    #[command(name = "pr-opened")]
    PrOpened(GithubPullRequestArgs),
    /// Emit a failed GitHub check or CI event.
    #[command(name = "check-failed")]
    CheckFailed(GithubCheckArgs),
    /// Emit a GitHub release published event.
    #[command(name = "release-published")]
    ReleasePublished(GithubReleaseArgs),
}

#[derive(Debug, Clone, Args)]
pub struct GithubIssueArgs {
    /// Repository owner or organization.
    #[arg(long)]
    pub owner: String,
    /// Repository display name.
    #[arg(long)]
    pub repo: String,
    /// Issue number.
    #[arg(long)]
    pub number: u64,
    /// One-line issue title.
    #[arg(long)]
    pub title: String,
    /// Issue author display name.
    #[arg(long)]
    pub author: Option<String>,
    /// Redacted synthetic issue URL.
    #[arg(long)]
    pub url: Option<String>,
    /// Override the delivery channel.
    #[arg(long)]
    pub channel: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub struct GithubPullRequestArgs {
    /// Repository owner or organization.
    #[arg(long)]
    pub owner: String,
    /// Repository display name.
    #[arg(long)]
    pub repo: String,
    /// Pull request number.
    #[arg(long)]
    pub number: u64,
    /// One-line pull request title.
    #[arg(long)]
    pub title: String,
    /// Head branch name.
    #[arg(long)]
    pub branch: String,
    /// Base branch name.
    #[arg(long)]
    pub base_branch: Option<String>,
    /// Head commit SHA, 7 to 64 hex characters.
    #[arg(long)]
    pub commit: Option<String>,
    /// Pull request author display name.
    #[arg(long)]
    pub author: Option<String>,
    /// Redacted synthetic pull request URL.
    #[arg(long)]
    pub url: Option<String>,
    /// Override the delivery channel.
    #[arg(long)]
    pub channel: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub struct GithubCheckArgs {
    /// Repository owner or organization.
    #[arg(long)]
    pub owner: String,
    /// Repository display name.
    #[arg(long)]
    pub repo: String,
    /// Workflow or check name.
    #[arg(long)]
    pub workflow: String,
    /// Check status such as failure, success, or cancelled.
    #[arg(long)]
    pub status: String,
    /// Branch name for the check.
    #[arg(long)]
    pub branch: String,
    /// Commit SHA, 7 to 64 hex characters.
    #[arg(long)]
    pub commit: Option<String>,
    /// One-line check title or summary.
    #[arg(long)]
    pub title: Option<String>,
    /// Redacted synthetic check URL.
    #[arg(long)]
    pub url: Option<String>,
    /// Override the delivery channel.
    #[arg(long)]
    pub channel: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub struct GithubReleaseArgs {
    /// Repository owner or organization.
    #[arg(long)]
    pub owner: String,
    /// Repository display name.
    #[arg(long)]
    pub repo: String,
    /// Release tag.
    #[arg(long)]
    pub tag: String,
    /// One-line release title.
    #[arg(long)]
    pub title: Option<String>,
    /// Release author display name.
    #[arg(long)]
    pub author: Option<String>,
    /// Redacted synthetic release URL.
    #[arg(long)]
    pub url: Option<String>,
    /// Override the delivery channel.
    #[arg(long)]
    pub channel: Option<String>,
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
    /// Remove Hermes gateway hook files installed by hermeship.
    UninstallHooks(UninstallHooksArgs),
}

#[derive(Debug, Clone, Args)]
pub struct HermesHookArgs {
    /// Source provider for the hook payload.
    #[arg(long, default_value = "gateway")]
    pub provider: String,
    /// Hook payload JSON. Use `-` to read stdin.
    #[arg(long)]
    pub payload: String,
}

#[derive(Debug, Clone, Args)]
pub struct InstallHooksArgs {
    /// Install scope for Hermes hooks.
    #[arg(long, default_value = "global")]
    pub scope: String,
    /// Hermes home directory. Defaults to HERMES_HOME or ~/.hermes.
    #[arg(long)]
    pub home: Option<PathBuf>,
    /// Print files that would be written without changing disk.
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,
    /// Override existing hook files.
    #[arg(long, default_value_t = false)]
    pub force: bool,
}

#[derive(Debug, Clone, Args)]
pub struct UninstallHooksArgs {
    /// Hermes home directory. Defaults to HERMES_HOME or ~/.hermes.
    #[arg(long)]
    pub home: Option<PathBuf>,
    /// Print files that would be removed without changing disk.
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,
}

#[derive(Debug, Clone, Args)]
pub struct SetupArgs {
    /// Read the Discord bot token from stdin.
    #[arg(long)]
    pub discord_token_stdin: bool,
    /// Read the Discord bot token from this environment variable.
    #[arg(long)]
    pub discord_token_env: Option<String>,
    /// Set the default delivery channel in [defaults].
    #[arg(long)]
    pub default_channel: Option<String>,
    /// Set daemon.base_url.
    #[arg(long)]
    pub daemon_url: Option<String>,
    /// Print planned setup changes without writing config.
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,
}

#[derive(Debug, Clone, Args)]
pub struct InstallArgs {
    /// Hermeship home directory. Defaults to ~/.hermeship.
    #[arg(long)]
    pub home: Option<PathBuf>,
    /// Replace an existing generated config scaffold.
    #[arg(long, default_value_t = false)]
    pub force: bool,
    /// Print files that would be created without changing disk.
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,
}

#[derive(Debug, Clone, Args)]
pub struct UninstallArgs {
    /// Hermeship home directory. Defaults to ~/.hermeship.
    #[arg(long)]
    pub home: Option<PathBuf>,
    /// Hermes home directory for safe Hermes hook removal.
    #[arg(long)]
    pub hermes_home: Option<PathBuf>,
    /// Remove Hermeship config.toml.
    #[arg(long, default_value_t = false)]
    pub remove_config: bool,
    /// Remove Hermeship state and logs directories.
    #[arg(long, default_value_t = false)]
    pub remove_state: bool,
    /// Remove Hermeship-managed Hermes gateway hooks.
    #[arg(long, default_value_t = false)]
    pub remove_hooks: bool,
    /// Print files that would be removed without changing disk.
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,
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
    use std::path::PathBuf;

    use clap::Parser;

    use serde_json::json;

    use super::{
        Cli, Commands, ConfigCommand, GitCommands, GithubCommands, HermesCommands, ReleaseCommands,
    };
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
    fn parses_setup_install_and_uninstall_commands() {
        let setup = Cli::parse_from([
            "hermeship",
            "setup",
            "--discord-token-stdin",
            "--default-channel",
            "ops",
            "--daemon-url",
            "http://127.0.0.1:25296",
            "--dry-run",
        ]);
        assert!(matches!(
            setup.command,
            Some(Commands::Setup(args))
                if args.discord_token_stdin
                    && args.discord_token_env.is_none()
                    && args.default_channel.as_deref() == Some("ops")
                    && args.daemon_url.as_deref() == Some("http://127.0.0.1:25296")
                    && args.dry_run
        ));

        let install = Cli::parse_from([
            "hermeship",
            "install",
            "--home",
            "/tmp/hermeship-home",
            "--force",
            "--dry-run",
        ]);
        assert!(matches!(
            install.command,
            Some(Commands::Install(args))
                if args.home == Some(PathBuf::from("/tmp/hermeship-home"))
                    && args.force
                    && args.dry_run
        ));

        let uninstall = Cli::parse_from([
            "hermeship",
            "uninstall",
            "--home",
            "/tmp/hermeship-home",
            "--hermes-home",
            "/tmp/hermes-home",
            "--remove-config",
            "--remove-state",
            "--remove-hooks",
            "--dry-run",
        ]);
        assert!(matches!(
            uninstall.command,
            Some(Commands::Uninstall(args))
                if args.home == Some(PathBuf::from("/tmp/hermeship-home"))
                    && args.hermes_home == Some(PathBuf::from("/tmp/hermes-home"))
                    && args.remove_config
                    && args.remove_state
                    && args.remove_hooks
                    && args.dry_run
        ));
    }

    #[test]
    fn rejects_plaintext_discord_token_arg() {
        let error =
            Cli::try_parse_from(["hermeship", "setup", "--discord-token", "synthetic-token"])
                .unwrap_err()
                .to_string();

        assert!(
            error.contains("unexpected argument '--discord-token'"),
            "{error}"
        );
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
    fn parses_git_commit_command() {
        let cli = Cli::parse_from([
            "hermeship",
            "git",
            "commit",
            "--repo",
            "hermeship",
            "--branch",
            "main",
            "--commit",
            "1234567890abcdef1234567890abcdef12345678",
            "--summary",
            "ship git source",
            "--channel",
            "ops",
            "--repo-path",
            "/tmp/hermeship",
            "--worktree-path",
            "/tmp/hermeship-worktree",
            "--author-name",
            "Synthetic Author",
            "--author-email",
            "synthetic@example.invalid",
        ]);

        match cli.command {
            Some(Commands::Git {
                command: GitCommands::Commit(args),
            }) => {
                assert_eq!(args.repo, "hermeship");
                assert_eq!(args.branch, "main");
                assert_eq!(args.commit, "1234567890abcdef1234567890abcdef12345678");
                assert_eq!(args.summary, "ship git source");
                assert_eq!(args.channel.as_deref(), Some("ops"));
                assert_eq!(args.repo_path, Some(PathBuf::from("/tmp/hermeship")));
                assert_eq!(
                    args.worktree_path,
                    Some(PathBuf::from("/tmp/hermeship-worktree"))
                );
                assert_eq!(args.author_name.as_deref(), Some("Synthetic Author"));
                assert_eq!(
                    args.author_email.as_deref(),
                    Some("synthetic@example.invalid")
                );
            }
            other => panic!("expected git commit command, got {other:?}"),
        }
    }

    #[test]
    fn parses_git_branch_changed_command() {
        let cli = Cli::parse_from([
            "hermeship",
            "git",
            "branch-changed",
            "--repo",
            "hermeship",
            "--old-branch",
            "main",
            "--new-branch",
            "codex/milestone-8-git",
            "--channel",
            "ops",
            "--repo-path",
            "/tmp/hermeship",
            "--worktree-path",
            "/tmp/hermeship-worktree",
        ]);

        match cli.command {
            Some(Commands::Git {
                command: GitCommands::BranchChanged(args),
            }) => {
                assert_eq!(args.repo, "hermeship");
                assert_eq!(args.old_branch, "main");
                assert_eq!(args.new_branch, "codex/milestone-8-git");
                assert_eq!(args.channel.as_deref(), Some("ops"));
                assert_eq!(args.repo_path, Some(PathBuf::from("/tmp/hermeship")));
                assert_eq!(
                    args.worktree_path,
                    Some(PathBuf::from("/tmp/hermeship-worktree"))
                );
            }
            other => panic!("expected git branch-changed command, got {other:?}"),
        }
    }

    #[test]
    fn parses_github_issue_pr_check_and_release_commands() {
        let issue = Cli::parse_from([
            "hermeship",
            "github",
            "issue-opened",
            "--owner",
            "posp",
            "--repo",
            "hermeship",
            "--number",
            "42",
            "--title",
            "Add deterministic GitHub source",
            "--author",
            "synthetic-user",
            "--url",
            "https://github.example.invalid/posp/hermeship/issues/42",
            "--channel",
            "ops",
        ]);
        match issue.command {
            Some(Commands::Github {
                command: GithubCommands::IssueOpened(args),
            }) => {
                assert_eq!(args.owner, "posp");
                assert_eq!(args.repo, "hermeship");
                assert_eq!(args.number, 42);
                assert_eq!(args.title, "Add deterministic GitHub source");
                assert_eq!(args.author.as_deref(), Some("synthetic-user"));
                assert_eq!(args.channel.as_deref(), Some("ops"));
            }
            other => panic!("expected github issue-opened command, got {other:?}"),
        }

        let pr = Cli::parse_from([
            "hermeship",
            "github",
            "pr-opened",
            "--owner",
            "posp",
            "--repo",
            "hermeship",
            "--number",
            "17",
            "--title",
            "Ship GitHub source",
            "--branch",
            "codex/milestone-8-github",
            "--base-branch",
            "main",
            "--commit",
            "1234567890abcdef1234567890abcdef12345678",
        ]);
        assert!(matches!(
            pr.command,
            Some(Commands::Github {
                command: GithubCommands::PrOpened(args),
            }) if args.number == 17
                && args.branch == "codex/milestone-8-github"
                && args.base_branch.as_deref() == Some("main")
                && args.commit.as_deref() == Some("1234567890abcdef1234567890abcdef12345678")
        ));

        let check = Cli::parse_from([
            "hermeship",
            "github",
            "check-failed",
            "--owner",
            "posp",
            "--repo",
            "hermeship",
            "--workflow",
            "ci",
            "--status",
            "failure",
            "--branch",
            "main",
            "--title",
            "cargo test failed",
        ]);
        assert!(matches!(
            check.command,
            Some(Commands::Github {
                command: GithubCommands::CheckFailed(args),
            }) if args.workflow == "ci"
                && args.status == "failure"
                && args.branch == "main"
                && args.title.as_deref() == Some("cargo test failed")
        ));

        let release = Cli::parse_from([
            "hermeship",
            "github",
            "release-published",
            "--owner",
            "posp",
            "--repo",
            "hermeship",
            "--tag",
            "v0.1.0",
            "--title",
            "Hermeship v0.1.0",
        ]);
        assert!(matches!(
            release.command,
            Some(Commands::Github {
                command: GithubCommands::ReleasePublished(args),
            }) if args.tag == "v0.1.0"
                && args.title.as_deref() == Some("Hermeship v0.1.0")
        ));
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
            "--home",
            "/tmp/hermes",
            "--dry-run",
            "--force",
        ]);

        match cli.command {
            Some(Commands::Hermes {
                command: HermesCommands::InstallHooks(args),
            }) => {
                assert_eq!(args.scope, "global");
                assert_eq!(args.home, Some(PathBuf::from("/tmp/hermes")));
                assert!(args.dry_run);
                assert!(args.force);
            }
            other => panic!("expected hermes install-hooks command, got {other:?}"),
        }
    }

    #[test]
    fn parses_hermes_uninstall_hooks_command() {
        let cli = Cli::parse_from([
            "hermeship",
            "hermes",
            "uninstall-hooks",
            "--home",
            "/tmp/hermes",
            "--dry-run",
        ]);

        match cli.command {
            Some(Commands::Hermes {
                command: HermesCommands::UninstallHooks(args),
            }) => {
                assert_eq!(args.home, Some(PathBuf::from("/tmp/hermes")));
                assert!(args.dry_run);
            }
            other => panic!("expected hermes uninstall-hooks command, got {other:?}"),
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
            "setup ",
            "send ",
            "emit ",
            "explain ",
            "config show",
            "config path",
            "config verify",
            "hermes hook",
            "hermes install-hooks",
            "hermes uninstall-hooks",
            "git commit",
            "git branch-changed",
            "github issue-opened",
            "github pr-opened",
            "github check-failed",
            "github release-published",
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
