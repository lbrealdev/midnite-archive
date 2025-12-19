# midnite-archive

## Setup

Pre-requirements:

- yt-dlp

### yt-dlp - Default Installation Method 

// to do

### yt-dlp - Install Using **uv**

If you are using `uv`, run the following command to install `yt-dlp`:
```shell
uv tool install yt-dlp
```

Once installed, check the yt-dlp version:
```shell
yt-dlp --version
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
