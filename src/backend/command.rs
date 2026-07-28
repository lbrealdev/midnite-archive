//! Legacy `std::process::Command` implementations of yt-dlp operations.
//!
//! These are the raw command paths used by the sync facade when
//! `ytd-rs-backend` is disabled. [`CommandBackend`] (feature-gated) wraps them
//! for the async trait without calling back into the facade.

use crate::backend::{ensure_archive_parent, list_archive_path, url_archive_path};
use crate::types::{Channel, Video};
use crate::yt_dlp::parse_channel_list_output;
use anyhow::{Context, Result, bail};
use std::path::Path;
use std::process::Command;

#[cfg(feature = "ytd-rs-backend")]
use crate::backend::YtDlpBackend;
#[cfg(feature = "ytd-rs-backend")]
use async_trait::async_trait;

/// Generate a channel list and return structured Video data.
pub fn generate_channel_list(
    channel: &Channel,
    output_file: &Path,
    filter: Option<&str>,
) -> Result<Vec<Video>> {
    let channel_url = channel.url();

    let mut cmd = Command::new("yt-dlp");
    cmd.args(["--flat-playlist", "--print", "%(title)s-%(id)s"]);

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

    let stdout = String::from_utf8_lossy(&output.stdout);
    let videos = parse_channel_list_output(&stdout, channel);

    std::fs::write(output_file, &output.stdout)
        .with_context(|| format!("Failed to write output file: {:?}", output_file))?;

    Ok(videos)
}

pub fn download_from_url(url: &str, output_dir: &Path) -> Result<()> {
    let deno_path = which::which("deno").context("Failed to find deno executable path")?;

    let archive_file = url_archive_path(output_dir);
    ensure_archive_parent(&archive_file);
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

    let archive_file = list_archive_path(output_dir, list_file);
    ensure_archive_parent(&archive_file);
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

/// Download comments for a specific video.
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

/// Async trait wrapper around the sync Command implementations.
///
/// Calls [`command`] functions directly (not the [`crate::yt_dlp`] facade) so
/// the facade can select this backend without recursing.
#[cfg(feature = "ytd-rs-backend")]
#[derive(Debug, Default, Clone, Copy)]
pub struct CommandBackend;

#[cfg(feature = "ytd-rs-backend")]
#[async_trait]
impl YtDlpBackend for CommandBackend {
    async fn generate_channel_list(
        &self,
        channel: &Channel,
        output_file: &Path,
        filter: Option<&str>,
    ) -> Result<Vec<Video>> {
        generate_channel_list(channel, output_file, filter)
    }

    async fn download_from_url(&self, url: &str, output_dir: &Path) -> Result<()> {
        download_from_url(url, output_dir)
    }

    async fn download_from_file(
        &self,
        list_file: &Path,
        output_dir: &Path,
        total_videos: usize,
        downloaded_count: usize,
    ) -> Result<()> {
        download_from_file(list_file, output_dir, total_videos, downloaded_count)
    }

    async fn download_comments(&self, list_file: &Path, output_dir: &Path) -> Result<()> {
        download_comments(list_file, output_dir)
    }

    async fn download_comments_for_video(&self, video: &Video, output_dir: &Path) -> Result<()> {
        download_comments_for_video(video, output_dir)
    }
}
