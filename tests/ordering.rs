//! Canonical order is physical append order. Timestamps describe; they never
//! decide.

mod common;

use common::*;
use witnessglass::{
    AnyRecord, Event, ReportedIntent, SessionEnded, SessionStarted, Tail, append, replay_bytes,
    replay_file,
};

fn started() -> Event {
    Event::SessionStarted(SessionStarted { source: None })
}

fn ended() -> Event {
    Event::SessionEnded(SessionEnded { reason: None })
}

#[test]
fn replay_is_deterministic() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("session.ndjson");

    append(&path, &session_started(), ts("2026-08-02T18:00:00Z")).expect("boundary");
    append(
        &path,
        &reported_intent("First.", None),
        ts("2026-08-02T18:00:01Z"),
    )
    .expect("intent");
    append(
        &path,
        &tool_requested(TOOL_CALL),
        ts("2026-08-02T18:00:02Z"),
    )
    .expect("request");

    let first = replay_file(&path).expect("replay");
    let second = replay_file(&path).expect("replay");
    assert_eq!(first, second);

    // Byte-for-byte stable when re-rendered, not merely equal as values.
    let render = |replay: &witnessglass::Replay| {
        replay
            .records
            .iter()
            .map(|r| serde_json::to_string(r).expect("serialize"))
            .collect::<Vec<_>>()
    };
    assert_eq!(render(&first), render(&second));
}

#[test]
fn equal_timestamps_need_no_tie_breaker() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("session.ndjson");

    let same = ts("2026-08-02T18:00:00Z");
    append(&path, &reported_intent("one", None), same).expect("one");
    append(&path, &reported_intent("two", None), same).expect("two");
    append(&path, &reported_intent("three", None), same).expect("three");

    let replay = replay_file(&path).expect("replay");
    assert_eq!(sequences(&replay.records), vec![1, 2, 3]);
    assert!(replay.records.iter().all(|r| r.recorded_at() == same));
    assert_eq!(intent_texts(&replay.records), vec!["one", "two", "three"]);
}

#[test]
fn a_clock_moving_backwards_does_not_reorder_replay() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("session.ndjson");

    // The clock steps backwards between appends, as it can after an NTP
    // correction. Append order is unaffected.
    append(
        &path,
        &reported_intent("first appended", None),
        ts("2026-08-02T18:00:10Z"),
    )
    .expect("one");
    append(
        &path,
        &reported_intent("second appended", None),
        ts("2026-08-02T17:59:00Z"),
    )
    .expect("two");
    append(
        &path,
        &reported_intent("third appended", None),
        ts("2026-08-02T18:00:05Z"),
    )
    .expect("three");

    let replay = replay_file(&path).expect("replay");
    assert_eq!(
        intent_texts(&replay.records),
        vec!["first appended", "second appended", "third appended"]
    );
    assert_eq!(sequences(&replay.records), vec![1, 2, 3]);

    // Proof that no timestamp sort happened: the timestamps are genuinely out
    // of order in the replayed stream.
    assert!(replay.records[1].recorded_at() < replay.records[0].recorded_at());
    assert!(replay.records[2].recorded_at() > replay.records[1].recorded_at());
}

#[test]
fn a_duplicate_sequence_is_rejected() {
    let recording = ndjson(&[
        raw_record(1, "2026-08-02T18:00:00Z", SESSION, started()),
        raw_record(1, "2026-08-02T18:00:01Z", SESSION, ended()),
    ]);
    let err = replay_bytes(recording.as_bytes()).expect_err("duplicate sequence");
    assert!(
        matches!(
            err,
            witnessglass::Error::SequenceViolation {
                line: 2,
                expected: 2,
                found: 1
            }
        ),
        "unexpected error: {err}"
    );
}

#[test]
fn a_decreasing_sequence_is_rejected() {
    let recording = ndjson(&[
        raw_record(1, "2026-08-02T18:00:00Z", SESSION, started()),
        raw_record(2, "2026-08-02T18:00:01Z", SESSION, ended()),
        raw_record(1, "2026-08-02T18:00:02Z", SESSION, ended()),
    ]);
    let err = replay_bytes(recording.as_bytes()).expect_err("decreasing sequence");
    assert!(
        matches!(err, witnessglass::Error::SequenceViolation { line: 3, .. }),
        "unexpected error: {err}"
    );
}

#[test]
fn a_skipped_sequence_is_rejected() {
    // A gap cannot be distinguished from a deletion, so the history is
    // ambiguous and the recording is refused rather than silently accepted.
    let recording = ndjson(&[
        raw_record(1, "2026-08-02T18:00:00Z", SESSION, started()),
        raw_record(3, "2026-08-02T18:00:01Z", SESSION, ended()),
    ]);
    let err = replay_bytes(recording.as_bytes()).expect_err("skipped sequence");
    assert!(
        matches!(
            err,
            witnessglass::Error::SequenceViolation {
                line: 2,
                expected: 2,
                found: 3
            }
        ),
        "unexpected error: {err}"
    );
}

#[test]
fn a_sequence_not_starting_at_one_is_rejected() {
    let recording = ndjson(&[raw_record(7, "2026-08-02T18:00:00Z", SESSION, started())]);
    let err = replay_bytes(recording.as_bytes()).expect_err("bad first sequence");
    assert!(
        matches!(err, witnessglass::Error::SequenceViolation { line: 1, .. }),
        "unexpected error: {err}"
    );
}

#[test]
fn one_recording_is_one_session() {
    let recording = ndjson(&[
        raw_record(1, "2026-08-02T18:00:00Z", SESSION, started()),
        raw_record(2, "2026-08-02T18:00:01Z", OTHER_SESSION, ended()),
    ]);
    let err = replay_bytes(recording.as_bytes()).expect_err("session mismatch");
    assert!(
        matches!(err, witnessglass::Error::SessionMismatch { line: 2, .. }),
        "unexpected error: {err}"
    );
}

#[test]
fn appending_another_session_to_a_recording_is_refused() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("session.ndjson");

    append(&path, &session_started(), ts("2026-08-02T18:00:00Z")).expect("boundary");

    let stray = emission(OTHER_SESSION, witnessglass::Channel::Recorder, started());
    let err = append(&path, &stray, ts("2026-08-02T18:00:01Z")).expect_err("session mismatch");
    assert!(
        matches!(err, witnessglass::Error::EmissionSessionMismatch { .. }),
        "unexpected error: {err}"
    );

    // The refusal wrote nothing.
    let replay = replay_file(&path).expect("replay");
    assert_eq!(replay.records.len(), 1);
    assert_eq!(replay.tail, Tail::Complete);
}

fn intent_texts(records: &[AnyRecord]) -> Vec<&str> {
    records
        .iter()
        .map(|record| match v2_event(record) {
            Event::ReportedIntent(ReportedIntent { text, .. }) => text.as_str(),
            other => panic!("expected reported intent, got {}", other.kind()),
        })
        .collect()
}
