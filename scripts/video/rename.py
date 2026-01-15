#!/usr/bin/env python3
"""
rename.py - Unified script for renaming video files.

Supports simple (spaces only) and advanced (special characters) renaming modes,
with options for recursive processing, dry-run, and verbosity.
"""

import argparse
import os
import re
import sys
from pathlib import Path

# Characters to replace in advanced mode
BAD_CHARS = r'[ /:：⧸\'"()[\]&|*?<>]'


def sanitize_filename_simple(filename: str) -> str:
    """Replace spaces with underscores (simple mode)."""
    return filename.replace(" ", "_")


def sanitize_filename_advanced(filename: str) -> str:
    """Replace bad characters with underscores, keeping hyphens (advanced mode)."""
    return re.sub(BAD_CHARS, "_", filename)


def rename_files(
    directory: Path,
    extensions: list,
    mode: str,
    recursive: bool,
    dry_run: bool,
    verbose: bool,
):
    """Rename files in the directory matching the extensions."""
    pattern = "**/*" if recursive else "*"
    renamed_count = 0

    sanitizer = (
        sanitize_filename_simple if mode == "simple" else sanitize_filename_advanced
    )

    for ext in extensions:
        for file_path in directory.glob(f"{pattern}.{ext}"):
            if file_path.is_file():
                new_name = sanitizer(file_path.name)
                if new_name != file_path.name:
                    new_path = file_path.parent / new_name
                    # Handle conflicts (advanced mode style)
                    if mode == "advanced":
                        counter = 1
                        while new_path.exists():
                            stem = new_path.stem
                            suffix = new_path.suffix
                            new_path = new_path.parent / f"{stem}_{counter}{suffix}"
                            counter += 1

                    if dry_run:
                        print(f"Would rename: {file_path} -> {new_path}")
                    else:
                        try:
                            file_path.rename(new_path)
                            if verbose:
                                print(f"Renamed: {file_path} -> {new_path}")
                        except OSError as e:
                            print(f"Error renaming {file_path}: {e}", file=sys.stderr)
                            continue
                    renamed_count += 1

    if dry_run:
        print(f"Dry run complete. Would rename {renamed_count} files.")
    else:
        print(f"Renamed {renamed_count} files.")


def main():
    parser = argparse.ArgumentParser(
        description="Unified script for renaming video files.",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Modes:
  simple: Replace spaces with underscores (like old rename.sh).
  advanced: Replace special characters with underscores (like old special_rename.py).

Examples:
  python3 rename.py --mode simple /path/to/videos
  python3 rename.py --mode advanced -r -v /path/to/videos
  python3 rename.py --mode advanced -n -e mkv mp4 /path/to/videos
        """,
    )
    parser.add_argument("directory", type=Path, help="Directory containing video files")
    parser.add_argument(
        "--mode",
        choices=["simple", "advanced"],
        default="advanced",
        help="Renaming mode (default: advanced)",
    )
    parser.add_argument(
        "-r",
        "--recursive",
        action="store_true",
        help="Process subdirectories recursively",
    )
    parser.add_argument(
        "-n", "--dry-run", action="store_true", help="Preview changes without renaming"
    )
    parser.add_argument(
        "-v", "--verbose", action="store_true", help="Show each rename operation"
    )
    parser.add_argument(
        "-e",
        "--extensions",
        nargs="+",
        default=["mkv", "mp4", "description"],
        help="File extensions to process (default: mkv mp4 description)",
    )

    args = parser.parse_args()

    if not args.directory.is_dir():
        print(f"Error: {args.directory} is not a directory", file=sys.stderr)
        sys.exit(1)

    print("########################################")
    print("#              Rename Tool             #")
    print("########################################")
    print(f"Directory: {args.directory}")
    print(f"Mode: {args.mode}")
    print(f"Extensions: {', '.join(args.extensions)}")
    print(f"Recursive: {'Yes' if args.recursive else 'No'}")
    print(f"Dry run: {'Yes' if args.dry_run else 'No'}")
    print()

    rename_files(
        args.directory,
        args.extensions,
        args.mode,
        args.recursive,
        args.dry_run,
        args.verbose,
    )


if __name__ == "__main__":
    main()
