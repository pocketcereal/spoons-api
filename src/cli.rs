//! Command-line interface definitions.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Spoons API - A REST API for music data.
#[derive(Debug, Parser)]
#[command(name = "spoons-api")]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// Available commands.
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Start the API server.
    Start(StartArgs),
}

/// Arguments for the start command.
#[derive(Debug, Parser)]
pub struct StartArgs {
    /// Path to the configuration file.
    #[arg(short, long, env = "SPOONS_CONFIG")]
    pub config: Option<PathBuf>,

    /// Override the server port.
    #[arg(short, long, env = "SPOONS_PORT")]
    pub port: Option<u16>,
}

/// Parse command-line arguments.
pub fn parse() -> Cli {
    Cli::parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_start_command() {
        let cli = Cli::try_parse_from(["spoons-api", "start"]).unwrap();
        match cli.command {
            Commands::Start(args) => {
                assert!(args.config.is_none());
                assert!(args.port.is_none());
            }
        }
    }

    #[test]
    fn test_cli_start_with_config() {
        let cli = Cli::try_parse_from(["spoons-api", "start", "--config", "config.yaml"]).unwrap();
        match cli.command {
            Commands::Start(args) => {
                assert_eq!(args.config, Some(PathBuf::from("config.yaml")));
            }
        }
    }
}
