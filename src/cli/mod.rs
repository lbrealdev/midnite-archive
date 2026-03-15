mod comments;
mod download;
mod generate;
mod rename;

pub use comments::execute as comments;
pub use download::execute as download;
pub use generate::execute as generate;
pub use rename::execute as rename;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = env!("CARGO_PKG_NAME"), version = env!("CARGO_PKG_VERSION"), about = ABOUT, long_about = LONG_ABOUT)]
pub struct Cli {
    /// Increase verbosity level (can be used multiple times, e.g., -v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate video list from a YouTube channel (name or URL)
    Generate {
        /// YouTube channel (e.g., @severo12, severo12, or channel URL)
        channel: String,
        /// Filter videos by title pattern (e.g., --filter "live" or --filter "title*=live")
        #[arg(short, long)]
        filter: Option<String>,
    },
    /// Download videos from a list file or URL
    Download {
        /// Input file (URL list) or single YouTube URL
        input: String,
    },
    /// Download comments from a video list
    Comments {
        /// File containing YouTube URLs
        list_file: PathBuf,
    },
    /// Rename video files (sanitize special characters)
    Rename {
        /// Directory containing video files
        directory: PathBuf,
        /// Process subdirectories recursively
        #[arg(short, long)]
        recursive: bool,
        /// Preview changes without renaming
        #[arg(short, long)]
        dry_run: bool,
        /// Show each rename operation
        #[arg(short, long)]
        verbose: bool,
        /// File extensions to process
        #[arg(short = 'e', long, default_values = ["mkv", "mp4", "description"])]
        extensions: Vec<String>,
    },
}

const ABOUT: &str = "Midnite Archive CLI";
const LONG_ABOUT: &str = "YouTube archiving tool for Midnite/Akae Beka content";
