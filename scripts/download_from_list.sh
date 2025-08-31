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
echo "#         Channel List Download        #"
echo "########################################"

if [[ ! -f "$CHANNEL_LIST_FILE_PATH" ]]; then
  echo "The file $CHANNEL_LIST_FILE_PATH was not found!"
  exit 1
fi

YT_CHANNEL_LIST_FILE_FULL_PATH=$(readlink -f "$CHANNEL_LIST_FILE_PATH")

printf "\nYouTube channel list path: %s\n" "$YT_CHANNEL_LIST_FILE_FULL_PATH"

YT_CHANNEL_FILE_STEM=$(echo "$YT_CHANNEL_LIST_FILE_FULL_PATH" | grep -oP '[^/]+$')

YT_CHANNEL_DIRECTORY="${YT_CHANNEL_FILE_STEM%%-*}"

# -P '~/Desktop/yt-dlp-videos/videos/'

echo "Checking if videos download directory exists..."

if [[ ! -d "$YT_CHANNEL_DIRECTORY/videos" ]]; then
  echo "The $YT_CHANNEL_DIRECTORY directory has been created."
  mkdir -p "$YT_CHANNEL_DIRECTORY/videos"
fi

echo "Downloading from list..."
echo ""

cd "$YT_CHANNEL_DIRECTORY/videos" || exit
yt-dlp -cw -o "%(title)s-%(id)s.%(ext)s" -a "$YT_CHANNEL_LIST_FILE" --embed-thumbnail --write-description --embed-metadata

echo "Done!"
