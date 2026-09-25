use crate::backend::{count_ids_in_archive, read_archive_lines};
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

    // Read videos and count totals for tracking stats
    let videos = match list_file.read_videos() {
        Ok((videos, unparseable)) => {
            let count = videos.len();
            if !unparseable.is_empty() {
                tracing::warn!("Skipped {} unparseable lines", unparseable.len());
            }
            tracing::info!("Found {} videos in list", count);
            for video in &videos[..5.min(videos.len())] {
                tracing::info!("  - {}", video);
            }
            if videos.len() > 5 {
                tracing::info!("  ... and {} more", videos.len() - 5);
            }
            videos
        }
        Err(e) => {
            tracing::warn!("Could not parse video list: {}", e);
            Vec::new()
        }
    };

    let total_videos = videos.len();
    let list_ids: Vec<&str> = videos.iter().map(|video| video.id.as_ref()).collect();
    let archive_dir = download_dir.join(".archive");

    // Already downloaded = ids from this list that appear in the channel archive
    // (union of every *.archive), not the raw line count of one list-stem file.
    let already_downloaded = if total_videos > 0 {
        let lines = read_archive_lines(&archive_dir).with_context(|| {
            format!(
                "Failed to read download archive in {}",
                download_dir.display()
            )
        })?;
        let already = count_ids_in_archive(list_ids.iter().copied(), &lines);
        let remaining = total_videos.saturating_sub(already);
        println!("📊 Download Statistics:");
        println!("   Total videos: {}", total_videos);
        println!("   Already downloaded: {}", already);
        println!("   Remaining to download: {}", remaining);
        if remaining > 0 {
            println!();
        }
        already
    } else {
        0
    };

    tracing::info!("Starting download...");
    yt_dlp::download_from_file(
        &list_file.path,
        &download_dir,
        total_videos,
        already_downloaded,
    )
    .with_context(|| "Download failed")?;

    if total_videos > 0 {
        let lines = read_archive_lines(&archive_dir).with_context(|| {
            format!(
                "Failed to read download archive in {}",
                download_dir.display()
            )
        })?;
        let downloaded_after = count_ids_in_archive(list_ids.iter().copied(), &lines);
        let newly_downloaded = downloaded_after.saturating_sub(already_downloaded);
        if newly_downloaded > 0 {
            println!(
                "✓ Downloaded {} new video(s) this session",
                newly_downloaded
            );
            println!(
                "   Progress: {}/{} videos complete",
                downloaded_after, total_videos
            );
        }
    }

    Ok(())
}
