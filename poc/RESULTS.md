# PoC Results — yt-dlp Rust wrappers

Issue: [#50](https://github.com/lbrealdev/midnite-archive/issues/50)

Date: 2026-07-14  
Host: Linux x86_64 (cloud agent + local maintainer)  
Test URL: `https://www.youtube.com/watch?v=jNQXAC9IVRw` (“Me at the zoo”)

## Environments

| Env | Result |
|-----|--------|
| Cloud agent | Both backends hit YouTube bot-check (`Sign in to confirm you’re not a bot`). Compile/API/size metrics still stand. |
| Local maintainer | **`just poc::ytdrs` full success** (metadata + download + description sidecar + progress). **`just poc::ytdlp`** metadata OK, then reported `download_ok` with `progress_events=0` / `elapsed_ms=0` and empty `out/` → recipe failed. |

## Matrix

| Metric | `ytd-rs` 0.2.1 | `boul2gom/yt-dlp` (=2.1.0) |
|--------|----------------|----------------------------|
| Integration LOC | 97 | 120 |
| Release compile (clean) | ~23 s | ~81 s |
| Binary size (release) | 1,460,408 B | 12,844,248 B |
| Binary size (stripped) | 1,037,632 B (~1.0 MiB) | 9,756,856 B (~9.3 MiB) |
| `cargo tree` lines | 60 | 430 |
| Structured metadata API | pass (`get_info`) | pass (`fetch_video_infos`) |
| Progress API | pass (`download_process` line stream) | pass API shape; **local live: 0 events** |
| Arbitrary yt-dlp args | **pass** (`.arg` / `.arg_with`) | **fail for our needs** (high-level download API; no raw flag path used) |
| EJS + Deno flags | **pass** (can pass `--remote-components` / `--js-runtimes`) | **unknown / not exposed** in the API used here |
| yt-dlp/ffmpeg install | requires system PATH | **pass** (`with_new_binaries`, ~1.3 s here) |
| Error quality | clear yt-dlp stderr via anyhow | clear command failure + stderr; **false success risk** if empty out |
| Live YouTube fetch (cloud) | blocked (bot check) | blocked (bot check) |
| Live YouTube fetch (local) | **pass** (full download) | **fail** (claimed success, empty out) |
| License (repo file) | MIT (`LICENSE`) | GPL-3.0-only |
| License (crates.io metadata) | `unknown` | `GPL-3.0-only` |
| Latest crates.io usability | ok | **2.7.x fails resolve** (`lofty` 0.23.2/0.23.3 yanked); PoC pinned `=2.1.0` |

## Notes

### `ytd-rs`

- Smallest integration and binary by a wide margin.
- Best match for midnite-archive’s current need: keep yt-dlp flags (`--download-archive`, `--write-comments`, EJS/Deno, output templates).
- Progress is raw stdout lines — fine for CLI/TUI after light parsing.
- Local run confirmed EJS/Deno + `--write-description` + real download output.
- Does not manage binaries; Linux/macOS/Windows packaging must install yt-dlp/ffmpeg (and Deno for EJS) separately.
- Clarify crates.io license metadata with upstream (repo LICENSE is MIT).

### `boul2gom/yt-dlp`

- Stronger productized downloader: auto binary install, typed progress callbacks/hooks, richer model.
- Much heavier compile graph and ~9× larger stripped binary in this PoC.
- GPL-3.0-only would force midnite-archive GPL if linked into the shipped binary.
- Latest published versions (2.7.x) currently do not resolve on crates.io because of yanked `lofty` dependencies — a real adoption risk until fixed upstream.
- Without a documented raw-arg escape hatch in the path we exercised, it is a poor fit for preserving midnite-archive’s exact yt-dlp flags.
- Local live run showed a false-success path (API returned Ok with no progress and no files). The PoC now fails closed when `out/` is empty.

## Pre-migration baseline

For build/binary/dependency numbers that include the **current** `midnite-archive`
CLI (not only the PoCs), see [`BASELINE.md`](BASELINE.md). Re-measure that matrix
after integrating `ytd-rs` into the main crate.

## Recommendation

**Proceed with `ytd-rs` as the preferred wrapper for the next implementation issue**, behind a midnite-archive-owned backend boundary.

Revisit `boul2gom/yt-dlp` only if:

1. crates.io latest versions resolve again, and
2. raw yt-dlp argument passthrough (or equivalent) is proven for archive/comments/EJS, and
3. the project explicitly accepts GPL-3.0, and
4. the empty-output / zero-progress false-success behavior is fixed upstream.

## How to reproduce

```shell
export PATH="$HOME/.local/bin:$HOME/.deno/bin:$PATH"

just poc::build
just poc::ytdrs
just poc::ytdlp
just poc::clean
```

Or manually:

```shell
cargo clean --manifest-path poc/ytd-rs/Cargo.toml
cargo build --release --manifest-path poc/ytd-rs/Cargo.toml
POC_OUT=out cargo run --release --manifest-path poc/ytd-rs/Cargo.toml

cargo clean --manifest-path poc/boul2gom/Cargo.toml
cargo build --release --manifest-path poc/boul2gom/Cargo.toml
POC_OUT=out POC_LIBS=libs cargo run --release --manifest-path poc/boul2gom/Cargo.toml
```

Cookies may be required for YouTube in some environments; see yt-dlp FAQ on `--cookies` / `--cookies-from-browser`.
