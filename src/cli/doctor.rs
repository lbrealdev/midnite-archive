use crate::yt_dlp::{self, ToolProbe};
use anyhow::{Result, bail};
use std::io::{self, IsTerminal, Write};

/// Severity for doctor checks.
#[derive(Debug, Clone, Copy)]
enum Severity {
    /// Required for all yt-dlp workflows.
    Required,
    /// Required for `download` (EJS / Deno runtime).
    RequiredForDownload,
}

struct Check {
    probe: ToolProbe,
    severity: Severity,
    hint: &'static str,
}

pub fn execute() -> Result<()> {
    let checks = [
        Check {
            probe: yt_dlp::probe_yt_dlp(),
            severity: Severity::Required,
            hint: "Install yt-dlp: https://github.com/yt-dlp/yt-dlp#installation",
        },
        Check {
            probe: yt_dlp::probe_ffmpeg(),
            severity: Severity::Required,
            hint: "Install ffmpeg (needed for merge/embed). See https://ffmpeg.org/download.html",
        },
        Check {
            probe: yt_dlp::probe_deno(),
            severity: Severity::RequiredForDownload,
            hint: "Install deno (needed for download / EJS): https://deno.land/",
        },
    ];

    let mut out = String::new();
    render_report(&mut out, &checks, io::stdout().is_terminal());
    print!("{out}");
    let _ = io::stdout().flush();

    if checks.iter().any(|c| !c.probe.ok) {
        bail!("doctor found failing checks; install missing tools and re-run");
    }

    Ok(())
}

fn render_report(out: &mut String, checks: &[Check], bold_sections: bool) {
    out.push_str("midnite-archive doctor\n\n");
    push_section(out, "tools", bold_sections);

    for check in checks {
        render_check(out, check);
    }
}

fn push_section(out: &mut String, name: &str, bold: bool) {
    if bold {
        out.push_str("\x1b[1m");
        out.push_str(name);
        out.push_str(":\x1b[0m\n");
    } else {
        out.push_str(name);
        out.push_str(":\n");
    }
}

fn render_check(out: &mut String, check: &Check) {
    let probe = &check.probe;
    let status = if probe.ok { "ok" } else { "FAIL" };
    let version = probe.version.as_deref().unwrap_or("-");
    let path = probe
        .path
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "-".to_string());

    out.push_str(&format!("  {}:\n", probe.name));
    out.push_str(&format!("    status:  {status}\n"));
    out.push_str(&format!("    version: {version}\n"));
    out.push_str(&format!("    path:    {path}\n"));

    if matches!(check.severity, Severity::RequiredForDownload) {
        out.push_str("    note:    needed for download\n");
    }

    if !probe.ok {
        if let Some(err) = &probe.error {
            out.push_str(&format!("    error:   {err}\n"));
            tracing::info!("{}: {}", probe.name, err);
        }
        out.push_str(&format!("    hint:    {}\n", check.hint));
    } else {
        tracing::info!("{} ok version={} path={}", probe.name, version, path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yt_dlp::ToolProbe;
    use std::path::PathBuf;

    fn ok_check(name: &'static str, version: &str, path: &str, severity: Severity) -> Check {
        Check {
            probe: ToolProbe {
                name,
                ok: true,
                version: Some(version.into()),
                path: Some(PathBuf::from(path)),
                error: None,
            },
            severity,
            hint: "unused",
        }
    }

    #[test]
    fn render_report_sectioned_plain() {
        let checks = [
            ok_check(
                "yt-dlp",
                "2026.07.04",
                "/home/ops/.local/share/mise/installs/yt-dlp/latest/yt-dlp",
                Severity::Required,
            ),
            ok_check(
                "ffmpeg",
                "4.1.11-0+deb10u1",
                "/usr/bin/ffmpeg",
                Severity::Required,
            ),
            ok_check(
                "deno",
                "2.9.2",
                "/home/ops/.local/share/mise/installs/deno/2.9.2/bin/deno",
                Severity::RequiredForDownload,
            ),
        ];

        let mut out = String::new();
        render_report(&mut out, &checks, false);

        assert_eq!(
            out,
            "\
midnite-archive doctor

tools:
  yt-dlp:
    status:  ok
    version: 2026.07.04
    path:    /home/ops/.local/share/mise/installs/yt-dlp/latest/yt-dlp
  ffmpeg:
    status:  ok
    version: 4.1.11-0+deb10u1
    path:    /usr/bin/ffmpeg
  deno:
    status:  ok
    version: 2.9.2
    path:    /home/ops/.local/share/mise/installs/deno/2.9.2/bin/deno
    note:    needed for download
"
        );
    }

    #[test]
    fn render_report_fail_includes_error_and_hint() {
        let checks = [Check {
            probe: ToolProbe {
                name: "yt-dlp",
                ok: false,
                version: None,
                path: None,
                error: Some("yt-dlp not found on PATH".into()),
            },
            severity: Severity::Required,
            hint: "Install yt-dlp: https://github.com/yt-dlp/yt-dlp#installation",
        }];

        let mut out = String::new();
        render_report(&mut out, &checks, false);

        assert!(out.contains("status:  FAIL"));
        assert!(out.contains("error:   yt-dlp not found on PATH"));
        assert!(out.contains("hint:    Install yt-dlp:"));
        assert!(!out.contains("\x1b["));
    }

    #[test]
    fn bold_section_uses_ansi_when_requested() {
        let mut out = String::new();
        push_section(&mut out, "tools", true);
        assert_eq!(out, "\x1b[1mtools:\x1b[0m\n");
    }
}
