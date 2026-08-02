//! The raw stream v1 record: envelope, provenance, and event vocabulary.
//!
//! A recording is UTF-8 NDJSON. One newline-terminated line is exactly one
//! complete record. Records are appended and never rewritten.
//!
//! The envelope carries the things a later reader needs in order to know what
//! it is holding — schema version, session, append sequence, recorded time, and
//! where the event came from — and keeps them separate from the event payload
//! itself.

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// Schema version implemented by this build. Every record carries it, and a
/// reader refuses any other value rather than guessing.
pub const SCHEMA_VERSION: u32 = 1;

/// The epistemic channel an event arrived on.
///
/// This is the distinction the whole project rests on: what an agent *says* and
/// what the machinery *sees* are different kinds of claim, and neither is
/// silently promoted into the other.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Channel {
    /// Semantics the agent supplied about itself: intent, hypotheses, plans,
    /// friendly descriptions. Useful, and not ground truth.
    Reported,
    /// Operational activity the capture mechanism witnessed: tool invocation,
    /// completion, failure. Solid within its coverage, silent outside it, and
    /// unable to explain why anything was wanted.
    Observed,
    /// Facts about the recording process itself, asserted by the recorder.
    Recorder,
}

impl Channel {
    /// Stable lowercase name, as it appears in a record.
    pub fn as_str(self) -> &'static str {
        match self {
            Channel::Reported => "reported",
            Channel::Observed => "observed",
            Channel::Recorder => "recorder",
        }
    }
}

/// Where a record came from.
///
/// `adapter` names the integration that produced the event and `mechanism`
/// names the specific capture point within it — enough for a reader to work out
/// what that source could and could not see. There is deliberately no numeric
/// fidelity score: a made-up number would imply a precision nobody has measured.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Provenance {
    /// Epistemic channel this event arrived on.
    pub channel: Channel,
    /// Integration that produced the event, e.g. `"manual"`.
    pub adapter: String,
    /// Capture point within that integration, e.g. `"cli-stdin"`.
    pub mechanism: String,
}

/// Agent-supplied semantics. A claim, recorded as a claim.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReportedIntent {
    /// What the agent said, in its own words.
    pub text: String,
    /// Optional correlation to a tool call this statement is about.
    ///
    /// Sharing an id with a tool observation correlates the two. It does not
    /// merge them, and it does not make either one evidence for the other.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

/// A tool invocation the capture mechanism witnessed beginning.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservedToolStarted {
    /// Stable id correlating this call's lifecycle. Maps to a Claude Code
    /// hook's `tool_use_id`.
    pub tool_call_id: String,
    /// Tool name as delivered by the capture mechanism.
    pub tool_name: String,
    /// Arguments as delivered, stored uninterpreted. Nothing is dropped, but
    /// JSON normalization applies: the value survives semantically, not
    /// byte-for-byte.
    pub arguments: serde_json::Value,
}

/// How a tool call ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolOutcome {
    /// The call completed.
    Succeeded,
    /// The call failed.
    Failed,
}

/// A tool invocation the capture mechanism witnessed ending.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservedToolFinished {
    /// Same id as the corresponding start, when a start was observed at all.
    pub tool_call_id: String,
    /// Completion or failure.
    pub outcome: ToolOutcome,
    /// Result as delivered, stored uninterpreted. Nothing is dropped, but JSON
    /// normalization applies: the value survives semantically, not
    /// byte-for-byte.
    pub result: serde_json::Value,
}

/// The v1 event vocabulary.
///
/// Deliberately small. Anything richer is a derived projection's problem, not
/// the raw stream's.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Event {
    /// Opening session boundary.
    SessionStarted,
    /// Closing session boundary.
    SessionEnded,
    /// Reported semantics.
    ReportedIntent(ReportedIntent),
    /// Observed start of a tool call.
    ObservedToolStarted(ObservedToolStarted),
    /// Observed end of a tool call.
    ObservedToolFinished(ObservedToolFinished),
}

impl Event {
    /// Stable name of this event kind, as it appears in a record.
    pub fn kind(&self) -> &'static str {
        match self {
            Event::SessionStarted => "session_started",
            Event::SessionEnded => "session_ended",
            Event::ReportedIntent(_) => "reported_intent",
            Event::ObservedToolStarted(_) => "observed_tool_started",
            Event::ObservedToolFinished(_) => "observed_tool_finished",
        }
    }

    /// Channels this event kind may legitimately arrive on.
    ///
    /// Intent can only be reported — no mechanism observes it. Tool lifecycle
    /// can only be observed — an agent describing a tool call is making a
    /// statement, not witnessing one. Session boundaries may be asserted by the
    /// recorder or witnessed by a capture mechanism, but are never a semantic
    /// claim.
    fn allowed_channels(&self) -> &'static [Channel] {
        match self {
            Event::SessionStarted | Event::SessionEnded => &[Channel::Recorder, Channel::Observed],
            Event::ReportedIntent(_) => &[Channel::Reported],
            Event::ObservedToolStarted(_) | Event::ObservedToolFinished(_) => &[Channel::Observed],
        }
    }

    /// Human-readable rendering of [`Event::allowed_channels`].
    fn allowed_channels_display(&self) -> &'static str {
        match self {
            Event::SessionStarted | Event::SessionEnded => "recorder, observed",
            Event::ReportedIntent(_) => "reported",
            Event::ObservedToolStarted(_) | Event::ObservedToolFinished(_) => "observed",
        }
    }

    /// Reject an event presented on a channel it cannot come from.
    pub(crate) fn check_channel(&self, channel: Channel) -> Result<()> {
        if self.allowed_channels().contains(&channel) {
            Ok(())
        } else {
            Err(Error::ChannelNotAllowed {
                event: self.kind(),
                channel,
                allowed: self.allowed_channels_display(),
            })
        }
    }
}

/// What an emitter hands to the recorder.
///
/// The emitter supplies the session, the provenance, and the event. The
/// recorder supplies the schema version, the append sequence, and the recorded
/// timestamp — those are facts about the act of recording, and an emitter is
/// not in a position to assert them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Emission {
    /// Session this event belongs to.
    pub session_id: String,
    /// Where the event came from.
    pub provenance: Provenance,
    /// The event itself.
    pub event: Event,
}

impl Emission {
    /// Reject an emission whose event and channel are incoherent.
    pub fn validate(&self) -> Result<()> {
        self.event.check_channel(self.provenance.channel)
    }
}

/// One complete record: exactly one line of a recording.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    /// Schema version of this record.
    pub schema_version: u32,
    /// Session this recording belongs to.
    pub session_id: String,
    /// Strictly increasing append sequence, starting at 1. This, and nothing
    /// else, defines canonical replay order.
    pub sequence: u64,
    /// Wall-clock time the recorder wrote this record. Descriptive metadata.
    /// It never determines order.
    pub recorded_at: jiff::Timestamp,
    /// Where the event came from.
    pub provenance: Provenance,
    /// The event itself.
    pub event: Event,
}

impl Record {
    /// Validate the invariants a v1 record must satisfy on its own, without
    /// reference to its neighbours.
    pub(crate) fn validate_self(&self, line: usize) -> Result<()> {
        if self.schema_version != SCHEMA_VERSION {
            return Err(Error::UnsupportedSchemaVersion {
                line,
                found: u64::from(self.schema_version),
                supported: SCHEMA_VERSION,
            });
        }
        self.event
            .check_channel(self.provenance.channel)
            .map_err(|err| Error::Corruption {
                line,
                reason: err.to_string(),
            })
    }
}
