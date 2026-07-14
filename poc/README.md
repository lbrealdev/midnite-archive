# yt-dlp wrapper PoC

Tracks [#50](https://github.com/lbrealdev/midnite-archive/issues/50).

Two tiny, independent example crates. Same concrete job. No midnite-archive
command reimplementation.

| Crate | Wrapper | Manifest |
|-------|---------|----------|
| `poc/ytd-rs` | [`ytd-rs`](https://crates.io/crates/ytd-rs) | `poc/ytd-rs/Cargo.toml` |
| `poc/boul2gom` | [`yt-dlp` / boul2gom](https://crates.io/crates/yt-dlp) | `poc/boul2gom/Cargo.toml` |

## Prerequisites

- Rust stable
- For `ytd-rs`: `yt-dlp` and `deno` on `PATH`
- For `boul2gom`: network (crate can download yt-dlp/ffmpeg into `libs/`); Deno still useful for EJS if used

## Run

```shell
export PATH="$HOME/.local/bin:$HOME/.deno/bin:$PATH"

# ytd-rs
POC_OUT=out cargo run --release --manifest-path poc/ytd-rs/Cargo.toml

# boul2gom (pins yt-dlp =2.1.0; see RESULTS.md)
POC_OUT=out POC_LIBS=libs cargo run --release --manifest-path poc/boul2gom/Cargo.toml
```

Optional:

```shell
POC_ERROR=1 cargo run --release --manifest-path poc/ytd-rs/Cargo.toml -- 'https://www.youtube.com/watch?v=INVALID____'
```

## Metrics

See [RESULTS.md](RESULTS.md).
