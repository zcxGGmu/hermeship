use anyhow::Result;
use clap::Parser;
use hermeship::cli::{Cli, Commands, ConfigCommand, HermesCommands, ReleaseCommands};
use hermeship::client::DaemonClient;
use hermeship::config::AppConfig;
use hermeship::daemon::HealthResponse;
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
            print_placeholder("send", IncomingEvent::custom(channel, message))
        }
        Commands::Emit(args) => print_placeholder("emit", args.into_event()?),
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
