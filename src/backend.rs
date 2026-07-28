//! YtDlp backend boundary.
//!
//! Abstracts over the mechanism used to invoke `yt-dlp` so that the CLI can
//! migrate from raw `std::process::Command` to the async `ytd-rs` wrapper
//! (Issue #66) without changing command behaviour.
//!
//! Two implementations are provided:
//! - [`CommandBackend`]: legacy sync `std::process::Command` calls, kept as a
//!   testable fallback. It wraps the existing free functions in [`crate::yt_dlp`]
//!   via `tokio::task::spawn_blocking`.
//! - [`YtdRsBackend`]: new async implementation backed by the `ytd-rs` crate.
//!
//! Probe helpers ([`crate::yt_dlp::ToolProbe`], `probe_*`, `check_*`) stay sync
//! and are intentionally NOT part of this trait.

use crate::types::{Channel, Video};
use crate::yt_dlp;
use anyhow::{Context, Result};
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use ytd_rs::YtDlp;

/// Backend abstraction over the subset of `yt-dlp` operations the CLI uses.
///
/// All methods are async so that subsequent migration issues (#67-#71) can
/// swap implementations per-command without touching call sites. Sync commands
/// (doctor, rename) do not use this trait and remain synchronous.
#[async_trait]
pub trait YtDlpBackend: Send + Sync {
    /// Flat-playlist metadata fetch, returning parsed [`Video`]s and writing
    /// the raw title-id list to `output_file`.
    async fn generate_channel_list(
        &self,
        channel: &Channel,
        output_file: &Path,
        filter: Option<&str>,
    ) -> Result<Vec<Video>>;

    /// Download a single URL into `output_dir` using the EJS/Deno archive flow.
    async fn download_from_url(&self, url: &str, output_dir: &Path) -> Result<()>;

    /// Download every URL listed in `list_file` into `output_dir`.
    async fn download_from_file(
        &self,
        list_file: &Path,
        output_dir: &Path,
        total_videos: usize,
        downloaded_count: usize,
    ) -> Result<()>;

    /// Download comments for every URL listed in `list_file` into `output_dir`.
    async fn download_comments(&self, list_file: &Path, output_dir: &Path) -> Result<()>;

    /// Download comments for a single [`Video`] into `output_dir`.
    async fn download_comments_for_video(&self, video: &Video, output_dir: &Path) -> Result<()>;
}

/// Legacy backend that shells out to `yt-dlp` via `std::process::Command`.
///
/// Wraps the synchronous free functions in [`crate::yt_dlp`], offloading the
/// blocking work to `tokio::task::spawn_blocking` so the trait stays async.
#[derive(Debug, Default, Clone, Copy)]
pub struct CommandBackend;

#[async_trait]
impl YtDlpBackend for CommandBackend {
    async fn generate_channel_list(
        &self,
        channel: &Channel,
        output_file: &Path,
        filter: Option<&str>,
    ) -> Result<Vec<Video>> {
        let channel = channel.clone();
        let output_file = output_file.to_path_buf();
        let filter = filter.map(|s| s.to_string());
        tokio::task::spawn_blocking(move || {
            yt_dlp::generate_channel_list(&channel, &output_file, filter.as_deref())
        })
        .await
        .context("generate_channel_list task panicked")?
    }

    async fn download_from_url(&self, url: &str, output_dir: &Path) -> Result<()> {
        let url = url.to_string();
        let output_dir = output_dir.to_path_buf();
        tokio::task::spawn_blocking(move || yt_dlp::download_from_url(&url, &output_dir))
            .await
            .context("download_from_url task panicked")?
    }

    async fn download_from_file(
        &self,
        list_file: &Path,
        output_dir: &Path,
        total_videos: usize,
        downloaded_count: usize,
    ) -> Result<()> {
        let list_file = list_file.to_path_buf();
        let output_dir = output_dir.to_path_buf();
        tokio::task::spawn_blocking(move || {
            yt_dlp::download_from_file(&list_file, &output_dir, total_videos, downloaded_count)
        })
        .await
        .context("download_from_file task panicked")?
    }

    async fn download_comments(&self, list_file: &Path, output_dir: &Path) -> Result<()> {
        let list_file = list_file.to_path_buf();
        let output_dir = output_dir.to_path_buf();
        tokio::task::spawn_blocking(move || yt_dlp::download_comments(&list_file, &output_dir))
            .await
            .context("download_comments task panicked")?
    }

    async fn download_comments_for_video(&self, video: &Video, output_dir: &Path) -> Result<()> {
        let video = video.clone();
        let output_dir = output_dir.to_path_buf();
        tokio::task::spawn_blocking(move || {
            yt_dlp::download_comments_for_video(&video, &output_dir)
        })
        .await
        .context("download_comments_for_video task panicked")?
    }
}

/// Async backend backed by the `ytd-rs` crate.
#[derive(Debug, Default, Clone, Copy)]
pub struct YtdRsBackend;

impl YtdRsBackend {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl YtDlpBackend for YtdRsBackend {
    async fn generate_channel_list(
        &self,
        channel: &Channel,
        output_file: &Path,
        filter: Option<&str>,
    ) -> Result<Vec<Video>> {
        let channel_url = channel.url();

        let mut ytd = YtDlp::new(&channel_url)
            .arg("--flat-playlist")
            .arg_with("--print", "%(title)s-%(id)s");
        if let Some(pattern) = filter {
            ytd = ytd.arg_with("--match-title", pattern);
        }

        let result = ytd
            .download()
            .await
            .with_context(|| format!("Failed to run yt-dlp for channel: {}", channel.name))?;

        let stdout = result.output().to_string();
        let videos = yt_dlp::parse_channel_list_output(&stdout, channel);

        std::fs::write(output_file, &stdout)
            .with_context(|| format!("Failed to write output file: {:?}", output_file))?;

        Ok(videos)
    }

    async fn download_from_url(&self, url: &str, output_dir: &Path) -> Result<()> {
        let deno_path = which::which("deno").context("Failed to find deno executable path")?;

        let archive_file = prepare_archive_file(output_dir, "downloads.archive")?;

        let ytd = build_download_builder(url, &deno_path, &archive_file, output_dir);
        ytd.download()
            .await
            .with_context(|| format!("Failed to run yt-dlp for URL: {}", url))?;
        Ok(())
    }

    async fn download_from_file(
        &self,
        list_file: &Path,
        output_dir: &Path,
        _total_videos: usize,
        _downloaded_count: usize,
    ) -> Result<()> {
        let deno_path = which::which("deno").context("Failed to find deno executable path")?;

        let archive_name = list_file
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("archive");
        let archive_file = prepare_archive_file(output_dir, &format!("{archive_name}.archive"))?;

        // No positional URL: yt-dlp reads URLs from `-a <list_file>`.
        let ytd = build_download_builder("", &deno_path, &archive_file, output_dir)
            .arg_with("-a", list_file.to_string_lossy().to_string());
        ytd.download()
            .await
            .with_context(|| format!("Failed to run yt-dlp for file: {:?}", list_file))?;
        Ok(())
    }

    async fn download_comments(&self, list_file: &Path, output_dir: &Path) -> Result<()> {
        let ytd = YtDlp::new_multiple(Vec::new())
            .arg_with("-o", "%(id)s.comments.json")
            .arg_with("-P", output_dir.to_string_lossy().to_string())
            .arg_with("-a", list_file.to_string_lossy().to_string())
            .arg("--write-comments")
            .arg("--skip-download")
            .arg("--no-colors");
        ytd.download()
            .await
            .with_context(|| format!("Failed to run yt-dlp for comments: {:?}", list_file))?;
        Ok(())
    }

    async fn download_comments_for_video(&self, video: &Video, output_dir: &Path) -> Result<()> {
        let ytd = YtDlp::new(video.url())
            .arg_with("-o", "%(id)s.comments.json")
            .arg_with("-P", output_dir.to_string_lossy().to_string())
            .arg("--write-comments")
            .arg("--skip-download")
            .arg("--no-colors");
        ytd.download()
            .await
            .with_context(|| format!("Failed to run yt-dlp for video: {}", video.id))?;
        Ok(())
    }
}

/// Create `<output_dir>/.archive/<name>`, logging (not failing) on mkdir error.
fn prepare_archive_file(output_dir: &Path, name: &str) -> Result<PathBuf> {
    let archive_dir = output_dir.join(".archive");
    if let Err(e) = std::fs::create_dir_all(&archive_dir) {
        tracing::warn!("Failed to create archive directory: {}", e);
    }
    let archive_file = archive_dir.join(name);
    tracing::info!("Using download archive: {}", archive_file.display());
    Ok(archive_file)
}

/// Shared yt-dlp arg set for the EJS/Deno download flow (url + file variants).
fn build_download_builder(
    url: &str,
    deno_path: &Path,
    archive_file: &Path,
    output_dir: &Path,
) -> YtDlp {
    YtDlp::new(url)
        .arg("-cw")
        .arg_with("-o", "%(title)s-%(id)s.%(ext)s")
        .arg("--embed-thumbnail")
        .arg("--write-description")
        .arg("--embed-metadata")
        .arg("--no-colors")
        .arg("--remote-components")
        .arg("ejs:npm")
        .arg_with(
            "--js-runtimes",
            format!("deno:{}", deno_path.display()),
        )
        .arg_with(
            "--download-archive",
            archive_file.to_string_lossy().to_string(),
        )
        .arg_with("-P", output_dir.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backends_are_object_safe() {
        let backends: Vec<Box<dyn YtDlpBackend>> = vec![
            Box::new(CommandBackend),
            Box::new(YtdRsBackend::new()),
        ];
        assert_eq!(backends.len(), 2);
    }

    #[test]
    fn command_backend_default_constructs() {
        let _b: Box<dyn YtDlpBackend> = Box::new(CommandBackend);
    }

    #[test]
    fn ytdrs_backend_default_constructs() {
        let _b: Box<dyn YtDlpBackend> = Box::new(YtdRsBackend);
    }
}
