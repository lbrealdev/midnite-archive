# midnite-archive

## Setup

// to do

## Usage

Generate YouTube channel videos list:
```shell
./scripts/yt/channel_list_generate.sh <youtube-channel>
```

Download videos from list:
```shell
./scripts/yt/channel_list_download.sh <generated-list>
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
