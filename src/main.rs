#![warn(clippy::all)]

use anyhow::Result;
use clap::Parser;
use midnite_archive::cli::{Cli, Commands};
use std::process;
use tracing_subscriber::EnvFilter;

fn setup_logging(verbose: u8) {
    let level = match verbose {
        0 => "off",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    setup_logging(cli.verbose);

    tracing::info!("Starting midnite-archive");

    match cli.command {
        Commands::Generate { channel, filter } => {
            tracing::debug!("Executing generate command for channel: {}", channel);
            midnite_archive::cli::generate(&channel, filter.as_deref())?;
        }
        Commands::Download { input } => {
            tracing::debug!("Executing download command with input: {}", input);
            midnite_archive::cli::download(&input)?;
        }
        Commands::Comments { list_file } => {
            tracing::debug!("Executing comments command with file: {:?}", list_file);
            midnite_archive::cli::comments(&list_file)?;
        }
        Commands::Rename {
            directory,
            recursive,
            dry_run,
            verbose,
            extensions,
        } => {
            tracing::debug!(
                "Executing rename command: directory={:?}, recursive={}, dry_run={}",
                directory,
                recursive,
                dry_run
            );
            midnite_archive::cli::rename(&directory, recursive, dry_run, verbose, &extensions)?;
        }
    }

    tracing::info!("midnite-archive completed successfully");
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        tracing::error!("Application error: {}", e);
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
