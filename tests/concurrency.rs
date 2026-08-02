//! Claude Code runs hooks as separate, short-lived, parallel processes, so
//! racing emitters are the normal case rather than an exotic one. Deciding the
//! next sequence number and writing the record is one serialized transaction;
//! these tests hold it to that.

mod common;

use std::collections::BTreeSet;
use std::io::Write;
use std::process::{Command, Stdio};

use common::*;
use witnessglass::{Event, ReportedIntent, Tail, append, replay_file};

/// Text of every reported-intent record in a replayed recording.
fn intent_texts(records: &[witnessglass::AnyRecord]) -> BTreeSet<String> {
    records
        .iter()
        .map(|record| match v2_event(record) {
            Event::ReportedIntent(ReportedIntent { text, .. }) => text.clone(),
            other => panic!("expected reported intent, got {}", other.kind()),
        })
        .collect()
}

const THREADS: usize = 16;
const PER_THREAD: usize = 4;

#[test]
fn concurrent_in_process_appenders_produce_intact_uniquely_ordered_records() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("session.ndjson");

    std::thread::scope(|scope| {
        for thread in 0..THREADS {
            let path = path.clone();
            scope.spawn(move || {
                for index in 0..PER_THREAD {
                    let text = format!("emitter-{thread:02}-{index:02}");
                    append(
                        &path,
                        &reported_intent(&text, None),
                        ts("2026-08-02T18:00:00Z"),
                    )
                    .expect("append should succeed under contention");
                }
            });
        }
    });

    let replay = replay_file(&path).expect("replay");
    assert_eq!(replay.tail, Tail::Complete);
    assert_eq!(replay.records.len(), THREADS * PER_THREAD);

    // Sequence numbers are exactly 1..=n with no duplicates and no gaps. That
    // is already enforced by replay, so reaching this point proves the append
    // transaction never handed the same number to two racing writers.
    let expected: Vec<u64> = (1..=(THREADS * PER_THREAD) as u64).collect();
    assert_eq!(sequences(&replay.records), expected);

    // Every emission landed exactly once, and no record was lost or doubled.
    let mut expected_texts = BTreeSet::new();
    for thread in 0..THREADS {
        for index in 0..PER_THREAD {
            expected_texts.insert(format!("emitter-{thread:02}-{index:02}"));
        }
    }
    assert_eq!(intent_texts(&replay.records), expected_texts);
}

#[test]
fn concurrent_short_lived_processes_do_not_interleave_records() {
    // The real shape of the problem: independent processes, each alive only
    // long enough to write one record, exactly as a hook would be.
    const PROCESSES: usize = 8;

    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("session.ndjson");
    let binary = env!("CARGO_BIN_EXE_witnessglass");

    let mut children = Vec::new();
    for index in 0..PROCESSES {
        let child = Command::new(binary)
            .arg("append")
            .arg("--recording")
            .arg(&path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn emitter");
        children.push((index, child));
    }

    for (index, child) in &mut children {
        let payload = serde_json::to_string(&reported_intent(&format!("process-{index:02}"), None))
            .expect("serialize emission");
        child
            .stdin
            .take()
            .expect("stdin")
            .write_all(payload.as_bytes())
            .expect("write emission");
    }

    for (index, child) in children {
        let output = child.wait_with_output().expect("wait");
        assert!(
            output.status.success(),
            "emitter {index} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let replay = replay_file(&path).expect("replay");
    assert_eq!(replay.tail, Tail::Complete);
    assert_eq!(replay.records.len(), PROCESSES);
    assert_eq!(
        sequences(&replay.records),
        (1..=PROCESSES as u64).collect::<Vec<_>>()
    );

    let expected: BTreeSet<String> = (0..PROCESSES).map(|i| format!("process-{i:02}")).collect();
    assert_eq!(intent_texts(&replay.records), expected);
}
