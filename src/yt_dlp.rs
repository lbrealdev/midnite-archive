use crate::types::{Channel, Video, VideoId};
use anyhow::{Context, Result, bail};
use std::path::Path;
use std::process::Command;

pub fn check_available() -> Result<()> {
    match Command::new("yt-dlp").arg("--version").output() {
        Ok(output) if output.status.success() => Ok(()),
        _ => bail!("yt-dlp not found. Please install it first."),
    }
}

pub fn check_deno_available() -> Result<()> {
    match Command::new("deno").arg("--version").output() {
        Ok(output) if output.status.success() => Ok(()),
        _ => bail!("deno not found. Please install it first."),
    }
}

/// Generate a channel list and return structured Video data
pub fn generate_channel_list(
    channel: &Channel,
    output_file: &Path,
    filter: Option<&str>,
) -> Result<Vec<Video>> {
    let channel_url = channel.url();

    // Build command with optional filter
    let mut cmd = Command::new("yt-dlp");
    cmd.args(["--flat-playlist", "--print", "%(title)s-%(id)s"]);

    // Add match-filter if provided
    if let Some(pattern) = filter {
        cmd.args(["--match-title", pattern]);
    }

    let output = cmd
        .arg(&channel_url)
        .output()
        .with_context(|| format!("Failed to run yt-dlp for channel: {}", channel.name))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "yt-dlp failed with exit code: {:?}\n{}",
            output.status.code(),
            stderr
        );
    }

    // Parse output and create structured Video objects
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut videos = Vec::new();

    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Parse "title-video_id" format
        // YouTube IDs are always 11 characters and can contain hyphens
        // So we extract the last 11 characters as the ID
        if line.len() >= 12 {
            // Need at least 1 char for title + hyphen + 11 chars for ID
            let (title_part, id_part) = line.split_at(line.len() - 11);

            // Remove trailing hyphen from title if present
            let title = title_part.strip_suffix('-').unwrap_or(title_part);

            match VideoId::new(id_part) {
                Ok(id) => {
                    let video = Video::new(id, title, channel.clone());
                    videos.push(video);
                }
                Err(e) => {
                    tracing::debug!("Failed to parse video ID '{}': {}", id_part, e);
                }
            }
        } else {
            tracing::debug!("Line too short to contain valid video ID: {}", line);
        }
    }

    tracing::info!("Successfully parsed {} videos from output", videos.len());

    // Also write to file for backward compatibility
    std::fs::write(output_file, &output.stdout)
        .with_context(|| format!("Failed to write output file: {:?}", output_file))?;

    Ok(videos)
}

pub fn download_from_url(url: &str, output_dir: &Path) -> Result<()> {
    let deno_path = which::which("deno").context("Failed to find deno executable path")?;

    // Create archive path for single URL downloads: <output_dir>/.archive/downloads.archive
    let archive_dir = output_dir.join(".archive");
    if let Err(e) = std::fs::create_dir_all(&archive_dir) {
        tracing::warn!("Failed to create archive directory: {}", e);
    }

    let archive_file = archive_dir.join("downloads.archive");
    tracing::info!("Using download archive: {}", archive_file.display());

    let status = Command::new("yt-dlp")
        .args([
            "-cw",
            "-o",
            "%(title)s-%(id)s.%(ext)s",
            "--embed-thumbnail",
            "--write-description",
            "--embed-metadata",
            "--no-colors",
            "--remote-components",
            "ejs:npm",
            "--js-runtimes",
        ])
        .arg(format!("deno:{}", deno_path.display()))
        .arg("--download-archive")
        .arg(&archive_file)
        .args(["-P", &output_dir.to_string_lossy()])
        .arg(url)
        .status()
        .with_context(|| format!("Failed to run yt-dlp for URL: {}", url))?;

    if !status.success() {
        bail!("yt-dlp download failed with exit code: {:?}", status.code());
    }

    Ok(())
}

pub fn download_from_file(
    list_file: &Path,
    output_dir: &Path,
    _total_videos: usize,
    _downloaded_count: usize,
) -> Result<()> {
    let deno_path = which::which("deno").context("Failed to find deno executable path")?;

    // Create archive path: <channel>/.archive/<list-file-name>.archive
    let archive_dir = output_dir.join(".archive");
    if let Err(e) = std::fs::create_dir_all(&archive_dir) {
        tracing::warn!("Failed to create archive directory: {}", e);
    }

    let archive_file = archive_dir
        .join(
            list_file
                .file_stem()
                .unwrap_or(std::ffi::OsStr::new("archive")),
        )
        .with_extension("archive");

    tracing::info!("Using download archive: {}", archive_file.display());

    let status = Command::new("yt-dlp")
        .args([
            "-cw",
            "-o",
            "%(title)s-%(id)s.%(ext)s",
            "--embed-thumbnail",
            "--write-description",
            "--embed-metadata",
            "--no-colors",
            "--remote-components",
            "ejs:npm",
            "--js-runtimes",
        ])
        .arg(format!("deno:{}", deno_path.display()))
        .arg("--download-archive")
        .arg(&archive_file)
        .args(["-P", &output_dir.to_string_lossy()])
        .args(["-a", &list_file.to_string_lossy()])
        .status()
        .with_context(|| format!("Failed to run yt-dlp for file: {:?}", list_file))?;

    if !status.success() {
        bail!("yt-dlp download failed with exit code: {:?}", status.code());
    }

    Ok(())
}

pub fn download_comments(list_file: &Path, output_dir: &Path) -> Result<()> {
    let status = Command::new("yt-dlp")
        .args(["-o", "%(id)s.comments.json"])
        .args(["-P", &output_dir.to_string_lossy()])
        .args(["-a", &list_file.to_string_lossy()])
        .args(["--write-comments", "--skip-download", "--no-colors"])
        .status()
        .with_context(|| format!("Failed to run yt-dlp for comments: {:?}", list_file))?;

    if !status.success() {
        bail!(
            "yt-dlp comments download failed with exit code: {:?}",
            status.code()
        );
    }

    Ok(())
}

/// Download comments for a specific video
pub fn download_comments_for_video(video: &Video, output_dir: &Path) -> Result<()> {
    let status = Command::new("yt-dlp")
        .args(["-o", "%(id)s.comments.json"])
        .args(["-P", &output_dir.to_string_lossy()])
        .args(["--write-comments", "--skip-download", "--no-colors"])
        .arg(video.url())
        .status()
        .with_context(|| format!("Failed to run yt-dlp for video: {}", video.id))?;

    if !status.success() {
        bail!(
            "yt-dlp comments download failed with exit code: {:?}",
            status.code()
        );
    }

    Ok(())
}
