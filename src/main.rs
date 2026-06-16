use std::io::{self, Read};

use anyhow::{Context, Result};
use clap::Parser;
use hermeship::cli::{
    Cli, Commands, ConfigCommand, GitCommands, GithubCommands, HermesCommands, ReleaseCommands,
};
use hermeship::client::DaemonClient;
use hermeship::config::AppConfig;
use hermeship::daemon::{EventAcceptedResponse, HealthResponse};
use hermeship::events::IncomingEvent;
use hermeship::hermes::HermesHookEnvelope;
use hermeship::hooks::{HookInstallOptions, HookInstallReport, HookUninstallReport};
use hermeship::lifecycle::{InstallOptions, SetupOptions, UninstallOptions};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() {
    if let Err(error) = real_main().await {
        eprintln!("hermeship error: {error}");
        std::process::exit(1);
    }
}

async fn real_main() -> Result<()> {
    let cli = Cli::parse();
    let config_was_overridden = cli.config.is_some();
    let config_path = cli.config_path();

    match cli.command.unwrap_or(Commands::Start { port: None }) {
        Commands::Start { port } => {
            let mut config = AppConfig::load_or_default(&config_path)?;
            if let Some(port) = port {
                config.daemon.port = port;
                config.daemon.base_url = None;
            }
            config.validate()?;
            hermeship::daemon::serve(config, VERSION).await
        }
        Commands::Status => {
            let config = AppConfig::load_or_default(&config_path)?;
            let health = DaemonClient::from_config(&config.daemon).health().await?;
            print_health(&health);
            Ok(())
        }
        Commands::Send { channel, message } => {
            let config = AppConfig::load_or_default(&config_path)?;
            let accepted = submit_event(&config, IncomingEvent::custom(channel, message)).await?;
            print_event_accepted("send", &accepted);
            Ok(())
        }
        Commands::Emit(args) => {
            let config = AppConfig::load_or_default(&config_path)?;
            let accepted = submit_event(&config, args.into_event()?).await?;
            print_event_accepted("emit", &accepted);
            Ok(())
        }
        Commands::Explain(args) => {
            let config = AppConfig::load_or_default(&config_path)?;
            print!("{}", explain_event(&config, args.into_event()?)?);
            Ok(())
        }
        Commands::Config { command } => match command.unwrap_or(ConfigCommand::Show) {
            ConfigCommand::Show => {
                let config = AppConfig::load_or_default(&config_path)?;
                println!("{}", config.to_redacted_pretty_toml()?);
                Ok(())
            }
            ConfigCommand::Path => {
                println!("{}", config_path.display());
                Ok(())
            }
            ConfigCommand::Verify => {
                let config = AppConfig::load_or_default(&config_path)?;
                config.validate()?;
                println!("config ok: {}", config_path.display());
                Ok(())
            }
        },
        Commands::Setup(args) => {
            let discord_token = read_setup_token(&args, io::stdin().lock())?;
            let report = hermeship::lifecycle::setup(&SetupOptions {
                config_path,
                discord_token,
                default_channel: args.default_channel,
                daemon_url: args.daemon_url,
                dry_run: args.dry_run,
            })?;
            print!("{}", report.render());
            Ok(())
        }
        Commands::Hermes { command } => match command {
            HermesCommands::Hook(args) => {
                let config = AppConfig::load_or_default(&config_path)?;
                let payload = read_payload_arg(&args.payload, io::stdin().lock())?;
                let hook = serde_json::from_str::<HermesHookEnvelope>(&payload)
                    .with_context(|| "hermes hook --payload must be valid JSON")?
                    .with_source_default(args.provider);
                let accepted = submit_hermes_hook(&config, &hook).await?;
                print_event_accepted("hermes hook", &accepted);
                Ok(())
            }
            HermesCommands::InstallHooks(args) => {
                ensure_global_hermes_hook_scope(&args.scope)?;
                let hermes_home = args
                    .home
                    .unwrap_or_else(hermeship::hooks::default_hermes_home);
                let report = hermeship::hooks::install_hermes_hooks(&HookInstallOptions {
                    hermes_home,
                    hermeship_bin: Some(
                        std::env::current_exe()
                            .context("failed to resolve current hermeship binary")?,
                    ),
                    force: args.force,
                    dry_run: args.dry_run,
                })?;
                print_hook_install_report(&report);
                Ok(())
            }
            HermesCommands::UninstallHooks(args) => {
                let hermes_home = args
                    .home
                    .unwrap_or_else(hermeship::hooks::default_hermes_home);
                let report = hermeship::hooks::uninstall_hermes_hooks(hermes_home, args.dry_run)?;
                print_hook_uninstall_report(&report);
                Ok(())
            }
        },
        Commands::Git { command } => {
            let config = AppConfig::load_or_default(&config_path)?;
            let accepted = submit_event(&config, git_command_into_event(command)?).await?;
            print_event_accepted("git", &accepted);
            Ok(())
        }
        Commands::Github { command } => {
            let config = AppConfig::load_or_default(&config_path)?;
            let accepted = submit_event(&config, github_command_into_event(command)?).await?;
            print_event_accepted("github", &accepted);
            Ok(())
        }
        Commands::Install(args) => {
            let home = args.home.unwrap_or_else(hermeship::lifecycle::default_home);
            let config_path = lifecycle_config_path(config_path, config_was_overridden, &home);
            let report = hermeship::lifecycle::install(&InstallOptions {
                home,
                config_path,
                force: args.force,
                dry_run: args.dry_run,
            })?;
            print!("{}", report.render());
            Ok(())
        }
        Commands::Uninstall(args) => {
            let home = args.home.unwrap_or_else(hermeship::lifecycle::default_home);
            let config_path = lifecycle_config_path(config_path, config_was_overridden, &home);
            let hermes_home = if args.remove_hooks {
                Some(
                    args.hermes_home
                        .unwrap_or_else(hermeship::hooks::default_hermes_home),
                )
            } else {
                args.hermes_home
            };
            let report = hermeship::lifecycle::uninstall(&UninstallOptions {
                home,
                config_path,
                hermes_home,
                remove_config: args.remove_config,
                remove_state: args.remove_state,
                remove_hooks: args.remove_hooks,
                dry_run: args.dry_run,
            })?;
            print!("{}", report.render());
            Ok(())
        }
        Commands::Release { command } => match command {
            ReleaseCommands::Preflight { version } => {
                let repo_root = std::env::current_dir().context("failed to resolve current dir")?;
                let report = hermeship::release_preflight::run_preflight(&repo_root, &version)?;
                print!("{}", report.render());
                if report.ok() {
                    Ok(())
                } else {
                    anyhow::bail!("release preflight failed")
                }
            }
        },
    }
}

fn lifecycle_config_path(
    default_path: std::path::PathBuf,
    was_overridden: bool,
    home: &std::path::Path,
) -> std::path::PathBuf {
    if was_overridden {
        default_path
    } else {
        home.join("config.toml")
    }
}

fn read_setup_token(args: &hermeship::cli::SetupArgs, stdin: impl Read) -> Result<Option<String>> {
    match (args.discord_token_stdin, args.discord_token_env.as_ref()) {
        (true, Some(_)) => {
            anyhow::bail!("use only one of --discord-token-stdin or --discord-token-env")
        }
        (true, None) => read_secret_stdin(stdin).map(Some),
        (false, Some(name)) => {
            let value = std::env::var(name)
                .with_context(|| format!("environment variable {name} is not set"))?;
            let token = value.trim();
            if token.is_empty() {
                anyhow::bail!("environment variable {name} is empty");
            }
            Ok(Some(token.to_string()))
        }
        (false, None) => Ok(None),
    }
}

fn read_secret_stdin(mut stdin: impl Read) -> Result<String> {
    let mut buffer = String::new();
    stdin
        .read_to_string(&mut buffer)
        .context("failed to read Discord token from stdin")?;
    let token = buffer.trim();
    if token.is_empty() {
        anyhow::bail!("--discord-token-stdin received empty stdin");
    }
    Ok(token.to_string())
}

fn ensure_global_hermes_hook_scope(scope: &str) -> Result<()> {
    if scope.trim() == "global" {
        return Ok(());
    }

    anyhow::bail!("Hermes gateway hook install currently supports only --scope global")
}

fn print_hook_install_report(report: &HookInstallReport) {
    if report.dry_run {
        println!(
            "hermes hooks dry-run: would write {}",
            report.hook_dir.display()
        );
        for path in &report.planned_files {
            println!("  would write {}", path.display());
        }
        return;
    }

    println!("hermes hooks installed: {}", report.hook_dir.display());
    for path in &report.written_files {
        println!("  wrote {}", path.display());
    }
    for path in &report.skipped_files {
        println!("  skipped existing {}", path.display());
    }
}

fn print_hook_uninstall_report(report: &HookUninstallReport) {
    if report.dry_run {
        println!(
            "hermes hooks dry-run: would remove {}",
            report.hook_dir.display()
        );
        for path in &report.planned_paths {
            println!("  would remove {}", path.display());
        }
        return;
    }

    if report.removed_paths.is_empty() {
        println!("hermes hooks not installed: {}", report.hook_dir.display());
        return;
    }

    println!("hermes hooks removed:");
    for path in &report.removed_paths {
        println!("  removed {}", path.display());
    }
}

async fn submit_event(config: &AppConfig, event: IncomingEvent) -> Result<EventAcceptedResponse> {
    let client = DaemonClient::from_config(&config.daemon);
    client.post_event(&event).await
}

async fn submit_hermes_hook(
    config: &AppConfig,
    hook: &HermesHookEnvelope,
) -> Result<EventAcceptedResponse> {
    let client = DaemonClient::from_config(&config.daemon);
    client.post_hermes_hook(hook).await
}

fn git_command_into_event(command: GitCommands) -> Result<IncomingEvent> {
    match command {
        GitCommands::Commit(args) => {
            hermeship::source::git::commit_event(hermeship::source::git::GitCommitInput {
                repo: args.repo,
                branch: args.branch,
                commit: args.commit,
                summary: args.summary,
                channel: args.channel,
                repo_path: args.repo_path.map(path_to_string),
                worktree_path: args.worktree_path.map(path_to_string),
                author_name: args.author_name,
                author_email: args.author_email,
            })
        }
        GitCommands::BranchChanged(args) => Ok(hermeship::source::git::branch_changed_event(
            hermeship::source::git::GitBranchChangedInput {
                repo: args.repo,
                old_branch: args.old_branch,
                new_branch: args.new_branch,
                channel: args.channel,
                repo_path: args.repo_path.map(path_to_string),
                worktree_path: args.worktree_path.map(path_to_string),
            },
        )),
    }
}

fn github_command_into_event(command: GithubCommands) -> Result<IncomingEvent> {
    match command {
        GithubCommands::IssueOpened(args) => hermeship::source::github::issue_opened_event(
            hermeship::source::github::GithubIssueInput {
                owner: args.owner,
                repo: args.repo,
                number: args.number,
                title: args.title,
                author: args.author,
                url: args.url,
                channel: args.channel,
            },
        ),
        GithubCommands::PrOpened(args) => hermeship::source::github::pull_request_opened_event(
            hermeship::source::github::GithubPullRequestInput {
                owner: args.owner,
                repo: args.repo,
                number: args.number,
                title: args.title,
                branch: args.branch,
                base_branch: args.base_branch,
                commit: args.commit,
                author: args.author,
                url: args.url,
                channel: args.channel,
            },
        ),
        GithubCommands::CheckFailed(args) => hermeship::source::github::check_failed_event(
            hermeship::source::github::GithubCheckInput {
                owner: args.owner,
                repo: args.repo,
                workflow: args.workflow,
                status: args.status,
                branch: args.branch,
                commit: args.commit,
                title: args.title,
                url: args.url,
                channel: args.channel,
            },
        ),
        GithubCommands::ReleasePublished(args) => {
            hermeship::source::github::release_published_event(
                hermeship::source::github::GithubReleaseInput {
                    owner: args.owner,
                    repo: args.repo,
                    tag: args.tag,
                    title: args.title,
                    author: args.author,
                    url: args.url,
                    channel: args.channel,
                },
            )
        }
    }
}

fn path_to_string(path: std::path::PathBuf) -> String {
    path.to_string_lossy().into_owned()
}

fn explain_event(config: &AppConfig, event: IncomingEvent) -> Result<String> {
    let sanitized = hermeship::privacy::sanitize_payload(&event.payload, &config.privacy);
    let event = IncomingEvent {
        payload: sanitized,
        ..event
    };
    let envelope = hermeship::event::compat::from_incoming_event(&event)?;
    let router = hermeship::router::Router::new(config.clone());

    Ok(router.explain(&envelope).to_string())
}

fn read_payload_arg(payload: &str, mut stdin: impl Read) -> Result<String> {
    if payload != "-" {
        return Ok(payload.to_string());
    }

    let mut buffer = String::new();
    stdin
        .read_to_string(&mut buffer)
        .context("failed to read hermes hook payload from stdin")?;
    let payload = buffer.trim();
    if payload.is_empty() {
        anyhow::bail!("hermes hook --payload - received empty stdin");
    }
    Ok(payload.to_string())
}

fn print_event_accepted(name: &str, accepted: &EventAcceptedResponse) {
    println!(
        "{name} queued: id={} kind={} queue={} pending={} capacity={}",
        accepted.id,
        accepted.canonical_kind,
        accepted.queue.status,
        accepted.queue.pending,
        accepted.queue.capacity
    );
}

fn print_health(health: &HealthResponse) {
    println!("hermeship daemon: {}", health.status);
    println!("version: {}", health.version);
    println!(
        "queue: {} (pending={}, capacity={})",
        health.queue.status, health.queue.pending, health.queue.capacity
    );
    let sinks = if health.configured_sinks.is_empty() {
        "none".to_string()
    } else {
        health.configured_sinks.join(", ")
    };
    println!("configured sinks: {sinks}");
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use tokio::sync::mpsc;

    use clap::Parser;
    use hermeship::cli::{Cli, Commands};
    use hermeship::config::AppConfig;
    use hermeship::daemon::daemon_router_with_queue;
    use hermeship::event::{EventBody, EventEnvelope};
    use hermeship::events::IncomingEvent;
    use hermeship::hermes::HermesHookEnvelope;

    use super::{
        VERSION, explain_event, git_command_into_event, github_command_into_event,
        read_payload_arg, read_setup_token, submit_event, submit_hermes_hook,
    };

    #[tokio::test]
    async fn daemon_emit_command_posts_event_to_daemon() {
        let config = AppConfig::default();
        let (queue_tx, mut queue_rx) = mpsc::channel(8);
        let (base_url, server) = spawn_test_daemon(config.clone(), queue_tx).await;
        let mut config = config;
        config.daemon.base_url = Some(base_url);

        let accepted = submit_event(
            &config,
            IncomingEvent::new("hermes.agent.started", json!({ "session_id": "demo" })),
        )
        .await
        .unwrap();

        assert!(accepted.queued);
        assert_eq!(accepted.canonical_kind, "hermes.agent.started");
        assert_eq!(accepted.queue.pending, 1);

        let envelope = queue_rx.try_recv().unwrap();
        server.abort();

        assert_eq!(envelope.canonical_kind(), "hermes.agent.started");
    }

    #[tokio::test]
    async fn daemon_send_command_posts_custom_event_to_daemon() {
        let config = AppConfig::default();
        let (queue_tx, mut queue_rx) = mpsc::channel(8);
        let (base_url, server) = spawn_test_daemon(config.clone(), queue_tx).await;
        let mut config = config;
        config.daemon.base_url = Some(base_url);

        let accepted = submit_event(
            &config,
            IncomingEvent::custom(Some("ops".to_string()), "hello".to_string()),
        )
        .await
        .unwrap();

        assert!(accepted.queued);
        assert_eq!(accepted.canonical_kind, "custom");
        assert_eq!(accepted.queue.pending, 1);

        let envelope = queue_rx.try_recv().unwrap();
        server.abort();

        assert_eq!(envelope.canonical_kind(), "custom");
        match envelope.body {
            EventBody::Custom(body) => {
                assert_eq!(body.message, "hello");
                assert_eq!(body.payload.unwrap()["summary"], "hello");
            }
            other => panic!("expected custom event, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn daemon_git_commit_command_posts_git_event_to_daemon() {
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
        ]);
        let Some(Commands::Git { command }) = cli.command else {
            panic!("expected git command");
        };
        let event = git_command_into_event(command).unwrap();

        let config = AppConfig::default();
        let (queue_tx, mut queue_rx) = mpsc::channel(8);
        let (base_url, server) = spawn_test_daemon(config.clone(), queue_tx).await;
        let mut config = config;
        config.daemon.base_url = Some(base_url);

        let accepted = submit_event(&config, event).await.unwrap();

        assert!(accepted.queued);
        assert_eq!(accepted.canonical_kind, "git.commit");
        assert_eq!(accepted.queue.pending, 1);

        let envelope = queue_rx.try_recv().unwrap();
        server.abort();

        assert_eq!(envelope.canonical_kind(), "git.commit");
        assert_eq!(envelope.metadata.channel_hint.as_deref(), Some("ops"));
        assert_eq!(envelope.metadata.repo_name.as_deref(), Some("hermeship"));
        assert_eq!(
            envelope.metadata.repo_path.as_deref(),
            Some("/tmp/hermeship")
        );
        assert_eq!(envelope.metadata.branch.as_deref(), Some("main"));
        match envelope.body {
            EventBody::GitCommit(body) => {
                assert_eq!(body.repo, "hermeship");
                assert_eq!(body.short_sha, "1234567");
                assert_eq!(body.summary, "ship git source");
            }
            other => panic!("expected GitCommit, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn daemon_git_branch_changed_command_posts_git_event_to_daemon() {
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
            "--repo-path",
            "/tmp/hermeship",
            "--worktree-path",
            "/tmp/hermeship-worktree",
        ]);
        let Some(Commands::Git { command }) = cli.command else {
            panic!("expected git command");
        };
        let event = git_command_into_event(command).unwrap();

        let config = AppConfig::default();
        let (queue_tx, mut queue_rx) = mpsc::channel(8);
        let (base_url, server) = spawn_test_daemon(config.clone(), queue_tx).await;
        let mut config = config;
        config.daemon.base_url = Some(base_url);

        let accepted = submit_event(&config, event).await.unwrap();

        assert!(accepted.queued);
        assert_eq!(accepted.canonical_kind, "git.branch-changed");

        let envelope = queue_rx.try_recv().unwrap();
        server.abort();

        assert_eq!(envelope.canonical_kind(), "git.branch-changed");
        assert_eq!(envelope.metadata.repo_name.as_deref(), Some("hermeship"));
        assert_eq!(
            envelope.metadata.branch.as_deref(),
            Some("codex/milestone-8-git")
        );
        match envelope.body {
            EventBody::GitBranchChanged(body) => {
                assert_eq!(body.repo, "hermeship");
                assert_eq!(body.old_branch, "main");
                assert_eq!(body.new_branch, "codex/milestone-8-git");
            }
            other => panic!("expected GitBranchChanged, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn daemon_github_issue_command_posts_github_event_to_daemon() {
        let cli = Cli::parse_from([
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
            "--channel",
            "ops",
        ]);
        let Some(Commands::Github { command }) = cli.command else {
            panic!("expected github command");
        };
        let event = github_command_into_event(command).unwrap();

        let config = AppConfig::default();
        let (queue_tx, mut queue_rx) = mpsc::channel(8);
        let (base_url, server) = spawn_test_daemon(config.clone(), queue_tx).await;
        let mut config = config;
        config.daemon.base_url = Some(base_url);

        let accepted = submit_event(&config, event).await.unwrap();

        assert!(accepted.queued);
        assert_eq!(accepted.canonical_kind, "github.issue-opened");
        assert_eq!(accepted.queue.pending, 1);

        let envelope = queue_rx.try_recv().unwrap();
        server.abort();

        assert_eq!(envelope.canonical_kind(), "github.issue-opened");
        assert_eq!(envelope.metadata.channel_hint.as_deref(), Some("ops"));
        assert_eq!(envelope.metadata.repo_name.as_deref(), Some("hermeship"));
        match envelope.body {
            EventBody::GithubIssue(body) => {
                assert_eq!(body.owner, "posp");
                assert_eq!(body.repo, "hermeship");
                assert_eq!(body.number, 42);
                assert_eq!(body.title, "Add deterministic GitHub source");
            }
            other => panic!("expected GithubIssue, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn daemon_hermes_hook_command_posts_hook_to_daemon() {
        let config = AppConfig::default();
        let (queue_tx, mut queue_rx) = mpsc::channel(8);
        let (base_url, server) = spawn_test_daemon(config.clone(), queue_tx).await;
        let mut config = config;
        config.daemon.base_url = Some(base_url);
        let hook: HermesHookEnvelope = serde_json::from_value(json!({
            "event": "agent:start",
            "context": {
                "session_id": "demo"
            }
        }))
        .unwrap();

        let accepted = submit_hermes_hook(&config, &hook).await.unwrap();

        assert!(accepted.queued);
        assert_eq!(accepted.canonical_kind, "hermes.agent.started");
        assert_eq!(accepted.queue.pending, 1);

        let envelope = queue_rx.try_recv().unwrap();
        server.abort();

        assert_eq!(envelope.canonical_kind(), "hermes.agent.started");
        assert_eq!(envelope.metadata.provider.as_deref(), Some("hermes"));
        assert_eq!(envelope.metadata.source.as_deref(), Some("gateway"));
    }

    #[test]
    fn hermes_hook_payload_dash_reads_from_stdin() {
        let payload = read_payload_arg(
            "-",
            r#"{"event":"agent:start","context":{"session_id":"demo"}}"#.as_bytes(),
        )
        .unwrap();

        assert_eq!(
            payload,
            r#"{"event":"agent:start","context":{"session_id":"demo"}}"#
        );
    }

    #[test]
    fn hermes_hook_payload_inline_json_is_used_directly() {
        let payload = read_payload_arg(
            r#"{"event":"agent:start"}"#,
            r#"{"event":"ignored"}"#.as_bytes(),
        )
        .unwrap();

        assert_eq!(payload, r#"{"event":"agent:start"}"#);
    }

    #[test]
    fn setup_token_stdin_reads_secret_without_cli_arg() {
        let args = hermeship::cli::SetupArgs {
            discord_token_stdin: true,
            discord_token_env: None,
            default_channel: None,
            daemon_url: None,
            dry_run: false,
        };

        let token = read_setup_token(&args, b"synthetic-token\n".as_slice()).unwrap();

        assert_eq!(token.as_deref(), Some("synthetic-token"));
    }

    #[test]
    fn setup_token_rejects_empty_stdin_and_conflicting_sources() {
        let empty = hermeship::cli::SetupArgs {
            discord_token_stdin: true,
            discord_token_env: None,
            default_channel: None,
            daemon_url: None,
            dry_run: false,
        };
        let error = read_setup_token(&empty, b"   \n".as_slice())
            .unwrap_err()
            .to_string();
        assert!(error.contains("empty stdin"), "{error}");

        let conflicting = hermeship::cli::SetupArgs {
            discord_token_stdin: true,
            discord_token_env: Some("HERMESHIP_SETUP_DISCORD_TOKEN".to_string()),
            default_channel: None,
            daemon_url: None,
            dry_run: false,
        };
        let error = read_setup_token(&conflicting, b"synthetic-token".as_slice())
            .unwrap_err()
            .to_string();
        assert!(error.contains("use only one"), "{error}");
    }

    #[test]
    fn router_explain_event_prints_route_diagnostics_without_daemon() {
        let config = AppConfig {
            routes: vec![
                hermeship::config::RouteRule {
                    event: "hermes.agent.*".to_string(),
                    channel: Some("ops".to_string()),
                    format: Some(hermeship::config::MessageFormat::Alert),
                    ..hermeship::config::RouteRule::default()
                },
                hermeship::config::RouteRule {
                    event: "hermes.agent.*".to_string(),
                    filter: std::collections::BTreeMap::from([(
                        "platform".to_string(),
                        "discord".to_string(),
                    )]),
                    channel: Some("discord-only".to_string()),
                    ..hermeship::config::RouteRule::default()
                },
            ],
            ..AppConfig::default()
        };
        let event = IncomingEvent::new(
            "hermes.agent.started",
            json!({
                "platform": "telegram",
                "session_id": "demo"
            }),
        );

        let output = explain_event(&config, event).unwrap();

        assert!(output.contains("event: hermes.agent.started"));
        assert!(
            output.contains("route candidates: hermes.agent.started, hermes.agent.*, hermes.*")
        );
        assert!(output.contains("[MATCH] #0"));
        assert!(output.contains("DiscordChannel(\"ops\")"));
        assert!(output.contains("[skip] #1"));
        assert!(output.contains("platform expected \"discord\" actual \"telegram\""));
    }

    async fn spawn_test_daemon(
        config: AppConfig,
        queue_tx: mpsc::Sender<EventEnvelope>,
    ) -> (String, tokio::task::JoinHandle<std::io::Result<()>>) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let router = daemon_router_with_queue(config, VERSION, queue_tx);
        let server = tokio::spawn(async move { axum::serve(listener, router).await });

        (format!("http://{address}"), server)
    }
}
