//! `scripts/check-recording.sh`.
//!
//! The script exists to answer a structural question without displaying the
//! recording, so most of what follows asserts absence: nothing on stdout, and
//! no payload marker on either stream. Absence alone would also be satisfied by
//! a script that silenced replay entirely, so the verdict on stderr is asserted
//! too — a check that says nothing is not a check.
//!
//! One test asserts a leak rather than its absence. A corrupt record's parser
//! diagnostic can quote the bytes it choked on, and pinning that down is the
//! only way the limit stays the documented size instead of quietly growing.
//!
//! Every recording here is synthetic and lives in a temporary directory. The
//! repository's own recordings are never read.

#![cfg(unix)]

mod common;

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use common::{SESSION, TOOL_CALL, ndjson, raw_record};
use witnessglass::{Event, SessionEnded, SessionStarted, ToolRequested};

/// A string that appears nowhere else, so finding it in an output stream means
/// an event body escaped.
const MARKER: &str = "SYNTHETIC-PAYLOAD-MARKER-8Q4ZK";

/// A throwaway tree with the shape the script resolves against: the script
/// itself, the built binary where it expects to find it, and somewhere to put
/// synthetic recordings.
fn fake_repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("temp dir");
    let root = dir.path();
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"));

    std::fs::create_dir_all(root.join("scripts")).expect("scripts");
    std::fs::create_dir_all(root.join("target/debug")).expect("target");
    std::fs::create_dir_all(root.join("recordings")).expect("recordings");

    std::fs::copy(
        repo.join("scripts/check-recording.sh"),
        root.join("scripts/check-recording.sh"),
    )
    .expect("copy script");
    std::fs::copy(
        env!("CARGO_BIN_EXE_witnessglass"),
        root.join("target/debug/witnessglass"),
    )
    .expect("copy binary");

    dir
}

fn binary(root: &Path) -> PathBuf {
    root.join("target/debug/witnessglass")
}

fn check(root: &Path, args: &[&str]) -> Output {
    Command::new("bash")
        .arg(root.join("scripts/check-recording.sh"))
        .args(args)
        .output()
        .expect("run check-recording.sh")
}

/// Check a recording by path, which is the only way to exercise a path
/// containing spaces honestly.
fn check_path(root: &Path, recording: &Path) -> Output {
    check(root, &[recording.to_str().expect("utf-8 path")])
}

fn code(output: &Output) -> i32 {
    output.status.code().expect("exit code, not a signal")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn write_recording(root: &Path, name: &str, contents: &str) -> PathBuf {
    let path = root.join("recordings").join(name);
    std::fs::write(&path, contents).expect("write recording");
    path
}

/// Three complete records, one of which carries the marker in its payload.
fn complete_recording() -> String {
    let records = [
        raw_record(
            1,
            "2026-08-02T18:00:00Z",
            SESSION,
            Event::SessionStarted(SessionStarted { source: None }),
        ),
        raw_record(
            2,
            "2026-08-02T18:00:01Z",
            SESSION,
            Event::ToolRequested(ToolRequested {
                tool_use_id: TOOL_CALL.to_owned(),
                tool_name: "SyntheticTool".to_owned(),
                requested_input: serde_json::json!({ "command": MARKER }),
            }),
        ),
        raw_record(
            3,
            "2026-08-02T18:00:02Z",
            SESSION,
            Event::SessionEnded(SessionEnded { reason: None }),
        ),
    ];
    ndjson(&records)
}

/// The same records followed by an unterminated fragment that also holds the
/// marker. Replay must neither parse the fragment nor echo it.
fn truncated_recording() -> String {
    let mut text = complete_recording();
    text.push_str(&format!(
        "{{\"schema_version\":2,\"session_id\":\"{SESSION}\",\"sequence\":4,\
         \"event\":{{\"tool_requested\":{{\"requested_input\":{{\"command\":\"{MARKER}\""
    ));
    text
}

/// A newline-terminated record that was written whole and cannot be understood.
/// The damage is structural — a renamed key — so the marker sitting in the same
/// record is not what the parser complains about.
fn corrupt_recording() -> String {
    complete_recording().replacen("\"tool_name\"", "\"tool_nayme\"", 1)
}

#[test]
fn a_complete_recording_checks_as_complete() {
    let dir = fake_repo();
    let root = dir.path();
    let recording = write_recording(root, "complete.ndjson", &complete_recording());

    let output = check_path(root, &recording);

    assert_eq!(code(&output), 0, "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "", "the records must not be displayed");
    assert!(
        stderr(&output).contains("recording is complete"),
        "the verdict must stay visible: {}",
        stderr(&output)
    );
    assert!(stderr(&output).contains("3 record(s)"));
}

#[test]
fn a_truncated_tail_checks_as_incomplete() {
    let dir = fake_repo();
    let root = dir.path();
    let recording = write_recording(root, "truncated.ndjson", &truncated_recording());

    let output = check_path(root, &recording);

    assert_eq!(code(&output), 2, "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "");
    assert!(
        stderr(&output).contains("INCOMPLETE"),
        "stderr: {}",
        stderr(&output)
    );
    // The intact prefix is still evidence and the summary still says how much
    // of it there was.
    assert!(stderr(&output).contains("3 record(s)"));
}

#[test]
fn a_corrupt_record_checks_as_a_failure() {
    let dir = fake_repo();
    let root = dir.path();
    let recording = write_recording(root, "corrupt.ndjson", &corrupt_recording());

    let output = check_path(root, &recording);

    assert_eq!(code(&output), 1, "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "");
    assert!(
        stderr(&output).contains("corrupt record"),
        "stderr: {}",
        stderr(&output)
    );
}

#[test]
fn an_empty_recording_checks_as_complete() {
    // Nothing was ever written, and nothing is missing from the end of nothing.
    // Pinned by a test because it is a judgement, not an accident.
    let dir = fake_repo();
    let root = dir.path();
    let recording = write_recording(root, "empty.ndjson", "");

    let output = check_path(root, &recording);

    assert_eq!(code(&output), 0, "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "");
    assert!(stderr(&output).contains("0 record(s)"));
}

#[test]
fn a_missing_recording_is_a_failure() {
    let dir = fake_repo();
    let root = dir.path();

    let output = check_path(root, &root.join("recordings/absent.ndjson"));

    assert_eq!(code(&output), 1);
    assert_eq!(stdout(&output), "");
    assert!(!stderr(&output).is_empty(), "a failure must say something");
}

#[test]
fn a_directory_in_place_of_a_recording_is_a_failure() {
    let dir = fake_repo();
    let root = dir.path();

    let output = check_path(root, &root.join("recordings"));

    assert_eq!(code(&output), 1);
    assert_eq!(stdout(&output), "");
}

#[test]
fn a_missing_binary_is_a_failure_that_names_the_fix() {
    let dir = fake_repo();
    let root = dir.path();
    let recording = write_recording(root, "complete.ndjson", &complete_recording());
    std::fs::remove_file(binary(root)).expect("remove binary");

    let output = check_path(root, &recording);

    assert_eq!(code(&output), 1);
    assert_eq!(stdout(&output), "");
    assert!(
        stderr(&output).contains("cargo build"),
        "a missing build is an operator problem with a fix: {}",
        stderr(&output)
    );
}

#[test]
fn a_binary_that_cannot_be_executed_is_a_failure() {
    let dir = fake_repo();
    let root = dir.path();
    let recording = write_recording(root, "complete.ndjson", &complete_recording());
    std::fs::set_permissions(binary(root), std::fs::Permissions::from_mode(0o644))
        .expect("drop the execute bit");

    let output = check_path(root, &recording);

    assert_eq!(code(&output), 1);
    assert_eq!(stdout(&output), "");
    assert!(stderr(&output).contains("cargo build"));
}

#[test]
fn a_replay_that_reaches_no_verdict_is_a_failure_and_says_so() {
    // replay exits 0, 1, or 2. Anything else — a signal, an OOM kill — is not a
    // statement about the recording, and must not be reported as one.
    let dir = fake_repo();
    let root = dir.path();
    let recording = write_recording(root, "complete.ndjson", &complete_recording());
    std::fs::write(binary(root), "#!/bin/sh\nexit 3\n").expect("write stub");

    let output = check_path(root, &recording);

    assert_eq!(
        code(&output),
        1,
        "the contract is 0, 1, or 2 and nothing else"
    );
    assert_eq!(stdout(&output), "");
    assert!(
        stderr(&output).contains("without reaching a verdict"),
        "a human must not read this as corruption: {}",
        stderr(&output)
    );
}

#[test]
fn no_arguments_is_refused() {
    let dir = fake_repo();
    let output = check(dir.path(), &[]);

    assert_eq!(code(&output), 1);
    assert_eq!(stdout(&output), "");
    assert!(stderr(&output).contains("exactly one recording path"));
}

#[test]
fn extra_arguments_are_refused() {
    let dir = fake_repo();
    let root = dir.path();
    let recording = write_recording(root, "complete.ndjson", &complete_recording());
    let path = recording.to_str().expect("utf-8 path");

    for args in [
        vec![path, path],
        vec![path, "--verbose"],
        // The help option must not smuggle a malformed invocation to exit 0,
        // which is the code that means "complete recording".
        vec!["--help", path],
        vec!["-h", "--extra"],
    ] {
        let output = check(root, &args);
        assert_eq!(code(&output), 1, "should have refused {args:?}");
        assert_eq!(stdout(&output), "", "refusing {args:?}");
    }
}

#[test]
fn an_empty_path_argument_is_a_failure() {
    let dir = fake_repo();
    let output = check(dir.path(), &[""]);

    assert_eq!(code(&output), 1);
    assert_eq!(stdout(&output), "");
}

#[test]
fn the_help_option_prints_usage() {
    let dir = fake_repo();

    for option in ["-h", "--help"] {
        let output = check(dir.path(), &[option]);
        assert_eq!(code(&output), 0);
        assert!(
            stdout(&output).contains("scripts/check-recording.sh <RECORDING>"),
            "{option} printed: {}",
            stdout(&output)
        );
        // The exit codes are the contract, so they are in the help text.
        assert!(stdout(&output).contains("truncated tail"));
    }
}

#[test]
fn a_path_containing_spaces_is_checked() {
    let dir = fake_repo();
    let root = dir.path();
    let recording = write_recording(
        root,
        "a recording with spaces.ndjson",
        &complete_recording(),
    );

    let output = check_path(root, &recording);

    assert_eq!(code(&output), 0, "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "");
    assert!(stderr(&output).contains("recording is complete"));
}

#[test]
fn nothing_reaches_stdout_whatever_the_recording_turns_out_to_be() {
    let dir = fake_repo();
    let root = dir.path();

    let cases = [
        ("complete.ndjson", complete_recording()),
        ("truncated.ndjson", truncated_recording()),
        ("corrupt.ndjson", corrupt_recording()),
        ("empty.ndjson", String::new()),
    ];

    for (name, contents) in cases {
        let recording = write_recording(root, name, &contents);
        let output = check_path(root, &recording);
        assert_eq!(stdout(&output), "", "{name} put records on stdout");
    }

    // And for the invocations that never reach a recording at all.
    for args in [vec![], vec!["one", "two"]] {
        assert_eq!(stdout(&check(root, &args)), "", "for args {args:?}");
    }
}

#[test]
fn a_payload_marker_reaches_neither_stdout_nor_stderr() {
    let dir = fake_repo();
    let root = dir.path();

    let cases = [
        ("complete.ndjson", complete_recording()),
        // The marker is inside the unterminated fragment here, which replay
        // must never decode, let alone report.
        ("truncated.ndjson", truncated_recording()),
        ("corrupt.ndjson", corrupt_recording()),
    ];

    for (name, contents) in cases {
        assert!(contents.contains(MARKER), "{name} fixture lost its marker");

        let recording = write_recording(root, name, &contents);
        let output = check_path(root, &recording);

        assert!(!stdout(&output).contains(MARKER), "{name} leaked on stdout");
        assert!(!stderr(&output).contains(MARKER), "{name} leaked on stderr");
    }
}

#[test]
fn a_corrupt_record_can_quote_its_own_bytes_in_a_parser_diagnostic() {
    // The documented exception, asserted rather than hoped for. A record that
    // was written whole and holds a payload value in a slot that cannot take it
    // makes the parser name the value it rejected. That diagnostic comes from
    // the shared parser, so the script cannot suppress it without becoming a
    // second opinion about what a recording says — and a check that edits the
    // parser's complaint is worse than one that admits this limit.
    //
    // Failing this test means the limit moved. Either it was fixed at the
    // parser, in which case say so and delete this; or it grew, in which case
    // the payload-silence claim in the script header and in
    // docs/claude-adapter.md is no longer accurate.
    let dir = fake_repo();
    let root = dir.path();

    let mangled =
        complete_recording().replacen("\"sequence\":2", &format!("\"sequence\":\"{MARKER}\""), 1);
    let recording = write_recording(root, "mistyped.ndjson", &mangled);

    let output = check_path(root, &recording);

    assert_eq!(code(&output), 1);
    assert_eq!(stdout(&output), "", "stdout stays silent even here");
    assert!(
        stderr(&output).contains(MARKER),
        "the known limit is that this diagnostic quotes the rejected value: {}",
        stderr(&output)
    );
}

#[test]
fn checking_a_recording_changes_nothing_on_disk() {
    let dir = fake_repo();
    let root = dir.path();

    write_recording(root, "complete.ndjson", &complete_recording());
    write_recording(root, "truncated.ndjson", &truncated_recording());
    write_recording(root, "corrupt.ndjson", &corrupt_recording());

    let before = snapshot(root);

    for name in ["complete.ndjson", "truncated.ndjson", "corrupt.ndjson"] {
        check_path(root, &root.join("recordings").join(name));
    }
    check(root, &["--help"]);
    check(root, &[]);

    // The whole tree, not just the recordings: this also says the script wrote
    // no scratch file, created no state directory, and built nothing.
    assert_eq!(before, snapshot(root), "checking must be read-only");
}

/// Every file under `root`, by relative path and contents, sorted.
fn snapshot(root: &Path) -> Vec<(String, Vec<u8>)> {
    let mut files = Vec::new();
    let mut pending = vec![root.to_path_buf()];

    while let Some(dir) = pending.pop() {
        for entry in std::fs::read_dir(&dir).expect("read dir") {
            let path = entry.expect("dir entry").path();
            if path.is_dir() {
                pending.push(path);
            } else {
                let relative = path
                    .strip_prefix(root)
                    .expect("path under root")
                    .to_string_lossy()
                    .into_owned();
                files.push((relative, std::fs::read(&path).expect("read file")));
            }
        }
    }

    files.sort();
    files
}
