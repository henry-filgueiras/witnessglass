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
        /// Version this build supports.
        supported: u32,
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
                "line {line}: unsupported schema version {found}; this build implements \
                 schema version {supported}"
            ),
            Error::Corruption { line, reason } => {
                write!(f, "line {line}: corrupt record: {reason}")
            }
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
