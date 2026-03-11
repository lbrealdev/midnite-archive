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
pub fn generate_channel_list(channel: &Channel, output_file: &Path) -> Result<Vec<Video>> {
    let channel_url = channel.url();

    let output = Command::new("yt-dlp")
        .args(["--flat-playlist", "--print", "%(title)s-%(id)s"])
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
        if let Some((title, id_str)) = line.rsplit_once('-')
            && let Ok(id) = VideoId::new(id_str)
        {
            let video = Video::new(id, title, channel.clone());
            videos.push(video);
        }
    }

    // Also write to file for backward compatibility
    std::fs::write(output_file, &output.stdout)
        .with_context(|| format!("Failed to write output file: {:?}", output_file))?;

    Ok(videos)
}

pub fn download_from_url(url: &str, output_dir: &Path) -> Result<()> {
    let deno_path = which::which("deno").context("Failed to find deno executable path")?;

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
        .args(["-P", &output_dir.to_string_lossy()])
        .arg(url)
        .status()
        .with_context(|| format!("Failed to run yt-dlp for URL: {}", url))?;

    if !status.success() {
        bail!("yt-dlp download failed with exit code: {:?}", status.code());
    }

    Ok(())
}

pub fn download_from_file(list_file: &Path, output_dir: &Path) -> Result<()> {
    let deno_path = which::which("deno").context("Failed to find deno executable path")?;

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
