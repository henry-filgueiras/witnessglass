//! Raw stream v2: requested and effective tool evidence, kept separate.
//!
//! v2 exists for one reason. A cooperative Claude Code integration does not
//! deliver "a tool call started" and "a tool call finished". It delivers, at
//! distinct and independently-droppable moments:
//!
//! * a **request** the model constructed, which may afterwards be modified,
//!   denied, deferred, or never executed;
//! * a **success**, carrying the input actually sent and the response;
//! * a **failure**, carrying the effective input, an error, and whether the call
//!   was interrupted;
//! * a **denial**, which is not an execution failure and must not be filed as
//!   one.
//!
//! v1 had one `observed_tool_started` and one `observed_tool_finished` with a
//! success/failure flag. Mapping the pre-execution hook onto "started" would
//! record a request as an execution, and collapsing denial into failure would
//! make "the agent was stopped" indistinguishable from "the agent tried and it
//! broke". Both are exactly the quiet promotion decision:2 forbids, so the
//! vocabulary changed instead of the truth.
//!
//! # Causal context
//!
//! The envelope carries a [`Context`] holding only identifiers the integration
//! actually supplies: `prompt_id`, and inside a subagent `agent_id` and
//! `agent_type`. Nothing here invents a root agent id, a parent id, a span id,
//! or a hierarchy. Where Claude supplies a parent identifier it is recorded as
//! delivered; where it does not, the field is absent and stays absent. Parentage
//! is never inferred from timing.
//!
//! This is the empirical precursor to a possible agent-local causal overlay, not
//! its implementation.

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::record::{Channel, Provenance};

/// Schema version of the records in this module.
pub const SCHEMA_VERSION: u32 = 2;

/// Causal context supplied by the integration, and nothing else.
///
/// Every field is optional because every field is genuinely optional in the
/// payloads this is built from. An absent field means the integration did not
/// supply it, which is a fact about coverage worth preserving; it is never
/// filled in from somewhere else.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Context {
    /// Identifier of the user prompt this event belongs to, when supplied.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_id: Option<String>,
    /// Identifier of the agent this event came from, when the event came from a
    /// subagent and the integration said so.
    ///
    /// This is the *current* agent, not a parent and not a child. Subagent
    /// lifecycle events do not populate it: there the identifier names the
    /// subagent being started or stopped, so it belongs in the event payload,
    /// where it cannot be mistaken for the identity of the emitter.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    /// Type or name of the current agent, when supplied.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_type: Option<String>,
}

impl Context {
    /// Whether the integration supplied no causal context at all.
    pub fn is_empty(&self) -> bool {
        self.prompt_id.is_none() && self.agent_id.is_none() && self.agent_type.is_none()
    }
}

/// Opening session boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionStarted {
    /// How the session started, as reported by the integration — for Claude
    /// Code one of `startup`, `resume`, `clear`, `compact`, `fork`. Stored as a
    /// string rather than an enum so that a value this build has never heard of
    /// is recorded rather than rejected. Absent when the recorder asserted the
    /// boundary itself.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

/// Closing session boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionEnded {
    /// Why the session ended, as reported by the integration. Stored as a
    /// string for the same reason as [`SessionStarted::source`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
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
    pub tool_use_id: Option<String>,
}

/// A tool request the agent constructed.
///
/// This is evidence that a request existed, and nothing more. It is not evidence
/// that the call ran, that it ran with this input, or that it ran at all. The
/// request may still be modified, denied, deferred, or dropped.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolRequested {
    /// Identifier correlating this call's records.
    pub tool_use_id: String,
    /// Tool name as delivered.
    pub tool_name: String,
    /// The input as requested, stored uninterpreted. Nothing is dropped, but
    /// JSON normalization applies: the value survives semantically, not
    /// byte-for-byte.
    pub requested_input: serde_json::Value,
}

/// A tool call that executed successfully.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolSucceeded {
    /// Identifier correlating this call's records.
    pub tool_use_id: String,
    /// Tool name as delivered.
    pub tool_name: String,
    /// The input actually sent to the tool, which may differ from any requested
    /// input recorded earlier. Stored uninterpreted.
    pub effective_input: serde_json::Value,
    /// The tool's response, stored uninterpreted. This is the tool-level result
    /// the integration reports, not an account of what the tool did internally.
    pub response: serde_json::Value,
    /// Execution time in milliseconds, when the integration supplied one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
}

/// A tool call that executed and failed.
///
/// Distinct from [`ToolDenied`]: this call ran. Something went wrong while it
/// was running, or it was interrupted while running.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolFailed {
    /// Identifier correlating this call's records.
    pub tool_use_id: String,
    /// Tool name as delivered.
    pub tool_name: String,
    /// The input actually sent to the tool. Stored uninterpreted.
    pub effective_input: serde_json::Value,
    /// Error as reported by the integration, stored as delivered.
    pub error: String,
    /// Whether the call was interrupted, when the integration said either way.
    /// Absent means it did not say, which is not the same as `false`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interrupted: Option<bool>,
    /// Elapsed time in milliseconds before the failure, when supplied.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
}

/// A tool call that was denied permission.
///
/// This call did not run. Recording it as a failure would make "the agent was
/// stopped from doing this" indistinguishable from "the agent did this and it
/// broke", which are different facts about a session and licence different
/// conclusions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolDenied {
    /// Identifier correlating this call's records.
    pub tool_use_id: String,
    /// Tool name as delivered.
    pub tool_name: String,
    /// The input as requested and refused. Never executed, so it is a requested
    /// input rather than an effective one.
    pub requested_input: serde_json::Value,
}

/// A subagent the integration reported starting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubagentStarted {
    /// The identifier of the subagent being started. This is the child, not the
    /// emitter and not a parent.
    pub agent_id: String,
    /// Type or name of the subagent, when supplied.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_type: Option<String>,
    /// Parent agent identifier, recorded only when the integration supplied one.
    /// Never inferred, and in particular never inferred from timing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_agent_id: Option<String>,
    /// Parent agent type, recorded only when supplied.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_agent_type: Option<String>,
}

/// A subagent the integration reported stopping.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubagentStopped {
    /// The identifier of the subagent that stopped.
    pub agent_id: String,
    /// Type or name of the subagent, when supplied.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_type: Option<String>,
    /// Parent agent identifier, recorded only when supplied. Never inferred.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_agent_id: Option<String>,
    /// Parent agent type, recorded only when supplied.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_agent_type: Option<String>,
}

/// The v2 event vocabulary.
///
/// Still deliberately small. The growth over v1 is entirely in the tool
/// lifecycle, where v1 was making a claim the evidence could not support.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Event {
    /// Opening session boundary.
    SessionStarted(SessionStarted),
    /// Closing session boundary.
    SessionEnded(SessionEnded),
    /// Reported semantics.
    ReportedIntent(ReportedIntent),
    /// A request was constructed. Not proof of execution.
    ToolRequested(ToolRequested),
    /// A call executed successfully.
    ToolSucceeded(ToolSucceeded),
    /// A call executed and failed.
    ToolFailed(ToolFailed),
    /// A call was denied and did not execute.
    ToolDenied(ToolDenied),
    /// A subagent started.
    SubagentStarted(SubagentStarted),
    /// A subagent stopped.
    SubagentStopped(SubagentStopped),
}

impl Event {
    /// Stable name of this event kind, as it appears in a record.
    pub fn kind(&self) -> &'static str {
        match self {
            Event::SessionStarted(_) => "session_started",
            Event::SessionEnded(_) => "session_ended",
            Event::ReportedIntent(_) => "reported_intent",
            Event::ToolRequested(_) => "tool_requested",
            Event::ToolSucceeded(_) => "tool_succeeded",
            Event::ToolFailed(_) => "tool_failed",
            Event::ToolDenied(_) => "tool_denied",
            Event::SubagentStarted(_) => "subagent_started",
            Event::SubagentStopped(_) => "subagent_stopped",
        }
    }

    /// The correlation identifier this event carries, if any.
    ///
    /// Correlation is offered as a convenience for building a derived view. It
    /// does not merge anything, and two records sharing an id remain two
    /// records with two provenances.
    pub fn tool_use_id(&self) -> Option<&str> {
        match self {
            Event::ReportedIntent(intent) => intent.tool_use_id.as_deref(),
            Event::ToolRequested(requested) => Some(&requested.tool_use_id),
            Event::ToolSucceeded(succeeded) => Some(&succeeded.tool_use_id),
            Event::ToolFailed(failed) => Some(&failed.tool_use_id),
            Event::ToolDenied(denied) => Some(&denied.tool_use_id),
            _ => None,
        }
    }

    /// Channels this event kind may legitimately arrive on.
    ///
    /// Intent can only be reported — no mechanism observes it. Tool and subagent
    /// lifecycle can only be observed — an agent describing a call is making a
    /// statement, not witnessing one. Session boundaries may be asserted by the
    /// recorder or witnessed by a capture mechanism, but are never a semantic
    /// claim.
    fn allowed_channels(&self) -> &'static [Channel] {
        match self {
            Event::SessionStarted(_) | Event::SessionEnded(_) => {
                &[Channel::Recorder, Channel::Observed]
            }
            Event::ReportedIntent(_) => &[Channel::Reported],
            Event::ToolRequested(_)
            | Event::ToolSucceeded(_)
            | Event::ToolFailed(_)
            | Event::ToolDenied(_)
            | Event::SubagentStarted(_)
            | Event::SubagentStopped(_) => &[Channel::Observed],
        }
    }

    /// Human-readable rendering of [`Event::allowed_channels`].
    fn allowed_channels_display(&self) -> &'static str {
        match self {
            Event::SessionStarted(_) | Event::SessionEnded(_) => "recorder, observed",
            Event::ReportedIntent(_) => "reported",
            _ => "observed",
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
/// The emitter supplies the session, the causal context, the provenance, and the
/// event. The recorder supplies the schema version, the append sequence, and the
/// recorded timestamp — those are facts about the act of recording, and an
/// emitter is not in a position to assert them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Emission {
    /// Session this event belongs to.
    pub session_id: String,
    /// Causal context supplied by the integration.
    #[serde(default, skip_serializing_if = "Context::is_empty")]
    pub context: Context,
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

/// One complete v2 record: exactly one line of a v2 recording.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    /// Schema version of this record. Always 2.
    pub schema_version: u32,
    /// Session this recording belongs to.
    pub session_id: String,
    /// Strictly increasing append sequence, starting at 1.
    ///
    /// This is the recorder's acquisition order, and it is the canonical storage
    /// order of the recording. Under parallel hook processes it is not
    /// automatically a total causal order for the agent's execution: two calls
    /// that completed concurrently are recorded in whichever order their hook
    /// processes won the lock. Per-call correlation and supplied durations can
    /// support a derived causal view; raw replay does not reorder.
    pub sequence: u64,
    /// Wall-clock time the recorder wrote this record. Descriptive metadata.
    /// It never determines order.
    pub recorded_at: jiff::Timestamp,
    /// Causal context supplied by the integration, omitted when it supplied
    /// none.
    #[serde(default, skip_serializing_if = "Context::is_empty")]
    pub context: Context,
    /// Where the event came from.
    pub provenance: Provenance,
    /// The event itself.
    pub event: Event,
}

impl Record {
    /// Validate the invariants a v2 record must satisfy on its own, without
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
