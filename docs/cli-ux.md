# CLI UX Spec

Contract for how `midnite-archive` talks to the user: stdout vs logs, path layout, and per-command success output.

This is a product/UX spec for the CLI, not a dump of tracing logs. Verbose `INFO` lines belong behind `-v`; default runs should stay short and actionable.

## Principles

1. **Quiet by default.** Without `-v`, print only what the user needs next (counts, paths, status).
2. **Verbose is opt-in.** `-v` / `-vv` / `-vvv` enable tracing (`info` / `debug` / `trace`). Progress chatter stays there.
3. **Errors are explicit.** Failures go to stderr as `Error: …` and exit non-zero.
4. **Artifacts are printed.** When a command creates files the user will pass to another command, print those paths on stdout.
5. **One summary style.** Prefer a leading `✓` line, then indented detail lines. Avoid mixing unrelated ornaments.

## Global flags

| Flag | Behavior |
|------|----------|
| `-v` | `INFO` tracing |
| `-vv` | `DEBUG` tracing |
| `-vvv` | `TRACE` tracing |
| (none) | tracing off unless `RUST_LOG` is set |

## On-disk layout

Per channel (handle or `UC…` id as directory name):

```text
{channel}/
  lists/      # generate output
  videos/     # download output (+ .archive/)
  comments/   # comments output
```

### List file naming

```text
{channel}-list-title[-filtered]-{YYYYMMDDHHMMSS}.txt
{channel}-list-url[-filtered]-{YYYYMMDDHHMMSS}.txt
```

- `-filtered` is present only when `--filter` was used.
- Title file: yt-dlp flat-playlist lines (`title-id`).
- URL file: one `https://www.youtube.com/watch?v=…` per line.

Channel names may contain hyphens (`foo-bar`). Downstream commands recover the channel from the filename by splitting on `-list`.

## Commands

### `generate`

**Input:** handle (`severo12`), `@handle`, `https://www.youtube.com/@handle`, or `https://www.youtube.com/channel/UC…`.

**Options:**

| Option | Meaning |
|--------|---------|
| `--filter REGEX` | Passed to yt-dlp `--match-title` (title regex, not `--match-filter` syntax) |

**Default success (stdout):**

```text
✓ 412 videos
  Title: severo12/lists/severo12-list-title-20260403023745.txt
  URLs: severo12/lists/severo12-list-url-20260403023745.txt
```

**Filtered success (stdout):**

```text
$ midnite-archive generate @nomotrouble123 --filter midnite
✓ Filter 'midnite' applied - 233 videos
  Title: nomotrouble123/lists/nomotrouble123-list-title-filtered-20260403023745.txt
  URLs: nomotrouble123/lists/nomotrouble123-list-url-filtered-20260403023745.txt
```

With `-v`, also expect channel resolution, fetch progress, and parse counts as `INFO` lines.

### `download`

**Input:** a list file path, or a single `http(s)` YouTube URL.

**Layout:**

- List file → `{channel}/videos/` with `.archive/{list-stem}.archive`
- Single URL → `downloads/` with `.archive/downloads.archive`

**Default success (stdout) for list downloads:**

```text
📊 Download Statistics:
   Total videos: 233
   Already downloaded: 10
   Remaining to download: 223

✓ Downloaded 5 new video(s) this session
   Progress: 15/233 videos complete
✓ Done!
```

Single-URL runs may only print `✓ Done!`.

### `comments`

**Input:** a URL list file.

**Layout:** `{channel}/comments/` (`%(id)s.comments.json`).

**Default success (stdout):**

```text
✓ Done!
```

With `-v`, expect list parsing and per-video previews as `INFO` lines.

### `rename`

**Input:** a directory of media/sidecar files.

**Options:** `-d/--dry-run`, `-r/--recursive`, `-e/--extensions`.

**Behavior notes:**

- Dry-run builds a source → renamed table.
- Today the table is emitted via tracing (`INFO`), so **dry-run is easiest to see with `-v`** until stdout is unified.

## Error shape

```text
Error: <message>
```

Exit code `1`. Prefer actionable messages (missing yt-dlp/deno, bad path, invalid channel).

## Consistency backlog

Tracked here so future CLI changes stay aligned with this spec:

- [ ] Make `download` / `comments` summaries as informative as `generate` (paths + counts on stdout by default).
- [ ] Print `rename` dry-run table on stdout (not only via `-v` tracing).
- [ ] Align status ornaments (`✓` vs stats emoji) under one style.
- [ ] Prefer channel display without forcing `@` when the input is a `UC…` channel id.
