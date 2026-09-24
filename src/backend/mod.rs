//! YtDlp backend boundary.
//!
//! Abstracts over the mechanism used to invoke `yt-dlp` so that the CLI can
//! use the async `ytd-rs` wrapper (Issue #66) without changing command
//! behaviour.
//!
//! - [`ytdrs`]: async `ytd-rs` adapter (the only backend).
//! - [`process`]: midnite-owned tokio runner for streaming media downloads.
//!
//! The sync facade in [`crate::yt_dlp`] calls this backend via a process-wide
//! Tokio runtime. Probe helpers stay sync and are intentionally NOT part of
//! this boundary.

use anyhow::{Context, Result};
use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Public classification API for the ytd-rs adapter and a future TUI.
pub mod events;

pub(crate) mod process;
pub mod ytdrs;

pub use ytdrs::YtdRsBackend;

use crate::types::{Channel, Video};
use async_trait::async_trait;

/// Canonical list-download archive name inside `<output_dir>/.archive/`.
const CHANNEL_ARCHIVE_FILE: &str = "videos.archive";

/// Backend abstraction over the subset of `yt-dlp` operations the CLI uses.
///
/// The sync facade uses `runtime().block_on(...)` — never call `block_on`
/// from an async context.
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

/// Archive path for every list download of a channel: `<output_dir>/.archive/videos.archive`.
pub fn channel_archive_path(output_dir: &Path) -> PathBuf {
    output_dir.join(".archive").join(CHANNEL_ARCHIVE_FILE)
}

/// Video id from a yt-dlp archive line (`youtube <id>`). The id is the last field.
pub fn archive_line_id(line: &str) -> &str {
    line.split_whitespace().next_back().unwrap_or(line)
}

/// How many `list_ids` already appear in `archive_lines`.
pub fn count_ids_in_archive<'a>(
    list_ids: impl IntoIterator<Item = &'a str>,
    archive_lines: &[String],
) -> usize {
    let archive_ids: HashSet<&str> = archive_lines
        .iter()
        .map(|line| archive_line_id(line))
        .collect();
    list_ids
        .into_iter()
        .filter(|id| archive_ids.contains(id))
        .count()
}

/// Union of every `*.archive` file in `archive_dir`.
///
/// First occurrence wins. `videos.archive` is read first, then the other files
/// by name. A missing directory is an empty list. This does not write.
pub fn read_archive_lines(archive_dir: &Path) -> Result<Vec<String>> {
    let files = list_archive_files(archive_dir)
        .with_context(|| format!("Failed to read archive directory {}", archive_dir.display()))?;
    let mut seen = HashSet::new();
    let mut lines = Vec::new();
    for file in files {
        let content = fs::read_to_string(&file)
            .with_context(|| format!("Failed to read archive {}", file.display()))?;
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let owned = line.to_string();
            if seen.insert(owned.clone()) {
                lines.push(owned);
            }
        }
    }
    Ok(lines)
}

/// Write the archive union into `videos.archive` when a line is still missing
/// or the file has no trailing newline. Other `*.archive` files stay on disk.
///
/// Returns the canonical path even when there is nothing to write yet.
pub fn materialize_channel_archive(output_dir: &Path) -> Result<PathBuf> {
    let path = channel_archive_path(output_dir);
    let Some(dir) = path.parent() else {
        return Ok(path);
    };
    fs::create_dir_all(dir)
        .with_context(|| format!("Failed to create archive directory {}", dir.display()))?;
    let union = read_archive_lines(dir)?;
    if union.is_empty() {
        return Ok(path);
    }
    let existing = if path.is_file() {
        Some(
            fs::read_to_string(&path)
                .with_context(|| format!("Failed to read archive {}", path.display()))?,
        )
    } else {
        None
    };
    if existing
        .as_deref()
        .is_none_or(|raw| archive_needs_write(raw, &union))
    {
        fs::write(&path, canonical_archive_body(&union))
            .with_context(|| format!("Failed to write archive {}", path.display()))?;
    }
    Ok(path)
}

fn list_archive_files(archive_dir: &Path) -> io::Result<Vec<PathBuf>> {
    let entries = match fs::read_dir(archive_dir) {
        Ok(entries) => entries,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => return Err(err),
    };
    let mut files = Vec::new();
    for entry in entries {
        let path = entry?.path();
        if path.is_file() && path.extension() == Some(OsStr::new("archive")) {
            files.push(path);
        }
    }
    files.sort_by(|left, right| {
        let left_canonical = left.file_name() == Some(OsStr::new(CHANNEL_ARCHIVE_FILE));
        let right_canonical = right.file_name() == Some(OsStr::new(CHANNEL_ARCHIVE_FILE));
        right_canonical
            .cmp(&left_canonical)
            .then_with(|| left.file_name().cmp(&right.file_name()))
    });
    Ok(files)
}

fn archive_needs_write(raw: &str, union: &[String]) -> bool {
    let mut seen = HashSet::new();
    for line in raw.lines() {
        let line = line.trim();
        if !line.is_empty() {
            seen.insert(line.to_string());
        }
    }
    union.iter().any(|line| !seen.contains(line)) || !raw.ends_with('\n')
}

fn canonical_archive_body(lines: &[String]) -> String {
    let mut body = lines.join("\n");
    body.push('\n');
    body
}

/// Legacy per-list archive path: `<output_dir>/.archive/<stem>.archive`.
///
/// List downloads use [`channel_archive_path`] instead, so every list of a
/// channel shares one archive. This remains for callers that still address a
/// single list-stem file.
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
    if let Some(dir) = archive_file.parent()
        && let Err(e) = std::fs::create_dir_all(dir)
    {
        tracing::warn!("Failed to create archive directory: {}", e);
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

    #[test]
    fn ytdrs_backend_default_constructs() {
        let _b: Box<dyn YtDlpBackend> = Box::new(YtdRsBackend);
    }

    #[test]
    fn channel_archive_path_is_videos_archive() {
        let path = channel_archive_path(Path::new("/out"));
        assert_eq!(path, PathBuf::from("/out/.archive/videos.archive"));
    }

    #[test]
    fn archive_line_id_is_last_field() {
        assert_eq!(archive_line_id("youtube dQw4w9WgXcQ"), "dQw4w9WgXcQ");
        assert_eq!(archive_line_id("dQw4w9WgXcQ"), "dQw4w9WgXcQ");
    }

    #[test]
    fn read_archive_lines_prefers_videos_archive_and_dedupes() {
        let dir = tempfile::tempdir().unwrap();
        let archive = dir.path().join(".archive");
        fs::create_dir(&archive).unwrap();
        fs::write(archive.join("c.archive"), "youtube ccc\nyoutube aaa\n").unwrap();
        fs::write(archive.join("a.archive"), "youtube aaa\nyoutube bbb\n").unwrap();
        fs::write(archive.join("videos.archive"), "youtube bbb\n").unwrap();
        fs::write(archive.join("notes.txt"), "youtube zzz\n").unwrap();

        let lines = read_archive_lines(&archive).unwrap();

        assert_eq!(
            lines,
            vec![
                "youtube bbb".to_string(),
                "youtube aaa".to_string(),
                "youtube ccc".to_string(),
            ]
        );
        let raw = fs::read_to_string(archive.join("videos.archive")).unwrap();
        assert_eq!(raw, "youtube bbb\n");
    }

    #[test]
    fn read_archive_lines_missing_dir_is_empty() {
        let dir = tempfile::tempdir().unwrap();
        let lines = read_archive_lines(&dir.path().join(".archive")).unwrap();
        assert!(lines.is_empty());
    }

    #[test]
    fn materialize_folds_legacy_files_without_deleting_them() {
        let dir = tempfile::tempdir().unwrap();
        let archive = dir.path().join(".archive");
        fs::create_dir(&archive).unwrap();
        fs::write(archive.join("b.archive"), "youtube bbb\nyoutube aaa\n").unwrap();
        fs::write(archive.join("a.archive"), "youtube aaa\n").unwrap();

        let path = materialize_channel_archive(dir.path()).unwrap();

        assert_eq!(path, archive.join("videos.archive"));
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            "youtube aaa\nyoutube bbb\n"
        );
        assert!(archive.join("a.archive").is_file());
        assert!(archive.join("b.archive").is_file());
    }

    #[test]
    fn materialize_folds_a_missing_legacy_line_after_existing_ids() {
        let dir = tempfile::tempdir().unwrap();
        let archive = dir.path().join(".archive");
        fs::create_dir(&archive).unwrap();
        fs::write(archive.join("videos.archive"), "youtube bbb\n").unwrap();
        fs::write(archive.join("old.archive"), "youtube aaa\n").unwrap();

        materialize_channel_archive(dir.path()).unwrap();

        assert_eq!(
            fs::read_to_string(archive.join("videos.archive")).unwrap(),
            "youtube bbb\nyoutube aaa\n"
        );
        assert!(archive.join("old.archive").is_file());
    }

    #[test]
    fn materialize_skips_rewrite_when_union_already_present() {
        let dir = tempfile::tempdir().unwrap();
        let archive = dir.path().join(".archive");
        fs::create_dir(&archive).unwrap();
        let canonical = archive.join("videos.archive");
        fs::write(&canonical, "youtube bbb\nyoutube aaa\n").unwrap();
        fs::write(archive.join("a.archive"), "youtube aaa\n").unwrap();
        let before = fs::read(&canonical).unwrap();

        materialize_channel_archive(dir.path()).unwrap();

        assert_eq!(fs::read(&canonical).unwrap(), before);
    }

    #[test]
    fn materialize_rewrites_when_missing_trailing_newline() {
        let dir = tempfile::tempdir().unwrap();
        let archive = dir.path().join(".archive");
        fs::create_dir(&archive).unwrap();
        fs::write(archive.join("videos.archive"), "youtube aaa").unwrap();

        materialize_channel_archive(dir.path()).unwrap();

        assert_eq!(
            fs::read_to_string(archive.join("videos.archive")).unwrap(),
            "youtube aaa\n"
        );
    }

    #[test]
    fn materialize_empty_does_not_create_archive_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = materialize_channel_archive(dir.path()).unwrap();
        assert_eq!(path, dir.path().join(".archive").join("videos.archive"));
        assert!(!path.exists());
        assert!(path.parent().unwrap().is_dir());
    }

    #[test]
    fn count_ids_ignores_archive_ids_outside_the_list() {
        let lines = vec![
            "youtube aaaaaaaaaaa".to_string(),
            "youtube bbbbbbbbbbb".to_string(),
            "youtube ccccccccccc".to_string(),
            "youtube ddddddddddd".to_string(),
            "youtube eeeeeeeeeee".to_string(),
        ];
        let list = ["aaaaaaaaaaa", "bbbbbbbbbbb", "fffffffffff"];
        assert_eq!(count_ids_in_archive(list, &lines), 2);
    }
}
