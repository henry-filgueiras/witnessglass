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
//! # Schema versions
//!
//! A recording written under any schema version this build knows is replayable,
//! so an existing v1 recording stays readable after v2 arrives. What is refused
//! is a recording that *mixes* versions: the first record establishes the
//! version and every later record must agree. Records come back as [`AnyRecord`]
//! rather than one flattened type, so a reader cannot pick up a v1 record
//! believing it holds v2 evidence.
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
//!
//! The fragment is treated as opaque bytes, not text. A write cut short inside
//! a multibyte character leaves invalid UTF-8 behind, and that says nothing
//! about the complete records preceding it. So the input is split at its final
//! newline *before* anything is decoded, and only the newline-terminated prefix
//! is required to be valid UTF-8.

use std::path::Path;

use crate::error::{Error, Result};
use crate::record::{AnyRecord, parse_line, probe_schema_version};

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
    pub records: Vec<AnyRecord>,
    /// Schema version the recording is written in, established by its first
    /// record. `None` for a recording holding no complete records.
    pub schema_version: Option<u64>,
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
    // Split as bytes, before any decoding. The fragment is the part nothing is
    // known about, so it must also be the part nothing is attempted on: a write
    // interrupted mid-record can stop inside a multibyte character, producing
    // invalid UTF-8 by construction. Validating the whole input first would let
    // those bytes condemn every complete record in front of them.
    let (complete, fragment): (&[u8], &[u8]) = match bytes.iter().rposition(|&b| b == b'\n') {
        Some(index) => bytes.split_at(index + 1),
        None => (&[], bytes),
    };

    let tail = if fragment.is_empty() {
        Tail::Complete
    } else {
        Tail::Truncated {
            byte_offset: complete.len() as u64,
            bytes: fragment.len(),
        }
    };

    // Only the newline-terminated prefix is required to be valid UTF-8. Inside
    // a complete record, invalid UTF-8 is still corruption: that record was
    // written whole and is wrong.
    let text = std::str::from_utf8(complete).map_err(|err| {
        let valid = err.valid_up_to();
        Error::Corruption {
            line: complete[..valid].iter().filter(|&&b| b == b'\n').count() + 1,
            reason: format!("recording is not valid UTF-8 at byte {valid}"),
        }
    })?;

    let mut records = Vec::new();
    let mut session_id: Option<String> = None;
    let mut schema_version: Option<u64> = None;

    for (index, line) in text.lines().enumerate() {
        let line_number = index + 1;

        if line.trim().is_empty() {
            return Err(Error::Corruption {
                line: line_number,
                reason: "blank line is not a record".to_owned(),
            });
        }

        // The first record establishes the recording's vocabulary. A later
        // record declaring a different one is refused rather than read against
        // a schema it does not claim to use.
        let version = probe_schema_version(line, line_number)?;
        match schema_version {
            None => schema_version = Some(version),
            Some(expected) if expected != version => {
                return Err(Error::MixedSchemaVersions {
                    line: line_number,
                    expected,
                    found: version,
                });
            }
            Some(_) => {}
        }

        let record = parse_line(line, line_number)?;

        // Sequence is the canonical order, so it must be unambiguous: it starts
        // at 1 and advances by exactly 1. A duplicate, a decrease, or a gap all
        // leave a reader unable to say what the history was — a gap in
        // particular cannot be distinguished from a deletion — so all three are
        // rejected rather than repaired.
        let expected = line_number as u64;
        if record.sequence() != expected {
            return Err(Error::SequenceViolation {
                line: line_number,
                expected,
                found: record.sequence(),
            });
        }

        match &session_id {
            None => session_id = Some(record.session_id().to_owned()),
            Some(expected) if expected != record.session_id() => {
                return Err(Error::SessionMismatch {
                    line: line_number,
                    expected: expected.clone(),
                    found: record.session_id().to_owned(),
                });
            }
            Some(_) => {}
        }

        records.push(record);
    }

    Ok(Replay {
        records,
        schema_version,
        tail,
    })
}
