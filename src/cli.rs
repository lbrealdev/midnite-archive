use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "midnite")]
#[command(about = "YouTube archiving tool for Midnite/Akae Beka content", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate video list from a YouTube channel
    Generate {
        /// YouTube channel (e.g., @severo12 or severo12)
        channel: String,
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
