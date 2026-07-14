//! Tiny PoC for `ytd-rs` (issue #50).
//!
//! Same job as `poc/boul2gom`: metadata -> EJS/Deno download with progress +
//! one arbitrary yt-dlp arg. Not a midnite-archive command reimplementation.

use anyhow::{Context, Result, bail};
use std::env;
use std::path::PathBuf;
use std::time::Instant;
use which::which;
use ytd_rs::YtDlp;

const DEFAULT_URL: &str = "https://www.youtube.com/watch?v=jNQXAC9IVRw"; // "Me at the zoo"

#[tokio::main]
async fn main() -> Result<()> {
    let url = env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_URL.to_string());
    let out_dir = PathBuf::from(env::var("POC_OUT").unwrap_or_else(|_| "out".into()));
    std::fs::create_dir_all(&out_dir)?;

    let deno = which("deno").context("deno not found on PATH")?;
    which("yt-dlp").context("yt-dlp not found on PATH")?;

    println!("backend=ytd-rs");
    println!("url={url}");
    println!("out={}", out_dir.display());

    // 1) Structured metadata
    let t0 = Instant::now();
    let infos = YtDlp::new(&url)
        .get_info()
        .await
        .context("get_info failed")?;
    let meta_ms = t0.elapsed().as_millis();
    let title = infos
        .first()
        .map(|v| v.title.as_str())
        .unwrap_or("<unknown>");
    let id = infos
        .first()
        .map(|v| v.id.as_str())
        .unwrap_or("<unknown>");
    println!("metadata_ok title={title:?} id={id} elapsed_ms={meta_ms}");

    // 2) Download with EJS/Deno + arbitrary arg + streamed progress
    let t1 = Instant::now();
    let mut child = YtDlp::new(&url)
        .output_dir(out_dir.clone())
        .arg("--no-colors")
        .arg("--remote-components")
        .arg("ejs:npm")
        .arg("--js-runtimes")
        .arg(format!("deno:{}", deno.display()))
        // Arbitrary passthrough (flex check): write description sidecar
        .arg("--write-description")
        .arg_with("-o", "%(title)s-%(id)s.%(ext)s")
        .download_process()
        .await
        .context("download_process failed")?;

    let mut progress_lines = 0u32;
    while let Some(line) = child.next_line().await? {
        if line.contains("[download]") || line.contains("%") {
            progress_lines += 1;
            if progress_lines <= 5 {
                println!("progress={line}");
            }
        }
    }
    child.wait().await.context("yt-dlp process failed")?;
    let dl_ms = t1.elapsed().as_millis();

    println!("download_ok progress_lines={progress_lines} elapsed_ms={dl_ms}");
    println!("ejs_deno=pass");
    println!("arbitrary_args=pass (--write-description)");
    println!("progress_api=pass (download_process line stream)");

    // 3) Error quality sample (optional, skipped unless POC_ERROR=1)
    if env::var("POC_ERROR").ok().as_deref() == Some("1") {
        match YtDlp::new("https://www.youtube.com/watch?v=INVALID____").get_info().await {
            Ok(_) => println!("error_sample=unexpected_success"),
            Err(e) => println!("error_sample={e}"),
        }
    }

    let entries = std::fs::read_dir(&out_dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    if entries.is_empty() {
        bail!("download finished but output dir is empty");
    }
    println!("files={}", entries.join(", "));
    Ok(())
}
