//! Tiny PoC for `yt-dlp` (boul2gom) — issue #50.
//!
//! Same concrete job as `poc/ytd-rs`: metadata + download + progress metrics.
//! Not a reimplementation of midnite-archive commands.
//!
//! Note: crates.io `yt-dlp 2.7.x` currently fails to resolve (`lofty` yanked),
//! so this PoC pins `=2.1.0` (last easily fetchable version with hooks/progress).

use anyhow::{Context, Result, bail};
use std::env;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;
use yt_dlp::Downloader;

const DEFAULT_URL: &str = "https://www.youtube.com/watch?v=jNQXAC9IVRw"; // "Me at the zoo"

#[tokio::main]
async fn main() -> Result<()> {
    let url = env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_URL.to_string());
    let out_dir = PathBuf::from(env::var("POC_OUT").unwrap_or_else(|_| "out".into()));
    let libs_dir = PathBuf::from(env::var("POC_LIBS").unwrap_or_else(|_| "libs".into()));
    std::fs::create_dir_all(&out_dir)?;
    std::fs::create_dir_all(&libs_dir)?;

    println!("backend=boul2gom/yt-dlp");
    println!("crate_version=2.1.0 (pinned; 2.7.x unresolved due to yanked lofty)");
    println!("url={url}");
    println!("out={}", out_dir.display());

    let t_install = Instant::now();
    let downloader = Downloader::with_new_binaries(&libs_dir, &out_dir)
        .await
        .context("with_new_binaries failed")?
        .build()
        .await
        .context("Downloader::build failed")?;
    println!(
        "binary_management=pass elapsed_ms={}",
        t_install.elapsed().as_millis()
    );

    let deno = which::which("deno").ok();
    println!(
        "system_deno={}",
        deno.as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "missing".into())
    );

    let t0 = Instant::now();
    let video = downloader
        .fetch_video_infos(&url)
        .await
        .context("fetch_video_infos failed")?;
    println!(
        "metadata_ok title={:?} id={} elapsed_ms={}",
        video.title,
        video.id,
        t0.elapsed().as_millis()
    );

    let progress_events = Arc::new(AtomicUsize::new(0));
    let progress_events_cb = progress_events.clone();
    let outfile = format!("{}-{}.mp4", sanitize(&video.title), video.id);

    let t1 = Instant::now();
    let _bytes = downloader
        .download_video_with_progress(&video, &outfile, move |done, total| {
            let n = progress_events_cb.fetch_add(1, Ordering::SeqCst) + 1;
            if n <= 5 {
                println!("progress=done={done} total={total}");
            }
        })
        .await
        .context("download_video_with_progress failed")?;
    let events = progress_events.load(Ordering::SeqCst);
    let dl_ms = t1.elapsed().as_millis();
    println!("download_reported file={outfile} progress_events={events} elapsed_ms={dl_ms}");

    // Honest gaps vs midnite-archive needs:
    println!("ejs_deno=unknown (no documented raw --remote-components/--js-runtimes passthrough in used API)");
    println!("arbitrary_args=fail-for-our-needs (high-level API; cannot pass --write-description / --download-archive here)");
    println!("progress_api=shape-ok (download_video_with_progress callback)");

    if env::var("POC_ERROR").ok().as_deref() == Some("1") {
        match downloader
            .fetch_video_infos("https://www.youtube.com/watch?v=INVALID____")
            .await
        {
            Ok(_) => println!("error_sample=unexpected_success"),
            Err(e) => println!("error_sample={e}"),
        }
    }

    let path = out_dir.join(&outfile);
    let any = std::fs::read_dir(&out_dir)?
        .filter_map(|e| e.ok())
        .any(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false));
    if !path.exists() && !any {
        bail!(
            "download reported success but output dir is empty \
             (progress_events={events}, elapsed_ms={dl_ms})"
        );
    }
    if events == 0 {
        bail!(
            "download reported success with zero progress events \
             (file may exist, but progress API did not fire; elapsed_ms={dl_ms})"
        );
    }
    println!("download_ok file={outfile} progress_events={events} elapsed_ms={dl_ms}");
    Ok(())
}

fn sanitize(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .take(80)
        .collect()
}
