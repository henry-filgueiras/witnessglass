//! Reading a recording back.
//!
//! # Canonical order
//!
//! Replay yields records in physical append order, which the sequence number
//! represents exactly. Replay never sorts by timestamp and never consults a
//! timestamp for ordering at all. Wall-clock time on a record is descriptive
//! metadata: it says when the recorder wrote the record, not where the record
//! belongs. Two records may share a timestamp — no tie-breaker is needed,
//! because sequence already decides. A clock that moves backwards mid-session
//! produces a recording with descending timestamps and a perfectly intact
//! order.
//!
//! # Damage
//!
//! Two different kinds of damage are distinguished, because they license
//! different conclusions:
//!
//! * A **corrupt** record is newline-terminated — it was written whole — but
//!   cannot be understood. That is a real defect and fails loudly.
//! * A **truncated tail** is a final fragment with no terminating newline. It
//!   means the recording stops mid-record: an emitter died, a disk filled, a
//!   copy was cut short. The valid prefix is still evidence, so replay returns
//!   it and reports the recording as incomplete. The fragment is never parsed
//!   and never presented as an event.

use std::path::Path;

use crate::error::{Error, Result};
use crate::record::{Record, SCHEMA_VERSION};

/// Whether a recording ended cleanly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tail {
    /// The final record was newline-terminated. Nothing is known to be missing.
    Complete,
    /// The recording ends mid-record. Everything before the fragment is intact;
    /// the fragment itself is not an event and is not reported as one.
    Truncated {
        /// Byte offset where the unterminated fragment begins.
        byte_offset: u64,
        /// Length of the fragment in bytes.
        bytes: usize,
    },
}

impl Tail {
    /// Whether the recording is known to be incomplete.
    pub fn is_truncated(&self) -> bool {
        matches!(self, Tail::Truncated { .. })
    }
}

/// The result of replaying a recording.
#[derive(Debug, Clone, PartialEq)]
pub struct Replay {
    /// Records in canonical append order.
    pub records: Vec<Record>,
    /// Whether anything is known to be missing from the end.
    pub tail: Tail,
}

/// Replay a recording from disk.
pub fn replay_file(recording: &Path) -> Result<Replay> {
    let bytes = std::fs::read(recording)?;
    replay_bytes(&bytes)
}

/// Replay a recording already held in memory.
///
/// The whole recording is read at once. That is honest for the session-sized
/// files this kernel produces and is a deliberate deferral, not a claim about
/// arbitrarily large recordings.
pub fn replay_bytes(bytes: &[u8]) -> Result<Replay> {
    let text = std::str::from_utf8(bytes).map_err(|err| {
        let valid = err.valid_up_to();
        Error::Corruption {
            line: bytes[..valid].iter().filter(|&&b| b == b'\n').count() + 1,
            reason: format!("recording is not valid UTF-8 at byte {valid}"),
        }
    })?;

    // Split the complete, newline-terminated prefix from any trailing fragment.
    let (complete, fragment) = match text.rfind('\n') {
        Some(index) => (&text[..=index], &text[index + 1..]),
        None => ("", text),
    };

    let tail = if fragment.is_empty() {
        Tail::Complete
    } else {
        Tail::Truncated {
            byte_offset: complete.len() as u64,
            bytes: fragment.len(),
        }
    };

    let mut records = Vec::new();
    let mut session_id: Option<String> = None;

    for (index, line) in complete.lines().enumerate() {
        let line_number = index + 1;

        if line.trim().is_empty() {
            return Err(Error::Corruption {
                line: line_number,
                reason: "blank line is not a record".to_owned(),
            });
        }

        check_schema_version(line, line_number)?;

        let record: Record = serde_json::from_str(line).map_err(|err| Error::Corruption {
            line: line_number,
            reason: err.to_string(),
        })?;
        record.validate_self(line_number)?;

        // Sequence is the canonical order, so it must be unambiguous: it starts
        // at 1 and advances by exactly 1. A duplicate, a decrease, or a gap all
        // leave a reader unable to say what the history was — a gap in
        // particular cannot be distinguished from a deletion — so all three are
        // rejected rather than repaired.
        let expected = line_number as u64;
        if record.sequence != expected {
            return Err(Error::SequenceViolation {
                line: line_number,
                expected,
                found: record.sequence,
            });
        }

        match &session_id {
            None => session_id = Some(record.session_id.clone()),
            Some(expected) if *expected != record.session_id => {
                return Err(Error::SessionMismatch {
                    line: line_number,
                    expected: expected.clone(),
                    found: record.session_id.clone(),
                });
            }
            Some(_) => {}
        }

        records.push(record);
    }

    Ok(Replay { records, tail })
}

/// Read only the schema version, so an unsupported version is reported as such
/// instead of surfacing as a confusing parse failure against the v1 shape.
fn check_schema_version(line: &str, line_number: usize) -> Result<()> {
    #[derive(serde::Deserialize)]
    struct VersionProbe {
        schema_version: u64,
    }

    let probe: VersionProbe = serde_json::from_str(line).map_err(|err| Error::Corruption {
        line: line_number,
        reason: format!("could not read schema_version: {err}"),
    })?;

    if probe.schema_version != u64::from(SCHEMA_VERSION) {
        return Err(Error::UnsupportedSchemaVersion {
            line: line_number,
            found: probe.schema_version,
            supported: SCHEMA_VERSION,
        });
    }
    Ok(())
}
