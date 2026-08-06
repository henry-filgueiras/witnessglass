//! **Disposable research experiment.** sprint:4, task:14.
//!
//! Nothing in this module is product code, and nothing outside it — not the
//! recorder, not the schema, not [`crate::replay`], not [`crate::inspection`],
//! not [`crate::view`], not the CLI — depends on any of it. It exists to answer
//! one question and to be deleted if the answer is no:
//!
//! > Can a validated WitnessGlass event stream be projected into a regular,
//! > normalized multivariate behavioral signal that preserves enough observable
//! > temporal structure for numerical algorithms to recover deliberately
//! > injected motifs, regime changes, or multiscale behavior?
//!
//! It is reachable from `examples/behavioral-signal.rs` and from
//! `tests/behavioral_signal.rs`, and from nowhere else. `witnessglass --help`
//! does not mention it. Deleting this module, that example, that test, and
//! `fixtures/synthetic-behavioral-oracle.ndjson` removes the experiment
//! completely and leaves the crate exactly as sprint:3 left it.
//!
//! # What it is allowed to be
//!
//! [`signal`] is a projection in the sense `CLAUDE.md` §3 means, and it is held
//! to the same conditions as [`crate::inspection`]: derived, disposable,
//! rebuildable from raw, borrowing rather than owning, reading no file and
//! consulting no clock. It consumes an already-validated [`crate::Inspection`]
//! rather than raw NDJSON, because decision:6 puts the meaning of a recording in
//! exactly one place and a second reader of the raw stream would be a second
//! opinion about what a recording says.
//!
//! # What it is not allowed to be
//!
//! No detector. This round produces the numbers a Matrix Profile, a changepoint
//! search, or a wavelet transform would be handed; it does not run one, and the
//! arithmetic here goes no further than what validating the substrate needs.
//!
//! No semantic classification. See [`signal`]'s own header for the list of
//! candidate dimensions that were considered and refused, and the specific
//! evidence each one would have needed.

// sprint:10's one small static page over the boundary-refinement specimens.
// Deleting the visualization is deleting this file.
// sprint:15's adversarial gauntlet, built against inverse-frequency weighting
// rather than against the permutation null. Commissioning evidence, not a search.
pub mod adversarial;
pub mod boundary_page;
// sprint:8's event-native motif experiment. No dependency and no feature gate:
// it is a few hundred lines of arithmetic over the projection everything else
// here reads.
// sprint:13's challenger to the sprint:11 statistic. Same permutation null,
// asked a conditional question and evaluated exactly. Touches nothing it tests.
pub mod conditional_null;
// sprint:16's exposure study: where two known failure surfaces lie relative to
// the recordings this project actually has. Measures; repairs nothing.
pub mod envelope;
pub mod event_sequence;
// sprint:12's adversarial gauntlet against sprint:11's statistic. Calls the
// machinery and changes none of it; deleting the attack is deleting this file.
pub mod gauntlet;
// sprint:14's representation audit. Formalizes what a scorer restricted to the
// mark-only representation can see, and a preregistered family of functions of
// it. Not a statistic and not a repair.
pub mod haar;
pub mod identifiability;
// sprint:6's Matrix Profile experiment. Behind a non-default feature because it
// is the one part of this experiment with a third-party dependency, and a
// default build of the product must not link a numerics stack for research code.
#[cfg(feature = "experiment-matrix-profile")]
pub mod matrix_profile;
pub mod oracle;
// sprint:17's candidate repairs and the semantic contract they are derived
// from. Proposes; adopts nothing, and changes no statistic anything else uses.
pub mod repair;
pub mod signal;
// sprint:7's Behavioral Spectroscope projection. Behind the same feature as the
// Matrix Profile it renders, since it cannot be assembled without one.
#[cfg(feature = "experiment-matrix-profile")]
pub mod spectroscope;
