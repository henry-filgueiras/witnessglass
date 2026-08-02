//! One recording, one schema version.
//!
//! v1 was frozen rather than deleted, because recordings written under it exist
//! and a flight recorder that cannot read its own older recordings is not much
//! of a flight recorder. What is refused is a recording that mixes versions.

mod common;

use common::*;
use witnessglass::record::v1;
use witnessglass::{
    Error, Event, SessionEnded, SessionStarted, Tail, append, replay_bytes, replay_file,
};

fn v1_recording() -> String {
    ndjson_v1(&[
        raw_v1_record(
            1,
            "2026-08-01T09:00:00Z",
            SESSION,
            v1::Event::SessionStarted,
        ),
        raw_v1_record(
            2,
            "2026-08-01T09:00:01Z",
            SESSION,
            v1::Event::ReportedIntent(v1::ReportedIntent {
                text: "Recorded before v2 existed.".to_owned(),
                tool_call_id: Some(TOOL_CALL.to_owned()),
            }),
        ),
        raw_v1_record(
            3,
            "2026-08-01T09:00:02Z",
            SESSION,
            v1::Event::ObservedToolStarted(v1::ObservedToolStarted {
                tool_call_id: TOOL_CALL.to_owned(),
                tool_name: "SyntheticTool".to_owned(),
                arguments: serde_json::json!({ "target": "/synthetic/example" }),
            }),
        ),
        raw_v1_record(
            4,
            "2026-08-01T09:00:03Z",
            SESSION,
            v1::Event::ObservedToolFinished(v1::ObservedToolFinished {
                tool_call_id: TOOL_CALL.to_owned(),
                outcome: v1::ToolOutcome::Succeeded,
                result: serde_json::json!({ "status": "synthetic" }),
            }),
        ),
        raw_v1_record(5, "2026-08-01T09:00:04Z", SESSION, v1::Event::SessionEnded),
    ])
}

#[test]
fn an_existing_v1_recording_still_replays() {
    let replay = replay_bytes(v1_recording().as_bytes()).expect("v1 recording should replay");

    assert_eq!(replay.schema_version, Some(1));
    assert_eq!(replay.tail, Tail::Complete);
    assert_eq!(
        kinds(&replay.records),
        vec![
            "session_started",
            "reported_intent",
            "observed_tool_started",
            "observed_tool_finished",
            "session_ended",
        ]
    );
    assert_eq!(sequences(&replay.records), vec![1, 2, 3, 4, 5]);

    // The v1 payload comes back as a v1 payload. A reader cannot pick it up
    // believing it holds v2 evidence, which matters because v1's "tool started"
    // means something v2 deliberately refuses to claim.
    for record in &replay.records {
        assert_eq!(record.schema_version(), 1);
        assert!(record.as_v2().is_none());
    }
    let v1::Event::ObservedToolStarted(started) =
        &replay.records[2].as_v1().expect("a v1 record").event
    else {
        panic!("expected observed_tool_started");
    };
    assert_eq!(started.tool_call_id, TOOL_CALL);
}

#[test]
fn a_v1_recording_round_trips_byte_for_byte() {
    // Replaying and re-rendering an old recording must not quietly rewrite it
    // into the current schema.
    let original = v1_recording();
    let replay = replay_bytes(original.as_bytes()).expect("replay");

    let mut rendered = String::new();
    for record in &replay.records {
        rendered.push_str(&serde_json::to_string(record).expect("serialize"));
        rendered.push('\n');
    }
    assert_eq!(rendered, original);
}

#[test]
fn a_v2_recording_replays() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("session.ndjson");

    append(&path, &session_started(), ts("2026-08-02T18:00:00Z")).expect("boundary");
    append(
        &path,
        &tool_requested(TOOL_CALL),
        ts("2026-08-02T18:00:01Z"),
    )
    .expect("request");
    append(
        &path,
        &tool_succeeded(TOOL_CALL),
        ts("2026-08-02T18:00:02Z"),
    )
    .expect("success");

    let replay = replay_file(&path).expect("replay");
    assert_eq!(replay.schema_version, Some(2));
    assert_eq!(
        kinds(&replay.records),
        vec!["session_started", "tool_requested", "tool_succeeded"]
    );
    assert!(replay.records.iter().all(|r| r.as_v2().is_some()));
}

#[test]
fn a_recording_that_mixes_versions_is_refused() {
    let mut recording = v1_recording();
    recording.push_str(&ndjson(&[raw_record(
        6,
        "2026-08-02T18:00:00Z",
        SESSION,
        Event::SessionEnded(SessionEnded { reason: None }),
    )]));

    let err = replay_bytes(recording.as_bytes()).expect_err("mixed versions");
    assert!(
        matches!(
            err,
            Error::MixedSchemaVersions {
                line: 6,
                expected: 1,
                found: 2
            }
        ),
        "unexpected error: {err}"
    );
}

#[test]
fn a_recording_that_mixes_versions_the_other_way_is_also_refused() {
    let mut recording = ndjson(&[raw_record(
        1,
        "2026-08-02T18:00:00Z",
        SESSION,
        Event::SessionStarted(SessionStarted { source: None }),
    )]);
    recording.push_str(&ndjson_v1(&[raw_v1_record(
        2,
        "2026-08-02T18:00:01Z",
        SESSION,
        v1::Event::SessionEnded,
    )]));

    let err = replay_bytes(recording.as_bytes()).expect_err("mixed versions");
    assert!(
        matches!(
            err,
            Error::MixedSchemaVersions {
                line: 2,
                expected: 2,
                found: 1
            }
        ),
        "unexpected error: {err}"
    );
}

#[test]
fn appending_a_v2_record_to_a_v1_recording_is_refused() {
    // The refusal has to exist at the append boundary too. Replay rejecting a
    // mixed recording after the fact would mean the appender had already
    // destroyed a readable v1 recording by writing to it.
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("session.ndjson");
    let original = v1_recording();
    std::fs::write(&path, &original).expect("write");

    let err = append(
        &path,
        &reported_intent("should not land", None),
        ts("2026-08-02T18:00:00Z"),
    )
    .expect_err("version mismatch");
    assert!(
        matches!(
            err,
            Error::AppendVersionMismatch {
                recording: 1,
                writing: 2
            }
        ),
        "unexpected error: {err}"
    );

    // Byte-for-byte unchanged, and still replayable as v1.
    assert_eq!(std::fs::read_to_string(&path).expect("read"), original);
    assert_eq!(replay_file(&path).expect("replay").schema_version, Some(1));
}

#[test]
fn the_claude_adapter_refuses_to_append_to_a_v1_recording() {
    // The same refusal reached through the adapter: a user who activates the
    // hooks against a directory holding an older recording gets a loud failure,
    // not a spliced recording.
    use std::io::Write;
    use std::process::{Command, Stdio};

    let dir = tempfile::tempdir().expect("temp dir");
    let session = "resumed-v1-session";
    let recording = dir.path().join(format!("{session}.ndjson"));
    let original = ndjson_v1(&[raw_v1_record(
        1,
        "2026-08-01T09:00:00Z",
        session,
        v1::Event::SessionStarted,
    )]);
    std::fs::write(&recording, &original).expect("write");

    let mut child = Command::new(env!("CARGO_BIN_EXE_witnessglass"))
        .arg("claude-hook")
        .arg("--recordings-dir")
        .arg(dir.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn");
    let payload = serde_json::json!({
        "hook_event_name": "SessionEnd",
        "session_id": session,
        "reason": "clear",
    })
    .to_string();
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(payload.as_bytes())
        .expect("write");
    let output = child.wait_with_output().expect("wait");

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("schema version"),
        "stderr did not explain the refusal: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(std::fs::read_to_string(&recording).expect("read"), original);
}
