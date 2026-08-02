//! Raw stream v1: frozen.
//!
//! This is the vocabulary the synthetic kernel was built and tested against,
//! settled by decision:3. It is kept because recordings written under it exist
//! and must stay replayable. Nothing writes it any more.
//!
//! It is frozen rather than extended because it makes a claim it cannot support
//! once a real adapter is attached. `observed_tool_started` says a tool call was
//! witnessed *beginning*, and there is no cooperative Claude hook that witnesses
//! that. The hook that fires before a call fires after the request is
//! constructed and before it is processed, and the call may then be modified,
//! denied, deferred, or never executed at all. Stretching v1 around that would
//! mean recording a request as though it were an execution, which is exactly the
//! kind of quiet promotion decision:2 forbids. See [`super::v2`].

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::record::{Channel, Provenance};

/// Schema version of the records in this module.
pub const SCHEMA_VERSION: u32 = 1;

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

/// A tool invocation the capture mechanism claimed to witness beginning.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservedToolStarted {
    /// Stable id correlating this call's lifecycle.
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

/// The v1 event vocabulary. Five kinds, frozen.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Event {
    /// Opening session boundary.
    SessionStarted,
    /// Closing session boundary.
    SessionEnded,
    /// Reported semantics.
    ReportedIntent(ReportedIntent),
    /// Claimed observation of a tool call beginning.
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
    fn check_channel(&self, channel: Channel) -> Result<()> {
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

/// One complete v1 record: exactly one line of a v1 recording.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    /// Schema version of this record. Always 1.
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
                supported: crate::record::SUPPORTED_SCHEMA_VERSIONS,
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
