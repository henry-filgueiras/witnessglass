//! Corruption, truncation, and unsupported versions are different claims about
//! a recording, and the reader must not blur them.

mod common;

use common::*;
use witnessglass::{
    Channel, Error, Event, ReportedIntent, SessionEnded, SessionStarted, Tail, append,
    replay_bytes, replay_file,
};

fn started() -> Event {
    Event::SessionStarted(SessionStarted { source: None })
}

fn ended() -> Event {
    Event::SessionEnded(SessionEnded { reason: None })
}

#[test]
fn an_unknown_schema_version_is_refused_not_guessed_at() {
    let mut record = raw_record(1, "2026-08-02T18:00:00Z", SESSION, started());
    record.schema_version = 97;
    let recording = ndjson(&[record]);

    let err = replay_bytes(recording.as_bytes()).expect_err("unsupported version");
    assert!(
        matches!(
            err,
            Error::UnsupportedSchemaVersion {
                line: 1,
                found: 97,
                ..
            }
        ),
        "unexpected error: {err}"
    );
    assert!(err.to_string().contains("unsupported schema version 97"));
}

#[test]
fn an_unknown_version_later_in_the_stream_is_still_refused() {
    let mut second = raw_record(2, "2026-08-02T18:00:01Z", SESSION, ended());
    second.schema_version = 99;
    let recording = ndjson(&[
        raw_record(1, "2026-08-02T18:00:00Z", SESSION, started()),
        second,
    ]);

    // Refused as a version mix rather than an unknown version: the recording
    // established schema v2 on line 1, so line 2 disagreeing with it is the
    // first thing wrong with it.
    let err = replay_bytes(recording.as_bytes()).expect_err("unsupported version");
    assert!(
        matches!(
            err,
            Error::MixedSchemaVersions {
                line: 2,
                expected: 2,
                found: 99
            }
        ),
        "unexpected error: {err}"
    );
}

#[test]
fn a_malformed_complete_record_is_corruption() {
    let mut recording = ndjson(&[raw_record(1, "2026-08-02T18:00:00Z", SESSION, started())]);
    recording.push_str("{\"schema_version\":2,\"this\":\"is not a record\"}\n");

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
    let mut recording = ndjson(&[raw_record(1, "2026-08-02T18:00:00Z", SESSION, started())]);
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
    // intent, so the record is not a valid record.
    let mut record = raw_record(
        1,
        "2026-08-02T18:00:00Z",
        SESSION,
        Event::ReportedIntent(ReportedIntent {
            text: "synthetic".to_owned(),
            tool_use_id: None,
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
            tool_use_id: None,
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

    let mut bad = tool_requested(TOOL_CALL);
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
    let fragment = "{\"schema_version\":2,\"session_id\":\"sess-synthetic-0001\",\"sequ";
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
        raw_record(1, "2026-08-02T18:00:00Z", SESSION, started()),
        raw_record(2, "2026-08-02T18:00:01Z", SESSION, ended()),
    ]);
    let without_final_newline = &full[..full.len() - 1];

    let replay = replay_bytes(without_final_newline.as_bytes()).expect("prefix readable");
    assert_eq!(replay.records.len(), 1);
    assert!(replay.tail.is_truncated());
    assert_eq!(replay.records[0].event_kind(), "session_started");
}

#[test]
fn a_recording_that_is_only_a_fragment_replays_as_no_events() {
    let replay = replay_bytes(b"{\"schema_version\":2,\"sess").expect("readable");
    assert!(replay.records.is_empty());
    assert_eq!(replay.schema_version, None);
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
    text.push_str("{\"schema_version\":2,\"sess");
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
fn invalid_utf8_inside_a_complete_record_is_corruption() {
    // Newline-terminated: this record was written whole, and it is wrong.
    let err = replay_bytes(b"{\"schema_version\":2,\xff\xfe}\n").expect_err("corruption");
    assert!(
        matches!(err, Error::Corruption { line: 1, .. }),
        "unexpected error: {err}"
    );
}

#[test]
fn invalid_utf8_in_a_later_complete_record_is_still_corruption() {
    let mut recording =
        ndjson(&[raw_record(1, "2026-08-02T18:00:00Z", SESSION, started())]).into_bytes();
    recording.extend_from_slice(b"{\"schema_version\":2,\xff\xfe}\n");

    let err = replay_bytes(&recording).expect_err("corruption");
    assert!(
        matches!(err, Error::Corruption { line: 2, .. }),
        "unexpected error: {err}"
    );
}

#[test]
fn an_unterminated_invalid_utf8_fragment_does_not_condemn_the_prefix() {
    // An emitter killed mid-write can stop inside a multibyte character, so the
    // fragment is invalid UTF-8 by construction. It says nothing whatsoever
    // about the complete records in front of it.
    let mut recording = ndjson(&[
        raw_record(1, "2026-08-02T18:00:00Z", SESSION, started()),
        raw_record(2, "2026-08-02T18:00:01Z", SESSION, ended()),
    ])
    .into_bytes();
    let complete_len = recording.len();
    recording.extend_from_slice(b"{\"schema_version\":2,\"session_id\":\"\xf0\x9f");

    let replay = replay_bytes(&recording).expect("prefix survives an undecodable fragment");
    assert_eq!(replay.records.len(), 2);
    assert_eq!(
        replay.tail,
        Tail::Truncated {
            byte_offset: complete_len as u64,
            bytes: 36
        }
    );
}

#[test]
fn a_recording_of_only_invalid_unterminated_bytes_replays_as_no_events() {
    // Nothing decodable, no newline anywhere: zero records, and the whole file
    // reported as an incomplete fragment rather than as a decoding failure.
    let recording: &[u8] = b"\xff\xfe\x00\x80not a record\xf0\x9f";

    let replay = replay_bytes(recording).expect("readable");
    assert!(replay.records.is_empty());
    assert_eq!(
        replay.tail,
        Tail::Truncated {
            byte_offset: 0,
            bytes: recording.len()
        }
    );
}

#[test]
fn a_record_torn_midway_through_a_multibyte_character_keeps_its_prefix() {
    let full = ndjson(&[
        raw_record(1, "2026-08-02T18:00:00Z", SESSION, started()),
        raw_record(
            2,
            "2026-08-02T18:00:01Z",
            SESSION,
            Event::ReportedIntent(ReportedIntent {
                // Synthetic text chosen to put multibyte characters in the
                // second record so the cut can land inside one.
                text: "señal sintética 🔭".to_owned(),
                tool_use_id: None,
            }),
        ),
    ]);

    // Cut two bytes into the four-byte telescope, mid-character.
    let telescope = full.find('🔭').expect("multibyte character present");
    let cut = telescope + 2;
    let torn = &full.as_bytes()[..cut];
    assert!(
        std::str::from_utf8(torn).is_err(),
        "the cut should genuinely tear a character"
    );

    let replay = replay_bytes(torn).expect("prefix survives a torn character");
    assert_eq!(replay.records.len(), 1);
    assert_eq!(replay.records[0].event_kind(), "session_started");
    assert!(replay.tail.is_truncated());

    // The fragment is accounted for as bytes, not decoded.
    let first_line_len = full.find('\n').expect("newline") + 1;
    assert_eq!(
        replay.tail,
        Tail::Truncated {
            byte_offset: first_line_len as u64,
            bytes: cut - first_line_len
        }
    );
}

#[test]
fn a_recording_at_the_maximum_sequence_refuses_further_appends() {
    // Hand-built: the appender can never produce this, and replay would reject
    // the recording for starting at the wrong sequence. The append path reads
    // only the final record, which is exactly the path under test.
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("session.ndjson");

    let mut record = raw_record(1, "2026-08-02T18:00:00Z", SESSION, started());
    record.sequence = u64::MAX;
    let recording = ndjson(&[record]);
    std::fs::write(&path, &recording).expect("write");

    let err = append(
        &path,
        &reported_intent("should not land", None),
        ts("2026-08-02T18:00:01Z"),
    )
    .expect_err("sequence exhausted");
    assert!(
        matches!(err, Error::SequenceExhausted { last: u64::MAX }),
        "unexpected error: {err}"
    );

    // Byte-for-byte unchanged by the refusal.
    assert_eq!(std::fs::read(&path).expect("read"), recording.into_bytes());
}
