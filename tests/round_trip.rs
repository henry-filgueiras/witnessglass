//! The core claim: a session's evidence can be emitted and replayed without
//! the two epistemic channels being merged along the way.

mod common;

use common::*;
use witnessglass::{Channel, Event, Tail, ToolOutcome, append, replay_file};

#[test]
fn session_intent_and_tool_lifecycle_survive_a_round_trip() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("session.ndjson");

    append(&path, &session_started(), ts("2026-08-02T18:00:00Z")).expect("boundary");
    append(
        &path,
        &reported_intent("I am going to inspect the synthetic target.", None),
        ts("2026-08-02T18:00:01Z"),
    )
    .expect("intent");
    append(&path, &tool_started(TOOL_CALL), ts("2026-08-02T18:00:02Z")).expect("start");
    append(
        &path,
        &tool_finished(TOOL_CALL, ToolOutcome::Succeeded),
        ts("2026-08-02T18:00:03Z"),
    )
    .expect("finish");
    append(&path, &session_ended(), ts("2026-08-02T18:00:04Z")).expect("boundary");

    let replay = replay_file(&path).expect("replay");
    assert_eq!(replay.tail, Tail::Complete);
    assert_eq!(replay.records.len(), 5);

    let kinds: Vec<&str> = replay.records.iter().map(|r| r.event.kind()).collect();
    assert_eq!(
        kinds,
        vec![
            "session_started",
            "reported_intent",
            "observed_tool_started",
            "observed_tool_finished",
            "session_ended",
        ]
    );

    let sequences: Vec<u64> = replay.records.iter().map(|r| r.sequence).collect();
    assert_eq!(sequences, vec![1, 2, 3, 4, 5]);

    for record in &replay.records {
        assert_eq!(record.schema_version, witnessglass::SCHEMA_VERSION);
        assert_eq!(record.session_id, SESSION);
    }
}

#[test]
fn one_record_per_line_and_nothing_is_rewritten() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("session.ndjson");

    append(&path, &session_started(), ts("2026-08-02T18:00:00Z")).expect("boundary");
    let after_first = std::fs::read(&path).expect("read");

    append(&path, &tool_started(TOOL_CALL), ts("2026-08-02T18:00:01Z")).expect("start");
    let after_second = std::fs::read(&path).expect("read");

    // The second append extended the file; it did not touch the first record.
    assert!(after_second.starts_with(&after_first));
    assert_eq!(after_second.iter().filter(|&&b| b == b'\n').count(), 2);
    assert!(after_second.ends_with(b"\n"));

    // Each line parses on its own.
    let text = String::from_utf8(after_second).expect("utf-8");
    for line in text.lines() {
        serde_json::from_str::<witnessglass::Record>(line).expect("line is a complete record");
    }
}

#[test]
fn reported_and_observed_provenance_stay_distinguishable() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("session.ndjson");

    append(
        &path,
        &reported_intent("Claimed: the synthetic check passed.", None),
        ts("2026-08-02T18:00:00Z"),
    )
    .expect("intent");
    append(
        &path,
        &tool_finished(TOOL_CALL, ToolOutcome::Failed),
        ts("2026-08-02T18:00:01Z"),
    )
    .expect("finish");

    let replay = replay_file(&path).expect("replay");

    assert_eq!(replay.records[0].provenance.channel, Channel::Reported);
    assert_eq!(replay.records[1].provenance.channel, Channel::Observed);
    for record in &replay.records {
        assert_eq!(record.provenance.adapter, ADAPTER);
        assert_eq!(record.provenance.mechanism, MECHANISM);
    }

    // The recording preserves a claim of success alongside an observed failure
    // without reconciling them. Disagreement is evidence, not a defect.
    let Event::ReportedIntent(intent) = &replay.records[0].event else {
        panic!("expected reported intent");
    };
    assert!(intent.text.contains("passed"));
    let Event::ObservedToolFinished(finished) = &replay.records[1].event else {
        panic!("expected observed finish");
    };
    assert_eq!(finished.outcome, ToolOutcome::Failed);
}

#[test]
fn tool_lifecycle_correlates_by_id_without_being_collapsed() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("session.ndjson");

    // A friendly description supplied by the agent and the hook-delivered tool
    // name share a correlation id. They are two claims of different kinds, and
    // they stay two records.
    append(
        &path,
        &reported_intent("Checking the synthetic target.", Some(TOOL_CALL)),
        ts("2026-08-02T18:00:00Z"),
    )
    .expect("intent");
    append(&path, &tool_started(TOOL_CALL), ts("2026-08-02T18:00:01Z")).expect("start");
    append(
        &path,
        &tool_finished(TOOL_CALL, ToolOutcome::Succeeded),
        ts("2026-08-02T18:00:02Z"),
    )
    .expect("finish");

    let replay = replay_file(&path).expect("replay");
    assert_eq!(replay.records.len(), 3);

    let ids: Vec<Option<&str>> = replay
        .records
        .iter()
        .map(|record| match &record.event {
            Event::ReportedIntent(intent) => intent.tool_call_id.as_deref(),
            Event::ObservedToolStarted(started) => Some(started.tool_call_id.as_str()),
            Event::ObservedToolFinished(finished) => Some(finished.tool_call_id.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(ids, vec![Some(TOOL_CALL); 3]);

    // Correlated, but not fused: the channels and kinds remain separate, and
    // the agent's words appear only in the reported record.
    let channels: Vec<Channel> = replay
        .records
        .iter()
        .map(|r| r.provenance.channel)
        .collect();
    assert_eq!(
        channels,
        vec![Channel::Reported, Channel::Observed, Channel::Observed]
    );

    let observed_text = serde_json::to_string(&replay.records[1]).expect("serialize")
        + &serde_json::to_string(&replay.records[2]).expect("serialize");
    assert!(!observed_text.contains("Checking the synthetic target."));
}

#[test]
fn a_finish_without_an_observed_start_is_recorded_as_it_arrived() {
    // A capture mechanism that missed the start of a call has a blind spot.
    // Refusing the finish would delete the only evidence that the call existed,
    // so the raw stream takes it exactly as delivered and leaves the pairing to
    // whoever reads it.
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("session.ndjson");

    append(
        &path,
        &tool_finished("toolu_synthetic_orphan", ToolOutcome::Succeeded),
        ts("2026-08-02T18:00:00Z"),
    )
    .expect("orphan finish is accepted");

    let replay = replay_file(&path).expect("replay");
    assert_eq!(replay.records.len(), 1);
    assert_eq!(replay.records[0].event.kind(), "observed_tool_finished");
}

#[test]
fn replay_of_an_empty_recording_is_empty_and_complete() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("session.ndjson");
    std::fs::write(&path, b"").expect("write");

    let replay = replay_file(&path).expect("replay");
    assert!(replay.records.is_empty());
    assert_eq!(replay.tail, Tail::Complete);
}
