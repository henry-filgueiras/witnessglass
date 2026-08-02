//! Record envelopes: what is shared across schema versions, and how a line of a
//! recording is turned into a record of the right version.
//!
//! A recording is UTF-8 NDJSON. One newline-terminated line is exactly one
//! complete record. Records are appended and never rewritten.
//!
//! # Two schema versions
//!
//! [`v1`] is the synthetic kernel's vocabulary, frozen. It is readable and no
//! longer writable. [`v2`] is what this build writes, and it exists because v1
//! could not represent Claude Code's tool lifecycle honestly — see the crate
//! documentation and `docs/claude-adapter.md`.
//!
//! A recording uses one schema version throughout. Replay refuses a recording
//! that mixes them and the appender refuses to add a record of one version to a
//! recording of another, because a stream whose records mean different things at
//! different lines is not a canonical history of anything.

pub mod v1;
pub mod v2;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// Schema version this build writes.
pub const SCHEMA_VERSION: u32 = 2;

/// Schema versions this build can replay, for error messages.
pub const SUPPORTED_SCHEMA_VERSIONS: &str = "1, 2";

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
    /// Operational activity the capture mechanism witnessed: a tool request
    /// being constructed, a call completing, failing, or being denied. Solid
    /// within its coverage, silent outside it, and unable to explain why
    /// anything was wanted.
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
    /// Integration that produced the event, e.g. `"claude-code"`.
    pub adapter: String,
    /// Capture point within that integration, e.g.
    /// `"command-hook:PostToolUse"`.
    pub mechanism: String,
}

/// A record of whichever schema version its recording uses.
///
/// Replay yields these so that an existing v1 recording stays readable without
/// being silently reinterpreted as v2. A reader that needs the payload matches
/// on the variant, which forces it to acknowledge which vocabulary it is
/// holding.
#[derive(Debug, Clone, PartialEq)]
pub enum AnyRecord {
    /// A frozen schema v1 record.
    V1(v1::Record),
    /// A schema v2 record.
    V2(v2::Record),
}

impl AnyRecord {
    /// Schema version of this record.
    pub fn schema_version(&self) -> u32 {
        match self {
            AnyRecord::V1(record) => record.schema_version,
            AnyRecord::V2(record) => record.schema_version,
        }
    }

    /// Session this record belongs to.
    pub fn session_id(&self) -> &str {
        match self {
            AnyRecord::V1(record) => &record.session_id,
            AnyRecord::V2(record) => &record.session_id,
        }
    }

    /// Position in the canonical append chain.
    pub fn sequence(&self) -> u64 {
        match self {
            AnyRecord::V1(record) => record.sequence,
            AnyRecord::V2(record) => record.sequence,
        }
    }

    /// Wall-clock time the recorder wrote this record. Descriptive metadata; it
    /// never determines order.
    pub fn recorded_at(&self) -> jiff::Timestamp {
        match self {
            AnyRecord::V1(record) => record.recorded_at,
            AnyRecord::V2(record) => record.recorded_at,
        }
    }

    /// Where this record came from.
    pub fn provenance(&self) -> &Provenance {
        match self {
            AnyRecord::V1(record) => &record.provenance,
            AnyRecord::V2(record) => &record.provenance,
        }
    }

    /// Stable name of the event kind, as it appears in the record.
    ///
    /// The v1 and v2 vocabularies do not share names, which is deliberate: a
    /// reader comparing kinds across versions should notice that it is doing so.
    pub fn event_kind(&self) -> &'static str {
        match self {
            AnyRecord::V1(record) => record.event.kind(),
            AnyRecord::V2(record) => record.event.kind(),
        }
    }

    /// The v2 record, if this is one.
    pub fn as_v2(&self) -> Option<&v2::Record> {
        match self {
            AnyRecord::V2(record) => Some(record),
            AnyRecord::V1(_) => None,
        }
    }

    /// The v1 record, if this is one.
    pub fn as_v1(&self) -> Option<&v1::Record> {
        match self {
            AnyRecord::V1(record) => Some(record),
            AnyRecord::V2(_) => None,
        }
    }

    /// Validate the invariants this record must satisfy on its own, without
    /// reference to its neighbours.
    pub(crate) fn validate_self(&self, line: usize) -> Result<()> {
        match self {
            AnyRecord::V1(record) => record.validate_self(line),
            AnyRecord::V2(record) => record.validate_self(line),
        }
    }
}

impl Serialize for AnyRecord {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        match self {
            AnyRecord::V1(record) => record.serialize(serializer),
            AnyRecord::V2(record) => record.serialize(serializer),
        }
    }
}

/// Read only the schema version, so an unsupported version is reported as such
/// instead of surfacing as a confusing parse failure against some other shape.
pub(crate) fn probe_schema_version(line: &str, line_number: usize) -> Result<u64> {
    #[derive(Deserialize)]
    struct VersionProbe {
        schema_version: u64,
    }

    let probe: VersionProbe = serde_json::from_str(line).map_err(|err| Error::Corruption {
        line: line_number,
        reason: format!("could not read schema_version: {err}"),
    })?;
    Ok(probe.schema_version)
}

/// Parse one complete line into a record of its declared schema version.
///
/// The version is read first and dispatched on, so a record is never parsed
/// against a vocabulary it does not claim to use.
pub(crate) fn parse_line(line: &str, line_number: usize) -> Result<AnyRecord> {
    let version = probe_schema_version(line, line_number)?;
    let record = match version {
        1 => AnyRecord::V1(serde_json::from_str(line).map_err(|err| Error::Corruption {
            line: line_number,
            reason: err.to_string(),
        })?),
        2 => AnyRecord::V2(serde_json::from_str(line).map_err(|err| Error::Corruption {
            line: line_number,
            reason: err.to_string(),
        })?),
        found => {
            return Err(Error::UnsupportedSchemaVersion {
                line: line_number,
                found,
                supported: SUPPORTED_SCHEMA_VERSIONS,
            });
        }
    };
    record.validate_self(line_number)?;
    Ok(record)
}
