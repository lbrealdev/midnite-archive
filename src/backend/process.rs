//! Midnite-owned streaming process runner for yt-dlp downloads.
//!
//! Used only by the two streaming download paths. Pipes and drains both
//! stdout and stderr to EOF via cancel-safe `tokio::select!`.
//!
//! **Unix cancel:** the child is spawned in its own process group
//! (`process_group(0)`). Cancel sends `SIGTERM` to the group, drains for a
//! 3s grace period, escalates to `SIGKILL`, then reaps. `ESRCH` is treated
//! as success (idempotent). `kill_on_drop(true)` is a drop/panic backstop.
//!
//! **Windows limitation:** cancel uses `Child::start_kill()` on the direct
//! child only (no job objects). Grandchildren are not guaranteed to die.
//! CI is linux-musl.

use crate::backend::events::{YtDlpEvent, classify_and_emit, classify_and_emit_stderr};
use anyhow::{Context, Result, anyhow};
use std::collections::VecDeque;
use std::ffi::OsStr;
use std::future::Future;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufRead, AsyncBufReadExt, BufReader};
use tokio::process::{Child, ChildStderr, ChildStdout, Command};
use tokio::time::timeout;

const STDERR_TAIL: usize = 20;
const CANCEL_GRACE: Duration = Duration::from_secs(3);

/// Outcome of a streaming run that exited without a spawn/IO/`wait` error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RunOutcome {
    Success,
    Cancelled,
}

/// Spawn `program` with `args`, stream stdout/stderr until EOF, and honour
/// `cancel`. Nonzero exit becomes `Err` with the last 20 stderr lines.
pub(crate) async fn run_streaming<A, I, S, C, F>(
    program: A,
    args: I,
    cancel: C,
    on_event: F,
) -> Result<RunOutcome>
where
    A: AsRef<OsStr>,
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
    C: Future<Output = ()>,
    F: FnMut(YtDlpEvent),
{
    let mut cmd = Command::new(program.as_ref());
    cmd.args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("LC_ALL", "en_US.UTF-8")
        .env("PYTHONIOENCODING", "utf-8")
        .kill_on_drop(true);

    #[cfg(unix)]
    {
        cmd.process_group(0);
    }

    let mut child = cmd
        .spawn()
        .with_context(|| format!("failed to spawn {}", program.as_ref().to_string_lossy()))?;

    let child_pid = child.id();
    let result = run_streaming_inner(&mut child, cancel, on_event).await;
    // Every exit path must signal the process group: `kill_on_drop` SIGKILLs
    // only the direct child, leaving deno/ffmpeg grandchildren alive.
    let _ = signal_kill_captured(&mut child, child_pid);
    let _ = child.wait().await;
    result
}

/// Streaming loop + wait. The outer function always group-kills and reaps.
async fn run_streaming_inner<C, F>(
    child: &mut Child,
    cancel: C,
    mut on_event: F,
) -> Result<RunOutcome>
where
    C: Future<Output = ()>,
    F: FnMut(YtDlpEvent),
{
    let mut stdout = BufReader::new(
        child
            .stdout
            .take()
            .context("stdout pipe missing after spawn")?,
    );
    let mut stderr = BufReader::new(
        child
            .stderr
            .take()
            .context("stderr pipe missing after spawn")?,
    );
    let mut stdout_done = false;
    let mut stderr_done = false;
    let mut stderr_tail: VecDeque<String> = VecDeque::new();

    tokio::pin!(cancel);

    let cancelled = loop {
        tokio::select! {
            _ = &mut cancel => break true,
            line = read_line_lossy(&mut stdout), if !stdout_done => {
                match line.context("reading stdout")? {
                    Some(line) => on_event(classify_and_emit(&line)),
                    None => stdout_done = true,
                }
            }
            line = read_line_lossy(&mut stderr), if !stderr_done => {
                match line.context("reading stderr")? {
                    Some(line) => {
                        on_event(classify_and_emit_stderr(&line));
                        push_tail(&mut stderr_tail, line);
                    }
                    None => stderr_done = true,
                }
            }
            // `else` would never fire: `cancel` stays enabled (e.g. pending()).
            _ = std::future::ready(()), if stdout_done && stderr_done => break false,
        }
    };

    if cancelled {
        cancel_and_reap(
            child,
            &mut stdout,
            &mut stderr,
            &mut stdout_done,
            &mut stderr_done,
            &mut stderr_tail,
            &mut on_event,
        )
        .await?;
        return Ok(RunOutcome::Cancelled);
    }

    let status = child.wait().await.context("waiting for process")?;
    if status.success() {
        Ok(RunOutcome::Success)
    } else {
        Err(exit_error(status.code(), &stderr_tail))
    }
}

/// Read one line of bytes, decode with lossy UTF-8. `None` at EOF.
/// A final chunk without a trailing newline is still returned as a line.
async fn read_line_lossy<R>(reader: &mut R) -> std::io::Result<Option<String>>
where
    R: AsyncBufRead + Unpin,
{
    let mut buf = Vec::new();
    let n = reader.read_until(b'\n', &mut buf).await?;
    if n == 0 {
        return Ok(None);
    }
    if buf.last() == Some(&b'\n') {
        buf.pop();
        if buf.last() == Some(&b'\r') {
            buf.pop();
        }
    }
    Ok(Some(String::from_utf8_lossy(&buf).into_owned()))
}

fn push_tail(tail: &mut VecDeque<String>, line: String) {
    if tail.len() == STDERR_TAIL {
        tail.pop_front();
    }
    tail.push_back(line);
}

fn exit_error(code: Option<i32>, tail: &VecDeque<String>) -> anyhow::Error {
    let tail_text = tail.iter().cloned().collect::<Vec<_>>().join("\n");
    anyhow!("process exited with status {code:?}\n{tail_text}")
}

async fn drain_pipes(
    stdout: &mut BufReader<ChildStdout>,
    stderr: &mut BufReader<ChildStderr>,
    stdout_done: &mut bool,
    stderr_done: &mut bool,
    stderr_tail: &mut VecDeque<String>,
    on_event: &mut impl FnMut(YtDlpEvent),
) {
    loop {
        tokio::select! {
            line = read_line_lossy(stdout), if !*stdout_done => {
                match line {
                    Ok(Some(line)) => on_event(classify_and_emit(&line)),
                    _ => *stdout_done = true,
                }
            }
            line = read_line_lossy(stderr), if !*stderr_done => {
                match line {
                    Ok(Some(line)) => {
                        on_event(classify_and_emit_stderr(&line));
                        push_tail(stderr_tail, line);
                    }
                    _ => *stderr_done = true,
                }
            }
            else => break,
        }
    }
}

async fn cancel_and_reap(
    child: &mut Child,
    stdout: &mut BufReader<ChildStdout>,
    stderr: &mut BufReader<ChildStderr>,
    stdout_done: &mut bool,
    stderr_done: &mut bool,
    stderr_tail: &mut VecDeque<String>,
    on_event: &mut impl FnMut(YtDlpEvent),
) -> Result<()> {
    signal_term(child)?;

    let _ = timeout(
        CANCEL_GRACE,
        drain_pipes(
            stdout,
            stderr,
            stdout_done,
            stderr_done,
            stderr_tail,
            on_event,
        ),
    )
    .await;

    match timeout(CANCEL_GRACE, child.wait()).await {
        Ok(status) => {
            status.context("reaping cancelled process")?;
        }
        Err(_elapsed) => {
            signal_kill(child)?;
            child.wait().await.context("reaping cancelled process")?;
        }
    }
    Ok(())
}

#[cfg(unix)]
fn signal_process_group(pid: u32, sig: i32) -> std::io::Result<()> {
    let rc = unsafe { libc::kill(-(pid as i32), sig) };
    if rc == 0 {
        return Ok(());
    }
    let err = std::io::Error::last_os_error();
    if err.raw_os_error() == Some(libc::ESRCH) {
        Ok(())
    } else {
        Err(err)
    }
}

#[cfg(unix)]
fn signal_term(child: &mut Child) -> Result<()> {
    let Some(pid) = child.id() else {
        return Ok(());
    };
    signal_process_group(pid, libc::SIGTERM).context("SIGTERM process group")?;
    Ok(())
}

#[cfg(unix)]
fn signal_kill(child: &mut Child) -> Result<()> {
    signal_kill_captured(child, None)
}

/// SIGKILL the process group. `captured_pid` is used after `wait()` has
/// already reaped the child (`Child::id()` then returns `None`).
fn signal_kill_captured(child: &mut Child, captured_pid: Option<u32>) -> Result<()> {
    #[cfg(unix)]
    {
        let Some(pid) = child.id().or(captured_pid) else {
            return Ok(());
        };
        signal_process_group(pid, libc::SIGKILL).context("SIGKILL process group")?;
        Ok(())
    }
    #[cfg(windows)]
    {
        let _ = captured_pid;
        signal_kill(child)
    }
}

/// Windows: `start_kill` terminates the direct child only (no job object).
#[cfg(windows)]
fn signal_term(child: &mut Child) -> Result<()> {
    start_kill_idempotent(child)
}

#[cfg(windows)]
fn signal_kill(child: &mut Child) -> Result<()> {
    start_kill_idempotent(child)
}

#[cfg(windows)]
fn start_kill_idempotent(child: &mut Child) -> Result<()> {
    match child.start_kill() {
        Ok(()) => Ok(()),
        Err(e)
            if matches!(
                e.kind(),
                std::io::ErrorKind::InvalidInput | std::io::ErrorKind::NotFound
            ) =>
        {
            Ok(())
        }
        Err(e) => Err(e).context("start_kill child"),
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use tokio::sync::oneshot;

    fn collect() -> (impl FnMut(YtDlpEvent), Arc<Mutex<Vec<YtDlpEvent>>>) {
        let events = Arc::new(Mutex::new(Vec::new()));
        let slot = Arc::clone(&events);
        let on_event = move |event: YtDlpEvent| {
            slot.lock().expect("events mutex").push(event);
        };
        (on_event, events)
    }

    fn pid_alive(pid: u32) -> bool {
        let rc = unsafe { libc::kill(pid as i32, 0) };
        if rc == 0 {
            return true;
        }
        std::io::Error::last_os_error().raw_os_error() == Some(libc::EPERM)
    }

    async fn poll_until(mut pred: impl FnMut() -> bool, iters: u32) -> bool {
        for _ in 0..iters {
            if pred() {
                return true;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        pred()
    }

    #[tokio::test]
    async fn success_streams_stdout_and_stderr() {
        let (on_event, events) = collect();
        let outcome = run_streaming(
            "/bin/sh",
            [
                "-c",
                "echo '[download]  12.3% of 50.00MiB at 1.00MiB/s ETA 00:50'; echo 'WARNING: x' >&2",
            ],
            std::future::pending::<()>(),
            on_event,
        )
        .await
        .expect("run");
        assert_eq!(outcome, RunOutcome::Success);
        let events = events.lock().expect("events mutex");
        assert!(
            events
                .iter()
                .any(|e| matches!(e, YtDlpEvent::Progress { percent: Some(p), .. } if (*p - 12.3).abs() < 1e-4)),
            "missing progress: {events:?}"
        );
        assert!(
            events
                .iter()
                .any(|e| matches!(e, YtDlpEvent::Log { raw } if raw.contains("WARNING: x"))),
            "missing stderr log: {events:?}"
        );
    }

    #[tokio::test]
    async fn nonzero_exit_attaches_last_20_stderr_lines() {
        let script =
            "n=1; while [ \"$n\" -le 25 ]; do echo \"line $n\" >&2; n=$((n+1)); done; exit 7";
        let err = run_streaming(
            "/bin/sh",
            ["-c", script],
            std::future::pending::<()>(),
            |_| {},
        )
        .await
        .expect_err("nonzero exit");
        let msg = format!("{err:#}");
        assert!(msg.contains("status Some(7)"), "status missing: {msg}");
        assert!(msg.contains("line 6"), "expected first kept line: {msg}");
        assert!(msg.contains("line 25"), "expected last line: {msg}");
        assert!(
            !msg.contains("line 5\n"),
            "line 5 should have aged out: {msg}"
        );
    }

    #[tokio::test]
    async fn stderr_flood_does_not_deadlock() {
        let script = "n=0; while [ \"$n\" -lt 4000 ]; do echo pad-line-$n-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx >&2; n=$((n+1)); done; exit 1";
        let result = tokio::time::timeout(
            Duration::from_secs(10),
            run_streaming(
                "/bin/sh",
                ["-c", script],
                std::future::pending::<()>(),
                |_| {},
            ),
        )
        .await
        .expect("timed out draining stderr (pipe-fill deadlock?)");
        assert!(result.is_err(), "expected nonzero exit, got {result:?}");
    }

    #[tokio::test]
    async fn cancel_before_first_line() {
        let (on_event, events) = collect();
        let result = tokio::time::timeout(
            Duration::from_secs(5),
            run_streaming(
                "/bin/sh",
                ["-c", "sleep 30; echo first-line"],
                std::future::ready(()),
                on_event,
            ),
        )
        .await
        .expect("cancel-before-first-line timed out");
        assert_eq!(result.expect("run"), RunOutcome::Cancelled);
        let events = events.lock().expect("events mutex");
        assert!(
            !events.iter().any(|e| match e {
                YtDlpEvent::Error { raw }
                | YtDlpEvent::Progress { raw, .. }
                | YtDlpEvent::Log { raw } => raw.contains("first-line"),
            }),
            "stdout after cancel: {events:?}"
        );
    }

    #[tokio::test]
    async fn cancel_kills_grandchild() {
        let dir = tempfile::tempdir().expect("tempdir");
        let pidfile = dir.path().join("grandchild.pid");
        let script = dir.path().join("orphan.sh");
        std::fs::write(
            &script,
            "#!/bin/sh\n(\n  echo $$ > \"$1\"\n  while true; do sleep 0.05; done\n) &\nwhile true; do sleep 0.05; done\n",
        )
        .expect("write script");

        let (tx, rx) = oneshot::channel();
        let pidfile_arg = pidfile.clone();
        let streaming = run_streaming(
            "/bin/sh",
            [script.as_os_str(), pidfile_arg.as_os_str()],
            async {
                let _ = rx.await;
            },
            |_| {},
        );
        tokio::pin!(streaming);

        let grandchild = loop {
            tokio::select! {
                result = &mut streaming => {
                    panic!("process ended before grandchild pid was written: {result:?}");
                }
                _ = tokio::time::sleep(Duration::from_millis(20)) => {
                    if let Ok(text) = std::fs::read_to_string(&pidfile)
                        && let Ok(pid) = text.trim().parse::<u32>()
                    {
                        break pid;
                    }
                }
            }
        };

        tx.send(()).expect("send cancel");
        let outcome = tokio::time::timeout(Duration::from_secs(5), streaming)
            .await
            .expect("cancel timed out")
            .expect("run");
        assert_eq!(outcome, RunOutcome::Cancelled);

        let dead = poll_until(|| !pid_alive(grandchild), 100).await;
        assert!(
            dead,
            "grandchild pid {grandchild} still alive after cancel; \
             manual: cargo test --features ytd-rs-backend cancel_kills_grandchild -- --nocapture"
        );
    }

    #[tokio::test]
    async fn stderr_error_line_is_error_event() {
        let (on_event, events) = collect();
        let err = run_streaming(
            "/bin/sh",
            ["-c", "echo 'ERROR: boom' >&2; exit 2"],
            std::future::pending::<()>(),
            on_event,
        )
        .await
        .expect_err("nonzero");
        let msg = format!("{err:#}");
        assert!(msg.contains("ERROR: boom"), "{msg}");
        let events = events.lock().expect("events mutex");
        assert!(
            events
                .iter()
                .any(|e| matches!(e, YtDlpEvent::Error { raw } if raw == "ERROR: boom")),
            "{events:?}"
        );
    }

    #[tokio::test]
    async fn non_utf8_stdout_does_not_abort() {
        let dir = tempfile::tempdir().expect("tempdir");
        let pidfile = dir.path().join("grandchild.pid");
        let script = dir.path().join("non_utf8.sh");
        std::fs::write(
            &script,
            "#!/bin/sh\n(\n  echo $$ > \"$1\"\n  while true; do sleep 0.05; done\n) >/dev/null 2>&1 &\nwhile [ ! -s \"$1\" ]; do sleep 0.01; done\nprintf 'a\\n\\377b\\nc\\n'\n",
        )
        .expect("write script");

        let (on_event, events) = collect();
        let outcome = run_streaming(
            "/bin/sh",
            [script.as_os_str(), pidfile.as_os_str()],
            std::future::pending::<()>(),
            on_event,
        )
        .await
        .expect("run");
        assert_eq!(outcome, RunOutcome::Success);

        {
            let events = events.lock().expect("events mutex");
            let logs: Vec<&str> = events
                .iter()
                .filter_map(|e| match e {
                    YtDlpEvent::Log { raw } => Some(raw.as_str()),
                    _ => None,
                })
                .collect();
            assert!(logs.contains(&"a"), "missing 'a': {events:?}");
            assert!(
                logs.iter().any(|l| l.contains('b')),
                "missing lossy 'b' line: {events:?}"
            );
            assert!(logs.contains(&"c"), "missing 'c': {events:?}");
        }

        let grandchild = wait_for_pidfile(&pidfile).await;
        let dead = poll_until(|| !pid_alive(grandchild), 100).await;
        assert!(
            dead,
            "grandchild pid {grandchild} still alive after non-UTF-8 run"
        );
    }

    #[tokio::test]
    async fn eof_partial_line_without_newline() {
        let dir = tempfile::tempdir().expect("tempdir");
        let pidfile = dir.path().join("grandchild.pid");
        let script = dir.path().join("partial.sh");
        std::fs::write(
            &script,
            "#!/bin/sh\n(\n  echo $$ > \"$1\"\n  while true; do sleep 0.05; done\n) >/dev/null 2>&1 &\nwhile [ ! -s \"$1\" ]; do sleep 0.01; done\nprintf 'no-newline-tail'\n",
        )
        .expect("write script");

        let (on_event, events) = collect();
        let outcome = run_streaming(
            "/bin/sh",
            [script.as_os_str(), pidfile.as_os_str()],
            std::future::pending::<()>(),
            on_event,
        )
        .await
        .expect("run");
        assert_eq!(outcome, RunOutcome::Success);

        {
            let events = events.lock().expect("events mutex");
            assert!(
                events.iter().any(|e| match e {
                    YtDlpEvent::Log { raw } => raw == "no-newline-tail",
                    _ => false,
                }),
                "missing partial line: {events:?}"
            );
        }

        let grandchild = wait_for_pidfile(&pidfile).await;
        let dead = poll_until(|| !pid_alive(grandchild), 100).await;
        assert!(
            dead,
            "grandchild pid {grandchild} still alive after partial-line run"
        );
    }

    async fn wait_for_pidfile(pidfile: &std::path::Path) -> u32 {
        for _ in 0..100 {
            if let Ok(text) = std::fs::read_to_string(pidfile)
                && let Ok(pid) = text.trim().parse::<u32>()
            {
                return pid;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        panic!("pidfile {} was never written", pidfile.display());
    }
}
