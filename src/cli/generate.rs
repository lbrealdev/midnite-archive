use crate::yt_dlp;
use anyhow::{Context, Result, bail};
use chrono::Local;
use regex::Regex;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

const VALID_CHANNEL_PATTERN: &str = "^[a-zA-Z0-9_-]+$";

pub fn execute(channel: &str) -> Result<()> {
    println!("→ Generating video list for: {}", channel);

    let channel_name = parse_channel_name(channel);

    if !is_valid_channel_name(&channel_name) {
        bail!(
            "Invalid channel name '{}': must contain only alphanumeric characters, underscores, or hyphens",
            channel_name
        );
    }

    let timestamp = get_timestamp();
    let list_dir = Path::new(&channel_name).join("lists");

    let base_name = format!("{}-list-{}", channel_name, timestamp);
    let title_file = list_dir.join(format!(
        "{}.txt",
        base_name.replacen("list", "list-title", 1)
    ));
    let url_file = list_dir.join(format!("{}.txt", base_name.replacen("list", "list-url", 1)));

    println!();
    println!("YouTube channel name: @{}", channel_name);
    println!(
        "YouTube channel url: https://www.youtube.com/@{}",
        channel_name
    );
    println!();
    println!("Generating output files...");
    println!();
    println!("YouTube Channel file (title): {}", title_file.display());
    println!("YouTube Channel file (url): {}", url_file.display());
    println!();

    println!("Checking if {} directory exists...", channel_name);
    if !list_dir.exists() {
        fs::create_dir_all(&list_dir)
            .with_context(|| format!("Failed to create directory: {:?}", list_dir))?;
        println!("✓ Directory created: {}", channel_name);
    }

    println!();
    println!("Fetching channel list...");

    yt_dlp::check_available()?;

    yt_dlp::generate_channel_list(&channel_name, &title_file)
        .with_context(|| "Failed to generate channel list")?;

    generate_url_file(&title_file, &url_file).with_context(|| "Failed to generate URL file")?;

    println!("Testing...");

    println!();
    println!("✓ Done!");

    Ok(())
}

fn parse_channel_name(input: &str) -> String {
    let re = Regex::new(r"/@([^/]+)").unwrap();
    if let Some(caps) = re.captures(input) {
        caps[1].to_string()
    } else {
        input.strip_prefix('@').unwrap_or(input).to_string()
    }
}

fn is_valid_channel_name(name: &str) -> bool {
    let re = Regex::new(VALID_CHANNEL_PATTERN).unwrap();
    re.is_match(name)
}

fn get_timestamp() -> String {
    Local::now().format("%Y%m%d%H%M%S").to_string()
}

fn generate_url_file(title_file: &Path, url_file: &Path) -> Result<()> {
    let file = File::open(title_file)
        .with_context(|| format!("Failed to open title file: {:?}", title_file))?;
    let reader = BufReader::new(file);
    let id_regex = Regex::new(r"[A-Za-z0-9_-]{11}$").unwrap();

    let mut output = File::create(url_file)
        .with_context(|| format!("Failed to create URL file: {:?}", url_file))?;

    for line in reader.lines() {
        let line = line.context("Failed to read line from title file")?;
        if let Some(caps) = id_regex.captures(&line) {
            let video_id = &caps[0];
            let url = format!("https://www.youtube.com/watch?v={}", video_id);
            writeln!(output, "{}", url).with_context(|| "Failed to write URL to file")?;
        }
    }

    Ok(())
}
