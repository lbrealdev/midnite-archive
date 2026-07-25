use crate::yt_dlp::{self, ToolProbe};
use anyhow::{Result, bail};

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

    println!("midnite-archive doctor");

    let mut failed_required = false;

    for check in &checks {
        print_check(check);
        if !check.probe.ok {
            failed_required = true;
        }
    }

    if failed_required {
        bail!("doctor found failing checks; install missing tools and re-run");
    }

    Ok(())
}

fn print_check(check: &Check) {
    let probe = &check.probe;
    let status = if probe.ok { "ok" } else { "FAIL" };
    let version = probe.version.as_deref().unwrap_or("-");
    let path = probe
        .path
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "-".to_string());

    let note = match check.severity {
        Severity::Required => "",
        Severity::RequiredForDownload => "  (needed for download)",
    };

    println!(
        "  {:<8} {:<4}  {}  ({}){}",
        probe.name, status, version, path, note
    );

    if !probe.ok {
        if let Some(err) = &probe.error {
            tracing::info!("{}: {}", probe.name, err);
        }
        println!("           {}", check.hint);
    } else {
        tracing::info!("{} ok version={} path={}", probe.name, version, path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yt_dlp::ToolProbe;
    use std::path::PathBuf;

    #[test]
    fn print_check_formats_ok_probe() {
        let check = Check {
            probe: ToolProbe {
                name: "yt-dlp",
                ok: true,
                version: Some("2026.01.01".into()),
                path: Some(PathBuf::from("/usr/bin/yt-dlp")),
                error: None,
            },
            severity: Severity::Required,
            hint: "unused",
        };
        // Smoke: does not panic
        print_check(&check);
    }
}
