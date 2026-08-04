//! `scripts/probe-hook.sh` and `scripts/probe.sh`.
//!
//! The probe is the instrument this project uses to check the adapter, so a
//! defect in the probe is a defect in every measurement taken with it. The one
//! it could have had, and the reason these tests exist, is concurrency: Claude
//! runs matching hooks in parallel, and the capture used to be `cat >>` into a
//! single shared file. A payload larger than a pipe buffer takes several writes,
//! and two interleaved streams produce lines belonging to neither payload — a
//! corrupt capture that looks like a finding about the wire.
//!
//! Every payload here is synthetic and obviously so, and every value inside one
//! is a sentinel that `show` must never print.

#![cfg(unix)]

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

/// Value material that must never reach `show`'s output.
const SENTINEL: &str = "SYNTHETIC-PAYLOAD-VALUE-MUST-NOT-BE-PRINTED";

/// Each synthetic payload is this large, so a shared-file `cat >>` would need
/// several writes to land one and could interleave with a concurrent hook.
/// 64 KiB is the usual pipe buffer and the usual `cat` block size; this is
/// comfortably past both.
const PAYLOAD_FILLER_BYTES: usize = 512 * 1024;

/// How many hook processes write at once.
const CONCURRENT_HOOKS: usize = 8;

/// A throwaway tree shaped like the repository, holding only the two scripts.
fn fake_repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("temp dir");
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"));
    std::fs::create_dir_all(dir.path().join("scripts")).expect("scripts");
    for script in ["probe.sh", "probe-hook.sh"] {
        std::fs::copy(
            repo.join("scripts").join(script),
            dir.path().join("scripts").join(script),
        )
        .expect("copy script");
    }
    dir
}

fn spool(root: &Path) -> PathBuf {
    root.join(".witnessglass/probe/payloads")
}

fn probe(root: &Path, args: &[&str]) -> Output {
    Command::new("bash")
        .arg(root.join("scripts/probe.sh"))
        .args(args)
        .output()
        .expect("run probe.sh")
}

/// One synthetic hook payload, distinguishable from every other by index.
fn synthetic_payload(index: usize) -> String {
    let filler = SENTINEL.repeat(PAYLOAD_FILLER_BYTES / SENTINEL.len() + 1);
    serde_json::json!({
        "hook_event_name": "PostToolUse",
        "session_id": format!("synthetic-probe-session-{index}"),
        "tool_use_id": format!("toolu_synthetic_probe_{index}"),
        "tool_name": format!("SyntheticProbeTool{index}"),
        "duration_ms": index,
        "tool_input": { "command": format!("{SENTINEL}-command-{index}") },
        "tool_response": { "filler": filler },
    })
    .to_string()
}

/// Every completed capture in the spool, as raw bytes, in no particular order.
fn captures(root: &Path) -> Vec<Vec<u8>> {
    let mut found = Vec::new();
    let Ok(entries) = std::fs::read_dir(spool(root)) else {
        return found;
    };
    for entry in entries {
        let path = entry.expect("entry").path();
        if path.extension().is_some_and(|ext| ext == "payload") {
            found.push(std::fs::read(&path).expect("read capture"));
        }
    }
    found
}

fn incomplete_files(root: &Path) -> Vec<PathBuf> {
    let dir = spool(root).join("incomplete");
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    entries.map(|e| e.expect("entry").path()).collect()
}

/// Drive `CONCURRENT_HOOKS` copies of the capture hook at once, each with its
/// own large payload, and return the payloads that were written.
fn capture_concurrently(root: &Path) -> Vec<String> {
    let payloads: Vec<String> = (0..CONCURRENT_HOOKS).map(synthetic_payload).collect();
    let spool = spool(root);

    let mut children: Vec<_> = (0..CONCURRENT_HOOKS)
        .map(|_| {
            Command::new("bash")
                .arg(root.join("scripts/probe-hook.sh"))
                .arg(&spool)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect("spawn probe-hook.sh")
        })
        .collect();

    // Every hook is already running before any of them is fed, so the writes
    // genuinely overlap. Feeding them one at a time would test nothing.
    let writers: Vec<_> = children
        .iter_mut()
        .zip(payloads.clone())
        .map(|(child, payload)| {
            let mut stdin = child.stdin.take().expect("stdin");
            std::thread::spawn(move || {
                stdin.write_all(payload.as_bytes()).expect("write payload");
            })
        })
        .collect();
    for writer in writers {
        writer.join().expect("writer thread");
    }

    for child in children {
        let output = child.wait_with_output().expect("wait");
        assert!(
            output.status.success(),
            "probe-hook.sh exited {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            output.stdout.is_empty(),
            "probe-hook.sh wrote to stdout, which Claude reads as a decision: {}",
            String::from_utf8_lossy(&output.stdout)
        );
    }

    payloads
}

#[test]
fn concurrent_hooks_each_produce_one_whole_parseable_capture() {
    let dir = fake_repo();
    let root = dir.path();

    let payloads = capture_concurrently(root);

    let captured = captures(root);
    assert_eq!(
        captured.len(),
        CONCURRENT_HOOKS,
        "expected one completed capture per hook invocation"
    );
    assert!(
        incomplete_files(root).is_empty(),
        "a completed run left partial captures behind"
    );

    // Independently parseable: the failure this replaced produced files that
    // held pieces of two payloads and parsed as neither.
    let mut seen_sessions = Vec::new();
    for bytes in &captured {
        let text = std::str::from_utf8(bytes).expect("capture is UTF-8");
        let payload: serde_json::Value =
            serde_json::from_str(text).expect("each capture parses on its own");
        seen_sessions.push(
            payload["session_id"]
                .as_str()
                .expect("session_id")
                .to_owned(),
        );
        // Byte-for-byte what the hook was handed. The probe's whole claim is
        // that these bytes have not been through anything.
        assert!(
            payloads.iter().any(|written| written.as_bytes() == bytes),
            "a capture does not match any payload that was written"
        );
    }
    seen_sessions.sort();
    let mut expected: Vec<String> = (0..CONCURRENT_HOOKS)
        .map(|i| format!("synthetic-probe-session-{i}"))
        .collect();
    expected.sort();
    assert_eq!(seen_sessions, expected, "a payload was lost or duplicated");
}

#[test]
fn show_reports_the_captures_without_printing_any_payload_value() {
    if which_python3().is_none() {
        eprintln!("skipping: probe.sh show needs python3, which is not on PATH");
        return;
    }
    let dir = fake_repo();
    let root = dir.path();
    capture_concurrently(root);

    let output = probe(root, &["show"]);
    assert!(
        output.status.success(),
        "probe.sh show failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8(output.stdout).expect("utf-8");

    // What it is allowed to say: hook names, key names, counts.
    assert!(text.contains("PostToolUse"), "{text}");
    assert!(
        text.contains(&format!("({CONCURRENT_HOOKS} payload(s))")),
        "{text}"
    );
    assert!(text.contains("duration_ms"), "{text}");
    assert!(text.contains("tool_response"), "{text}");
    assert!(
        text.contains(&format!("distinct tool_name values: {CONCURRENT_HOOKS}")),
        "{text}"
    );

    // What it must not say: anything that was inside a payload.
    assert!(
        !text.contains(SENTINEL),
        "show printed payload content: {text}"
    );
    assert!(
        !text.contains("SyntheticProbeTool"),
        "show named a tool, which is a payload value: {text}"
    );
    assert!(
        !text.contains("synthetic-probe-session-"),
        "show printed a session id from a payload: {text}"
    );
}

#[test]
fn a_partial_capture_is_distinguishable_from_a_completed_one() {
    if which_python3().is_none() {
        eprintln!("skipping: probe.sh show needs python3, which is not on PATH");
        return;
    }
    let dir = fake_repo();
    let root = dir.path();
    capture_concurrently(root);

    // A hook process killed mid-write leaves its temporary file behind. Staged
    // directly, because killing one at the right instant is not deterministic.
    let partial = spool(root).join("incomplete/20260804T000000Z-999-abcdef");
    std::fs::write(&partial, br#"{"hook_event_name":"PostToolUse","tool_inp"#).expect("partial");

    let text = String::from_utf8(probe(root, &["show"]).stdout).expect("utf-8");
    assert!(
        text.contains("1 incomplete capture(s)"),
        "an incomplete capture went unreported: {text}"
    );
    assert!(
        text.contains(&format!("({CONCURRENT_HOOKS} payload(s))")),
        "the partial capture was counted as a payload: {text}"
    );
    assert!(
        !text.contains("did not parse as JSON"),
        "the partial capture was parsed rather than set aside: {text}"
    );
    assert!(partial.exists(), "show removed a capture");
}

#[test]
fn clear_removes_only_what_the_spool_owns_and_says_what_it_kept() {
    let dir = fake_repo();
    let root = dir.path();
    capture_concurrently(root);

    // A capture from before the spool existed. Not this command's to delete.
    let legacy = root.join(".witnessglass/probe/raw-hooks.ndjson");
    let legacy_line = r#"{"hook_event_name":"PermissionDenied","session_id":"synthetic-legacy"}"#;
    std::fs::write(&legacy, format!("{legacy_line}\n")).expect("legacy capture");

    let output = probe(root, &["clear"]);
    assert!(output.status.success(), "clear failed");
    let text = String::from_utf8(output.stdout).expect("utf-8");
    assert!(
        text.contains(&format!("removed {CONCURRENT_HOOKS} completed capture(s)")),
        "clear did not say what it removed: {text}"
    );
    assert!(
        text.contains("KEPT:") && text.contains("raw-hooks.ndjson"),
        "clear did not say what it kept: {text}"
    );

    assert!(captures(root).is_empty(), "clear left completed captures");
    assert_eq!(
        std::fs::read_to_string(&legacy).expect("legacy survives"),
        format!("{legacy_line}\n"),
        "clear destroyed a pre-spool capture"
    );
}

#[test]
fn a_pre_spool_capture_is_still_read_rather_than_ignored() {
    if which_python3().is_none() {
        eprintln!("skipping: probe.sh show needs python3, which is not on PATH");
        return;
    }
    let dir = fake_repo();
    let root = dir.path();
    std::fs::create_dir_all(root.join(".witnessglass/probe")).expect("probe dir");
    std::fs::write(
        root.join(".witnessglass/probe/raw-hooks.ndjson"),
        format!(
            "{}\n",
            serde_json::json!({
                "hook_event_name": "SubagentStop",
                "session_id": "synthetic-legacy-session",
                "agent_transcript_path": SENTINEL,
            })
        ),
    )
    .expect("legacy capture");

    let text = String::from_utf8(probe(root, &["show"]).stdout).expect("utf-8");
    assert!(text.contains("SubagentStop  (1 payload(s))"), "{text}");
    assert!(text.contains("agent_transcript_path"), "{text}");
    assert!(text.contains("pre-spool capture still present"), "{text}");
    assert!(
        !text.contains(SENTINEL),
        "show printed a payload value: {text}"
    );
}

/// A hook installed by an earlier round points at the old single-file capture
/// path. It must spool beside that file rather than append into it.
#[test]
fn a_legacy_capture_path_is_spooled_beside_rather_than_appended_to() {
    let dir = fake_repo();
    let root = dir.path();
    let legacy = root.join(".witnessglass/probe/raw-hooks.ndjson");
    std::fs::create_dir_all(legacy.parent().expect("parent")).expect("probe dir");
    std::fs::write(&legacy, "{\"hook_event_name\":\"PostToolUse\"}\n").expect("legacy");

    let mut child = Command::new("bash")
        .arg(root.join("scripts/probe-hook.sh"))
        .arg(&legacy)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn");
    let payload = synthetic_payload(42);
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(payload.as_bytes())
        .expect("write");
    assert!(child.wait().expect("wait").success());

    assert_eq!(
        std::fs::read_to_string(&legacy).expect("legacy survives"),
        "{\"hook_event_name\":\"PostToolUse\"}\n",
        "the capture was appended to a pre-spool file shared with other hooks"
    );
    let captured = captures(root);
    assert_eq!(captured.len(), 1, "the payload was not spooled");
    assert_eq!(captured[0], payload.as_bytes(), "the payload was modified");
}

fn which_python3() -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|path| {
        std::env::split_paths(&path)
            .map(|dir| dir.join("python3"))
            .find(|candidate| candidate.is_file())
    })
}
