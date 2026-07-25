<!--
Apply this body to https://github.com/lbrealdev/midnite-archive/issues/64
(cloud agent token can create issues but cannot edit existing ones).
-->

## Summary

Replace the direct `std::process::Command` orchestration in `src/yt_dlp.rs` with [`ytd-rs`](https://crates.io/crates/ytd-rs), behind a midnite-archive-owned backend boundary.

This tracks the production migration after the wrapper PoC (#50).

## Context

- Decision: **use `ytd-rs`** — [`poc/RESULTS.md`](../blob/main/poc/RESULTS.md)
- Integration goals / spike criteria — [`docs/yt-dlp-integration.md`](../blob/main/docs/yt-dlp-integration.md)
- Pre-migration binary/compile baseline — [`poc/BASELINE.md`](../blob/main/poc/BASELINE.md)

## Architecture

```text
CLI / future TUI
        |
        v
midnite-archive yt_dlp facade (existing call sites)
        |
        v
YtDlpBackend (project types only)
        |
        +-- ytd-rs adapter   (feature: ytd-rs-backend)
        +-- Command path     (default during migration)
```

Do **not** expose `ytd-rs` types from CLI or domain modules.

## Decisions

| Topic | Decision |
|-------|----------|
| Health checks | Owned by `midnite-archive doctor` (#73) + shared probe helpers; CLI preflight keeps calling the same helpers |
| Tokio lifecycle | One process-wide `OnceLock` runtime, `current_thread` + `enable_all()`, facade uses `block_on`. Never call from an async context |
| Coexistence / rollback | Cargo feature `ytd-rs-backend`, **default off** during migration. Facade dispatches via `cfg`. Flip default-on after ports land; delete `Command` path after |
| MSRV | After adding deps, verify `cargo +1.85 build --release`; bump `rust-version` only with rationale. Add MSRV CI job when #66 lands |
| Cross-platform packaging (spike #8) | Documented macOS/Windows packaging path owned by #66 docs update |
| Unused `download_comments_for_video` | Port or delete with rationale in #70 (no call sites today) |

## Keep

- Existing CLI commands and UX
- Domain types (`Channel`, `ListFile`, `Video`, …)
- On-disk layout and archive path conventions
- Current yt-dlp flags/workflows (EJS/Deno, archive, comments, filters)
- System-installed `yt-dlp` / `ffmpeg` / `deno` (no auto-install)

## Out of scope

- TUI work
- Pure-Rust YouTube extractors
- Adopting `boul2gom/yt-dlp` / GPL
- CLI UX consistency backlog polish (separate from this migration)
- Expecting faster download wall-clock times (network/yt-dlp bound)

## Acceptance criteria (epic)

- [ ] All child issues closed or explicitly deferred
- [ ] `src/yt_dlp.rs` no longer shells out via raw `Command` for supported operations
- [ ] Spike criteria in `docs/yt-dlp-integration.md` satisfied
- [ ] Post-integration baseline numbers recorded in `poc/BASELINE.md`
- [ ] License metadata for `ytd-rs` confirmed before merge of the dependency
- [ ] Output parity: identical flags → identical archive entries → identical CLI archive-line-count stats
- [ ] Testing strategy: no live YouTube in CI; unit-test arg construction + path naming; local smoke via env flag
- [ ] `download_comments_for_video` disposition recorded in #70 (port or delete with rationale)

## Child issues

- [ ] #65 Confirm ytd-rs license metadata before dependency adoption
- [ ] #66 Add YtDlpBackend boundary and Tokio runtime at yt_dlp facade
- [ ] #67 Port download_from_url to ytd-rs with current flag parity
- [ ] #68 Port batch download and download-archive behavior to ytd-rs
- [ ] #69 Port generate_channel_list to ytd-rs (flat playlist + match-title)
- [ ] #70 Port comments downloads to ytd-rs
- [ ] #71 Expose cancellation-safe progress and error events from ytd-rs adapter
- [ ] #72 Re-measure post-integration baseline in poc/BASELINE.md
- [ ] #73 Add midnite-archive doctor command for dependency health checks

## Suggested order

1. #65 (license merge blocker)
2. #73 (doctor — can land independently)
3. #66 (backend boundary + Tokio + feature flag)
4. #67 → #68 (download ports)
5. #69, #70 (generate + comments)
6. #71 (progress events; best after at least one download port)
7. #72 (baseline re-measure after real CLI binary uses the adapter)
