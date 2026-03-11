use crate::types::ListFile;
use crate::yt_dlp;
use anyhow::{Context, Result, bail};
use std::fs;
use std::path::Path;

pub fn execute(input: &str) -> Result<()> {
    tracing::info!("Starting download: {}", input);

    yt_dlp::check_available()?;
    yt_dlp::check_deno_available()?;

    // Check if input is a URL (simple check without regex)
    if input.starts_with("http://") || input.starts_with("https://") {
        handle_url_download(input)?;
    } else if Path::new(input).exists() {
        handle_file_download(input)?;
    } else {
        bail!(
            "Input '{}' is neither a valid URL nor an existing file.",
            input
        );
    }

    tracing::info!("Download completed successfully");

    // Keep final output visible
    println!("✓ Done!");

    Ok(())
}

fn handle_url_download(url: &str) -> Result<()> {
    tracing::info!("Input type: Single YouTube URL");
    tracing::info!("YouTube URL: {}", url);

    let download_dir = std::path::PathBuf::from("downloads");

    tracing::debug!("Preparing download directory...");
    fs::create_dir_all(&download_dir)
        .with_context(|| format!("Failed to create directory: {:?}", download_dir))?;
    tracing::info!("Download directory ready: {}", download_dir.display());

    tracing::info!("Starting download...");
    yt_dlp::download_from_url(url, &download_dir).with_context(|| "Download failed")?;

    Ok(())
}

fn handle_file_download(input: &str) -> Result<()> {
    tracing::info!("Input type: YouTube URL list file");

    // Use strongly-typed ListFile
    let list_file = ListFile::from_path(input)
        .with_context(|| format!("Failed to process list file: {}", input))?;

    tracing::info!("YouTube channel list path: {}", list_file.path.display());
    tracing::info!("Detected channel: @{}", list_file.channel.name);

    let download_dir = list_file.channel.videos_dir();

    tracing::debug!("Preparing download directory...");
    fs::create_dir_all(&download_dir)
        .with_context(|| format!("Failed to create directory: {:?}", download_dir))?;
    tracing::info!("Download directory ready: {}", download_dir.display());

    let file_stem = list_file
        .path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    tracing::info!("Downloading from {} list...", file_stem);

    // Optionally read videos for logging/validation
    match list_file.read_videos() {
        Ok(videos) => {
            tracing::info!("Found {} videos in list", videos.len());
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

    tracing::info!("Starting download...");
    yt_dlp::download_from_file(&list_file.path, &download_dir)
        .with_context(|| "Download failed")?;

    Ok(())
}
