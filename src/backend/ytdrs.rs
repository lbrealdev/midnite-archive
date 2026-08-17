//! Async backend backed by the `ytd-rs` crate.
//!
//! Compiled only with the `ytd-rs-backend` feature.
//!
//! Media downloads stream yt-dlp stdout via [`YtDlp::download_process`] and
//! classify lines into [`crate::backend::events::YtDlpEvent`]. Default CLI
//! stays quiet (`-v` surfaces progress as `tracing` info).
//!
//! **Cancellation:** `ytd-rs` 0.2.1 `YtDlpChild` exposes only `next_line` /
//! `wait`. There is no public kill/pid API and the child is **not** killed on
//! drop. TTY SIGINT may still stop yt-dlp via the process group. Programmatic
//! cancel is blocked on upstream. stderr is piped but unread on this path
//! (pipe-fill risk; failed `wait()` does not include real stderr).

use crate::backend::events::{YtDlpEvent, classify_yt_dlp_line};
use crate::backend::{YtDlpBackend, ensure_archive_parent, list_archive_path, url_archive_path};
use crate::types::{Channel, Video};
use crate::yt_dlp::parse_channel_list_output;
use anyhow::{Context, Result};
use async_trait::async_trait;
use std::path::Path;
use ytd_rs::YtDlp;

/// Async backend backed by the `ytd-rs` crate.
#[derive(Debug, Default, Clone, Copy)]
pub struct YtdRsBackend;

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
        let videos = parse_channel_list_output(&stdout, channel);

        std::fs::write(output_file, &stdout)
            .with_context(|| format!("Failed to write output file: {:?}", output_file))?;

        Ok(videos)
    }

    async fn download_from_url(&self, url: &str, output_dir: &Path) -> Result<()> {
        let deno_path = which::which("deno").context("Failed to find deno executable path")?;

        let archive_file = url_archive_path(output_dir);
        ensure_archive_parent(&archive_file);
        tracing::info!("Using download archive: {}", archive_file.display());

        let ytd = build_download_builder(url, &deno_path, &archive_file, output_dir);
        run_download_process(ytd)
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

        let archive_file = list_archive_path(output_dir, list_file);
        ensure_archive_parent(&archive_file);
        tracing::info!("Using download archive: {}", archive_file.display());

        // yt-dlp reads URLs from `-a <list_file>`; start with an empty link list
        // (not YtDlp::new("")) so we never pass a spurious empty positional URL.
        let ytd = apply_download_args(
            YtDlp::new_multiple(Vec::new()),
            &deno_path,
            &archive_file,
            output_dir,
        )
        .arg_with("-a", list_file.to_string_lossy().to_string());
        run_download_process(ytd)
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

/// Stream yt-dlp stdout, classify lines, emit via tracing, then wait.
async fn run_download_process(ytd: YtDlp) -> Result<()> {
    let mut child = ytd.download_process().await?;
    while let Some(line) = child.next_line().await? {
        match classify_yt_dlp_line(&line) {
            YtDlpEvent::Progress { raw, percent } => {
                if let Some(percent) = percent {
                    tracing::info!(percent, "{}", raw);
                } else {
                    tracing::info!("{}", raw);
                }
            }
            YtDlpEvent::Log { raw } => {
                tracing::debug!("{}", raw);
            }
        }
    }
    child.wait().await?;
    Ok(())
}

/// Shared yt-dlp arg set for the EJS/Deno download flow (url + file variants).
fn build_download_builder(
    url: &str,
    deno_path: &Path,
    archive_file: &Path,
    output_dir: &Path,
) -> YtDlp {
    apply_download_args(YtDlp::new(url), deno_path, archive_file, output_dir)
}

fn apply_download_args(
    ytd: YtDlp,
    deno_path: &Path,
    archive_file: &Path,
    output_dir: &Path,
) -> YtDlp {
    ytd.arg("-cw")
        .arg_with("-o", "%(title)s-%(id)s.%(ext)s")
        .arg("--embed-thumbnail")
        .arg("--write-description")
        .arg("--embed-metadata")
        .arg("--no-colors")
        .arg("--remote-components")
        .arg("ejs:npm")
        .arg_with("--js-runtimes", format!("deno:{}", deno_path.display()))
        .arg_with(
            "--download-archive",
            archive_file.to_string_lossy().to_string(),
        )
        .arg_with("-P", output_dir.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn download_process_streams_version_when_yt_dlp_present() {
        if which::which("yt-dlp").is_err() {
            return;
        }
        let mut child = YtDlp::new("")
            .arg("--version")
            .download_process()
            .await
            .expect("download_process --version");
        let mut lines = Vec::new();
        while let Some(line) = child.next_line().await.expect("next_line") {
            lines.push(line);
        }
        child.wait().await.expect("wait");
        assert!(!lines.is_empty(), "expected yt-dlp --version stdout");
    }
}
