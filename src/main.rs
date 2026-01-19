//! Spoons API - A REST API for music data.

use spoons_api::cli::{self, Commands, StartArgs};
use spoons_api::config::{self, AppConfig};
use spoons_api::logging;
use spoons_api::server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = cli::parse();

    match args.command {
        Commands::Start(start_args) => run_server(start_args).await,
    }
}

async fn run_server(args: StartArgs) -> anyhow::Result<()> {
    let mut config = load_config(&args)?;

    // Apply CLI overrides
    if let Some(port) = args.port {
        config.server.port = port;
    }

    logging::init(&config.logging);

    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        port = config.server.port,
        "Starting Spoons API"
    );

    server::run(&config).await?;

    Ok(())
}

fn load_config(args: &StartArgs) -> anyhow::Result<AppConfig> {
    match &args.config {
        Some(path) => {
            tracing::info!(path = %path.display(), "Loading configuration");
            config::load_config(path).map_err(Into::into)
        }
        None => {
            tracing::info!("Using default configuration");
            Ok(AppConfig::default())
        }
    }
}
