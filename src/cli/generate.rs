use crate::types::{Channel, ChannelName, Video};
use crate::yt_dlp;
use anyhow::{Context, Result};
use chrono::Local;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

pub fn execute(channel_input: &str) -> Result<()> {
    tracing::info!("Generating video list for: {}", channel_input);

    // Parse and validate channel name using strongly-typed type
    let channel_name = ChannelName::parse(channel_input)?;
    let channel = Channel::new(channel_name);

    tracing::info!("YouTube channel name: @{}", channel.name);
    tracing::info!("YouTube channel url: {}", channel.url());

    let timestamp = get_timestamp();
    let list_dir = channel.lists_dir();

    let base_name = format!("{}-list-{}", channel.name, timestamp);
    let title_file = list_dir.join(format!(
        "{}.txt",
        base_name.replacen("list", "list-title", 1)
    ));
    let url_file = list_dir.join(format!("{}.txt", base_name.replacen("list", "list-url", 1)));

    tracing::info!("Generating output files...");
    tracing::debug!("Title file: {}", title_file.display());
    tracing::debug!("URL file: {}", url_file.display());

    tracing::debug!("Checking if {} directory exists...", channel.name);
    if !list_dir.exists() {
        fs::create_dir_all(&list_dir)
            .with_context(|| format!("Failed to create directory: {:?}", list_dir))?;
        tracing::info!("Directory created: {}", channel.name);
    } else {
        tracing::debug!("Directory already exists: {}", channel.name);
    }

    tracing::info!("Fetching channel list...");

    yt_dlp::check_available()?;

    // Generate list and get back structured video data
    let videos = yt_dlp::generate_channel_list(&channel, &title_file)
        .with_context(|| "Failed to generate channel list")?;

    tracing::info!("Fetched {} videos from channel", videos.len());

    // Write URL file using strongly-typed Video data
    generate_url_file(&videos, &url_file).with_context(|| "Failed to generate URL file")?;

    tracing::info!("Generated {} video entries", videos.len());
    tracing::info!("Done!");

    // Keep final output visible
    println!("✓ Generated {} video entries", videos.len());
    println!("  Title file: {}", title_file.display());
    println!("  URL file: {}", url_file.display());

    Ok(())
}

fn get_timestamp() -> String {
    Local::now().format("%Y%m%d%H%M%S").to_string()
}

fn generate_url_file(videos: &[Video], url_file: &PathBuf) -> Result<()> {
    let mut output = File::create(url_file)
        .with_context(|| format!("Failed to create URL file: {:?}", url_file))?;

    for video in videos {
        writeln!(output, "{}", video.url()).with_context(|| "Failed to write URL to file")?;
    }

    tracing::debug!("Written {} URLs to {}", videos.len(), url_file.display());

    Ok(())
}
