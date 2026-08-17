//! Midnite-owned yt-dlp stdout events (issue #71).
//!
//! Classification is independent of `ytd-rs` so unit tests run on the default
//! feature set. The ytd-rs adapter streams lines and emits these via `tracing`.

/// A classified yt-dlp stdout line.
#[derive(Debug, Clone, PartialEq)]
pub enum YtDlpEvent {
    /// A download-progress line (`[download]` plus a `%`).
    Progress { raw: String, percent: Option<f32> },
    /// Any other stdout line (destination, archive skip, merger, …).
    Log { raw: String },
}

/// Classify a single yt-dlp stdout line.
///
/// Progress requires both `[download]` and `%`. Percent is parsed from the
/// first `N%` / `N.N%` token when present.
pub fn classify_yt_dlp_line(line: &str) -> YtDlpEvent {
    if line.contains("[download]") && line.contains('%') {
        YtDlpEvent::Progress {
            raw: line.to_string(),
            percent: parse_download_percent(line),
        }
    } else {
        YtDlpEvent::Log {
            raw: line.to_string(),
        }
    }
}

fn parse_download_percent(line: &str) -> Option<f32> {
    let end = line.find('%')?;
    let start = line[..end]
        .rfind(|c: char| !(c.is_ascii_digit() || c == '.'))
        .map(|i| i + 1)
        .unwrap_or(0);
    line[start..end].parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn percent_of(event: &YtDlpEvent) -> Option<f32> {
        match event {
            YtDlpEvent::Progress { percent, .. } => *percent,
            YtDlpEvent::Log { .. } => None,
        }
    }

    #[test]
    fn download_percent_is_progress() {
        let line = "[download]  12.3% of 50.00MiB at 1.00MiB/s ETA 00:50";
        let event = classify_yt_dlp_line(line);
        assert!(matches!(&event, YtDlpEvent::Progress { raw, .. } if raw == line));
        let percent = percent_of(&event).expect("percent");
        assert!((percent - 12.3).abs() < 1e-4, "percent={percent}");
    }

    #[test]
    fn download_destination_is_log() {
        let line = "[download] Destination: foo.mp4";
        assert_eq!(
            classify_yt_dlp_line(line),
            YtDlpEvent::Log {
                raw: line.to_string()
            }
        );
    }

    #[test]
    fn archive_skip_is_log() {
        let line = "[download] foo has already been recorded in the archive";
        assert_eq!(
            classify_yt_dlp_line(line),
            YtDlpEvent::Log {
                raw: line.to_string()
            }
        );
    }

    #[test]
    fn merger_is_log() {
        let line = "[Merger] Merging formats into \"foo.mp4\"";
        assert_eq!(
            classify_yt_dlp_line(line),
            YtDlpEvent::Log {
                raw: line.to_string()
            }
        );
    }
}
