<!--
Apply this body to https://github.com/lbrealdev/midnite-archive/issues/66
(cloud agent token can create issues but cannot edit existing ones).
-->

## Parent

Part of #64. Depends on #65 (license) before merging the dependency.

## Summary

Introduce a midnite-owned backend boundary around yt-dlp operations, and add a Tokio runtime at that boundary so sync CLI call sites can use async `ytd-rs` without rewriting the whole CLI.

## Goals

- Keep current public functions / call sites in `cli/*` stable where practical
- Define project request/result/progress types (no `ytd-rs` types leaked)
- Add `ytd-rs` + minimal Tokio features behind the adapter
- Sync facade invokes async adapter via a shared runtime (see Decisions)
- Coexist with the old `Command` path behind a Cargo feature until ports land

## Decisions (pinned)

### Tokio runtime lifecycle

One process-wide runtime via `std::sync::OnceLock`, created on first use:

```rust
fn runtime() -> &'static tokio::runtime::Runtime {
    static RT: std::sync::OnceLock<tokio::runtime::Runtime> = std::sync::OnceLock::new();
    RT.get_or_init(|| {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("failed to create Tokio runtime")
    })
}
```

- Flavor: `current_thread` + `enable_all()` (subprocess I/O; no need for multi-thread today)
- Facade uses `runtime().block_on(...)`
- **Never** call `block_on` from inside an async context (panics). CLI stays sync, so this is fine — document so a future async command does not trip it

### Coexistence / rollback

Cargo feature `ytd-rs-backend`, **default off** during the migration:

- `#[cfg(not(feature = "ytd-rs-backend"))]` → existing `Command` path
- `#[cfg(feature = "ytd-rs-backend")]` → new `ytd-rs` adapter

Suggested layout:

- Facade stays in `src/yt_dlp.rs` (stable call sites for `cli/*`)
- `src/backend/command.rs` — current `Command` impl
- `src/backend/ytdrs.rs` — adapter impl

Lifecycle:

1. Ports (#67–#70) land behind the flag while it remains default-off (`main` still ships `Command`)
2. When epic criterion “no longer shells out via raw `Command`” is met, flip the feature to default-on
3. After smoke suite passes, delete `backend/command.rs` in a follow-up

Rollback = flip the feature back (one-commit revert) without deleting port code.

### MSRV

- Declared MSRV is `rust-version = "1.85"` in root `Cargo.toml`
- After adding `tokio` + `ytd-rs`, verify `cargo +1.85 build --release`
- If a transitive dep raises MSRV: pin that dep, or bump `rust-version` deliberately with rationale (and update `rust-toolchain.toml` if needed)
- Add a CI matrix / job entry on the declared MSRV so future transitive bumps are caught before release

### Doctor / diagnostics ownership

- User-facing `midnite-archive doctor` UX is **not** owned by this issue — see #73
- This issue may expose a midnite-owned `diagnostics()` helper on the backend boundary later; no `ytd-rs` types may leak from it
- Spike criterion #8 (documented macOS/Windows packaging path for system-installed yt-dlp/ffmpeg/deno) is owned by the docs update in this issue

## Non-goals

- Making all CLI commands fully async
- Changing stdout UX / command summaries
- Porting every operation in this issue (ports are separate child issues)
- Implementing the `doctor` CLI command (#73)

## Acceptance criteria

- [ ] Backend module/trait (or equivalent) exists with midnite-owned types
- [ ] `ytd-rs` and Tokio added to root `Cargo.toml` with justified feature set, behind `ytd-rs-backend`
- [ ] Shared `OnceLock` + `current_thread` runtime; sync facade uses `block_on`
- [ ] CLI / domain modules do not import `ytd-rs` types
- [ ] Old `Command` path remains the default until ports land (feature default-off)
- [ ] `cargo +1.85 build --release` passes (or MSRV bump documented with rationale)
- [ ] MSRV CI job/matrix entry added (or tracked follow-up linked here)
- [ ] macOS/Windows packaging path documented (spike criterion #8)
- [ ] `cargo test` / clippy still pass

## References

- [`docs/yt-dlp-integration.md`](../blob/main/docs/yt-dlp-integration.md)
- [`poc/ytd-rs/src/main.rs`](../blob/main/poc/ytd-rs/src/main.rs)
- Doctor command: #73
- Epic: #64
