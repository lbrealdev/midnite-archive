use crate::types::ListFile;
use crate::yt_dlp;
use anyhow::{Context, Result, bail};
use std::fs;
use std::path::Path;

pub fn execute(list_file: &Path) -> Result<()> {
    tracing::info!("Downloading comments from: {}", list_file.display());

    yt_dlp::check_available()?;

    if !list_file.exists() {
        bail!("The file {:?} was not found!", list_file);
    }

    // Use strongly-typed ListFile
    let list_file = ListFile::from_path(list_file)
        .with_context(|| format!("Failed to process list file: {:?}", list_file))?;

    tracing::info!("YouTube channel list path: {}", list_file.path.display());
    tracing::info!("Detected channel: @{}", list_file.channel.name);

    let comments_dir = list_file.channel.comments_dir();

    tracing::debug!("Checking if {} directory exists...", list_file.channel.name);

    if !list_file.channel.base_dir().exists() {
        fs::create_dir_all(&comments_dir)
            .with_context(|| format!("Failed to create directory: {:?}", comments_dir))?;
        tracing::info!("Directory created: {}", list_file.channel.name);
    } else {
        fs::create_dir_all(&comments_dir)
            .with_context(|| format!("Failed to create directory: {:?}", comments_dir))?;
        tracing::info!(
            "{} directory already exists, creating comments directory...",
            list_file.channel.name
        );
    }

    let file_stem = list_file
        .path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    tracing::info!("Downloading from {} list...", file_stem);

    // Optionally read videos for better tracking
    match list_file.read_videos() {
        Ok((videos, unparseable)) => {
            if !unparseable.is_empty() {
                tracing::warn!("Skipped {} unparseable lines", unparseable.len());
            }
            tracing::info!("Found {} videos to process comments for", videos.len());
            for video in &videos[..5.min(videos.len())] {
                tracing::info!("  - {}", video);
            }
            if videos.len() > 5 {
                tracing::info!("  ... and {} more", videos.len() - 5);
            }
        }
        Err(e) => {
            tracing::warn!("Could not parse video list: {}", e);
        }
    }

    let output_dir_full_path = fs::canonicalize(&comments_dir)
        .with_context(|| "Failed to resolve output directory path")?;

    tracing::info!("Starting comments download...");
    yt_dlp::download_comments(&list_file.path, &output_dir_full_path)
        .with_context(|| "Comments download failed")?;

    tracing::info!("Comments download completed successfully");

    // Keep final output visible
    println!("✓ Done!");

    Ok(())
}
