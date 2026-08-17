//! YtDlp backend boundary.
//!
//! Abstracts over the mechanism used to invoke `yt-dlp` so that the CLI can
//! migrate from raw `std::process::Command` to the async `ytd-rs` wrapper
//! (Issue #66) without changing command behaviour.
//!
//! - [`command`]: legacy sync `std::process::Command` implementations (default).
//! - [`ytdrs`]: async `ytd-rs` adapter, compiled only with `ytd-rs-backend`.
//!
//! The sync facade in [`crate::yt_dlp`] selects the backend via the feature flag.
//! Probe helpers stay sync and are intentionally NOT part of this boundary.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

pub mod command;
/// Always compiled so classifier tests run on the default (Command) build.
/// Public classification API for the ytd-rs adapter and a future TUI.
pub mod events;

#[cfg(feature = "ytd-rs-backend")]
pub mod ytdrs;

#[cfg(feature = "ytd-rs-backend")]
pub use command::CommandBackend;
#[cfg(feature = "ytd-rs-backend")]
pub use ytdrs::YtdRsBackend;

#[cfg(feature = "ytd-rs-backend")]
use crate::types::{Channel, Video};
#[cfg(feature = "ytd-rs-backend")]
use anyhow::Result;
#[cfg(feature = "ytd-rs-backend")]
use async_trait::async_trait;

/// Backend abstraction over the subset of `yt-dlp` operations the CLI uses.
///
/// Available when `ytd-rs-backend` is enabled. The sync facade uses
/// `runtime().block_on(...)` — never call `block_on` from an async context.
#[cfg(feature = "ytd-rs-backend")]
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

/// Archive path for single-URL downloads: `<output_dir>/.archive/downloads.archive`.
pub fn url_archive_path(output_dir: &Path) -> PathBuf {
    output_dir.join(".archive").join("downloads.archive")
}

/// Archive path for list-file downloads: `<output_dir>/.archive/<stem>.archive`.
///
/// Uses `join(file_stem).with_extension("archive")` so multi-extension names
/// like `foo.bar.txt` produce `foo.archive` (not `foo.bar.archive`).
pub fn list_archive_path(output_dir: &Path, list_file: &Path) -> PathBuf {
    output_dir
        .join(".archive")
        .join(list_file.file_stem().unwrap_or(OsStr::new("archive")))
        .with_extension("archive")
}

/// Ensure the parent directory of `archive_file` exists; warn (do not fail) on error.
pub fn ensure_archive_parent(archive_file: &Path) {
    if let Some(dir) = archive_file.parent() {
        if let Err(e) = std::fs::create_dir_all(dir) {
            tracing::warn!("Failed to create archive directory: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn list_archive_path_strips_to_stem() {
        let path = list_archive_path(Path::new("/out"), Path::new("foo.bar.txt"));
        assert_eq!(path, PathBuf::from("/out/.archive/foo.archive"));
    }

    #[test]
    fn list_archive_path_single_extension() {
        let path = list_archive_path(Path::new("/out"), Path::new("channel-list.txt"));
        assert_eq!(path, PathBuf::from("/out/.archive/channel-list.archive"));
    }

    #[test]
    fn url_archive_path_is_downloads_archive() {
        let path = url_archive_path(Path::new("/out"));
        assert_eq!(path, PathBuf::from("/out/.archive/downloads.archive"));
    }

    #[cfg(feature = "ytd-rs-backend")]
    #[test]
    fn command_backend_default_constructs() {
        let _b: Box<dyn YtDlpBackend> = Box::new(CommandBackend);
    }

    #[cfg(feature = "ytd-rs-backend")]
    #[test]
    fn ytdrs_backend_default_constructs() {
        let _b: Box<dyn YtDlpBackend> = Box::new(YtdRsBackend);
    }
}
