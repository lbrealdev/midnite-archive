#!/bin/bash

set -euo pipefail

# Get input argument, defaulting to empty if not provided
INPUT="${1:-}"

# Display usage information
usage() {
  echo "Usage: $0 <input>"
  echo "  <input> can be:"
  echo "    - A file containing YouTube URLs (one per line)"
  echo "    - A single YouTube video URL"
  echo ""
  echo "Examples:"
  echo "  $0 channel-list-url.txt"
  echo "  $0 https://www.youtube.com/watch?v=VIDEO_ID"
  exit 1
}

# Check if required commands are available
available() { command -v "$1" >/dev/null; }

# Validate arguments
if [ "$#" -lt 1 ]; then
  usage
fi

echo "########################################"
echo "#            YouTube Script            #"
echo "#            Download Video            #"
echo "########################################"

# Check if yt-dlp is available
if ! available yt-dlp; then
  echo "Error: yt-dlp is not installed. Please install it first."
  echo "Run: uv tool install yt-dlp"
  exit 1
fi

# Check if deno is available
if ! available deno; then
  echo "Error: deno is not installed. Please install it first."
  echo "Run: curl -fsSL https://deno.land/install.sh | sh"
  exit 1
fi

# Determine input type and validate
YT_DLP_ARGS=()

if [[ "$INPUT" =~ ^https?:// ]]; then
  # Input is a URL
  echo "Input type: Single YouTube URL"
  printf "YouTube URL: %s\n" "$INPUT"

  # Create a generic download directory
  DOWNLOAD_DIR="downloads"

  # yt-dlp arguments for single URL
  YT_DLP_ARGS=(
    -cw
    -o "%(title)s-%(id)s.%(ext)s"
    -P "$DOWNLOAD_DIR"
    --embed-thumbnail
    --write-description
    --embed-metadata
    --no-colors
    --remote-components ejs:npm
    --js-runtimes deno:"$(which deno)"
    "$INPUT"
  )

elif [[ -f "$INPUT" ]]; then
  # Input is a file
  echo "Input type: YouTube URL list file"

  YT_CHANNEL_LIST_FILE_FULL_PATH=$(realpath "$INPUT")
  printf "YouTube channel list path: %s\n" "$YT_CHANNEL_LIST_FILE_FULL_PATH"

  YT_CHANNEL_FILE_STEM=$(basename "$YT_CHANNEL_LIST_FILE_FULL_PATH")
  YT_CHANNEL_DIRECTORY="${YT_CHANNEL_FILE_STEM%%-*}"

  echo "Checking if videos download directory exists..."

  if [[ ! -d "$YT_CHANNEL_DIRECTORY" ]]; then
    mkdir -p "$YT_CHANNEL_DIRECTORY/videos"
    echo "The $YT_CHANNEL_DIRECTORY directory has been created."
  else
    mkdir -p "$YT_CHANNEL_DIRECTORY/videos"
    echo "$YT_CHANNEL_DIRECTORY directory already exists, creating videos directory..."
  fi

  DOWNLOAD_DIR="$YT_CHANNEL_DIRECTORY/videos"
  echo ""
  echo "Downloading from $YT_CHANNEL_FILE_STEM list..."
  echo ""

  # yt-dlp arguments for file input
  YT_DLP_ARGS=(
    -cw
    -o "%(title)s-%(id)s.%(ext)s"
    -P "$DOWNLOAD_DIR"
    -a "$YT_CHANNEL_LIST_FILE_FULL_PATH"
    --embed-thumbnail
    --write-description
    --embed-metadata
    --no-colors
    --remote-components ejs:npm
    --js-runtimes deno:"$(which deno)"
  )

else
  echo "Error: Input '$INPUT' is neither a valid URL nor an existing file."
  echo ""
  usage
fi

# Ensure download directory exists
mkdir -p "$DOWNLOAD_DIR"

# Execute yt-dlp with appropriate arguments
echo "Starting download..."
yt-dlp "${YT_DLP_ARGS[@]}"

echo ""
echo "Done!"
