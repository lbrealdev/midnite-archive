use anyhow::{Context, Result, bail};
use std::path::Path;
use std::process::{Command, Stdio};

pub fn check_available() -> Result<()> {
    let output = Command::new("yt-dlp")
        .arg("--version")
        .output()
        .context("yt-dlp not found. Please install it first.")?;

    if !output.status.success() {
        bail!("yt-dlp --version failed");
    }

    Ok(())
}

pub fn check_deno_available() -> Result<()> {
    let output = Command::new("deno")
        .arg("--version")
        .output()
        .context("deno not found. Please install it first.")?;

    if !output.status.success() {
        bail!("deno --version failed");
    }

    Ok(())
}

pub fn generate_channel_list(channel: &str, output_file: &Path) -> Result<()> {
    let channel_url = format!("https://www.youtube.com/@{}", channel);

    let status = Command::new("yt-dlp")
        .args(["--flat-playlist", "--print", "%(title)s-%(id)s"])
        .arg(&channel_url)
        .stdout(Stdio::from(
            std::fs::File::create(output_file)
                .with_context(|| format!("Failed to create output file: {:?}", output_file))?,
        ))
        .status()
        .with_context(|| format!("Failed to run yt-dlp for channel: {}", channel))?;

    if !status.success() {
        bail!("yt-dlp failed with exit code: {:?}", status.code());
    }

    Ok(())
}

pub fn download_from_url(url: &str, output_dir: &Path) -> Result<()> {
    let deno_path = which::which("deno").context("Failed to find deno executable path")?;

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
        .args(["-P", &output_dir.to_string_lossy()])
        .arg(url)
        .status()
        .with_context(|| format!("Failed to run yt-dlp for URL: {}", url))?;

    if !status.success() {
        bail!("yt-dlp download failed with exit code: {:?}", status.code());
    }

    Ok(())
}

pub fn download_from_file(list_file: &Path, output_dir: &Path) -> Result<()> {
    let deno_path = which::which("deno").context("Failed to find deno executable path")?;

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
