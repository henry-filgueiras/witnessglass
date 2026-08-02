//! Shared synthetic fixtures.
//!
//! Everything here is obviously fabricated on purpose. No fixture is derived
//! from a real session, a real prompt, a real source tree, or a real machine.

#![allow(dead_code)]

use witnessglass::record::v1;
use witnessglass::{
    AnyRecord, Channel, Context, Emission, Event, Provenance, Record, ReportedIntent,
    SCHEMA_VERSION, SessionEnded, SessionStarted, ToolFailed, ToolRequested, ToolSucceeded,
};

pub const SESSION: &str = "sess-synthetic-0001";
pub const OTHER_SESSION: &str = "sess-synthetic-0002";
pub const TOOL_CALL: &str = "toolu_synthetic_0001";
pub const ADAPTER: &str = "synthetic-adapter";
pub const MECHANISM: &str = "synthetic-test-harness";

/// Parse a fixed timestamp. Panics on a bad literal, which is a test bug.
pub fn ts(text: &str) -> jiff::Timestamp {
    text.parse().expect("synthetic timestamp should parse")
}

pub fn provenance(channel: Channel) -> Provenance {
    Provenance {
        channel,
        adapter: ADAPTER.to_owned(),
        mechanism: MECHANISM.to_owned(),
    }
}

pub fn emission(session: &str, channel: Channel, event: Event) -> Emission {
    Emission {
        session_id: session.to_owned(),
        context: Context::default(),
        provenance: provenance(channel),
        event,
    }
}

/// Recorder-asserted opening boundary.
pub fn session_started() -> Emission {
    emission(
        SESSION,
        Channel::Recorder,
        Event::SessionStarted(SessionStarted { source: None }),
    )
}

/// Recorder-asserted closing boundary.
pub fn session_ended() -> Emission {
    emission(
        SESSION,
        Channel::Recorder,
        Event::SessionEnded(SessionEnded { reason: None }),
    )
}

/// Something the agent said about itself.
pub fn reported_intent(text: &str, tool_use_id: Option<&str>) -> Emission {
    emission(
        SESSION,
        Channel::Reported,
        Event::ReportedIntent(ReportedIntent {
            text: text.to_owned(),
            tool_use_id: tool_use_id.map(str::to_owned),
        }),
    )
}

/// A tool request the capture mechanism witnessed being constructed.
pub fn tool_requested(tool_use_id: &str) -> Emission {
    emission(
        SESSION,
        Channel::Observed,
        Event::ToolRequested(ToolRequested {
            tool_use_id: tool_use_id.to_owned(),
            tool_name: "SyntheticTool".to_owned(),
            requested_input: serde_json::json!({ "target": "/synthetic/example" }),
        }),
    )
}

/// A tool call the capture mechanism witnessed completing.
pub fn tool_succeeded(tool_use_id: &str) -> Emission {
    emission(
        SESSION,
        Channel::Observed,
        Event::ToolSucceeded(ToolSucceeded {
            tool_use_id: tool_use_id.to_owned(),
            tool_name: "SyntheticTool".to_owned(),
            effective_input: serde_json::json!({ "target": "/synthetic/example" }),
            response: serde_json::json!({ "status": "synthetic" }),
            duration_ms: None,
        }),
    )
}

/// A tool call the capture mechanism witnessed failing.
pub fn tool_failed(tool_use_id: &str) -> Emission {
    emission(
        SESSION,
        Channel::Observed,
        Event::ToolFailed(ToolFailed {
            tool_use_id: tool_use_id.to_owned(),
            tool_name: "SyntheticTool".to_owned(),
            effective_input: serde_json::json!({ "target": "/synthetic/example" }),
            error: "synthetic failure".to_owned(),
            interrupted: None,
            duration_ms: None,
        }),
    )
}

/// Build a raw v2 record directly, so tests can hand-craft damaged recordings
/// that the appender would never produce.
pub fn raw_record(sequence: u64, recorded_at: &str, session: &str, event: Event) -> Record {
    let channel = match &event {
        Event::ReportedIntent(_) => Channel::Reported,
        Event::SessionStarted(_) | Event::SessionEnded(_) => Channel::Recorder,
        _ => Channel::Observed,
    };
    Record {
        schema_version: SCHEMA_VERSION,
        session_id: session.to_owned(),
        sequence,
        recorded_at: ts(recorded_at),
        context: Context::default(),
        provenance: provenance(channel),
        event,
    }
}

/// Build a raw schema v1 record.
///
/// v1 is frozen and nothing writes it any more, so a v1 recording can only be
/// constructed by hand. That is exactly what a test of v1 replay needs: these
/// stand in for recordings made before v2 existed.
pub fn raw_v1_record(
    sequence: u64,
    recorded_at: &str,
    session: &str,
    event: v1::Event,
) -> v1::Record {
    let channel = match &event {
        v1::Event::ReportedIntent(_) => Channel::Reported,
        v1::Event::SessionStarted | v1::Event::SessionEnded => Channel::Recorder,
        v1::Event::ObservedToolStarted(_) | v1::Event::ObservedToolFinished(_) => Channel::Observed,
    };
    v1::Record {
        schema_version: v1::SCHEMA_VERSION,
        session_id: session.to_owned(),
        sequence,
        recorded_at: ts(recorded_at),
        provenance: provenance(channel),
        event,
    }
}

/// Render v2 records as an NDJSON recording.
pub fn ndjson(records: &[Record]) -> String {
    let mut out = String::new();
    for record in records {
        out.push_str(&serde_json::to_string(record).expect("record should serialize"));
        out.push('\n');
    }
    out
}

/// Render v1 records as an NDJSON recording.
pub fn ndjson_v1(records: &[v1::Record]) -> String {
    let mut out = String::new();
    for record in records {
        out.push_str(&serde_json::to_string(record).expect("record should serialize"));
        out.push('\n');
    }
    out
}

/// The event kinds of a replayed recording, in order.
pub fn kinds(records: &[AnyRecord]) -> Vec<&'static str> {
    records.iter().map(AnyRecord::event_kind).collect()
}

/// The sequence numbers of a replayed recording, in order.
pub fn sequences(records: &[AnyRecord]) -> Vec<u64> {
    records.iter().map(AnyRecord::sequence).collect()
}

/// The v2 event of a replayed record, panicking if it is not v2.
pub fn v2_event(record: &AnyRecord) -> &Event {
    &record.as_v2().expect("expected a schema v2 record").event
}
