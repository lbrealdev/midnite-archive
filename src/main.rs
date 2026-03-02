#![warn(clippy::all)]

use anyhow::Result;
use clap::Parser;
use midnite_archive::{Cli, Commands};
use std::process;

fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate { channel } => {
            midnite_archive::commands::generate(&channel)?;
        }
        Commands::Download { input } => {
            midnite_archive::commands::download(&input)?;
        }
        Commands::Comments { list_file } => {
            midnite_archive::commands::comments(&list_file)?;
        }
        Commands::Rename {
            directory,
            recursive,
            dry_run,
            verbose,
            extensions,
        } => {
            midnite_archive::commands::rename(
                &directory,
                recursive,
                dry_run,
                verbose,
                &extensions,
            )?;
        }
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
