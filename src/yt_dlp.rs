use crate::types::{Channel, Video, VideoId};
use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Result of probing an external tool on PATH.
#[derive(Debug, Clone)]
pub struct ToolProbe {
    pub name: &'static str,
    pub ok: bool,
    pub version: Option<String>,
    pub path: Option<PathBuf>,
    pub error: Option<String>,
}

/// Probe a tool: resolve PATH, run version args, capture first version-ish line.
pub fn probe_tool(name: &'static str, version_args: &[&str]) -> ToolProbe {
    let path = which::which(name).ok();

    let Some(resolved) = path.as_ref() else {
        return ToolProbe {
            name,
            ok: false,
            version: None,
            path: None,
            error: Some(format!("{name} not found on PATH")),
        };
    };

    match Command::new(resolved).args(version_args).output() {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let version = extract_version_line(&stdout).or_else(|| extract_version_line(&stderr));
            ToolProbe {
                name,
                ok: true,
                version,
                path: Some(resolved.clone()),
                error: None,
            }
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            ToolProbe {
                name,
                ok: false,
                version: None,
                path: Some(resolved.clone()),
                error: Some(if stderr.is_empty() {
                    format!("{name} exited with status {:?}", output.status.code())
                } else {
                    stderr
                }),
            }
        }
        Err(e) => ToolProbe {
            name,
            ok: false,
            version: None,
            path: Some(resolved.clone()),
            error: Some(e.to_string()),
        },
    }
}

fn extract_version_line(text: &str) -> Option<String> {
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(|line| {
            // Prefer a compact token when the first line is like "yt-dlp 2026.01.01"
            // or "ffmpeg version 6.1.1 Copyright ..."
            if let Some(rest) = line.strip_prefix("ffmpeg version ") {
                rest.split_whitespace().next().unwrap_or(rest).to_string()
            } else if let Some((_, ver)) = line.split_once(' ') {
                // "deno 2.1.4 (stable, ...)" or "yt-dlp 2026.x.x"
                ver.split_whitespace()
                    .next()
                    .unwrap_or(ver)
                    .trim_end_matches(',')
                    .to_string()
            } else {
                line.to_string()
            }
        })
}

pub fn probe_yt_dlp() -> ToolProbe {
    probe_tool("yt-dlp", &["--version"])
}

pub fn probe_deno() -> ToolProbe {
    probe_tool("deno", &["--version"])
}

pub fn probe_ffmpeg() -> ToolProbe {
    // ffmpeg uses -version (single dash)
    probe_tool("ffmpeg", &["-version"])
}

pub fn check_available() -> Result<()> {
    let probe = probe_yt_dlp();
    if probe.ok {
        Ok(())
    } else {
        bail!(
            "yt-dlp not found. Please install it first.{}",
            probe.error.map(|e| format!(" ({e})")).unwrap_or_default()
        );
    }
}

pub fn check_deno_available() -> Result<()> {
    let probe = probe_deno();
    if probe.ok {
        Ok(())
    } else {
        bail!(
            "deno not found. Please install it first.{}",
            probe.error.map(|e| format!(" ({e})")).unwrap_or_default()
        );
    }
}

pub fn check_ffmpeg_available() -> Result<()> {
    let probe = probe_ffmpeg();
    if probe.ok {
        Ok(())
    } else {
        bail!(
            "ffmpeg not found. Please install it first.{}",
            probe.error.map(|e| format!(" ({e})")).unwrap_or_default()
        );
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_version_line_plain() {
        assert_eq!(
            extract_version_line("2026.01.01\n"),
            Some("2026.01.01".into())
        );
    }

    #[test]
    fn extract_version_line_deno() {
        assert_eq!(
            extract_version_line("deno 2.1.4 (stable, release, x86_64-unknown-linux-gnu)\n"),
            Some("2.1.4".into())
        );
    }

    #[test]
    fn extract_version_line_ffmpeg() {
        assert_eq!(
            extract_version_line("ffmpeg version 6.1.1 Copyright (c) 2000-2023\n"),
            Some("6.1.1".into())
        );
    }
}
