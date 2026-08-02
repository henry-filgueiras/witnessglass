//! Errors produced by the recording kernel.
//!
//! The distinctions here are load-bearing rather than cosmetic. Corruption,
//! truncation, an unsupported schema version, and an ambiguous sequence are
//! different claims about a recording, and a reader must be able to tell them
//! apart in order to say honestly what it does and does not know.

use std::fmt;

use crate::record::Channel;

/// Result alias for kernel operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Everything that can go wrong reading or appending to a raw recording.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// Underlying I/O failure.
    Io(std::io::Error),

    /// The emission or record could not be serialized.
    Serialize(serde_json::Error),

    /// The recording's final record is not newline-terminated, so appending to
    /// it would splice a new record onto a partial one. Refused.
    AppendAfterTruncation {
        /// Length in bytes of the unterminated trailing fragment.
        bytes: usize,
    },

    /// A record declares a schema version this build does not implement.
    UnsupportedSchemaVersion {
        /// 1-based line number.
        line: usize,
        /// Version found in the record.
        found: u64,
        /// Versions this build can replay.
        supported: &'static str,
    },

    /// A recording mixes schema versions. One recording uses one vocabulary
    /// throughout; a stream whose records mean different things at different
    /// lines is not a canonical history of anything.
    MixedSchemaVersions {
        /// 1-based line number of the offending record.
        line: usize,
        /// Version established by the recording's first record.
        expected: u64,
        /// Version found in this record.
        found: u64,
    },

    /// An emission was offered to a recording written under a different schema
    /// version. Appending would mix versions within one recording.
    AppendVersionMismatch {
        /// Schema version the recording already uses.
        recording: u32,
        /// Schema version this build writes.
        writing: u32,
    },

    /// A complete, newline-terminated record could not be understood.
    Corruption {
        /// 1-based line number.
        line: usize,
        /// Why the record was rejected.
        reason: String,
    },

    /// The sequence numbers do not form the strict 1, 2, 3, … chain that makes
    /// the canonical history unambiguous.
    SequenceViolation {
        /// 1-based line number.
        line: usize,
        /// Sequence number required at this position.
        expected: u64,
        /// Sequence number actually found.
        found: u64,
    },

    /// The recording's final record already carries the largest sequence a v1
    /// record can hold, so no further record can be appended without wrapping
    /// the append chain back to zero and restarting the canonical history.
    SequenceExhausted {
        /// Sequence number the final record carries.
        last: u64,
    },

    /// One recording is one session; a record disagreed with its recording.
    SessionMismatch {
        /// 1-based line number.
        line: usize,
        /// Session id established by the recording's first record.
        expected: String,
        /// Session id found in this record.
        found: String,
    },

    /// An emission was offered to a recording belonging to another session.
    EmissionSessionMismatch {
        /// Session id the recording already belongs to.
        recording: String,
        /// Session id carried by the rejected emission.
        emission: String,
    },

    /// An event was presented on an epistemic channel it cannot come from —
    /// reported intent claiming to be observed, or a tool observation claiming
    /// to be reported.
    ChannelNotAllowed {
        /// Event kind, as it appears in the record.
        event: &'static str,
        /// Channel the emitter claimed.
        channel: Channel,
        /// Channels this event kind may legitimately arrive on.
        allowed: &'static str,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(err) => write!(f, "i/o error: {err}"),
            Error::Serialize(err) => write!(f, "could not serialize record: {err}"),
            Error::AppendAfterTruncation { bytes } => write!(
                f,
                "refusing to append: the recording ends with a {bytes}-byte unterminated \
                 fragment, and appending would splice a new record onto a partial one"
            ),
            Error::UnsupportedSchemaVersion {
                line,
                found,
                supported,
            } => write!(
                f,
                "line {line}: unsupported schema version {found}; this build can replay \
                 schema version(s) {supported}"
            ),
            Error::MixedSchemaVersions {
                line,
                expected,
                found,
            } => write!(
                f,
                "line {line}: schema version {found} in a schema version {expected} recording; \
                 one recording uses one schema version throughout"
            ),
            Error::AppendVersionMismatch { recording, writing } => write!(
                f,
                "refusing to append: the recording uses schema version {recording} and this \
                 build writes schema version {writing}; one recording uses one schema version \
                 throughout"
            ),
            Error::Corruption { line, reason } => {
                write!(f, "line {line}: corrupt record: {reason}")
            }
            Error::SequenceExhausted { last } => write!(
                f,
                "refusing to append: the recording's final record carries sequence {last}, the \
                 largest a schema v1 record can hold; appending would wrap the append chain \
                 and restart the canonical history"
            ),
            Error::SequenceViolation {
                line,
                expected,
                found,
            } => write!(
                f,
                "line {line}: sequence {found} breaks the append chain; expected {expected}"
            ),
            Error::SessionMismatch {
                line,
                expected,
                found,
            } => write!(
                f,
                "line {line}: session id {found:?} does not match the recording's session \
                 {expected:?}; one recording is one session"
            ),
            Error::EmissionSessionMismatch {
                recording,
                emission,
            } => write!(
                f,
                "emission belongs to session {emission:?} but the recording belongs to \
                 session {recording:?}"
            ),
            Error::ChannelNotAllowed {
                event,
                channel,
                allowed,
            } => write!(
                f,
                "event {event:?} cannot arrive on the {} channel; allowed: {allowed}",
                channel.as_str()
            ),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(err) => Some(err),
            Error::Serialize(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}
