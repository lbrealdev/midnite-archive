# Pre-migration baseline — current CLI vs wrapper PoCs

Companion to [`RESULTS.md`](RESULTS.md) (wrapper selection for [#50](https://github.com/lbrealdev/midnite-archive/issues/50)).

This document records **build / binary / dependency** metrics for:

1. **Current production CLI** — `midnite-archive` (`src/yt_dlp.rs` via `std::process::Command`)
2. **`poc-ytd-rs`** — selected wrapper spike
3. **`poc-boul2gom`** — rejected wrapper spike (historical)

Use these numbers as the **before** snapshot when integrating `ytd-rs` into the main crate. After integration, re-run the same recipe and append an “after” row for `midnite-archive`.

## Caveats

- PoC binaries are **tiny demos**, not feature-equivalent to `midnite-archive`. Do not treat PoC size as the post-migration CLI size.
- Adding `ytd-rs` + Tokio to the main crate will increase `midnite-archive` size relative to today; the right comparison is **current CLI → CLI after adapter**, not CLI → PoC.
- These metrics are **local-binary / compile** costs. Download wall-clock time is dominated by network and yt-dlp itself; runtime parity belongs in a later harness (same flags, same archive state, median of N runs).

## Environment

| Item | Value |
|------|-------|
| Date (UTC) | 2026-07-23 |
| Host | Linux x86_64 |
| rustc | 1.97.1 (8bab26f4f 2026-07-14) |
| cargo | 1.97.1 (c980f4866 2026-06-30) |
| Profile | `cargo build --release` (clean) |
| Strip | GNU `strip` → separate stripped copy |

## Matrix

| Metric | `midnite-archive` 0.2.0 (current) | `poc-ytd-rs` 0.2.1 | `poc-boul2gom` (=2.1.0) |
|--------|-----------------------------------|-------------------|-------------------------|
| Role | Full CLI + sync `Command` yt-dlp | Wrapper PoC only | Wrapper PoC only |
| Clean release compile | **14.6 s** | 11.5 s | 67.0 s |
| Release binary size | **4,909,000 B (4.68 MiB)** | 1,459,664 B (1.39 MiB) | 12,850,480 B (12.26 MiB) |
| Stripped binary size | **3,670,256 B (3.50 MiB)** | 1,037,632 B (0.99 MiB) | 9,764,680 B (9.31 MiB) |
| `cargo tree` lines | 93 | 60 | 435 |
| Resolved packages (`cargo metadata`) | 121 | 42 | 276 |
| Direct deps | anyhow, chrono, clap, comfy-table, regex, tracing, tracing-subscriber, walkdir, which | anyhow, tokio, which, ytd-rs | anyhow, tokio, which, yt-dlp |
| Async runtime in binary | No | Yes (tokio) | Yes (tokio) |
| `--help` median startup | ~0.9 ms (5 runs) | n/a (not a CLI product) | n/a |

### Relative notes

- Current CLI is already **~3.5× larger stripped** than `poc-ytd-rs`, and **~2.7× smaller stripped** than `poc-boul2gom`.
- `poc-boul2gom` remains the heavy outlier (~9× `poc-ytd-rs` stripped; ~2.7× current CLI stripped).
- Clean compile of current CLI (~15 s) is in the same ballpark as `poc-ytd-rs` (~11 s); `boul2gom` is ~4–5× slower to compile.

## Direct dependency trees (`cargo tree --depth 1`)

### `midnite-archive`

```text
midnite-archive v0.2.0
├── anyhow
├── chrono
├── clap
├── comfy-table
├── regex
├── tracing
├── tracing-subscriber
├── walkdir
└── which
```

### `poc-ytd-rs`

```text
poc-ytd-rs v0.1.0
├── anyhow
├── tokio
├── which
└── ytd-rs
```

### `poc-boul2gom`

```text
poc-boul2gom v0.1.0
├── anyhow
├── tokio
├── which
└── yt-dlp (=2.1.0)
```

## What to measure after `ytd-rs` integration

Re-run this baseline recipe on the integrated `midnite-archive` and record:

| Metric | Why |
|--------|-----|
| Clean release compile time | Catch Tokio / graph growth |
| Release + stripped binary size | Primary size regression signal |
| Resolved package count / `cargo tree` lines | Dependency weight |
| `--help` startup | Sanity check for runtime init cost |
| Same-flags download median (N≥3) | Behavioral parity; expect ~yt-dlp-bound |
| Time to first progress line | Adapter streaming quality |
| Fail-closed empty-output check | Avoid boul2gom-style false success |

Expected outcome: modest binary/compile growth vs current CLI (Tokio + `ytd-rs`), still far below a `boul2gom`-style graph, with **no meaningful download speed win** over today’s `Command` path.

## How to reproduce

```shell
# From repo root
mise install
eval "$(mise activate bash)"   # or: mise exec -- ...

cargo clean
cargo build --release
strip -o /tmp/midnite-archive.stripped target/release/midnite-archive
stat -c 'release_bytes=%s' target/release/midnite-archive
stat -c 'stripped_bytes=%s' /tmp/midnite-archive.stripped
cargo tree --manifest-path Cargo.toml | wc -l
cargo metadata --format-version=1 --manifest-path Cargo.toml \
  | python3 -c 'import json,sys; print(len(json.load(sys.stdin)["resolve"]["nodes"]))'

cargo clean --manifest-path poc/ytd-rs/Cargo.toml
cargo build --release --manifest-path poc/ytd-rs/Cargo.toml
strip -o /tmp/poc-ytd-rs.stripped poc/ytd-rs/target/release/poc-ytd-rs
stat -c 'release_bytes=%s' poc/ytd-rs/target/release/poc-ytd-rs
stat -c 'stripped_bytes=%s' /tmp/poc-ytd-rs.stripped

cargo clean --manifest-path poc/boul2gom/Cargo.toml
cargo build --release --manifest-path poc/boul2gom/Cargo.toml
strip -o /tmp/poc-boul2gom.stripped poc/boul2gom/target/release/poc-boul2gom
stat -c 'release_bytes=%s' poc/boul2gom/target/release/poc-boul2gom
stat -c 'stripped_bytes=%s' /tmp/poc-boul2gom.stripped
```

Optional startup sample:

```shell
python3 - <<'PY'
import subprocess, time, statistics
vals=[]
for _ in range(5):
    t0=time.perf_counter()
    subprocess.run(["target/release/midnite-archive","--help"], stdout=subprocess.DEVNULL, check=True)
    vals.append(time.perf_counter()-t0)
print("median_s", statistics.median(vals), "runs", vals)
PY
```

## Relation to other docs

- [`RESULTS.md`](RESULTS.md) — wrapper bake-off decision (`ytd-rs` preferred)
- [`../docs/yt-dlp-integration.md`](../docs/yt-dlp-integration.md) — integration goals and spike acceptance criteria
- [`README.md`](README.md) — how to run the PoCs
