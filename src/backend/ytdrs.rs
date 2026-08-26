//! Async backend backed by the `ytd-rs` crate.
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
use std::future::Future;
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
        run_download("yt-dlp", args, wait_for_ctrl_c())
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
        run_download("yt-dlp", args, wait_for_ctrl_c())
            .await
            .with_context(|| format!("Failed to run yt-dlp for file: {:?}", list_file))?;
        Ok(())
    }

    async fn download_comments(&self, list_file: &Path, output_dir: &Path) -> Result<()> {
        let mut ytd = YtDlp::new_multiple(Vec::new());
        for arg in comments_from_file_args(list_file, output_dir) {
            ytd = ytd.arg(arg);
        }
        ytd.download()
            .await
            .with_context(|| format!("Failed to run yt-dlp for comments: {:?}", list_file))?;
        Ok(())
    }

    async fn download_comments_for_video(&self, video: &Video, output_dir: &Path) -> Result<()> {
        let video_url = video.url();
        let mut ytd = YtDlp::new(&video_url);
        for arg in comments_for_video_args(video, output_dir) {
            if arg != video_url {
                ytd = ytd.arg(arg);
            }
        }
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

/// Shared yt-dlp arg set for comments-only downloads (batch list-file path).
///
/// Flag order matches the previous Command / ytd-rs builder path:
/// `-o` template, `-P` dir, `-a` list, then `--write-comments --skip-download --no-colors`.
fn comments_from_file_args(list_file: &Path, output_dir: &Path) -> Vec<String> {
    vec![
        "-o".into(),
        "%(id)s.comments.json".into(),
        "-P".into(),
        output_dir.to_string_lossy().into_owned(),
        "-a".into(),
        list_file.to_string_lossy().into_owned(),
        "--write-comments".into(),
        "--skip-download".into(),
        "--no-colors".into(),
    ]
}

/// Shared yt-dlp arg set for comments-only downloads of a single video.
///
/// Same comment flags as [`comments_from_file_args`], without `-a`; the video
/// URL is last (passed to `YtDlp::new` as the link).
fn comments_for_video_args(video: &Video, output_dir: &Path) -> Vec<String> {
    vec![
        "-o".into(),
        "%(id)s.comments.json".into(),
        "-P".into(),
        output_dir.to_string_lossy().into_owned(),
        "--write-comments".into(),
        "--skip-download".into(),
        "--no-colors".into(),
        video.url(),
    ]
}

async fn run_download<A, I, S, C>(program: A, args: I, cancel: C) -> Result<()>
where
    A: AsRef<OsStr>,
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
    C: Future<Output = ()>,
{
    let outcome = run_streaming(program, args, cancel, |_| {}).await?;
    match outcome {
        RunOutcome::Success => Ok(()),
        RunOutcome::Cancelled => bail!("download cancelled"),
    }
}

/// First Ctrl-C cancels the download. A re-armed listener hard-exits on the
/// second SIGINT so the user can always escalate during grace / wait.
async fn wait_for_ctrl_c() {
    match tokio::signal::ctrl_c().await {
        Ok(()) => {
            tokio::spawn(async {
                if tokio::signal::ctrl_c().await.is_ok() {
                    std::process::exit(130);
                }
            });
        }
        Err(e) => {
            tracing::warn!("failed to listen for Ctrl-C: {e}");
            std::future::pending::<()>().await;
        }
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

    #[test]
    fn comments_from_file_args_include_comment_and_playlist_flags() {
        let list = Path::new("/tmp/channel-list.txt");
        let out = Path::new("/tmp/comments");
        let args = comments_from_file_args(list, out);

        assert!(
            args.windows(2)
                .any(|w| w[0] == "-o" && w[1] == "%(id)s.comments.json"),
            "missing comments output template: {args:?}"
        );
        assert!(
            args.windows(2)
                .any(|w| w[0] == "-P" && w[1] == "/tmp/comments"),
            "missing -P output dir: {args:?}"
        );
        assert!(
            args.windows(2)
                .any(|w| w[0] == "-a" && w[1] == "/tmp/channel-list.txt"),
            "missing playlist -a list file: {args:?}"
        );
        for flag in ["--write-comments", "--skip-download", "--no-colors"] {
            assert!(args.iter().any(|a| a == flag), "missing {flag}: {args:?}");
        }
        assert!(
            !args.iter().any(|a| a.contains("youtube.com/watch")),
            "batch path must not take a watch URL: {args:?}"
        );
    }

    #[test]
    fn comments_for_video_args_include_comment_flags_and_video_url() {
        use crate::types::{Channel, ChannelName, Video, VideoId};

        let video = Video::new(
            VideoId::new("dQw4w9WgXcQ").unwrap(),
            "Never Gonna Give You Up",
            Channel::new(ChannelName::new("testchannel").unwrap()),
        );
        let out = Path::new("/tmp/comments");
        let args = comments_for_video_args(&video, out);

        assert!(
            args.windows(2)
                .any(|w| w[0] == "-o" && w[1] == "%(id)s.comments.json"),
            "missing comments output template: {args:?}"
        );
        assert!(
            args.windows(2)
                .any(|w| w[0] == "-P" && w[1] == "/tmp/comments"),
            "missing -P output dir: {args:?}"
        );
        for flag in ["--write-comments", "--skip-download", "--no-colors"] {
            assert!(args.iter().any(|a| a == flag), "missing {flag}: {args:?}");
        }
        assert_eq!(
            args.last().map(String::as_str),
            Some("https://www.youtube.com/watch?v=dQw4w9WgXcQ"),
            "video URL must be last: {args:?}"
        );
        assert!(
            !args.iter().any(|a| a == "-a"),
            "per-video path must not use playlist -a: {args:?}"
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
            std::future::pending::<()>(),
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
            std::future::pending::<()>(),
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
        let (tx, rx) = tokio::sync::oneshot::channel();
        let err = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            run_download(
                "/bin/sh",
                [OsString::from("-c"), OsString::from("sleep 30")],
                async {
                    let _ = rx.await;
                },
            ),
        );
        tx.send(()).expect("send cancel");
        let err = err
            .await
            .expect("cancel timed out")
            .expect_err("cancelled download should be Err");
        let msg = format!("{err:#}");
        assert!(
            msg.contains("download cancelled"),
            "production mapping missing: {msg}"
        );
    }
}
