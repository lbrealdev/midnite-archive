use crate::backend::read_archive_lines;
use anyhow::{Context, Result, bail};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

struct ArchiveRow {
    name: String,
    path: String,
    count: usize,
}

pub fn execute_list() -> Result<()> {
    let rows = discover_archives(Path::new("."))?;
    let mut out = String::new();
    render_list(&mut out, &rows);
    print!("{out}");
    let _ = io::stdout().flush();
    Ok(())
}

pub fn execute_show(name: &str) -> Result<()> {
    let lines = load_archive(Path::new("."), name)?;
    let mut out = String::new();
    render_show(&mut out, name, &lines);
    print!("{out}");
    let _ = io::stdout().flush();
    Ok(())
}

fn discover_archives(root: &Path) -> Result<Vec<ArchiveRow>> {
    let mut channels = Vec::new();
    let mut downloads = None;
    let entries =
        fs::read_dir(root).with_context(|| format!("Failed to read {}", root.display()))?;
    for entry in entries {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let Some(name) = entry.file_name().into_string().ok() else {
            continue;
        };
        if name == "downloads" {
            let archive_dir = root.join("downloads").join(".archive");
            if archive_dir.is_dir() {
                let count = read_archive_lines(&archive_dir)?.len();
                downloads = Some(ArchiveRow {
                    name,
                    path: relative_path(root, &archive_dir),
                    count,
                });
            }
            continue;
        }
        let archive_dir = root.join(&name).join("videos").join(".archive");
        if archive_dir.is_dir() {
            let count = read_archive_lines(&archive_dir)?.len();
            channels.push(ArchiveRow {
                name,
                path: relative_path(root, &archive_dir),
                count,
            });
        }
    }
    channels.sort_by(|left, right| left.name.cmp(&right.name));
    if let Some(downloads) = downloads {
        channels.push(downloads);
    }
    Ok(channels)
}

fn load_archive(root: &Path, name: &str) -> Result<Vec<String>> {
    let base = if name == "downloads" {
        let dir = root.join("downloads");
        if !dir.is_dir() {
            bail!("downloads directory not found");
        }
        dir
    } else {
        let dir = root.join(name);
        if !dir.is_dir() {
            bail!("channel directory '{name}' not found");
        }
        dir.join("videos")
    };
    read_archive_lines(&base.join(".archive"))
}

fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}

fn render_list(out: &mut String, rows: &[ArchiveRow]) {
    let noun = if rows.len() == 1 {
        "archive"
    } else {
        "archives"
    };
    out.push_str(&format!("✓ {} {noun}\n", rows.len()));
    for row in rows {
        out.push_str(&format!(
            "  {}: {} ({} {})\n",
            row.name,
            row.path,
            row.count,
            id_noun(row.count)
        ));
    }
}

fn render_show(out: &mut String, name: &str, lines: &[String]) {
    out.push_str(&format!(
        "✓ {} {} in {name}\n",
        lines.len(),
        id_noun(lines.len())
    ));
    for line in lines {
        out.push_str(&format!("  {line}\n"));
    }
}

fn id_noun(count: usize) -> &'static str {
    if count == 1 { "id" } else { "ids" }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn render_list_names_channels_then_counts() {
        let rows = [
            ArchiveRow {
                name: "severo12".into(),
                path: "severo12/videos/.archive".into(),
                count: 120,
            },
            ArchiveRow {
                name: "downloads".into(),
                path: "downloads/.archive".into(),
                count: 1,
            },
        ];
        let mut out = String::new();
        render_list(&mut out, &rows);
        assert_eq!(
            out,
            "\
✓ 2 archives
  severo12: severo12/videos/.archive (120 ids)
  downloads: downloads/.archive (1 id)
"
        );
    }

    #[test]
    fn render_list_empty() {
        let mut out = String::new();
        render_list(&mut out, &[]);
        assert_eq!(out, "✓ 0 archives\n");
    }

    #[test]
    fn render_show_prints_archive_lines() {
        let lines = vec!["youtube aaa".to_string(), "youtube bbb".to_string()];
        let mut out = String::new();
        render_show(&mut out, "severo12", &lines);
        assert_eq!(
            out,
            "\
✓ 2 ids in severo12
  youtube aaa
  youtube bbb
"
        );
    }

    #[test]
    fn discover_lists_channels_then_downloads() {
        let root = tempfile::tempdir().unwrap();
        let channel = root.path().join("severo12").join("videos").join(".archive");
        fs::create_dir_all(&channel).unwrap();
        fs::write(channel.join("videos.archive"), "youtube aaa\nyoutube bbb\n").unwrap();
        fs::write(channel.join("old.archive"), "youtube bbb\nyoutube ccc\n").unwrap();

        let beta = root.path().join("beta").join("videos").join(".archive");
        fs::create_dir_all(&beta).unwrap();
        fs::write(beta.join("videos.archive"), "youtube ddd\n").unwrap();

        let downloads = root.path().join("downloads").join(".archive");
        fs::create_dir_all(&downloads).unwrap();
        fs::write(downloads.join("downloads.archive"), "youtube eee\n").unwrap();

        fs::create_dir_all(root.path().join("noise")).unwrap();
        fs::create_dir_all(root.path().join("downloads").join("videos")).unwrap();

        let rows = discover_archives(root.path()).unwrap();

        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].name, "beta");
        assert_eq!(rows[0].count, 1);
        assert_eq!(rows[0].path, "beta/videos/.archive");
        assert_eq!(rows[1].name, "severo12");
        assert_eq!(rows[1].count, 3);
        assert_eq!(rows[1].path, "severo12/videos/.archive");
        assert_eq!(rows[2].name, "downloads");
        assert_eq!(rows[2].count, 1);
        assert_eq!(rows[2].path, "downloads/.archive");
    }

    #[test]
    fn show_missing_channel_errors() {
        let root = tempfile::tempdir().unwrap();
        let err = load_archive(root.path(), "severo12").unwrap_err();
        assert!(
            err.to_string()
                .contains("channel directory 'severo12' not found"),
            "{err}"
        );
    }

    #[test]
    fn show_missing_downloads_errors() {
        let root = tempfile::tempdir().unwrap();
        let err = load_archive(root.path(), "downloads").unwrap_err();
        assert!(
            err.to_string().contains("downloads directory not found"),
            "{err}"
        );
    }

    #[test]
    fn show_unions_legacy_archives_without_writing() {
        let root = tempfile::tempdir().unwrap();
        let archive = root.path().join("severo12").join("videos").join(".archive");
        fs::create_dir_all(&archive).unwrap();
        fs::write(archive.join("old.archive"), "youtube aaa\n").unwrap();

        let lines = load_archive(root.path(), "severo12").unwrap();

        assert_eq!(lines, vec!["youtube aaa".to_string()]);
        assert!(!archive.join("videos.archive").exists());
    }
}
