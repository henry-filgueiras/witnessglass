//! WitnessGlass raw recording kernel.
//!
//! > WitnessGlass is a flight recorder for coding agents: declared intent,
//! > observed activity, and temporal replay.
//!
//! This is an experimental kernel, not a finished recorder. It implements an
//! append-only UTF-8 NDJSON session recording that can be written a record at a
//! time and replayed deterministically in append order, plus one passive adapter
//! that translates Claude Code command-hook payloads into records.
//!
//! # What a recording is
//!
//! One file is one session. Each line is one complete record: a schema version
//! stamp, the session id, a strictly increasing append sequence, the wall-clock
//! time the recorder wrote it, whatever causal context the integration supplied,
//! its [`Provenance`], and an [`Event`].
//!
//! # Schema versions
//!
//! [`record::v1`] is frozen and readable. [`record::v2`] is what this build
//! writes. v2 exists because v1's tool vocabulary made a claim that a real
//! cooperative integration cannot support: it had one "tool started" event, and
//! Claude Code has no hook that witnesses a call starting. What fires before a
//! call is a *request*, which may then be modified, denied, deferred, or never
//! executed. v2 keeps requested input, effective input, success, failure, and
//! denial as five distinct things rather than one.
//!
//! A recording uses one schema version throughout. Both versions replay; only v2
//! is written; mixing them within a recording is refused at both ends.
//!
//! # What the kernel refuses to do
//!
//! It does not merge reported and observed events into a single tidier
//! statement, it does not infer intent from commands or from temporal adjacency,
//! it does not invent agent parentage, and it does not reorder a recording to
//! make its timestamps look sensible. Correlating a reported intent with a tool
//! observation is left to whoever reads the stream, and correlation is not
//! fusion.
//!
//! # Deriving a view
//!
//! [`inspection`] projects a replayed recording into a derived, disposable
//! inspection model in which every derived claim carries the raw sequence
//! numbers supporting it. It is the only correlation layer in this crate, and it
//! borrows the raw records rather than owning them, so it cannot rewrite what it
//! derives from.
//!
//! # Order under concurrency
//!
//! `sequence` is the recorder's acquisition order and the canonical storage
//! order. Claude runs matching hooks as parallel processes, so under concurrency
//! it is not automatically a total causal order for the agent's execution. Raw
//! replay never reorders; deriving a causal view from correlation ids and
//! supplied durations is a projection's job.
//!
//! # Privacy
//!
//! A recording contains whatever the emitter puts in it, which in real use means
//! prompts, commands, paths, output, and credentials. Nothing here redacts
//! anything. Recordings are not safe to share.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod append;
pub mod claude;
pub mod error;
pub mod inspection;
pub mod record;
pub mod replay;
pub mod view;

pub use append::append;
pub use error::{Error, Result};
// The inspection projection is derived and disposable. Its types are internal
// and unstable; nothing outside this repository may depend on their shape.
pub use inspection::{Inspection, inspect};
// The current schema's vocabulary is re-exported unqualified. `Record` here is
// always a v2 record; a v1 record is reachable only as `record::v1::Record`, so
// no caller can pick one up believing it is the other.
pub use record::v2::{
    Context, Emission, Event, Record, ReportedIntent, SessionEnded, SessionStarted,
    SubagentStarted, SubagentStopped, ToolDenied, ToolFailed, ToolRequested, ToolSucceeded,
};
pub use record::{AnyRecord, Channel, Provenance, SCHEMA_VERSION, SUPPORTED_SCHEMA_VERSIONS};
pub use replay::{Replay, Tail, replay_bytes, replay_file};
pub use view::{Capability, Snapshot, Viewer};
