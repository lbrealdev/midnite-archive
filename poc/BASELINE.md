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

## After integration (2026-08-26, current main)

Post-migration snapshot of production `midnite-archive` on current `main` (`0ad1189`, crate **0.3.0**). Compile timings are idle-machine, 3-run clean `cargo build --release` (median + range). `--help` is a startup sanity check only — **no download-speed or runtime-parity claims**.

### Environment

| Item | Value |
|------|-------|
| Date (UTC) | 2026-08-26 |
| Host | Debian GNU/Linux 13 (trixie), Linux 6.12.105+deb13-amd64 x86_64, **2 cores** |
| rustc | 1.90.0 (1159e78c4 2025-09-14) |
| cargo | 1.90.0 (840b83a10 2025-07-30) |
| Strip | GNU strip (GNU Binutils for Debian) 2.44 |
| Profile | `cargo build --release` (clean) |
| Toolchain vs 2026-07-23 baseline | **1.97.1 → 1.90.0** (pinned `rust-toolchain.toml`). Baseline recorded no CPU/core count, so compile-time deltas vs that row are **not directly comparable** across hosts (this machine is 2 cores). |

### Matrix

| Metric | 2026-07-23 (pre) | After (2026-08-26) | Control `082f094` same-host 1.90.0 |
|--------|------------------|--------------------|-------------------------------------|
| Role | Full CLI + sync `Command` yt-dlp (0.2.0) | Full CLI + `ytd-rs` adapter (0.3.0) | Same source as 2026-07-23 row, rebuilt here |
| Clean release compile | **14.6 s** | **86.382 s** median (n=3, wall 86.320–86.952 s; cargo Finished `1m 26s` ×3) | **64.770 s** median (n=3, wall 64.654–64.860 s; cargo Finished `1m 04s` ×3) |
| Release binary size | **4,909,000 B (4.68 MiB)** | **5,716,424 B (5.45 MiB)** | **5,064,304 B (4.83 MiB)** |
| Stripped binary size | **3,670,256 B (3.50 MiB)** | **4,375,512 B (4.17 MiB)** | **3,929,736 B (3.75 MiB)** |
| `cargo tree` lines | 93 | 135 | 93 |
| Resolved packages (`cargo metadata`) | 121 | 132 | 121 |

Set B (cheap, no compile) on `082f094` + cargo 1.90.0 matched the historical tree/resolve counts (93 / 121). Set C (clean release rebuild of that commit on this host + 1.90.0) succeeded; the Control column is that rebuild.

### Commentary

The 2026-07-23 → After compile jump (14.6 s → 86 s) is **partly toolchain/host, not a clean migration delta**: rustc **1.97.1 → 1.90.0**, and this host is 2 cores with no baseline core count recorded. Binary size on the same pre-migration source also moved under 1.90.0 (release 4,909,000 → 5,064,304 B; stripped 3,670,256 → 3,929,736 B), so the 2026-07-23 → After size delta is likewise mixed.

The defensible same-host, same-toolchain (1.90.0) **migration** figure is Control `082f094` → After:

| Metric | Control → After |
|--------|-----------------|
| Clean compile (wall median) | **+21.612 s** (64.770 → 86.382 s; Finished `1m 04s` → `1m 26s`) |
| Release size | **+652,120 B** (5,064,304 → 5,716,424) |
| Stripped size | **+445,776 B** (3,929,736 → 4,375,512) |
| `cargo tree` lines | **+42** (93 → 135) |
| Resolved packages | **+11** (121 → 132) |

That growth is consistent with adding Tokio + `ytd-rs` (and related crates) to the production CLI. Direct deps after (`cargo tree --depth 1`): anyhow, async-trait, chrono, clap, comfy-table, libc, regex, tokio, tracing, tracing-subscriber, walkdir, which, ytd-rs.

`--help` median of 5 runs (sanity check only): After 0.001625 s (1.63 ms); Control 0.001492 s (1.49 ms).

### Direct dependency tree after (`cargo tree --depth 1`)

```text
midnite-archive v0.3.0
├── anyhow v1.0.104
├── async-trait v0.1.92 (proc-macro)
├── chrono v0.4.45
├── clap v4.6.6
├── comfy-table v7.2.2
├── libc v0.2.183
├── regex v1.13.1
├── tokio v1.53.1
├── tracing v0.1.44
├── tracing-subscriber v0.3.23
├── walkdir v2.5.0
├── which v8.0.5
└── ytd-rs v0.2.1
[dev-dependencies]
└── tempfile v3.27.0
```
