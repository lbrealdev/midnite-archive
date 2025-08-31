#!/bin/bash

CHANNEL_LIST_FILE="$1"

usage() {
  echo "Usage: $0 <youtube-channel-list>.txt"
  exit 1
}

# Check if there enough arguments
if [ "$#" -lt 1 ]; then
  usage
fi

echo "########################################"
echo "#             YouTube Script           #"
echo "#          Channel List Download       #"
echo "########################################"

if [[ ! -f "$CHANNEL_LIST_FILE" ]]; then
  echo "The file $CHANNEL_LIST_FILE was not found!"
  exit 1
fi

YT_CHANNEL_LIST_FILE="$CHANNEL_LIST_FILE"

printf "\nYouTube channel list: %s\n" "$YT_CHANNEL_LIST_FILE"

# -P '~/Desktop/yt-dlp-videos/videos/'

echo "Downloading from list..."
echo ""

yt-dlp -cw -o "%(title)s-%(id)s.%(ext)s" -a "$YT_CHANNEL_LIST_FILE" --embed-thumbnail --write-description --embed-metadata

echo "Done!"
