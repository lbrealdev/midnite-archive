use crate::types::ListFile;
use crate::yt_dlp;
use anyhow::{bail, Context, Result};
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

    // Read videos and count totals for tracking stats
    let total_videos = match list_file.read_videos() {
        Ok(videos) => {
            let count = videos.len();
            tracing::info!("Found {} videos in list", count);
            for video in &videos[..5.min(videos.len())] {
                tracing::info!("  - {}", video);
            }
            if videos.len() > 5 {
                tracing::info!("  ... and {} more", videos.len() - 5);
            }
            count
        }
        Err(e) => {
            tracing::warn!("Could not parse video list: {}", e);
            0
        }
    };

    // Check archive file for download tracking stats
    let archive_file = download_dir
        .join(".archive")
        .join(
            list_file
                .path
                .file_stem()
                .unwrap_or(std::ffi::OsStr::new("archive")),
        )
        .with_extension("archive");

    let downloaded_count = if archive_file.exists() {
        match std::fs::read_to_string(&archive_file) {
            Ok(content) => content.lines().filter(|line| !line.is_empty()).count(),
            Err(_) => 0,
        }
    } else {
        0
    };

    let remaining = total_videos.saturating_sub(downloaded_count);

    // Display tracking statistics
    if total_videos > 0 {
        println!("📊 Download Statistics:");
        println!("   Total videos: {}", total_videos);
        println!("   Already downloaded: {}", downloaded_count);
        println!("   Remaining to download: {}", remaining);
        if remaining > 0 {
            println!();
        }
    }

    tracing::info!("Starting download...");
    yt_dlp::download_from_file(
        &list_file.path,
        &download_dir,
        total_videos,
        downloaded_count,
    )
    .with_context(|| "Download failed")?;

    // Check archive again after download
    let new_downloaded_count = if archive_file.exists() {
        match std::fs::read_to_string(&archive_file) {
            Ok(content) => content.lines().filter(|line| !line.is_empty()).count(),
            Err(_) => downloaded_count,
        }
    } else {
        downloaded_count
    };

    let newly_downloaded = new_downloaded_count.saturating_sub(downloaded_count);

    if total_videos > 0 && newly_downloaded > 0 {
        println!(
            "✓ Downloaded {} new video(s) this session",
            newly_downloaded
        );
        println!(
            "   Progress: {}/{} videos complete",
            new_downloaded_count, total_videos
        );
    }

    Ok(())
}
