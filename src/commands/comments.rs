use crate::yt_dlp;
use anyhow::{Context, Result, bail};
use std::fs;
use std::path::Path;

pub fn execute(list_file: &Path) -> Result<()> {
    println!("########################################");
    println!("            YouTube Script            ");
    println!("        Download Video Comments       ");
    println!("########################################");

    yt_dlp::check_available().context("yt-dlp dependency check failed")?;

    if !list_file.exists() {
        bail!("The file {:?} was not found!", list_file);
    }

    let list_file_full_path = fs::canonicalize(list_file)
        .with_context(|| format!("Failed to resolve path: {:?}", list_file))?;

    println!();
    println!(
        "YouTube channel list path: {}",
        list_file_full_path.display()
    );

    let channel_name = extract_channel_name(&list_file_full_path);
    let comments_dir = Path::new(&channel_name).join("comments");

    println!();
    println!("Checking if {} directory exists...", channel_name);

    if !Path::new(&channel_name).exists() {
        fs::create_dir_all(&comments_dir)
            .with_context(|| format!("Failed to create directory: {:?}", comments_dir))?;
        println!("The {} directory has been created.", channel_name);
    } else {
        fs::create_dir_all(&comments_dir)
            .with_context(|| format!("Failed to create directory: {:?}", comments_dir))?;
        println!(
            "{} directory already exists, creating comments directory...",
            channel_name
        );
    }

    let file_stem = list_file_full_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    println!();
    println!("Downloading from {} list...", file_stem);
    println!();

    let output_dir_full_path = fs::canonicalize(Path::new(&channel_name).join("comments"))
        .with_context(|| "Failed to resolve output directory path")?;

    yt_dlp::download_comments(&list_file_full_path, &output_dir_full_path)
        .with_context(|| "Comments download failed")?;

    println!();
    println!("Done!");

    Ok(())
}

fn extract_channel_name(path: &Path) -> String {
    path.file_name()
        .and_then(|n| n.to_str())
        .and_then(|name| name.split('-').next())
        .unwrap_or("unknown")
        .to_string()
}
