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

# Characters to replace
BAD_CHARS = r'[ /:：⧸\'"()[\]&|*?<>,-]'


def sanitize_filename(filename: str) -> str:
    """Replace bad characters with underscores."""
    return re.sub(BAD_CHARS, "_", filename)


def rename_files(
    directory: Path,
    extensions: list,
    recursive: bool,
    dry_run: bool,
    verbose: bool,
):
    """Rename files in the directory matching the extensions."""
    pattern = "**/*" if recursive else "*"
    renamed_count = 0

    for ext in extensions:
        for file_path in directory.glob(f"{pattern}.{ext}"):
            if file_path.is_file():
                new_name = sanitize_filename(file_path.name)
                if new_name != file_path.name:
                    new_path = file_path.parent / new_name
                    # Handle conflicts
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
        description="Script for renaming video files by replacing special characters with underscores.",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  python3 rename.py /path/to/videos
  python3 rename.py -r -v /path/to/videos
  python3 rename.py -n -e mkv mp4 /path/to/videos
        """,
    )
    parser.add_argument("directory", type=Path, help="Directory containing video files")
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
    print(f"Extensions: {', '.join(args.extensions)}")
    print(f"Recursive: {'Yes' if args.recursive else 'No'}")
    print(f"Dry run: {'Yes' if args.dry_run else 'No'}")
    print()

    rename_files(
        args.directory,
        args.extensions,
        args.recursive,
        args.dry_run,
        args.verbose,
    )


if __name__ == "__main__":
    main()
