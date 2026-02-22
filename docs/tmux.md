# Using tmux for Long-Running Downloads

When downloading large YouTube archives, downloads can take hours or days. Using `tmux` ensures your downloads continue even if your terminal closes or connection drops.

## What is tmux?

`tmux` is a terminal multiplexer that allows you to create persistent terminal sessions that run in the background.

## Common Commands

### Create a new session

```shell
tmux new -s download
```

### Detach from current session

```
Ctrl-b + d
```

### List all sessions

```shell
tmux ls
```

### Reattach to a session

```shell
tmux attach-session -t download
```

### Kill a session

```shell
tmux kill-session -t download
```

## Typical Workflow

1. **Start a session** before beginning downloads:
   ```shell
   tmux new -s download
   midnite download channel-list-url.txt
   ```

2. **Detach** with `Ctrl-b + d` - download continues in background

3. **Reattach later** to check progress:
   ```shell
   tmux attach-session -t download
   ```

4. **Kill session** when done:
   ```shell
   tmux kill-session -t download
   ```
