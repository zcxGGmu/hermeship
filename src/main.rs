use anyhow::Result;
use clap::Parser;
use hermeship::cli::{Cli, Commands, ConfigCommand, HermesCommands, ReleaseCommands};
use hermeship::client::DaemonClient;
use hermeship::config::AppConfig;
use hermeship::daemon::{EventAcceptedResponse, HealthResponse};
use hermeship::events::IncomingEvent;

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
        Commands::Explain(args) => print_placeholder("explain", args.into_event()?),
        Commands::Config { command } => match command.unwrap_or(ConfigCommand::Show) {
            ConfigCommand::Show => {
                let config = AppConfig::load_or_default(&config_path)?;
                println!("{}", config.to_pretty_toml()?);
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
        Commands::Hermes { command } => match command {
            HermesCommands::Hook(args) => print_placeholder("hermes hook", args.payload),
            HermesCommands::InstallHooks(args) => {
                print_placeholder("hermes install-hooks", (args.scope, args.force))
            }
        },
        Commands::Install => print_placeholder("install", ()),
        Commands::Uninstall => print_placeholder("uninstall", ()),
        Commands::Release { command } => match command {
            ReleaseCommands::Preflight { version } => {
                print_placeholder("release preflight", version)
            }
        },
    }
}

fn print_placeholder(name: &str, _args: impl std::fmt::Debug) -> Result<()> {
    println!("{name} command parsed; implementation will arrive in a later milestone");
    Ok(())
}

async fn submit_event(config: &AppConfig, event: IncomingEvent) -> Result<EventAcceptedResponse> {
    let client = DaemonClient::from_config(&config.daemon);
    client.post_event(&event).await
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

    use hermeship::config::AppConfig;
    use hermeship::daemon::daemon_router_with_queue;
    use hermeship::event::{EventBody, EventEnvelope};
    use hermeship::events::IncomingEvent;

    use super::{VERSION, submit_event};

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
