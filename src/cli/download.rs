use crate::yt_dlp;
use anyhow::{Context, Result, bail};
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};

pub fn execute(input: &str) -> Result<()> {
    println!("→ Starting download: {}", input);

    yt_dlp::check_available()?;
    yt_dlp::check_deno_available()?;

    let url_pattern = Regex::new(r"^https?://").unwrap();

    if url_pattern.is_match(input) {
        handle_url_download(input)?;
    } else if Path::new(input).exists() {
        handle_file_download(input)?;
    } else {
        bail!(
            "Input '{}' is neither a valid URL nor an existing file.",
            input
        );
    }

    println!();
    println!("✓ Done!");

    Ok(())
}

fn handle_url_download(url: &str) -> Result<()> {
    println!();
    println!("Input type: Single YouTube URL");
    println!("YouTube URL: {}", url);

    let download_dir = PathBuf::from("downloads");

    println!();
    println!("Preparing download directory...");
    fs::create_dir_all(&download_dir)
        .with_context(|| format!("Failed to create directory: {:?}", download_dir))?;
    println!("Ready: {}", download_dir.display());

    println!();
    println!("Starting download...");
    yt_dlp::download_from_url(url, &download_dir).with_context(|| "Download failed")?;

    Ok(())
}

fn handle_file_download(input: &str) -> Result<()> {
    println!();
    println!("Input type: YouTube URL list file");

    let list_file =
        fs::canonicalize(input).with_context(|| format!("Failed to resolve path: {}", input))?;
    println!("YouTube channel list path: {}", list_file.display());

    let channel_name = extract_channel_name(&list_file);

    let download_dir = Path::new(&channel_name).join("videos");

    println!();
    println!("Preparing download directory...");
    fs::create_dir_all(&download_dir)
        .with_context(|| format!("Failed to create directory: {:?}", download_dir))?;
    println!("Ready: {}", download_dir.display());

    let file_stem = list_file
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    println!();
    println!("Downloading from {} list...", file_stem);
    println!();

    println!("Starting download...");
    yt_dlp::download_from_file(&list_file, &download_dir).with_context(|| "Download failed")?;

    Ok(())
}

fn extract_channel_name(path: &Path) -> String {
    path.file_name()
        .and_then(|n| n.to_str())
        .and_then(|name| name.split('-').next())
        .unwrap_or("unknown")
        .to_string()
}
