#!/bin/bash

CHANNEL_LIST_FILE_PATH="$1"

usage() {
  echo "Usage: $0 <youtube-channel-file-path>"
  exit 1
}

# Check if there enough arguments
if [ "$#" -lt 1 ]; then
  usage
fi

echo "########################################"
echo "#            YouTube Script            #"
echo "#            Download Video            #"
echo "########################################"

if [[ ! -f "$CHANNEL_LIST_FILE_PATH" ]]; then
  echo "The file $CHANNEL_LIST_FILE_PATH was not found!"
  exit 1
fi

YT_CHANNEL_LIST_FILE_FULL_PATH=$(realpath "$CHANNEL_LIST_FILE_PATH")

printf "\nYouTube channel list path: %s\n" "$YT_CHANNEL_LIST_FILE_FULL_PATH"

YT_CHANNEL_FILE_STEM=$(basename "$YT_CHANNEL_LIST_FILE_FULL_PATH")
YT_CHANNEL_DIRECTORY="${YT_CHANNEL_FILE_STEM%%-*}"

echo "Checking if videos download directory exists..."

if [[ ! -d "$YT_CHANNEL_DIRECTORY" ]]; then
  mkdir -p "$YT_CHANNEL_DIRECTORY/videos"
  echo "The $YT_CHANNEL_DIRECTORY directory has been created."
else
  mkdir "$YT_CHANNEL_DIRECTORY/videos"
  echo "$YT_CHANNEL_DIRECTORY directory already exists, creating videos directory..."
fi

echo ""
echo "Downloading from $YT_CHANNEL_FILE_STEM list..."
echo ""

YT_CHANNEL_DIRECTORY_FULL_PATH=$(realpath "$YT_CHANNEL_DIRECTORY/videos")
yt-dlp -cw -o "%(title)s-%(id)s.%(ext)s" -P "$YT_CHANNEL_DIRECTORY_FULL_PATH" -a "$YT_CHANNEL_LIST_FILE_FULL_PATH" --embed-thumbnail --write-description --embed-metadata --no-colors

echo ""
echo "Done!"
