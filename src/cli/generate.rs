use crate::types::{Channel, ChannelName, Video};
use crate::yt_dlp;
use anyhow::{Context, Result};
use chrono::Local;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

pub fn execute(channel_input: &str, filter: Option<&str>) -> Result<()> {
    tracing::info!("Generating video list for: {}", channel_input);

    if let Some(f) = filter {
        tracing::info!("Applying filter: {}", f);
    }

    // Parse and validate channel name using strongly-typed type
    let channel_name = ChannelName::parse(channel_input)?;
    let channel = Channel::new(channel_name);

    tracing::info!("YouTube channel name: @{}", channel.name);
    tracing::info!("YouTube channel url: {}", channel.url());

    let timestamp = get_timestamp();
    let list_dir = channel.lists_dir();

    let filtered_suffix = filter
        .as_ref()
        .map_or(String::new(), |_| "-filtered".to_string());
    let file_prefix = format!("{}-list-", channel.name);
    let title_file_path = list_dir.join(format!(
        "{}title{}-{}.txt",
        file_prefix, filtered_suffix, timestamp
    ));
    let url_file_path = list_dir.join(format!(
        "{}url{}-{}.txt",
        file_prefix, filtered_suffix, timestamp
    ));

    tracing::info!("Generating output files...");
    tracing::debug!("Title file: {}", title_file_path.display());
    tracing::debug!("URL file: {}", url_file_path.display());

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
    let videos = yt_dlp::generate_channel_list(&channel, &title_file_path, filter)
        .with_context(|| "Failed to generate channel list")?;

    tracing::info!("Fetched {} videos from channel", videos.len());

    // Write URL file using strongly-typed Video data
    generate_url_file(&videos, &url_file_path).with_context(|| "Failed to generate URL file")?;

    tracing::info!("Done!");

    // Keep final output visible
    if let Some(f) = filter {
        println!("✓ Filter '{}' applied - {} videos", f, videos.len());
    } else {
        println!("✓ {} videos", videos.len());
    }
    println!("  Title: {}", title_file_path.display());
    println!("  URLs: {}", url_file_path.display());

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
