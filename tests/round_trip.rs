//! The core claim: a session's evidence can be emitted and replayed without
//! the two epistemic channels being merged along the way.

mod common;

use common::*;
use witnessglass::{Channel, Event, Tail, append, replay_file};

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
    append(
        &path,
        &tool_requested(TOOL_CALL),
        ts("2026-08-02T18:00:02Z"),
    )
    .expect("request");
    append(
        &path,
        &tool_succeeded(TOOL_CALL),
        ts("2026-08-02T18:00:03Z"),
    )
    .expect("success");
    append(&path, &session_ended(), ts("2026-08-02T18:00:04Z")).expect("boundary");

    let replay = replay_file(&path).expect("replay");
    assert_eq!(replay.tail, Tail::Complete);
    assert_eq!(replay.records.len(), 5);
    assert_eq!(replay.schema_version, Some(2));

    assert_eq!(
        kinds(&replay.records),
        vec![
            "session_started",
            "reported_intent",
            "tool_requested",
            "tool_succeeded",
            "session_ended",
        ]
    );
    assert_eq!(sequences(&replay.records), vec![1, 2, 3, 4, 5]);

    for record in &replay.records {
        assert_eq!(record.schema_version(), witnessglass::SCHEMA_VERSION);
        assert_eq!(record.session_id(), SESSION);
    }
}

#[test]
fn a_request_is_not_recorded_as_an_execution() {
    // The whole reason v2 exists. A pre-execution hook proves a request was
    // constructed; it proves nothing about whether the call ran, and the record
    // must not be readable as though it did.
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("session.ndjson");

    append(
        &path,
        &tool_requested(TOOL_CALL),
        ts("2026-08-02T18:00:00Z"),
    )
    .expect("request");

    let replay = replay_file(&path).expect("replay");
    assert_eq!(kinds(&replay.records), vec!["tool_requested"]);

    // Nothing in the record claims a start, an outcome, or a response.
    let line = serde_json::to_string(&replay.records[0]).expect("serialize");
    assert!(!line.contains("started"), "request implies a start: {line}");
    assert!(
        !line.contains("outcome"),
        "request implies an outcome: {line}"
    );
    assert!(
        !line.contains("response"),
        "request implies a response: {line}"
    );
    assert!(line.contains("requested_input"));
}

#[test]
fn requested_input_and_effective_input_stay_distinct() {
    // Claude documents that a tool request may be modified before it executes.
    // If both inputs collapsed into one field, a recording could not show that
    // what ran was not what was asked for — which is precisely the kind of
    // divergence the project exists to preserve.
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("session.ndjson");

    let requested = emission(
        SESSION,
        Channel::Observed,
        Event::ToolRequested(witnessglass::ToolRequested {
            tool_use_id: TOOL_CALL.to_owned(),
            tool_name: "SyntheticTool".to_owned(),
            requested_input: serde_json::json!({ "target": "/synthetic/asked-for" }),
        }),
    );
    let succeeded = emission(
        SESSION,
        Channel::Observed,
        Event::ToolSucceeded(witnessglass::ToolSucceeded {
            tool_use_id: TOOL_CALL.to_owned(),
            tool_name: "SyntheticTool".to_owned(),
            effective_input: serde_json::json!({ "target": "/synthetic/actually-run" }),
            response: serde_json::json!({ "status": "synthetic" }),
            duration_ms: Some(1234),
        }),
    );

    append(&path, &requested, ts("2026-08-02T18:00:00Z")).expect("request");
    append(&path, &succeeded, ts("2026-08-02T18:00:01Z")).expect("success");

    let replay = replay_file(&path).expect("replay");

    let Event::ToolRequested(requested) = v2_event(&replay.records[0]) else {
        panic!("expected a request");
    };
    let Event::ToolSucceeded(succeeded) = v2_event(&replay.records[1]) else {
        panic!("expected a success");
    };

    assert_eq!(requested.requested_input["target"], "/synthetic/asked-for");
    assert_eq!(
        succeeded.effective_input["target"],
        "/synthetic/actually-run"
    );
    assert_ne!(requested.requested_input, succeeded.effective_input);

    // Correlated by id, and still two records with two inputs.
    assert_eq!(requested.tool_use_id, succeeded.tool_use_id);
    assert_eq!(succeeded.duration_ms, Some(1234));
}

#[test]
fn one_record_per_line_and_nothing_is_rewritten() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("session.ndjson");

    append(&path, &session_started(), ts("2026-08-02T18:00:00Z")).expect("boundary");
    let after_first = std::fs::read(&path).expect("read");

    append(
        &path,
        &tool_requested(TOOL_CALL),
        ts("2026-08-02T18:00:01Z"),
    )
    .expect("request");
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
    append(&path, &tool_failed(TOOL_CALL), ts("2026-08-02T18:00:01Z")).expect("failure");

    let replay = replay_file(&path).expect("replay");

    assert_eq!(replay.records[0].provenance().channel, Channel::Reported);
    assert_eq!(replay.records[1].provenance().channel, Channel::Observed);
    for record in &replay.records {
        assert_eq!(record.provenance().adapter, ADAPTER);
        assert_eq!(record.provenance().mechanism, MECHANISM);
    }

    // The recording preserves a claim of success alongside an observed failure
    // without reconciling them. Disagreement is evidence, not a defect.
    let Event::ReportedIntent(intent) = v2_event(&replay.records[0]) else {
        panic!("expected reported intent");
    };
    assert!(intent.text.contains("passed"));
    let Event::ToolFailed(failed) = v2_event(&replay.records[1]) else {
        panic!("expected observed failure");
    };
    assert_eq!(failed.error, "synthetic failure");
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
    assert_eq!(replay.records.len(), 3);

    let ids: Vec<Option<&str>> = replay
        .records
        .iter()
        .map(|record| v2_event(record).tool_use_id())
        .collect();
    assert_eq!(ids, vec![Some(TOOL_CALL); 3]);

    // Correlated, but not fused: the channels and kinds remain separate, and
    // the agent's words appear only in the reported record.
    let channels: Vec<Channel> = replay
        .records
        .iter()
        .map(|r| r.provenance().channel)
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
fn a_completion_without_a_recorded_request_is_recorded_as_it_arrived() {
    // A capture mechanism that missed the pre-execution hook has a blind spot.
    // Refusing the completion would delete the only evidence that the call
    // existed, so the raw stream takes it exactly as delivered and leaves the
    // pairing to whoever reads it.
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("session.ndjson");

    append(
        &path,
        &tool_succeeded("toolu_synthetic_orphan"),
        ts("2026-08-02T18:00:00Z"),
    )
    .expect("orphan completion is accepted");

    let replay = replay_file(&path).expect("replay");
    assert_eq!(replay.records.len(), 1);
    assert_eq!(replay.records[0].event_kind(), "tool_succeeded");
}

#[test]
fn replay_of_an_empty_recording_is_empty_and_complete() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("session.ndjson");
    std::fs::write(&path, b"").expect("write");

    let replay = replay_file(&path).expect("replay");
    assert!(replay.records.is_empty());
    assert_eq!(replay.schema_version, None);
    assert_eq!(replay.tail, Tail::Complete);
}
