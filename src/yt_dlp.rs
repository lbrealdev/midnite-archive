use crate::backend::{YtDlpBackend, YtdRsBackend};
use crate::types::{Channel, Video, VideoId};
use anyhow::{Result, bail};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Process-wide current-thread Tokio runtime for the sync facade.
///
/// Never call `block_on` from inside an async context (it panics).
fn runtime() -> &'static tokio::runtime::Runtime {
    static RT: std::sync::OnceLock<tokio::runtime::Runtime> = std::sync::OnceLock::new();
    RT.get_or_init(|| {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("failed to create Tokio runtime")
    })
}

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

/// Generate a channel list and return structured Video data.
pub fn generate_channel_list(
    channel: &Channel,
    output_file: &Path,
    filter: Option<&str>,
) -> Result<Vec<Video>> {
    runtime().block_on(YtdRsBackend.generate_channel_list(channel, output_file, filter))
}

/// Parse yt-dlp `--flat-playlist --print "%(title)s-%(id)s"` output into Videos.
pub fn parse_channel_list_output(stdout: &str, channel: &Channel) -> Vec<Video> {
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
    videos
}

pub fn download_from_url(url: &str, output_dir: &Path) -> Result<()> {
    runtime().block_on(YtdRsBackend.download_from_url(url, output_dir))
}

pub fn download_from_file(
    list_file: &Path,
    output_dir: &Path,
    total_videos: usize,
    downloaded_count: usize,
) -> Result<()> {
    runtime().block_on(YtdRsBackend.download_from_file(
        list_file,
        output_dir,
        total_videos,
        downloaded_count,
    ))
}

pub fn download_comments(list_file: &Path, output_dir: &Path) -> Result<()> {
    runtime().block_on(YtdRsBackend.download_comments(list_file, output_dir))
}

/// Download comments for a specific video.
pub fn download_comments_for_video(video: &Video, output_dir: &Path) -> Result<()> {
    runtime().block_on(YtdRsBackend.download_comments_for_video(video, output_dir))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ChannelName;

    fn test_channel() -> Channel {
        Channel::new(ChannelName::new("testchannel").unwrap())
    }

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

    #[test]
    fn parse_channel_list_normal_hyphenated_titles() {
        let channel = test_channel();
        let stdout = "\
My Cool Video-dQw4w9WgXcQ
Foo-Bar-Baz-abcdefghijk
Simple Title-xxxxxxxxxxx
";
        let videos = parse_channel_list_output(stdout, &channel);
        assert_eq!(videos.len(), 3);
        assert_eq!(videos[0].title, "My Cool Video");
        assert_eq!(videos[0].id.to_string(), "dQw4w9WgXcQ");
        assert_eq!(videos[1].title, "Foo-Bar-Baz");
        assert_eq!(videos[1].id.to_string(), "abcdefghijk");
        assert_eq!(videos[2].title, "Simple Title");
        assert_eq!(videos[2].id.to_string(), "xxxxxxxxxxx");
        assert_eq!(videos[0].channel, channel);
    }

    #[test]
    fn parse_channel_list_short_lines_are_skipped() {
        let channel = test_channel();
        // Fewer than 12 chars total (need title + hyphen + 11-char ID).
        let stdout = "\
short
a-bcdefghij
title-only
ok-dQw4w9WgXcQ
";
        let videos = parse_channel_list_output(stdout, &channel);
        assert_eq!(videos.len(), 1);
        assert_eq!(videos[0].title, "ok");
        assert_eq!(videos[0].id.to_string(), "dQw4w9WgXcQ");
    }

    #[test]
    fn parse_channel_list_invalid_video_ids_are_skipped() {
        let channel = test_channel();
        // 11 trailing chars but not a valid YouTube ID (invalid characters).
        let stdout = "\
Bad Chars-!!!!!!!!!!!
Spaces ID-aaaa bbbbb
Good One-dQw4w9WgXcQ
";
        let videos = parse_channel_list_output(stdout, &channel);
        assert_eq!(videos.len(), 1);
        assert_eq!(videos[0].title, "Good One");
        assert_eq!(videos[0].id.to_string(), "dQw4w9WgXcQ");
    }

    #[test]
    fn parse_channel_list_empty_input() {
        let channel = test_channel();
        assert!(parse_channel_list_output("", &channel).is_empty());
        assert!(parse_channel_list_output("\n\n\n", &channel).is_empty());
    }

    #[test]
    fn parse_channel_list_whitespace_lines() {
        let channel = test_channel();
        let stdout = "\
   \t  
  Spaced Title-dQw4w9WgXcQ  
\tAnother-abcdefghijk\t
   \n
";
        let videos = parse_channel_list_output(stdout, &channel);
        assert_eq!(videos.len(), 2);
        assert_eq!(videos[0].title, "Spaced Title");
        assert_eq!(videos[0].id.to_string(), "dQw4w9WgXcQ");
        assert_eq!(videos[1].title, "Another");
        assert_eq!(videos[1].id.to_string(), "abcdefghijk");
    }
}
