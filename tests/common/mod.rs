//! Shared synthetic fixtures.
//!
//! Everything here is obviously fabricated on purpose. No fixture is derived
//! from a real session, a real prompt, a real source tree, or a real machine.

#![allow(dead_code)]

use witnessglass::{
    Channel, Emission, Event, ObservedToolFinished, ObservedToolStarted, Provenance, Record,
    ReportedIntent, SCHEMA_VERSION, ToolOutcome,
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
        provenance: provenance(channel),
        event,
    }
}

/// Recorder-asserted opening boundary.
pub fn session_started() -> Emission {
    emission(SESSION, Channel::Recorder, Event::SessionStarted)
}

/// Recorder-asserted closing boundary.
pub fn session_ended() -> Emission {
    emission(SESSION, Channel::Recorder, Event::SessionEnded)
}

/// Something the agent said about itself.
pub fn reported_intent(text: &str, tool_call_id: Option<&str>) -> Emission {
    emission(
        SESSION,
        Channel::Reported,
        Event::ReportedIntent(ReportedIntent {
            text: text.to_owned(),
            tool_call_id: tool_call_id.map(str::to_owned),
        }),
    )
}

/// A tool call the capture mechanism witnessed beginning.
pub fn tool_started(tool_call_id: &str) -> Emission {
    emission(
        SESSION,
        Channel::Observed,
        Event::ObservedToolStarted(ObservedToolStarted {
            tool_call_id: tool_call_id.to_owned(),
            tool_name: "SyntheticTool".to_owned(),
            arguments: serde_json::json!({ "target": "/synthetic/example" }),
        }),
    )
}

/// A tool call the capture mechanism witnessed ending.
pub fn tool_finished(tool_call_id: &str, outcome: ToolOutcome) -> Emission {
    emission(
        SESSION,
        Channel::Observed,
        Event::ObservedToolFinished(ObservedToolFinished {
            tool_call_id: tool_call_id.to_owned(),
            outcome,
            result: serde_json::json!({ "status": "synthetic" }),
        }),
    )
}

/// Build a raw record directly, so tests can hand-craft damaged recordings that
/// the appender would never produce.
pub fn raw_record(sequence: u64, recorded_at: &str, session: &str, event: Event) -> Record {
    let channel = match &event {
        Event::ReportedIntent(_) => Channel::Reported,
        Event::ObservedToolStarted(_) | Event::ObservedToolFinished(_) => Channel::Observed,
        Event::SessionStarted | Event::SessionEnded => Channel::Recorder,
    };
    Record {
        schema_version: SCHEMA_VERSION,
        session_id: session.to_owned(),
        sequence,
        recorded_at: ts(recorded_at),
        provenance: provenance(channel),
        event,
    }
}

/// Render records as an NDJSON recording.
pub fn ndjson(records: &[Record]) -> String {
    let mut out = String::new();
    for record in records {
        out.push_str(&serde_json::to_string(record).expect("record should serialize"));
        out.push('\n');
    }
    out
}
