//! Async backend backed by the `ytd-rs` crate.
//!
//! Compiled only with the `ytd-rs-backend` feature.
//!
//! Media downloads stream yt-dlp stdout and stderr via a midnite-owned
//! [`tokio::process`] runner and classify lines into
//! [`crate::backend::events::YtDlpEvent`]. Default CLI stays quiet
//! (`-v` surfaces progress as `tracing` info).
//!
//! **Cancellation:** streaming downloads spawn yt-dlp in its own process
//! group. Ctrl-C is caught with `tokio::signal::ctrl_c()`, which SIGTERMs
//! the group, drains pipes for 3s, then SIGKILLs and reaps. `kill_on_drop(true)`
//! is a panic/drop backstop. A cancelled download returns `Err` (nonzero CLI
//! exit, no "done"). Generate and comments still use buffered `ytd-rs`
//! `download()` and are not cancellable. Windows cancel is `start_kill()` on
//! the direct child only (no job objects).

use crate::backend::process::{RunOutcome, run_streaming};
use crate::backend::{YtDlpBackend, ensure_archive_parent, list_archive_path, url_archive_path};
use crate::types::{Channel, Video};
use crate::yt_dlp::parse_channel_list_output;
use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use std::ffi::{OsStr, OsString};
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

        let mut args = download_args(&deno_path, &archive_file, output_dir);
        args.push(OsString::from(url));
        run_download("yt-dlp", args)
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

        let mut args = download_args(&deno_path, &archive_file, output_dir);
        args.push(OsString::from("-a"));
        args.push(list_file.as_os_str().to_owned());
        run_download("yt-dlp", args)
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

/// Shared yt-dlp arg set for the EJS/Deno download flow (url + file variants).
///
/// Paths are kept as [`OsString`] (no `to_string_lossy`). Includes `--newline`
/// so progress lines are one-per-line for the streaming classifier.
fn download_args(deno_path: &Path, archive_file: &Path, output_dir: &Path) -> Vec<OsString> {
    let mut js_runtime = OsString::from("deno:");
    js_runtime.push(deno_path.as_os_str());

    vec![
        OsString::from("-cw"),
        OsString::from("-o"),
        OsString::from("%(title)s-%(id)s.%(ext)s"),
        OsString::from("--embed-thumbnail"),
        OsString::from("--write-description"),
        OsString::from("--embed-metadata"),
        OsString::from("--no-colors"),
        OsString::from("--newline"),
        OsString::from("--remote-components"),
        OsString::from("ejs:npm"),
        OsString::from("--js-runtimes"),
        js_runtime,
        OsString::from("--download-archive"),
        archive_file.as_os_str().to_owned(),
        OsString::from("-P"),
        output_dir.as_os_str().to_owned(),
    ]
}

async fn run_download<A, I, S>(program: A, args: I) -> Result<()>
where
    A: AsRef<OsStr>,
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let outcome = run_streaming(
        program,
        args,
        async {
            match tokio::signal::ctrl_c().await {
                Ok(()) => {}
                Err(e) => {
                    tracing::warn!("failed to listen for Ctrl-C: {e}");
                    std::future::pending::<()>().await;
                }
            }
        },
        |_| {},
    )
    .await?;
    match outcome {
        RunOutcome::Success => Ok(()),
        RunOutcome::Cancelled => bail!("download cancelled"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn download_args_includes_newline_and_preserves_os_paths() {
        let deno = Path::new("/opt/deno");
        let archive = Path::new("/tmp/archives/foo.archive");
        let out = Path::new("/tmp/out dir");
        let args = download_args(deno, archive, out);

        assert!(
            args.iter().any(|a| a == "--newline"),
            "missing --newline: {args:?}"
        );
        assert!(
            args.iter().any(|a| a == "deno:/opt/deno"),
            "lossy or missing js-runtime: {args:?}"
        );
        assert!(
            args.iter().any(|a| a.as_os_str() == archive.as_os_str()),
            "archive path not passed as OsStr: {args:?}"
        );
        assert!(
            args.iter().any(|a| a.as_os_str() == out.as_os_str()),
            "output dir not passed as OsStr: {args:?}"
        );
        assert!(
            !args
                .iter()
                .any(|a| a.to_string_lossy().contains('\u{FFFD}'))
        );
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn run_download_success_with_fake_binary() {
        run_download(
            "/bin/sh",
            [
                OsString::from("-c"),
                OsString::from("echo '[download]  12.3% of 50.00MiB at 1.00MiB/s ETA 00:50'"),
            ],
        )
        .await
        .expect("fake success");
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn run_download_nonzero_attaches_stderr_tail() {
        let err = run_download(
            "/bin/sh",
            [
                OsString::from("-c"),
                OsString::from("echo 'ERROR: boom' >&2; exit 2"),
            ],
        )
        .await
        .expect_err("expected failure");
        let msg = format!("{err:#}");
        assert!(msg.contains("ERROR: boom"), "{msg}");
        assert!(msg.contains("status Some(2)"), "{msg}");
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn run_download_cancel_returns_err() {
        let outcome = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            run_streaming(
                "/bin/sh",
                ["-c", "sleep 30"],
                std::future::ready(()),
                |_| {},
            ),
        )
        .await
        .expect("cancel timed out")
        .expect("run");
        assert_eq!(outcome, RunOutcome::Cancelled);
        // Mirror production mapping: Cancelled becomes Err so CLI skips "done".
        let mapped: Result<()> = match outcome {
            RunOutcome::Success => Ok(()),
            RunOutcome::Cancelled => Err(anyhow::anyhow!("download cancelled")),
        };
        assert!(mapped.is_err());
    }
}
