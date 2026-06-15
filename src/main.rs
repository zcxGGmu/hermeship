use anyhow::Result;
use clap::Parser;
use hermeship::cli::{Cli, Commands, ConfigCommand, HermesCommands, ReleaseCommands};

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
        Commands::Start { port } => print_placeholder("start", port),
        Commands::Status => print_placeholder("status", ()),
        Commands::Send { channel, message } => print_placeholder("send", (channel, message)),
        Commands::Emit(args) => print_placeholder("emit", (args.event, args.payload)),
        Commands::Explain(args) => print_placeholder("explain", (args.event, args.payload)),
        Commands::Config { command } => match command.unwrap_or(ConfigCommand::Show) {
            ConfigCommand::Show => print_placeholder("config show", ()),
            ConfigCommand::Path => {
                println!("{}", config_path.display());
                Ok(())
            }
            ConfigCommand::Verify => print_placeholder("config verify", ()),
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
