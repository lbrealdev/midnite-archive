# yt-dlp wrapper PoC

Tracks [#50](https://github.com/lbrealdev/midnite-archive/issues/50).

Two tiny, independent example crates. Same concrete job. No midnite-archive
command reimplementation.

| Crate | Wrapper | Manifest |
|-------|---------|----------|
| `poc/ytd-rs` | [`ytd-rs`](https://crates.io/crates/ytd-rs) | `poc/ytd-rs/Cargo.toml` |
| `poc/boul2gom` | [`yt-dlp (rs)`](https://crates.io/crates/yt-dlp) | `poc/boul2gom/Cargo.toml` |

## Prerequisites

- Rust stable
- From repo root: `mise install` (tools from [`mise.toml`](../mise.toml), including `deno` and `yt-dlp`)
- For `ytd-rs`: `yt-dlp` and `deno` must resolve on `PATH` (`which deno`, `which yt-dlp`)
- For `boul2gom`: network (crate can download yt-dlp/ffmpeg into `libs/`); Deno still useful for EJS if used

The PoC crates resolve Deno themselves once it is on `PATH` — you do not need to pass `$(which deno)` manually.

## Run

```shell
# From repo root — activate mise tools for this shell
mise install
eval "$(mise activate bash)"   # use zsh/fish equivalent if needed

just poc::build
just poc::ytdrs
just poc::ytdlp
POC_VIDEO_URL='https://...' just poc::ytdrs
just poc::clean
```

One-shot without activating the shell:

```shell
mise exec -- just poc::ytdrs
```

If Deno was installed via the official installer instead of mise, add `$HOME/.deno/bin` to `PATH` (secondary; this repo uses mise).

Optional overrides:

```shell
just poc::ytdrs 'https://www.youtube.com/watch?v=VIDEO_ID'
POC_ERROR=1 just poc::ytdrs
```

Manual cargo (without Just):

```shell
# ytd-rs
POC_OUT=out cargo run --release --manifest-path poc/ytd-rs/Cargo.toml

# boul2gom (pins yt-dlp =2.1.0; see RESULTS.md)
POC_OUT=out POC_LIBS=libs cargo run --release --manifest-path poc/boul2gom/Cargo.toml
```

## Metrics

See [RESULTS.md](RESULTS.md).

**Decision:** prefer `ytd-rs` for production integration (smaller binary, raw
yt-dlp args + EJS/Deno, MIT). Local `just poc::ytdrs` succeeded; `boul2gom`
reported a false download success with an empty output dir.

## References

- [ytd-rs](https://crates.io/crates/ytd-rs)
- [ytd-rs (source)](https://github.com/narrrl/ytd-rs)
- [yt-dlp (rs)](https://crates.io/crates/yt-dlp)
- [yt-dlp (rs) (source)](https://github.com/boul2gom/yt-dlp)
- [yt-dlp](https://github.com/yt-dlp/yt-dlp)
- [Deno](https://github.com/denoland/deno)
