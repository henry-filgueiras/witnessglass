//! Corruption, truncation, and unsupported versions are different claims about
//! a recording, and the reader must not blur them.

mod common;

use common::*;
use witnessglass::{
    Channel, Error, Event, ReportedIntent, Tail, append, replay_bytes, replay_file,
};

#[test]
fn an_unknown_schema_version_is_refused_not_guessed_at() {
    let mut record = raw_record(1, "2026-08-02T18:00:00Z", SESSION, Event::SessionStarted);
    record.schema_version = 2;
    let recording = ndjson(&[record]);

    let err = replay_bytes(recording.as_bytes()).expect_err("unsupported version");
    assert!(
        matches!(
            err,
            Error::UnsupportedSchemaVersion {
                line: 1,
                found: 2,
                supported: 1
            }
        ),
        "unexpected error: {err}"
    );
    assert!(err.to_string().contains("unsupported schema version 2"));
}

#[test]
fn an_unknown_version_later_in_the_stream_is_still_refused() {
    let mut second = raw_record(2, "2026-08-02T18:00:01Z", SESSION, Event::SessionEnded);
    second.schema_version = 99;
    let recording = ndjson(&[
        raw_record(1, "2026-08-02T18:00:00Z", SESSION, Event::SessionStarted),
        second,
    ]);

    let err = replay_bytes(recording.as_bytes()).expect_err("unsupported version");
    assert!(
        matches!(
            err,
            Error::UnsupportedSchemaVersion {
                line: 2,
                found: 99,
                ..
            }
        ),
        "unexpected error: {err}"
    );
}

#[test]
fn a_malformed_complete_record_is_corruption() {
    let mut recording = ndjson(&[raw_record(
        1,
        "2026-08-02T18:00:00Z",
        SESSION,
        Event::SessionStarted,
    )]);
    recording.push_str("{\"schema_version\":1,\"this\":\"is not a record\"}\n");

    let err = replay_bytes(recording.as_bytes()).expect_err("corruption");
    assert!(
        matches!(err, Error::Corruption { line: 2, .. }),
        "unexpected error: {err}"
    );
}

#[test]
fn a_line_that_is_not_json_is_corruption() {
    let recording = "this is not json at all\n";
    let err = replay_bytes(recording.as_bytes()).expect_err("corruption");
    assert!(
        matches!(err, Error::Corruption { line: 1, .. }),
        "unexpected error: {err}"
    );
}

#[test]
fn a_blank_line_is_corruption_not_an_empty_event() {
    let mut recording = ndjson(&[raw_record(
        1,
        "2026-08-02T18:00:00Z",
        SESSION,
        Event::SessionStarted,
    )]);
    recording.push('\n');

    let err = replay_bytes(recording.as_bytes()).expect_err("corruption");
    assert!(
        matches!(err, Error::Corruption { line: 2, .. }),
        "unexpected error: {err}"
    );
}

#[test]
fn an_event_on_an_impossible_channel_is_corruption() {
    // Intent presented as though it had been observed. No mechanism observes
    // intent, so the record is not a valid v1 record.
    let mut record = raw_record(
        1,
        "2026-08-02T18:00:00Z",
        SESSION,
        Event::ReportedIntent(ReportedIntent {
            text: "synthetic".to_owned(),
            tool_call_id: None,
        }),
    );
    record.provenance.channel = Channel::Observed;
    let recording = ndjson(&[record]);

    let err = replay_bytes(recording.as_bytes()).expect_err("channel mismatch");
    assert!(
        matches!(err, Error::Corruption { line: 1, .. }),
        "unexpected error: {err}"
    );
    assert!(err.to_string().contains("observed"));
}

#[test]
fn emitting_intent_on_the_observed_channel_is_refused_at_the_source() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("session.ndjson");

    let bad = emission(
        SESSION,
        Channel::Observed,
        Event::ReportedIntent(ReportedIntent {
            text: "synthetic".to_owned(),
            tool_call_id: None,
        }),
    );
    let err = append(&path, &bad, ts("2026-08-02T18:00:00Z")).expect_err("channel mismatch");
    assert!(
        matches!(err, Error::ChannelNotAllowed { .. }),
        "unexpected error: {err}"
    );
}

#[test]
fn emitting_a_tool_observation_on_the_reported_channel_is_refused() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("session.ndjson");

    let mut bad = tool_started(TOOL_CALL);
    bad.provenance.channel = Channel::Reported;
    let err = append(&path, &bad, ts("2026-08-02T18:00:00Z")).expect_err("channel mismatch");
    assert!(
        matches!(err, Error::ChannelNotAllowed { .. }),
        "unexpected error: {err}"
    );
}

#[test]
fn a_truncated_tail_yields_the_valid_prefix_and_says_so() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("session.ndjson");

    append(&path, &session_started(), ts("2026-08-02T18:00:00Z")).expect("boundary");
    append(
        &path,
        &reported_intent("survives", None),
        ts("2026-08-02T18:00:01Z"),
    )
    .expect("intent");

    // An emitter died mid-write.
    let fragment = "{\"schema_version\":1,\"session_id\":\"sess-synthetic-0001\",\"sequ";
    let mut text = std::fs::read_to_string(&path).expect("read");
    let complete_len = text.len();
    text.push_str(fragment);
    std::fs::write(&path, &text).expect("write");

    let replay = replay_file(&path).expect("prefix is still readable");
    assert_eq!(replay.records.len(), 2);
    assert_eq!(
        replay.tail,
        Tail::Truncated {
            byte_offset: complete_len as u64,
            bytes: fragment.len()
        }
    );
    assert!(replay.tail.is_truncated());
}

#[test]
fn a_complete_looking_final_record_without_a_newline_is_still_truncated() {
    // The bytes happen to parse. Nothing proves they are all the bytes, so the
    // fragment is not promoted to an event.
    let full = ndjson(&[
        raw_record(1, "2026-08-02T18:00:00Z", SESSION, Event::SessionStarted),
        raw_record(2, "2026-08-02T18:00:01Z", SESSION, Event::SessionEnded),
    ]);
    let without_final_newline = &full[..full.len() - 1];

    let replay = replay_bytes(without_final_newline.as_bytes()).expect("prefix readable");
    assert_eq!(replay.records.len(), 1);
    assert!(replay.tail.is_truncated());
    assert_eq!(replay.records[0].event.kind(), "session_started");
}

#[test]
fn a_recording_that_is_only_a_fragment_replays_as_no_events() {
    let replay = replay_bytes(b"{\"schema_version\":1,\"sess").expect("readable");
    assert!(replay.records.is_empty());
    assert_eq!(
        replay.tail,
        Tail::Truncated {
            byte_offset: 0,
            bytes: 25
        }
    );
}

#[test]
fn appending_onto_a_truncated_tail_is_refused() {
    // Splicing a new record onto a partial one would manufacture a single
    // corrupt line out of two honest halves, and would destroy the evidence
    // that the recording had been cut short.
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("session.ndjson");

    append(&path, &session_started(), ts("2026-08-02T18:00:00Z")).expect("boundary");
    let mut text = std::fs::read_to_string(&path).expect("read");
    text.push_str("{\"schema_version\":1,\"sess");
    std::fs::write(&path, &text).expect("write");

    let err = append(
        &path,
        &reported_intent("should not land", None),
        ts("2026-08-02T18:00:01Z"),
    )
    .expect_err("append refused");
    assert!(
        matches!(err, Error::AppendAfterTruncation { bytes: 25 }),
        "unexpected error: {err}"
    );

    // The file is untouched by the refusal.
    assert_eq!(std::fs::read_to_string(&path).expect("read"), text);
}

#[test]
fn invalid_utf8_is_corruption() {
    let err = replay_bytes(b"{\"schema_version\":1,\xff\xfe}\n").expect_err("corruption");
    assert!(
        matches!(err, Error::Corruption { .. }),
        "unexpected error: {err}"
    );
}
