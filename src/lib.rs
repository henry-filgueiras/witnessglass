//! WitnessGlass raw recording kernel.
//!
//! > WitnessGlass is a flight recorder for coding agents: declared intent,
//! > observed activity, and temporal replay.
//!
//! This is an experimental kernel, not a finished recorder. It implements one
//! concrete thing: an append-only UTF-8 NDJSON session recording that can be
//! written a record at a time and replayed deterministically in append order.
//!
//! There is no adapter here — nothing yet connects this to Claude or to any
//! other agent. Events arrive from whoever calls [`append`] or the CLI.
//!
//! # What a recording is
//!
//! One file is one session. Each line is one complete [`Record`]: a
//! [`SCHEMA_VERSION`] stamp, the session id, a strictly increasing append
//! sequence, the wall-clock time the recorder wrote it, its [`Provenance`], and
//! an [`Event`].
//!
//! # What the kernel refuses to do
//!
//! It does not merge reported and observed events into a single tidier
//! statement, it does not infer intent from commands or from temporal
//! adjacency, and it does not reorder a recording to make its timestamps look
//! sensible. Correlating a reported intent with a tool observation is left to
//! whoever reads the stream, and correlation is not fusion.
//!
//! # Privacy
//!
//! A recording contains whatever the emitter puts in it, which in real use
//! means prompts, commands, paths, output, and credentials. Nothing here
//! redacts anything. Recordings are not safe to share.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod append;
pub mod error;
pub mod record;
pub mod replay;

pub use append::append;
pub use error::{Error, Result};
pub use record::{
    Channel, Emission, Event, ObservedToolFinished, ObservedToolStarted, Provenance, Record,
    ReportedIntent, SCHEMA_VERSION, ToolOutcome,
};
pub use replay::{Replay, Tail, replay_bytes, replay_file};
