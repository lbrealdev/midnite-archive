# midnite-archive

## Description

A collection of bash scripts for archiving YouTube content using yt-dlp with External JavaScript (EJS) support. Originally created for preserving the extensive discography of the band Midnite/Akae Beka, these tools can archive any YouTube channel, playlist, or individual video.

**Key Features:**
- Generate comprehensive video lists from YouTube channels
- Download videos with metadata and thumbnails
- Extract video comments and discussions
- Batch rename downloaded files
- EJS-powered reliability for handling YouTube's JavaScript challenges

**Requirements:** See [requirements](docs/requirements.md) for setup instructions.
**Troubleshooting:** Check [troubleshooting](docs/troubleshooting.md) for common issues.

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

## Mise

```shell
mise ls -l
```
