# midnite-archive

## Description

The `midnite-archive` repository contains a collection of useful tools for downloading the extensive content of the band Midnite/Akae Beka from our beloved teacher and guide Vaughn Benjamin.

Follow the [requirements](docs/requirements.md) guide to check if you have the necessary tools to run the `midnite-archive` scripts correctly.

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
