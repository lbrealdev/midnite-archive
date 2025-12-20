# midnite-archive

## Description

This repository contains some scripts to automate downloading videos from YouTube.

For the scripts to work correctly, make sure you have `yt-dlp` installed.

## Setup

### Installation 

// to do

### Install using **uv**

If you have `uv` installed, run the following command to install `yt-dlp` as a uv tool.
```shell
uv tool install yt-dlp
```

## Usage

Generate YouTube channel videos list:
```shell
./scripts/yt/channel_list_generate.sh <youtube-channel>
```

Download videos from list:
```shell
./scripts/yt/download_video.sh <generated-list>
```

Download videos comments from list:
```shell
./scripts/yt/download_video_comments.sh <generated-list>
```

Rename downloaded videos:
```shell
./scripts/video/rename.sh <directory>
```

## Tmux

Create a new session:
```shell
tmux new -s download
```

Detach current session:
```shell
Ctrl-b + d
```

List sessions:
```shell
tmux ls
```

Reattach to session `download`:
```shell
tmux attach-session -t download
```

Delete session:
```shell
tmux kill-session -t download
```
