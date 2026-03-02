use anyhow::{bail, Context, Result};
use comfy_table::{presets::UTF8_FULL, Table};
use regex::Regex;
use std::fs;
use std::path::Path;

const BAD_CHARS_PATTERN: &str = "[ /:：⧸'\"()\\[\\]\\\\&$@#|*?<>,-]";

pub fn execute(
    directory: &Path,
    recursive: bool,
    dry_run: bool,
    verbose: bool,
    extensions: &[String],
) -> Result<()> {
    println!("→ Renaming files in: {}", directory.display());

    if !directory.is_dir() {
        bail!("Error: {} is not a directory", directory.display());
    }

    println!(
        "Extensions: {}",
        extensions
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!("Recursive: {}", if recursive { "Yes" } else { "No" });
    println!("Dry run: {}", if dry_run { "Yes" } else { "No" });
    println!();

    let renames = collect_renames(directory, extensions, recursive)?;

    if renames.is_empty() {
        println!("No files to rename.");
        return Ok(());
    }

    if dry_run {
        display_rename_table(&renames);
        println!();
        println!("✓ Dry run complete. Would rename {} files.", renames.len());
    } else {
        let mut success_count = 0;
        for (source_path, new_name) in &renames {
            let new_path = source_path.parent().unwrap().join(new_name);
            let final_path = handle_conflict(&new_path);

            match fs::rename(source_path, &final_path) {
                Ok(()) => {
                    if verbose {
                        println!(
                            "Renamed: {} -> {}",
                            source_path.display(),
                            final_path.display()
                        );
                    }
                    success_count += 1;
                }
                Err(e) => {
                    eprintln!("✗ Error renaming {}: {}", source_path.display(), e);
                }
            }
        }
        println!("✓ Renamed {} files.", success_count);
    }

    Ok(())
}

type RenameList = Vec<(std::path::PathBuf, String)>;

fn collect_renames(directory: &Path, extensions: &[String], recursive: bool) -> Result<RenameList> {
    let re = Regex::new(BAD_CHARS_PATTERN).context("Failed to compile regex")?;
    let underscore_re = Regex::new(r"_+").context("Failed to compile underscore regex")?;

    let mut renames = Vec::new();

    for ext in extensions {
        let entries: Vec<_> = if recursive {
            walkdir::WalkDir::new(directory)
                .follow_links(false)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path()
                        .extension()
                        .map(|e| e == ext.as_str())
                        .unwrap_or(false)
                        && e.path().is_file()
                })
                .map(|e| e.path().to_path_buf())
                .collect()
        } else {
            fs::read_dir(directory)
                .with_context(|| format!("Failed to read directory: {:?}", directory))?
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path()
                        .extension()
                        .map(|ex| ex == ext.as_str())
                        .unwrap_or(false)
                        && e.path().is_file()
                })
                .map(|e| e.path())
                .collect()
        };

        for file_path in entries {
            let new_name = sanitize_filename(&file_path, &re, &underscore_re);
            if new_name != file_path.name() {
                renames.push((file_path, new_name));
            }
        }
    }

    Ok(renames)
}

fn display_rename_table(renames: &RenameList) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_header(vec!["Source", "Renamed"]);

    for (source_path, new_name) in renames {
        let source_name = source_path.name();
        table.add_row(vec![&source_name, new_name]);
    }

    println!("{table}");
}

fn sanitize_filename(path: &Path, re: &Regex, underscore_re: &Regex) -> String {
    let filename = match path.file_name().and_then(|n| n.to_str()) {
        Some(name) => name,
        None => return String::new(),
    };

    let (name, ext) = match filename.rsplit_once('.') {
        Some((n, e)) => (n, e),
        None => {
            return normalize_underscores(re.replace_all(filename, "_").as_ref(), underscore_re);
        }
    };

    if let Some(last_dash_pos) = name.rfind('-') {
        let title = &name[..last_dash_pos];
        let id_part = &name[last_dash_pos..];
        let sanitized_title = re.replace_all(title, "_");
        let normalized_title = normalize_underscores(&sanitized_title, underscore_re);
        format!("{}.{}", normalized_title + id_part, ext)
    } else {
        let sanitized_name = re.replace_all(name, "_");
        let normalized_name = normalize_underscores(&sanitized_name, underscore_re);
        format!("{}.{}", normalized_name, ext)
    }
}

fn normalize_underscores(s: &str, underscore_re: &Regex) -> String {
    let collapsed = underscore_re.replace_all(s, "_");
    collapsed.trim_matches('_').to_string()
}

fn handle_conflict(path: &Path) -> std::path::PathBuf {
    if !path.exists() {
        return path.to_path_buf();
    }

    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

    let mut counter = 1;
    loop {
        let new_name = format!("{}_{}.{}", stem, counter, ext);
        let new_path = path.parent().unwrap().join(new_name);
        if !new_path.exists() {
            return new_path;
        }
        counter += 1;
    }
}

trait PathExt {
    fn name(&self) -> String;
}

impl PathExt for Path {
    fn name(&self) -> String {
        self.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string()
    }
}
