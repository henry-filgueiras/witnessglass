//! **Disposable exploratory workflow.** sprint:21, task:31. Authorized by
//! decision:9 and by nothing else.
//!
//! A local, on-demand, deterministic corpus analyser. It reads a directory the
//! operator names, replays and validates each recording through
//! [`crate::replay_file`] and [`crate::inspection::inspect`], derives a second
//! projection beside the raw event stream, runs the existing cross-recording
//! search over the eligible corpus, assembles cross-session candidate families,
//! calibrates them against the exact first-order null, and hands back a
//! [`Facts`] document from which a report is rendered.
//!
//! # What this is not
//!
//! It is not a daemon, an index, a watcher, or a cache. It holds nothing between
//! invocations, writes nothing outside the output directory it is given, opens
//! no socket, and consults no model. It is not a product surface: nothing in the
//! crate outside this module, `examples/corpus-report.rs` and `tests/corpus.rs`
//! refers to it, and deleting those three files deletes the capability.
//!
//! # The two projections
//!
//! [`Projection::Raw`] is the established event projection —
//! [`super::event_sequence::project`], one mark per observed record, the mark
//! being a schema-tagged event kind plus the delivered tool name. sprint:8
//! through sprint:20 measured that projection and this module does not touch it.
//! Here it serves as the **instrument-grammar control**: whatever the recorder's
//! own request→outcome alternation explains is quarantined rather than reported.
//!
//! [`Projection::Workflow`] is new and derived. One [`Action`] per correlated
//! tool group, in canonical order of each group's earliest record, labelled with
//! a [`Category`] from a small versioned vocabulary. It exists because the raw
//! stream is the recorder's grammar rather than the agent's, and a human reading
//! it learns about the hook protocol.
//!
//! **A category is analyser shorthand for a delivered tool name.** It is not the
//! agent's intent, not a reported claim, and not an observed fact about what a
//! command did. `reported_intent` records contribute no action, and no reported
//! text is read anywhere in this module.
//!
//! # What may be claimed
//!
//! Prevalence — a shape appearing in N of M eligible sessions — is a
//! **description** and needs no null. A claim that a shape is unusual needs a
//! null measured on the projection the claim is about. sprint:20's collapse was
//! measured on the raw projection under the exact doublet null and may not be
//! transferred here by assertion, so [`calibrate`] runs the same null on
//! whichever projection is being reported, and the statistic it calibrates —
//! cross-session prevalence of an exact recurring shape — is **not** sprint:19's
//! `T`.
//!
//! # Privacy
//!
//! Output derived from a real recording is exactly as sensitive as that
//! recording. Nothing here redacts anything, and no output of this module may be
//! described as sanitized or safe to share. What it retains is what decision:8
//! permits: opaque session prefixes, counts, delivered tool names, derived
//! categories, numbers, and raw sequence numbers. Shell commands are classified
//! by their leading program name and **only the resulting category survives** —
//! never the command, a fragment of it, or a token of it.

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::event_sequence::{
    Alignment, ChannelScope, EventSequence, Mark, MarkedEvent, cross_pairs, dedupe_overlapping,
    project,
};
use super::identifiability::Observation;
use super::repair::candidate;
use super::transition_null::doublet_null_seeded;
use crate::inspection::{
    EventKind, GroupShape, Inspection, Sequence, ToolEvidence, V1Kind, V2Kind, inspect,
};
use crate::record::AnyRecord;
use crate::{Replay, replay_file};

// ---------------------------------------------------------------------------
// Identity of the analyser and of everything it decides
// ---------------------------------------------------------------------------

/// The analyser's own name, written into every document it produces.
pub const ANALYZER: &str = "witnessglass-corpus-report";

/// The analyser's version. Bumped when any decision below changes, so two
/// documents that disagree can be told apart without guessing.
pub const ANALYZER_VERSION: &str = "1";

/// Version of the category vocabulary in [`Category`] and [`classify_shell`].
pub const VOCABULARY_VERSION: u32 = 1;

/// Version of the [`Facts`] and [`Manifest`] serializations.
pub const DOCUMENT_VERSION: u32 = 1;

/// Span lengths searched, in events.
///
/// Short on purpose: a report lead is a pipeline a human reads in one breath,
/// and three to six actions is what that is. This is not sprint:9's ladder and
/// is not a claim about it.
pub const SPAN_LADDER: [usize; 4] = [3, 4, 5, 6];

/// Candidates retained per session pair per span length, after deduplication.
pub const KEEP_PER_PAIR: usize = 40;

/// Fewest actions a recording must contribute to be eligible.
pub const MIN_ACTIONS: usize = 12;

/// Fewest distinct categories a recording's action stream must carry.
pub const MIN_CATEGORIES: usize = 2;

/// Null replicates, unless the caller names another count.
pub const REPLICATES: usize = 999;

/// The tail below which a family is called unusual beyond local grammar.
pub const TAIL_THRESHOLD: f64 = 0.01;

/// The most eligible sessions one run can hold, bounded by the session bitset
/// the null pass counts coverage with.
pub const MAX_ELIGIBLE_SESSIONS: usize = 64;

/// Length of the opaque session prefix used as an identity everywhere.
pub const IDENTITY_PREFIX: usize = 8;

fn r1() -> &'static super::repair::Candidate {
    candidate("R1 pooled sum").expect("the ranked statistic must remain")
}

// ---------------------------------------------------------------------------
// The category vocabulary
// ---------------------------------------------------------------------------

/// Analyser shorthand for a delivered tool name.
///
/// **Not intent, not a reported claim, and not an observed fact about what a
/// command did.** Every variant is a deterministic function of a delivered tool
/// name and, for a shell call, of that command's leading program name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Category {
    /// A tool or leading program that reads without writing.
    Inspect,
    /// A tool that writes a file.
    Modify,
    /// A leading program that runs tests, type checks, or lints.
    Verify,
    /// A leading program that drives a version-control system.
    VersionControl,
    /// A tool that fetches from outside the machine.
    Research,
    /// A tool that starts a subagent.
    Delegate,
    /// A shell call whose leading program is in none of the rules above.
    /// Deliberately not guessed at.
    Shell,
    /// A delivered tool name in none of the rules above, or no delivered tool
    /// name at all.
    Other,
}

impl Category {
    /// Every category, in reporting order.
    pub const ALL: [Category; 8] = [
        Category::Inspect,
        Category::Modify,
        Category::Verify,
        Category::VersionControl,
        Category::Research,
        Category::Delegate,
        Category::Shell,
        Category::Other,
    ];

    /// The name used in every document and every rendered pipeline.
    pub fn as_str(self) -> &'static str {
        match self {
            Category::Inspect => "Inspect",
            Category::Modify => "Modify",
            Category::Verify => "Verify",
            Category::VersionControl => "VersionControl",
            Category::Research => "Research",
            Category::Delegate => "Delegate",
            Category::Shell => "Shell",
            Category::Other => "Other",
        }
    }

    /// The rule that assigns this category, in one line, for the report.
    pub fn rule(self) -> &'static str {
        match self {
            Category::Inspect => {
                "delivered tool Read, Grep, Glob, LS or NotebookRead; or a shell call whose \
                 leading program is a read-only utility"
            }
            Category::Modify => "delivered tool Edit, MultiEdit, Write or NotebookEdit",
            Category::Verify => {
                "a shell call whose leading program runs tests, type checks or lints"
            }
            Category::VersionControl => {
                "a shell call whose leading program is git, gh, jj, hg or svn"
            }
            Category::Research => "delivered tool WebSearch or WebFetch",
            Category::Delegate => "delivered tool Task or Agent",
            Category::Shell => "a shell call matching no rule above; not guessed at",
            Category::Other => "any other delivered tool name, or none",
        }
    }
}

/// Read-only shell programs. A command whose leading program is one of these is
/// [`Category::Inspect`] **even if a later stage of a pipeline does something
/// else**: only the leading program is read, and the report says so.
const INSPECT_PROGRAMS: [&str; 14] = [
    "cat", "ls", "head", "tail", "wc", "find", "which", "grep", "rg", "ag", "du", "stat", "diff",
    "tree",
];

/// Programs that are a test, type check, or lint on their own.
const VERIFY_PROGRAMS: [&str; 9] = [
    "pytest",
    "vitest",
    "jest",
    "tsc",
    "mypy",
    "ruff",
    "rspec",
    "phpunit",
    "gotestsum",
];

/// Programs that drive a version-control system.
const VCS_PROGRAMS: [&str; 5] = ["git", "gh", "jj", "hg", "svn"];

/// Subcommands that make `cargo` a verification call.
const CARGO_VERIFY: [&str; 5] = ["test", "clippy", "fmt", "check", "bench"];

/// Runner subcommands that make `npx` a verification call.
const NPX_VERIFY: [&str; 6] = ["tsc", "vitest", "jest", "eslint", "playwright", "mocha"];

/// Prefixes of a package script name that make it a verification call.
const SCRIPT_VERIFY_PREFIXES: [&str; 5] = ["test", "check", "lint", "typecheck", "verify"];

/// The last path segment of a token, lowercased.
fn program_name(token: &str) -> String {
    token
        .trim_matches(|c| c == '"' || c == '\'')
        .rsplit('/')
        .next()
        .unwrap_or(token)
        .to_ascii_lowercase()
}

/// Classify a shell command by its **leading program name only**.
///
/// The single concession to how commands are actually written: a command that
/// opens with `cd` is read from the token after its first `&&`. Nothing else is
/// parsed, no pipeline stage past the first is looked at, and **no part of the
/// command is returned or retained** — the return value is a category and
/// nothing else.
pub fn classify_shell(command: &str) -> Category {
    let tokens: Vec<&str> = command.split_whitespace().take(16).collect();
    let mut slice = tokens.as_slice();
    if slice
        .first()
        .is_some_and(|token| program_name(token) == "cd")
        && let Some(position) = slice.iter().position(|token| *token == "&&")
    {
        slice = &slice[position + 1..];
    }

    let Some(program) = slice.first().map(|token| program_name(token)) else {
        return Category::Shell;
    };
    let first = slice.get(1).map(|token| token.to_ascii_lowercase());
    let second = slice.get(2).map(|token| token.to_ascii_lowercase());
    let first = first.as_deref().unwrap_or_default();
    let second = second.as_deref().unwrap_or_default();

    if VERIFY_PROGRAMS.contains(&program.as_str()) {
        return Category::Verify;
    }
    let verifies = match program.as_str() {
        "cargo" => CARGO_VERIFY.contains(&first),
        "npm" | "pnpm" | "yarn" | "bun" => {
            first == "test"
                || (first == "run"
                    && SCRIPT_VERIFY_PREFIXES
                        .iter()
                        .any(|prefix| second.starts_with(prefix)))
        }
        "npx" => NPX_VERIFY.contains(&first),
        "node" | "deno" => first == "--test",
        "make" => SCRIPT_VERIFY_PREFIXES.iter().any(|p| first.starts_with(p)),
        _ => false,
    };
    if verifies {
        return Category::Verify;
    }
    if VCS_PROGRAMS.contains(&program.as_str()) {
        return Category::VersionControl;
    }
    if INSPECT_PROGRAMS.contains(&program.as_str()) {
        return Category::Inspect;
    }
    Category::Shell
}

/// Classify a delivered tool name, given the shell command when there is one.
///
/// `command` is consumed and discarded; only the returned [`Category`] survives.
pub fn categorize(tool_name: Option<&str>, command: Option<&str>) -> Category {
    match tool_name {
        Some("Read" | "Grep" | "Glob" | "LS" | "NotebookRead") => Category::Inspect,
        Some("Edit" | "MultiEdit" | "Write" | "NotebookEdit") => Category::Modify,
        Some("WebSearch" | "WebFetch") => Category::Research,
        Some("Task" | "Agent") => Category::Delegate,
        Some("Bash") => match command {
            Some(text) => classify_shell(text),
            None => Category::Shell,
        },
        _ => Category::Other,
    }
}

// ---------------------------------------------------------------------------
// The observed tool-action stream
// ---------------------------------------------------------------------------

/// What the recorder observed became of one correlated tool group.
///
/// Every variant is read off records that exist. Nothing is inferred from
/// adjacency, and no missing outcome is filled in with a plausible one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Outcome {
    /// A success record correlates to this group.
    Succeeded,
    /// A failure record correlates to this group: the call ran and went wrong.
    Failed,
    /// A denial record correlates to this group: the call did not run.
    Denied,
    /// An opening-side record with no outcome record in the examined scope.
    /// Not "still running".
    NoOutcomeObserved,
    /// Outcome records correlating to this group disagree about what became of
    /// the call. Preserved; nothing is chosen as canonical.
    Disagreeing,
}

impl Outcome {
    /// The name used in every document.
    pub fn as_str(self) -> &'static str {
        match self {
            Outcome::Succeeded => "succeeded",
            Outcome::Failed => "failed",
            Outcome::Denied => "denied",
            Outcome::NoOutcomeObserved => "no-outcome-observed",
            Outcome::Disagreeing => "disagreeing",
        }
    }

    /// How a pipeline renders it. Success is unmarked because it is the case a
    /// reader assumes; every other case is spelled out.
    pub fn suffix(self) -> &'static str {
        match self {
            Outcome::Succeeded => "",
            Outcome::Failed => "(failed)",
            Outcome::Denied => "(denied)",
            Outcome::NoOutcomeObserved => "(no outcome)",
            Outcome::Disagreeing => "(disagreeing)",
        }
    }
}

/// One derived action: one correlated tool group, placed in canonical order.
///
/// Carries no intent, no parentage, no causality, no concurrency and no
/// duration. It carries what the records say and the receipts that say it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Action {
    /// Position in this session's action stream.
    pub index: usize,
    /// The derived category. Analyser shorthand.
    pub category: Category,
    /// The delivered tool name, when every record in the group delivered the
    /// same one. `None` where none was delivered or where they disagree.
    pub tool_name: Option<String>,
    /// Whether records in the group delivered different tool names.
    pub tool_names_disagree: bool,
    /// The group's cardinality shape, exactly as [`crate::inspection`] classifies
    /// it.
    pub shape: GroupShape,
    /// The observed terminal outcome.
    pub outcome: Outcome,
    /// Lowest raw sequence number in the group.
    pub first_sequence: Sequence,
    /// Highest raw sequence number in the group.
    pub last_sequence: Sequence,
}

/// The schema-tagged event kind an action's mark carries.
///
/// Always the kind of a record that is actually in the group: the terminal
/// outcome's kind where there is one, the opening record's kind where there is
/// not. The mark's *identity* is this kind together with the derived category,
/// so a failed `Verify` and a successful one are different marks and a
/// failure-recovery shape survives into the search.
fn action_kind(schema_version: Option<u64>, outcome: Outcome) -> EventKind {
    if schema_version == Some(1) {
        return match outcome {
            Outcome::Succeeded | Outcome::Failed => EventKind::V1(V1Kind::ObservedToolFinished),
            _ => EventKind::V1(V1Kind::ObservedToolStarted),
        };
    }
    match outcome {
        Outcome::Succeeded => EventKind::V2(V2Kind::ToolSucceeded),
        Outcome::Failed => EventKind::V2(V2Kind::ToolFailed),
        Outcome::Denied => EventKind::V2(V2Kind::ToolDenied),
        Outcome::NoOutcomeObserved | Outcome::Disagreeing => EventKind::V2(V2Kind::ToolRequested),
    }
}

/// The mark label an action contributes, which is what the search compares.
///
/// Distinct from [`Mark::label`]: the workflow projection's marks carry a
/// derived category where the raw projection carries a delivered tool name, and
/// the two must never be mistaken for one another in a document.
fn action_mark_label(category: Category, outcome: Outcome) -> String {
    format!("{}{}", category.as_str(), outcome.suffix())
}

/// The shell command a tool group delivered, if any.
///
/// **The only place in this module that reads a payload.** The value is borrowed,
/// classified by the caller, and never stored.
fn group_command(record: &AnyRecord, sequence: Sequence) -> Option<&str> {
    let AnyRecord::V2(record) = record else {
        return None;
    };
    if record.sequence != sequence {
        return None;
    }
    let input = match &record.event {
        crate::Event::ToolRequested(event) => &event.requested_input,
        crate::Event::ToolSucceeded(event) => &event.effective_input,
        crate::Event::ToolFailed(event) => &event.effective_input,
        crate::Event::ToolDenied(event) => &event.requested_input,
        _ => return None,
    };
    input.get("command")?.as_str()
}

/// Read one tool group's observed outcome off its evidence.
fn group_outcome(evidence: &ToolEvidence) -> Outcome {
    if evidence.outcome_classes() > 1 {
        return Outcome::Disagreeing;
    }
    match evidence {
        ToolEvidence::V1 {
            finished_succeeded,
            finished_failed,
            ..
        } => {
            if !finished_succeeded.is_empty() {
                Outcome::Succeeded
            } else if !finished_failed.is_empty() {
                Outcome::Failed
            } else {
                Outcome::NoOutcomeObserved
            }
        }
        ToolEvidence::V2 {
            succeeded,
            failed,
            denied,
            ..
        } => {
            if !succeeded.is_empty() {
                Outcome::Succeeded
            } else if !failed.is_empty() {
                Outcome::Failed
            } else if !denied.is_empty() {
                Outcome::Denied
            } else {
                Outcome::NoOutcomeObserved
            }
        }
    }
}

/// Derive one session's observed tool-action stream.
///
/// One action per correlated tool group, in canonical order of each group's
/// earliest record. A [`GroupShape::ReportedIntentOnly`] group contributes **no**
/// action — it is a claim with no observation beside it, and promoting it here
/// would merge the two channels. Those groups are counted and returned.
pub fn action_stream(inspection: &Inspection<'_>) -> (Vec<Action>, usize) {
    let mut by_sequence: HashMap<Sequence, &AnyRecord> = HashMap::new();
    for record in inspection.records {
        by_sequence.insert(record.sequence(), record);
    }

    let mut actions = Vec::new();
    let mut reported_only = 0usize;
    for group in &inspection.tool_groups {
        if group.shape == GroupShape::ReportedIntentOnly {
            reported_only += 1;
            continue;
        }
        let outcome = group_outcome(&group.evidence);

        let names: Vec<&str> = group
            .delivered_tool_names
            .iter()
            .map(|delivered| delivered.value)
            .collect();
        let tool_names_disagree = names.len() > 1;
        let tool_name = if names.len() == 1 {
            Some(names[0])
        } else {
            None
        };

        // Receipts: the whole group, opening side and every outcome record.
        let mut receipts: Vec<Sequence> = group.evidence.opening().sequences().to_vec();
        receipts.extend(outcome_sequences(&group.evidence));
        receipts.sort_unstable();
        let first_sequence = receipts.first().copied().unwrap_or(group.first_sequence);
        let last_sequence = receipts.last().copied().unwrap_or(group.first_sequence);

        let command = receipts
            .iter()
            .find_map(|sequence| {
                by_sequence
                    .get(sequence)
                    .and_then(|record| group_command(record, *sequence))
            })
            .map(str::to_owned);
        let category = categorize(tool_name, command.as_deref());
        // The command string dies here. Nothing below this line can see it.
        drop(command);

        actions.push(Action {
            index: actions.len(),
            category,
            tool_name: tool_name.map(str::to_owned),
            tool_names_disagree,
            shape: group.shape,
            outcome,
            first_sequence,
            last_sequence,
        });
    }
    (actions, reported_only)
}

/// Every outcome record's sequence in a group, ascending.
fn outcome_sequences(evidence: &ToolEvidence) -> Vec<Sequence> {
    let mut all: Vec<Sequence> = match evidence {
        ToolEvidence::V1 {
            finished_succeeded,
            finished_failed,
            ..
        } => finished_succeeded
            .sequences()
            .iter()
            .chain(finished_failed.sequences())
            .copied()
            .collect(),
        ToolEvidence::V2 {
            succeeded,
            failed,
            denied,
            ..
        } => succeeded
            .sequences()
            .iter()
            .chain(failed.sequences())
            .chain(denied.sequences())
            .copied()
            .collect(),
    };
    all.sort_unstable();
    all
}

/// Project one session's action stream into the search's own sequence type.
///
/// The container is [`EventSequence`] and the machinery is unchanged; what
/// differs from [`project`] is the mark. Gaps are measured between consecutive
/// actions' earliest records, which is what the raw projection measures between
/// consecutive records — recorder timestamps, not execution durations.
pub fn workflow_sequence<'a>(
    inspection: &'a Inspection<'a>,
    actions: &[Action],
    labels: &'a [String],
) -> Option<EventSequence<'a>> {
    let extrema = inspection.timestamps.as_ref()?;
    let origin_ns = extrema.earliest.recorded_at.as_nanosecond();

    let mut at: HashMap<Sequence, jiff::Timestamp> = HashMap::new();
    for entry in &inspection.ledger {
        at.insert(entry.sequence, entry.recorded_at);
    }

    let mut events: Vec<MarkedEvent<'a>> = Vec::with_capacity(actions.len());
    let mut clamped_gaps = 0usize;
    let mut previous_ns: Option<i128> = None;
    for (action, label) in actions.iter().zip(labels) {
        let recorded_at = at.get(&action.first_sequence)?;
        let at_ns = recorded_at.as_nanosecond();
        let gap_from_previous_ms = previous_ns.map(|previous| {
            let delta = at_ns - previous;
            if delta < 0 {
                clamped_gaps += 1;
                0
            } else {
                u64::try_from(delta / 1_000_000).unwrap_or(0)
            }
        });
        previous_ns = Some(at_ns);
        events.push(MarkedEvent {
            sequence: Some(action.first_sequence),
            mark: Mark {
                kind: action_kind(inspection.schema_version, action.outcome),
                tool_name: Some(label.as_str()),
            },
            offset_ms: u64::try_from((at_ns - origin_ns) / 1_000_000).unwrap_or(0),
            gap_from_previous_ms,
        });
    }

    Some(EventSequence {
        channels: ChannelScope::Observed,
        events,
        origin: extrema.earliest.recorded_at,
        filtered_out: inspection.ledger.len().saturating_sub(actions.len()),
        clamped_gaps,
        non_monotonic: extrema.non_monotonic.clone(),
        scope: inspection.scope,
        session_id: inspection.session_id,
    })
}

// ---------------------------------------------------------------------------
// Which projection a number belongs to
// ---------------------------------------------------------------------------

/// Which derived lens a result was measured in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Projection {
    /// The established event projection: one mark per observed record, the mark
    /// being a schema-tagged event kind plus the delivered tool name.
    Raw,
    /// The workflow projection: one mark per correlated tool group, the mark
    /// being the group's terminal-outcome kind plus a derived category.
    Workflow,
}

impl Projection {
    /// The name used in every document.
    pub fn as_str(self) -> &'static str {
        match self {
            Projection::Raw => "raw-event",
            Projection::Workflow => "workflow-action",
        }
    }

    /// How a mark of this projection is rendered for a human.
    pub fn display(self, mark: &Mark<'_>) -> String {
        match self {
            // The raw mark is the recorder's own vocabulary and is shown as
            // such: nothing about it is translated.
            Projection::Raw => mark.label(),
            // The workflow mark's identity is entirely in its derived label; the
            // schema kind is redundant with the label's own outcome suffix.
            Projection::Workflow => mark.tool_name.unwrap_or("?").to_owned(),
        }
    }
}

// ---------------------------------------------------------------------------
// Inventory
// ---------------------------------------------------------------------------

/// Why a discovered file contributed nothing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SkipReason {
    /// The file could not be read or replayed as a recording.
    ReplayFailed,
    /// The recording holds no complete record, so no schema vocabulary and no
    /// session identity is established.
    NoCompleteRecords,
    /// The recording holds no timestamped record, so no sequence has an origin.
    NoTimestamps,
    /// The action stream is shorter than [`MIN_ACTIONS`].
    TooFewActions,
    /// The action stream carries fewer than [`MIN_CATEGORIES`] distinct
    /// categories.
    VocabularyTooSmall,
    /// The corpus already holds [`MAX_ELIGIBLE_SESSIONS`] eligible sessions.
    CorpusFull,
    /// Two discovered files carry the same session identity.
    DuplicateIdentity,
}

impl SkipReason {
    /// A sentence a human can read without the code beside them.
    pub fn explain(&self) -> &'static str {
        match self {
            SkipReason::ReplayFailed => "could not be read or replayed as a recording",
            SkipReason::NoCompleteRecords => "holds no complete record",
            SkipReason::NoTimestamps => "holds no timestamped record",
            SkipReason::TooFewActions => "too few observed actions to search",
            SkipReason::VocabularyTooSmall => "its actions carry too few distinct categories",
            SkipReason::CorpusFull => "the run's session limit was already reached",
            SkipReason::DuplicateIdentity => "another discovered file carries the same session id",
        }
    }
}

/// One discovered file, accounted for.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputRecord {
    /// Opaque session prefix, or the file stem's prefix where no session is
    /// established. Never a path and never a file name.
    pub identity: String,
    /// FNV-1a of the file's bytes, so two runs can be shown to have read the
    /// same input without the input appearing anywhere.
    pub fingerprint: String,
    /// File length in bytes.
    pub bytes: u64,
    /// Complete records replayed.
    pub records: usize,
    /// Whether the recording ends mid-record.
    pub truncated: bool,
    /// Actions the workflow projection derived, where it got that far.
    pub actions: Option<usize>,
    /// `None` when the file was included.
    pub skipped: Option<SkipReason>,
}

/// FNV-1a over bytes, rendered as lowercase hex. Not a cryptographic hash and
/// not used as one: it exists so two runs can be shown to have read identical
/// inputs.
pub fn fingerprint(bytes: &[u8]) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}")
}

/// The shortest prefix length at which every key in a corpus is distinguishable.
///
/// [`IDENTITY_PREFIX`] is enough for the UUID session ids a real recorder writes,
/// and lengthening it is what keeps two sessions from being reported as one when
/// it is not. The full key is used when no prefix separates them, because an
/// identity that does not identify is worse than a long one.
fn identity_width(keys: &[String]) -> usize {
    let longest = keys
        .iter()
        .map(|key| key.chars().count())
        .max()
        .unwrap_or(0);
    let mut width = IDENTITY_PREFIX;
    while width < longest {
        let mut prefixes: Vec<String> = keys
            .iter()
            .map(|key| key.chars().take(width).collect())
            .collect();
        prefixes.sort();
        let count = prefixes.len();
        prefixes.dedup();
        if prefixes.len() == count {
            return width;
        }
        width += 4;
    }
    longest.max(IDENTITY_PREFIX)
}

/// Every `*.ndjson` file in a directory, in file-name order.
///
/// Sorted rather than left in directory order, because directory order is not a
/// property of the corpus and a report that changes with it is not repeatable.
pub fn discover(directory: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut found: Vec<PathBuf> = std::fs::read_dir(directory)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .filter(|path| path.extension().is_some_and(|ext| ext == "ndjson"))
        .collect();
    found.sort();
    Ok(found)
}

/// One loaded recording, before eligibility is decided.
struct Loaded {
    /// The session id the recording establishes, or the file stem where it
    /// establishes none. Compared in full; **never** printed in full.
    key: String,
    identity: String,
    fingerprint: String,
    bytes: u64,
    replay: Option<Replay>,
}

fn load(path: &Path) -> Loaded {
    let stem = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("unknown");
    let raw = std::fs::read(path).ok();
    let bytes = raw.as_ref().map(|bytes| bytes.len() as u64).unwrap_or(0);
    let fingerprint = raw.as_deref().map(fingerprint).unwrap_or_default();
    let replay = replay_file(path).ok();
    let key = replay
        .as_ref()
        .and_then(|replay| replay.records.first())
        .map(|record| record.session_id())
        .unwrap_or(stem)
        .to_owned();
    Loaded {
        identity: String::new(),
        key,
        fingerprint,
        bytes,
        replay,
    }
}

// ---------------------------------------------------------------------------
// Search, retaining what the established search discards
// ---------------------------------------------------------------------------

/// One candidate the search kept, with everything needed to inspect it.
///
/// [`super::calibration::SearchOutcome`] keeps scores and discards coordinates,
/// because its question was about a maximum. This round's question is about a
/// shape, so the coordinates are the answer and are kept. **The search itself is
/// unchanged**: this calls `cross_pairs` and `dedupe_overlapping` exactly as
/// `complete_search` does, and `tests/corpus.rs` asserts the two agree on the
/// maximum R1.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RetainedCandidate {
    /// Opaque identity of the recording the `a` window came from.
    pub a_session: String,
    /// Opaque identity of the recording the `b` window came from.
    pub b_session: String,
    /// Span length in events.
    pub k: usize,
    /// Index of the `a` window's first event.
    pub a_start: usize,
    /// Index of the `b` window's first event.
    pub b_start: usize,
    /// Raw sequence range the `a` window covers.
    pub a_receipts: Option<(Sequence, Sequence)>,
    /// Raw sequence range the `b` window covers.
    pub b_receipts: Option<(Sequence, Sequence)>,
    /// The alignment decomposition, every component kept.
    pub alignment: Alignment,
    /// R1 at this candidate, the statistic the calibrated rounds read.
    pub r1: Option<f64>,
    /// The `a` window's canonical mark labels.
    pub a_marks: Vec<String>,
    /// The `b` window's canonical mark labels.
    pub b_marks: Vec<String>,
    /// Whether the two windows carry identical mark sequences. Only an exact
    /// candidate establishes cross-session support for a shape.
    pub exact: bool,
}

/// Canonical mark labels of a whole sequence, one per event.
fn canonical_labels(sequence: &EventSequence<'_>) -> Vec<String> {
    sequence
        .events
        .iter()
        .map(|event| event.mark.label())
        .collect()
}

/// Run the established search over one pair and retain what it kept.
pub fn retained_search(
    a_session: &str,
    b_session: &str,
    first: &EventSequence<'_>,
    second: &EventSequence<'_>,
    k: usize,
    keep: usize,
) -> Vec<RetainedCandidate> {
    let Some(ranked) = cross_pairs(first, second, k, usize::MAX) else {
        return Vec::new();
    };
    let kept = dedupe_overlapping(&ranked, keep);
    let a_labels = canonical_labels(first);
    let b_labels = canonical_labels(second);

    kept.iter()
        .map(|pair| {
            let (wa, wb) = (&pair.comparison.a, &pair.comparison.b);
            let r1 = Observation::of(
                first,
                (wa.start, wa.start + wa.k),
                second,
                (wb.start, wb.start + wb.k),
            )
            .and_then(|observation| (r1().score)(&observation));
            let a_marks = a_labels[wa.start..wa.start + wa.k].to_vec();
            let b_marks = b_labels[wb.start..wb.start + wb.k].to_vec();
            RetainedCandidate {
                a_session: a_session.to_owned(),
                b_session: b_session.to_owned(),
                k,
                a_start: wa.start,
                b_start: wb.start,
                a_receipts: wa.first_sequence.zip(wa.last_sequence),
                b_receipts: wb.first_sequence.zip(wb.last_sequence),
                alignment: pair.comparison.alignment,
                r1,
                exact: a_marks == b_marks,
                a_marks,
                b_marks,
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Families
// ---------------------------------------------------------------------------

/// Where one occurrence of a family sits in one session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Occurrence {
    /// Index of the occurrence's first event in that session's sequence.
    pub start: usize,
    /// Raw sequence number of the occurrence's first record.
    pub first_sequence: Option<Sequence>,
    /// Raw sequence number of the occurrence's last record.
    pub last_sequence: Option<Sequence>,
}

/// One session's contribution to a family.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionSupport {
    /// Opaque session identity.
    pub identity: String,
    /// Non-overlapping occurrences in this session.
    pub occurrences: usize,
    /// Up to three of them, with receipts, so the operator can go and look.
    pub receipts: Vec<Occurrence>,
}

/// Why a family is not a finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Quarantine {
    /// Two distinct marks or fewer: a shape that alternates or repeats says as
    /// little as a pair of lone impulses does.
    LowDiversity,
    /// One mark occupies two thirds of the window or more.
    MarkDominance,
    /// Raw projection only: the window alternates opening-side and outcome-side
    /// records, which is the recorder's protocol rather than the agent's
    /// behaviour.
    RequestOutcomeAlternation,
    /// Raw projection only: every mark in the window carries the same delivered
    /// tool name.
    SingleTool,
}

impl Quarantine {
    /// A sentence a human can read without the code beside them.
    pub fn explain(self) -> &'static str {
        match self {
            Quarantine::LowDiversity => "it is built from two kinds of step or fewer",
            Quarantine::MarkDominance => "one kind of step fills two thirds of it or more",
            Quarantine::RequestOutcomeAlternation => {
                "it is the recorder's request-then-outcome protocol, not a behaviour"
            }
            Quarantine::SingleTool => "every step in it is the same tool",
        }
    }
}

/// How much weight a family carries, in words rather than numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Strength {
    /// Survives the first-order-aware calibration on its own projection.
    PromisingBeyondLocalGrammar,
    /// Appears in at least half the eligible sessions, and is ordinary under
    /// that calibration.
    DescriptivelyCommon,
    /// Recurs across sessions, but in a minority of them.
    WeakLead,
    /// Quarantined as instrument grammar.
    ProbablyRecorderGrammar,
}

impl Strength {
    /// The label printed in the report.
    pub fn as_str(self) -> &'static str {
        match self {
            Strength::PromisingBeyondLocalGrammar => "promising beyond local grammar",
            Strength::DescriptivelyCommon => "descriptively common",
            Strength::WeakLead => "weak lead",
            Strength::ProbablyRecorderGrammar => "probably recorder grammar",
        }
    }
}

/// A calibrated tail for one family, on the projection it was found in.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FamilyCalibration {
    /// Null replicates run.
    pub replicates: usize,
    /// How many replicates produced a corpus whose **best** shape of this length
    /// reached this family's session count or better.
    pub exceedances: usize,
    /// `(1 + exceedances) / (replicates + 1)`. A Monte Carlo tail under the
    /// exact first-order null and this discovery statistic, and nothing else.
    pub tail: f64,
    /// Whether `tail <= TAIL_THRESHOLD`.
    pub exceptional: bool,
}

/// One cross-session recurring shape.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Family {
    /// Stable identifier: projection, span length, and rank within them.
    pub id: String,
    /// A name derived from the shape, for a human to refer to it by.
    pub name: String,
    /// The projection this family lives in.
    pub projection: Projection,
    /// Span length in events.
    pub k: usize,
    /// The canonical mark labels the search matched on.
    pub marks: Vec<String>,
    /// The same shape rendered for a human.
    pub pipeline: Vec<String>,
    /// The most common underlying delivered tool-name sequence across this
    /// family's occurrences, and how many of them carry it.
    pub tool_sequence: Option<Vec<String>>,
    /// Occurrences carrying [`Family::tool_sequence`].
    pub tool_sequence_share: usize,
    /// Eligible sessions holding at least one occurrence.
    pub sessions: usize,
    /// Eligible sessions in the corpus. The denominator, stated.
    pub eligible: usize,
    /// Total non-overlapping occurrences across the corpus.
    pub occurrences: usize,
    /// Per-session support, in corpus order.
    pub support: Vec<SessionSupport>,
    /// Distinct marks inside the shape.
    pub distinct_marks: usize,
    /// Quarantine flags, empty for a family that is not instrument grammar.
    pub quarantine: Vec<Quarantine>,
    /// The id of a longer shape that contains this one contiguously and holds
    /// exactly the same sessions. Such a shape is the same finding with less
    /// information, so it is flagged rather than reported — and flagged rather
    /// than deleted, so two corpora's facts documents stay comparable.
    pub subsumed_by: Option<String>,
    /// The id of the shape this one is a variant of: built from the same set of
    /// steps, in a different order or at a different length, and supported by no
    /// more sessions. `None` for a shape that is its own representative. Set
    /// after ranking, so the representative is always the best-supported member.
    pub variant_of: Option<String>,
    /// Other shapes over the same set of steps, folded into this one. Non-zero
    /// only on a representative.
    pub variants: usize,
    /// The most sessions any folded variant reached.
    pub variants_max_sessions: usize,
    /// The longest folded variant's pipeline, so the representative does not
    /// hide the longer shapes it stands for.
    pub longest_variant: Option<Vec<String>>,
    /// Sessions holding [`Family::longest_variant`].
    pub longest_variant_sessions: usize,
    /// Session pairs whose search produced this family as an exact candidate.
    pub discovered_by_pairs: usize,
    /// Best R1 over the exact candidates that discovered it.
    pub best_r1: Option<f64>,
    /// The calibrated tail, on this family's own projection.
    pub calibration: Option<FamilyCalibration>,
    /// The plain-language strength label.
    pub strength: Strength,
}

/// The name a shape is referred to by.
///
/// A run-length encoding of its own steps, so `Modify → Shell → Shell → Inspect`
/// is `Modify–Shell×2–Inspect`. Run-length encoding is a bijection on sequences,
/// so two different shapes never collide on a name and nothing has to be
/// disambiguated after the fact. A shape that returns to the step it opened with
/// is called a loop, which is the one thing a reader wants flagged.
fn family_name(pipeline: &[String]) -> String {
    let mut runs: Vec<(&str, usize)> = Vec::new();
    for step in pipeline {
        match runs.last_mut() {
            Some((name, count)) if *name == step.as_str() => *count += 1,
            _ => runs.push((step.as_str(), 1)),
        }
    }
    let joined = runs
        .iter()
        .map(|(name, count)| {
            if *count == 1 {
                (*name).to_owned()
            } else {
                format!("{name}×{count}")
            }
        })
        .collect::<Vec<_>>()
        .join("–");
    let loops = runs.len() > 2 && runs.first().map(|run| run.0) == runs.last().map(|run| run.0);
    if loops {
        format!("{joined} loop")
    } else {
        joined
    }
}

/// Greedy non-overlapping exact occurrences of `shape` in `labels`.
fn occurrences_of(labels: &[String], shape: &[String]) -> Vec<usize> {
    let k = shape.len();
    let mut found = Vec::new();
    let mut index = 0usize;
    while index + k <= labels.len() {
        if &labels[index..index + k] == shape {
            found.push(index);
            index += k;
        } else {
            index += 1;
        }
    }
    found
}

/// Whether `short` appears contiguously inside `long`.
fn is_contiguous_subsequence(short: &[String], long: &[String]) -> bool {
    if short.len() > long.len() {
        return false;
    }
    (0..=long.len() - short.len()).any(|start| &long[start..start + short.len()] == short)
}

// ---------------------------------------------------------------------------
// The null
// ---------------------------------------------------------------------------

/// Deterministic seed for one session in one replicate.
///
/// A round that reproduces must draw the same nulls, and a session must not draw
/// the same null as its neighbour. Mixed rather than concatenated so adjacent
/// replicates do not produce correlated streams in the LCG the null uses.
pub fn corpus_null_seed(replicate: usize, session: usize) -> u64 {
    let mut state = 0x9E37_79B9_7F4A_7C15u64
        ^ (replicate as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9)
        ^ (session as u64).wrapping_mul(0x94D0_49BB_1331_11EB);
    state ^= state >> 30;
    state = state.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    state ^= state >> 27;
    state = state.wrapping_mul(0x94D0_49BB_1331_11EB);
    state ^ (state >> 31)
}

/// The null distribution of the corpus's best shape, at one span length.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NullDistribution {
    /// Span length.
    pub k: usize,
    /// Replicates run.
    pub replicates: usize,
    /// How many eligible sessions the best shape reached, per replicate,
    /// ascending.
    pub best_sessions: Vec<usize>,
    /// Median of that.
    pub median: usize,
    /// Its 0.95 quantile.
    pub q95: usize,
    /// Its 0.99 quantile.
    pub q99: usize,
    /// The largest value any replicate produced.
    pub max: usize,
}

impl NullDistribution {
    /// Replicates whose best shape reached `sessions` or better.
    pub fn exceedances(&self, sessions: usize) -> usize {
        self.best_sessions
            .iter()
            .filter(|value| **value >= sessions)
            .count()
    }

    /// The Monte Carlo tail for an observed session count.
    pub fn tail(&self, sessions: usize) -> f64 {
        (1.0 + self.exceedances(sessions) as f64) / (self.replicates as f64 + 1.0)
    }
}

/// How often a session's exact first-order null returns the session itself.
///
/// sprint:20 §D5 measured this and it decides how much a tail means: where the
/// null is concentrated on the observation, the null distribution contains the
/// thing it is meant to be a foil for.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Degeneracy {
    /// Opaque session identity.
    pub identity: String,
    /// Replicates identical to the observed mark sequence.
    pub identical: usize,
    /// Replicates run.
    pub replicates: usize,
    /// `identical / replicates`.
    pub fraction: f64,
}

/// Index a sequence's marks against a shared vocabulary, growing it as needed.
fn index_marks<'a>(sequence: &EventSequence<'a>, vocabulary: &mut Vec<Mark<'a>>) -> Vec<u16> {
    sequence
        .events
        .iter()
        .map(|event| {
            let position = vocabulary.iter().position(|mark| *mark == event.mark);
            match position {
                Some(found) => found as u16,
                None => {
                    vocabulary.push(event.mark);
                    (vocabulary.len() - 1) as u16
                }
            }
        })
        .collect()
}

/// The greatest number of distinct sessions any one shape of length `k` covers.
fn best_coverage(indexed: &[Vec<u16>], k: usize) -> usize {
    let mut seen: HashMap<Vec<u16>, u64> = HashMap::new();
    for (session, marks) in indexed.iter().enumerate() {
        if marks.len() < k {
            continue;
        }
        let bit = 1u64 << session;
        for start in 0..=marks.len() - k {
            *seen.entry(marks[start..start + k].to_vec()).or_insert(0) |= bit;
        }
    }
    seen.values()
        .map(|mask| mask.count_ones() as usize)
        .max()
        .unwrap_or(0)
}

/// Run the exact first-order null over the whole corpus.
///
/// Every eligible session is replaced by a `doublet_null_seeded` replicate of
/// itself, which preserves that session's first-order transition counts, mark
/// multiset, length and both endpoints **exactly** and destroys longer-range
/// reuse. The same discovery statistic — the best shape's session coverage — is
/// then recomputed on the null corpus, at every span length, inside every
/// replicate.
pub fn calibrate(
    sequences: &[EventSequence<'_>],
    ladder: &[usize],
    replicates: usize,
) -> (Vec<NullDistribution>, Vec<usize>) {
    let mut per_k: Vec<Vec<usize>> = vec![Vec::with_capacity(replicates); ladder.len()];
    let mut identical: Vec<usize> = vec![0; sequences.len()];

    let mut observed_vocabulary: Vec<Mark<'_>> = Vec::new();
    let observed: Vec<Vec<u16>> = sequences
        .iter()
        .map(|sequence| index_marks(sequence, &mut observed_vocabulary))
        .collect();

    for replicate in 0..replicates {
        let nulls: Vec<EventSequence<'_>> = sequences
            .iter()
            .enumerate()
            .map(|(session, sequence)| {
                doublet_null_seeded(sequence, corpus_null_seed(replicate, session))
            })
            .collect();

        let mut vocabulary: Vec<Mark<'_>> = observed_vocabulary.clone();
        let indexed: Vec<Vec<u16>> = nulls
            .iter()
            .map(|sequence| index_marks(sequence, &mut vocabulary))
            .collect();
        for (session, marks) in indexed.iter().enumerate() {
            if *marks == observed[session] {
                identical[session] += 1;
            }
        }
        for (slot, k) in ladder.iter().enumerate() {
            per_k[slot].push(best_coverage(&indexed, *k));
        }
    }

    let distributions = ladder
        .iter()
        .zip(per_k)
        .map(|(k, mut values)| {
            values.sort_unstable();
            let at = |q: f64| -> usize {
                if values.is_empty() {
                    return 0;
                }
                let rank = (q * (values.len() as f64 - 1.0)).round() as usize;
                values[rank.min(values.len() - 1)]
            };
            NullDistribution {
                k: *k,
                replicates,
                median: at(0.50),
                q95: at(0.95),
                q99: at(0.99),
                max: values.last().copied().unwrap_or(0),
                best_sessions: values,
            }
        })
        .collect();

    (distributions, identical)
}

// ---------------------------------------------------------------------------
// The documents
// ---------------------------------------------------------------------------

/// Who produced a document, and under which fixed decisions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnalyzerIdentity {
    /// [`ANALYZER`].
    pub name: String,
    /// [`ANALYZER_VERSION`].
    pub version: String,
    /// [`VOCABULARY_VERSION`].
    pub vocabulary_version: u32,
    /// The crate version the analyser was built from.
    pub witnessglass_version: String,
}

impl Default for AnalyzerIdentity {
    fn default() -> Self {
        Self {
            name: ANALYZER.to_owned(),
            version: ANALYZER_VERSION.to_owned(),
            vocabulary_version: VOCABULARY_VERSION,
            witnessglass_version: env!("CARGO_PKG_VERSION").to_owned(),
        }
    }
}

/// Every knob that decides what the analysis does.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Configuration {
    /// Span lengths searched.
    pub span_ladder: Vec<usize>,
    /// Candidates retained per session pair per span length.
    pub keep_per_pair: usize,
    /// Null replicates.
    pub replicates: usize,
    /// The exceptional-tail threshold.
    pub tail_threshold: f64,
    /// Fewest actions a recording must contribute.
    pub min_actions: usize,
    /// Fewest distinct categories a recording's actions must carry.
    pub min_categories: usize,
    /// The null construction, by name.
    pub null: String,
    /// The ranked statistic, by name.
    pub statistic: String,
}

/// One session that made it into the analysis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionFacts {
    /// Opaque session identity.
    pub identity: String,
    /// Complete records replayed.
    pub records: usize,
    /// Whether the recording ends mid-record.
    pub truncated: bool,
    /// Events the raw projection retained.
    pub raw_events: usize,
    /// Actions the workflow projection derived.
    pub actions: usize,
    /// Correlation ids cited only by reported intent, which contribute no
    /// action.
    pub reported_intent_only_groups: usize,
    /// `reported_intent` records, counted and never read.
    pub reported_intent_records: usize,
    /// Actions per category, in [`Category::ALL`] order.
    pub by_category: Vec<(Category, usize)>,
    /// Actions per observed outcome.
    pub by_outcome: Vec<(Outcome, usize)>,
    /// Actions per delivered tool name, most common first.
    pub by_tool: Vec<(String, usize)>,
    /// Actions whose group shape is not a paired lifecycle.
    pub unpaired_actions: usize,
}

/// What one projection's search and calibration found.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectionFacts {
    /// Which lens.
    pub projection: Projection,
    /// Session pairs searched.
    pub pairs: usize,
    /// Candidates the search retained across every pair and span length.
    pub candidates: usize,
    /// Of those, ones whose two windows carry identical mark sequences.
    pub exact_candidates: usize,
    /// Families assembled from the exact candidates.
    pub families: Vec<Family>,
    /// The null distributions, one per span length.
    pub null: Vec<NullDistribution>,
    /// Per-session null degeneracy.
    pub degeneracy: Vec<Degeneracy>,
    /// Mean events per sequence in this projection.
    pub mean_length: f64,
}

/// The machine-readable analysis result. `report.md` is rendered from this and
/// from nothing else.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Facts {
    /// Document kind, so a file can identify itself.
    pub schema: String,
    /// [`DOCUMENT_VERSION`].
    pub schema_version: u32,
    /// Who produced it.
    pub analyzer: AnalyzerIdentity,
    /// The operator's label for this corpus.
    pub corpus_label: String,
    /// Files discovered in the directory.
    pub discovered: usize,
    /// Files that replayed into a usable recording.
    pub included: usize,
    /// Files accounted for but not analysed.
    pub skipped: usize,
    /// The prevalence denominator.
    pub eligible_sessions: usize,
    /// Whether any included recording ends mid-record.
    pub truncated_included: usize,
    /// The eligibility rules, in words, so the report can state them without
    /// the manifest beside it.
    pub eligibility: Vec<String>,
    /// Every discovered file that contributed nothing, by opaque identity and
    /// reason. Present so that nothing can disappear between the manifest and
    /// the report a human actually reads.
    pub set_aside: Vec<(String, SkipReason)>,
    /// The knobs.
    pub configuration: Configuration,
    /// Per-session facts, in corpus order.
    pub sessions: Vec<SessionFacts>,
    /// Corpus-wide actions per category.
    pub by_category: Vec<(Category, usize)>,
    /// Corpus-wide actions per delivered tool name, most common first.
    pub by_tool: Vec<(String, usize)>,
    /// Corpus-wide actions per observed outcome.
    pub by_outcome: Vec<(Outcome, usize)>,
    /// `reported_intent` records across the corpus. Counted, never read.
    pub reported_intent_records: usize,
    /// Correlation ids cited only by reported intent, which contribute no
    /// action.
    pub reported_intent_only_groups: usize,
    /// The workflow projection's results.
    pub workflow: ProjectionFacts,
    /// The raw projection's results, run as the instrument-grammar control.
    pub raw: ProjectionFacts,
    /// Ranked ids into [`ProjectionFacts::families`] of the workflow projection.
    pub leads: Vec<String>,
    /// Ids of quarantined families, ranked.
    pub background: Vec<String>,
    /// Ids of families whose calibrated tail cleared the threshold.
    pub exceptional: Vec<String>,
}

/// The reproducibility record. Deterministic: it holds no clock and no path.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Manifest {
    /// Document kind.
    pub schema: String,
    /// [`DOCUMENT_VERSION`].
    pub schema_version: u32,
    /// Who produced it.
    pub analyzer: AnalyzerIdentity,
    /// The operator's label for this corpus.
    pub corpus_label: String,
    /// The knobs.
    pub configuration: Configuration,
    /// The eligibility rules, in words.
    pub eligibility: Vec<String>,
    /// Every discovered file, in file-name order, included or skipped.
    pub inputs: Vec<InputRecord>,
    /// Files discovered.
    pub discovered: usize,
    /// Files included.
    pub included: usize,
    /// Files skipped.
    pub skipped: usize,
    /// The prevalence denominator.
    pub eligible_sessions: usize,
}

// ---------------------------------------------------------------------------
// The analysis
// ---------------------------------------------------------------------------

/// What the caller asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    /// Directory to discover recordings in.
    pub directory: PathBuf,
    /// The operator's label for this corpus.
    pub label: String,
    /// Null replicates.
    pub replicates: usize,
}

/// Everything one run produced.
#[derive(Debug, Clone, PartialEq)]
pub struct Analysis {
    /// The machine-readable result.
    pub facts: Facts,
    /// The reproducibility record.
    pub manifest: Manifest,
}

fn configuration(replicates: usize) -> Configuration {
    Configuration {
        span_ladder: SPAN_LADDER.to_vec(),
        keep_per_pair: KEEP_PER_PAIR,
        replicates,
        tail_threshold: TAIL_THRESHOLD,
        min_actions: MIN_ACTIONS,
        min_categories: MIN_CATEGORIES,
        null: "doublet — exact first-order transition-preserving shuffle (sprint:20)".to_owned(),
        statistic: "R1 pooled sum (sprint:17), read at the search's own winners".to_owned(),
    }
}

fn eligibility_rules() -> Vec<String> {
    vec![
        "the file's name ends in .ndjson".to_owned(),
        "the file replays as a recording".to_owned(),
        "the recording holds at least one complete record and one timestamp".to_owned(),
        format!("its observed tool-action stream holds at least {MIN_ACTIONS} actions"),
        format!("those actions carry at least {MIN_CATEGORIES} distinct categories"),
        "a truncated recording's valid prefix is included, and reported as truncated".to_owned(),
        "no two included recordings share a session identity".to_owned(),
    ]
}

/// Run the whole analysis.
///
/// Reads the directory, and nothing else. Writes nothing: serialization and
/// rendering are the caller's, so this function is a pure function of the files
/// it read.
pub fn analyze(request: &Request) -> std::io::Result<Analysis> {
    let paths = discover(&request.directory)?;
    let mut loaded: Vec<Loaded> = paths.iter().map(|path| load(path)).collect();
    // Opaque identities, wide enough that no two discovered recordings collide.
    let keys: Vec<String> = loaded.iter().map(|item| item.key.clone()).collect();
    let width = identity_width(&keys);
    for item in &mut loaded {
        item.identity = item.key.chars().take(width).collect();
    }

    // Replays first, so every inspection can borrow from a stable place. Moved
    // out rather than cloned: a corpus is tens of megabytes of parsed payload
    // and holding two copies of it buys nothing.
    let replays: Vec<Option<Replay>> = loaded.iter_mut().map(|item| item.replay.take()).collect();
    let inspections: Vec<Option<Inspection<'_>>> = replays
        .iter()
        .map(|replay| replay.as_ref().map(inspect))
        .collect();

    let streams: Vec<Option<(Vec<Action>, usize)>> = inspections
        .iter()
        .map(|inspection| inspection.as_ref().map(action_stream))
        .collect();

    // Eligibility, and an account of everything that failed it.
    let mut inputs: Vec<InputRecord> = Vec::with_capacity(loaded.len());
    let mut eligible: Vec<usize> = Vec::new();
    let mut seen_keys: Vec<String> = Vec::new();
    for (index, item) in loaded.iter().enumerate() {
        let replay = replays[index].as_ref();
        let records = replay.map(|replay| replay.records.len()).unwrap_or(0);
        let truncated = replay
            .map(|replay| replay.tail.is_truncated())
            .unwrap_or(false);
        let actions = streams[index].as_ref().map(|(actions, _)| actions.len());

        let skipped = if replay.is_none() {
            Some(SkipReason::ReplayFailed)
        } else if records == 0
            || inspections[index]
                .as_ref()
                .and_then(|i| i.session_id)
                .is_none()
        {
            Some(SkipReason::NoCompleteRecords)
        } else if inspections[index]
            .as_ref()
            .is_none_or(|i| i.timestamps.is_none())
        {
            Some(SkipReason::NoTimestamps)
        } else if seen_keys.contains(&item.key) {
            Some(SkipReason::DuplicateIdentity)
        } else if actions.unwrap_or(0) < MIN_ACTIONS {
            Some(SkipReason::TooFewActions)
        } else if distinct_categories(&streams[index]) < MIN_CATEGORIES {
            Some(SkipReason::VocabularyTooSmall)
        } else if eligible.len() >= MAX_ELIGIBLE_SESSIONS {
            Some(SkipReason::CorpusFull)
        } else {
            None
        };

        if skipped.is_none() {
            eligible.push(index);
            seen_keys.push(item.key.clone());
        }
        inputs.push(InputRecord {
            identity: item.identity.clone(),
            fingerprint: item.fingerprint.clone(),
            bytes: item.bytes,
            records,
            truncated,
            actions,
            skipped,
        });
    }

    // Labels have to outlive the sequences that borrow them.
    let labels: Vec<Vec<String>> = eligible
        .iter()
        .map(|index| {
            streams[*index]
                .as_ref()
                .map(|(actions, _)| {
                    actions
                        .iter()
                        .map(|action| action_mark_label(action.category, action.outcome))
                        .collect()
                })
                .unwrap_or_default()
        })
        .collect();

    let mut workflow_sequences: Vec<EventSequence<'_>> = Vec::with_capacity(eligible.len());
    let mut raw_sequences: Vec<EventSequence<'_>> = Vec::with_capacity(eligible.len());
    for (slot, index) in eligible.iter().enumerate() {
        let inspection = inspections[*index].as_ref().expect("eligible has a replay");
        let (actions, _) = streams[*index].as_ref().expect("eligible has a stream");
        let workflow = workflow_sequence(inspection, actions, &labels[slot])
            .expect("eligible has a timestamp origin");
        let raw =
            project(inspection, ChannelScope::Observed).expect("eligible has a timestamp origin");
        workflow_sequences.push(workflow);
        raw_sequences.push(raw);
    }

    let identities: Vec<String> = eligible
        .iter()
        .map(|index| loaded[*index].identity.clone())
        .collect();
    let action_slices: Vec<&[Action]> = eligible
        .iter()
        .map(|index| streams[*index].as_ref().expect("eligible").0.as_slice())
        .collect();

    let workflow = analyse_projection(
        Projection::Workflow,
        &identities,
        &workflow_sequences,
        Some(&action_slices),
        request.replicates,
    );
    let raw = analyse_projection(
        Projection::Raw,
        &identities,
        &raw_sequences,
        None,
        request.replicates,
    );

    let sessions: Vec<SessionFacts> = eligible
        .iter()
        .enumerate()
        .map(|(slot, index)| {
            let inspection = inspections[*index].as_ref().expect("eligible");
            let (actions, reported_only) = streams[*index].as_ref().expect("eligible");
            session_facts(
                &identities[slot],
                inspection,
                actions,
                *reported_only,
                raw_sequences[slot].len(),
            )
        })
        .collect();

    let by_category = Category::ALL
        .iter()
        .map(|category| {
            let count = sessions
                .iter()
                .flat_map(|session| &session.by_category)
                .filter(|(other, _)| other == category)
                .map(|(_, count)| *count)
                .sum();
            (*category, count)
        })
        .collect();
    let mut tools: BTreeMap<String, usize> = BTreeMap::new();
    for session in &sessions {
        for (name, count) in &session.by_tool {
            *tools.entry(name.clone()).or_insert(0) += count;
        }
    }
    let by_tool = rank_tools(tools);
    let reported_intent_records = sessions
        .iter()
        .map(|session| session.reported_intent_records)
        .sum();
    let reported_intent_only_groups = sessions
        .iter()
        .map(|session| session.reported_intent_only_groups)
        .sum();
    let by_outcome = [
        Outcome::Succeeded,
        Outcome::Failed,
        Outcome::Denied,
        Outcome::NoOutcomeObserved,
        Outcome::Disagreeing,
    ]
    .iter()
    .map(|outcome| {
        let count = sessions
            .iter()
            .flat_map(|session| &session.by_outcome)
            .filter(|(other, _)| other == outcome)
            .map(|(_, count)| *count)
            .sum();
        (*outcome, count)
    })
    .collect();

    let leads: Vec<String> = workflow
        .families
        .iter()
        .filter(|family| {
            family.quarantine.is_empty()
                && family.variant_of.is_none()
                && family.subsumed_by.is_none()
        })
        .map(|family| family.id.clone())
        .collect();
    let background: Vec<String> = workflow
        .families
        .iter()
        .chain(&raw.families)
        .filter(|family| !family.quarantine.is_empty())
        .map(|family| family.id.clone())
        .collect();
    let exceptional: Vec<String> = workflow
        .families
        .iter()
        .chain(&raw.families)
        .filter(|family| {
            family
                .calibration
                .as_ref()
                .is_some_and(|calibration| calibration.exceptional)
        })
        .map(|family| family.id.clone())
        .collect();

    let facts = Facts {
        schema: "witnessglass.corpus-facts".to_owned(),
        schema_version: DOCUMENT_VERSION,
        analyzer: AnalyzerIdentity::default(),
        corpus_label: request.label.clone(),
        discovered: inputs.len(),
        included: eligible.len(),
        skipped: inputs.len() - eligible.len(),
        eligible_sessions: eligible.len(),
        truncated_included: eligible
            .iter()
            .filter(|index| inputs[**index].truncated)
            .count(),
        eligibility: eligibility_rules(),
        set_aside: inputs
            .iter()
            .filter_map(|input| {
                input
                    .skipped
                    .clone()
                    .map(|reason| (input.identity.clone(), reason))
            })
            .collect(),
        configuration: configuration(request.replicates),
        sessions,
        by_category,
        by_tool,
        by_outcome,
        reported_intent_records,
        reported_intent_only_groups,
        workflow,
        raw,
        leads,
        background,
        exceptional,
    };

    let manifest = Manifest {
        schema: "witnessglass.corpus-manifest".to_owned(),
        schema_version: DOCUMENT_VERSION,
        analyzer: AnalyzerIdentity::default(),
        corpus_label: request.label.clone(),
        configuration: configuration(request.replicates),
        eligibility: eligibility_rules(),
        discovered: inputs.len(),
        included: eligible.len(),
        skipped: inputs.len() - eligible.len(),
        eligible_sessions: eligible.len(),
        inputs,
    };

    Ok(Analysis { facts, manifest })
}

fn distinct_categories(stream: &Option<(Vec<Action>, usize)>) -> usize {
    let Some((actions, _)) = stream else {
        return 0;
    };
    let mut seen: Vec<Category> = Vec::new();
    for action in actions {
        if !seen.contains(&action.category) {
            seen.push(action.category);
        }
    }
    seen.len()
}

fn rank_tools(tools: BTreeMap<String, usize>) -> Vec<(String, usize)> {
    let mut ranked: Vec<(String, usize)> = tools.into_iter().collect();
    ranked.sort_by(|left, right| right.1.cmp(&left.1).then(left.0.cmp(&right.0)));
    ranked
}

fn session_facts(
    identity: &str,
    inspection: &Inspection<'_>,
    actions: &[Action],
    reported_only: usize,
    raw_events: usize,
) -> SessionFacts {
    let by_category = Category::ALL
        .iter()
        .map(|category| {
            (
                *category,
                actions
                    .iter()
                    .filter(|action| action.category == *category)
                    .count(),
            )
        })
        .collect();
    let outcomes = [
        Outcome::Succeeded,
        Outcome::Failed,
        Outcome::Denied,
        Outcome::NoOutcomeObserved,
        Outcome::Disagreeing,
    ];
    let by_outcome = outcomes
        .iter()
        .map(|outcome| {
            (
                *outcome,
                actions
                    .iter()
                    .filter(|action| action.outcome == *outcome)
                    .count(),
            )
        })
        .collect();
    let mut tools: BTreeMap<String, usize> = BTreeMap::new();
    for action in actions {
        if let Some(name) = &action.tool_name {
            *tools.entry(name.clone()).or_insert(0) += 1;
        }
    }
    let reported_intent_records = inspection
        .ledger
        .iter()
        .filter(|entry| entry.kind == EventKind::V2(V2Kind::ReportedIntent))
        .count();

    SessionFacts {
        identity: identity.to_owned(),
        records: inspection.record_count(),
        truncated: inspection.scope.is_truncated(),
        raw_events,
        actions: actions.len(),
        reported_intent_only_groups: reported_only,
        reported_intent_records,
        by_category,
        by_outcome,
        by_tool: rank_tools(tools),
        unpaired_actions: actions
            .iter()
            .filter(|action| action.shape != GroupShape::PairedLifecycle)
            .count(),
    }
}

/// Search, group, count, calibrate and rank one projection.
fn analyse_projection(
    projection: Projection,
    identities: &[String],
    sequences: &[EventSequence<'_>],
    actions: Option<&[&[Action]]>,
    replicates: usize,
) -> ProjectionFacts {
    let ladder: Vec<usize> = SPAN_LADDER.to_vec();
    let labels: Vec<Vec<String>> = sequences.iter().map(canonical_labels).collect();

    let mut candidates: Vec<RetainedCandidate> = Vec::new();
    let mut pairs = 0usize;
    for a in 0..sequences.len() {
        for b in (a + 1)..sequences.len() {
            pairs += 1;
            for k in &ladder {
                candidates.extend(retained_search(
                    &identities[a],
                    &identities[b],
                    &sequences[a],
                    &sequences[b],
                    *k,
                    KEEP_PER_PAIR,
                ));
            }
        }
    }
    let exact_candidates = candidates.iter().filter(|c| c.exact).count();

    // A family is an exact mark sequence. Only an exact candidate — one whose
    // two windows carry identical marks — establishes cross-session support, so
    // there is no near-miss to chain through and `A≈B`, `B≈C` can never imply
    // `A≈C`.
    let mut seeds: BTreeMap<Vec<String>, (usize, Option<f64>)> = BTreeMap::new();
    for found in candidates.iter().filter(|c| c.exact) {
        let entry = seeds.entry(found.a_marks.clone()).or_insert((0, None));
        entry.0 += 1;
        entry.1 = match (entry.1, found.r1) {
            (Some(best), Some(new)) => Some(best.max(new)),
            (best, new) => best.or(new),
        };
    }

    let (null, identical) = if replicates > 0 {
        calibrate(sequences, &ladder, replicates)
    } else {
        (Vec::new(), vec![0; sequences.len()])
    };

    let mut families: Vec<Family> = seeds
        .iter()
        .map(|(marks, (discovered_by_pairs, best_r1))| {
            build_family(
                projection,
                marks,
                *discovered_by_pairs,
                *best_r1,
                identities,
                sequences,
                &labels,
                actions,
                &null,
            )
        })
        .collect();

    families.sort_by(|left, right| {
        right
            .sessions
            .cmp(&left.sessions)
            .then(right.occurrences.cmp(&left.occurrences))
            .then(right.k.cmp(&left.k))
            .then(left.marks.cmp(&right.marks))
    });
    for (rank, family) in families.iter_mut().enumerate() {
        family.id = format!("{}-{:03}", projection.as_str(), rank + 1);
    }
    // Flag — never delete — a shape that is a contiguous fragment of a longer
    // shape holding exactly the same sessions. Reporting both is reporting one
    // finding twice, and the longer one carries more information. It stays in
    // the document because a facts file that drops shapes depending on what else
    // the corpus happened to contain is not a document two corpora can be
    // compared through.
    let subsumers: Vec<Option<String>> = families
        .iter()
        .map(|family| {
            families
                .iter()
                .find(|longer| {
                    longer.id != family.id
                        && longer.k > family.k
                        && longer.sessions == family.sessions
                        && session_set(longer) == session_set(family)
                        && is_contiguous_subsequence(&family.marks, &longer.marks)
                })
                .map(|longer| longer.id.clone())
        })
        .collect();
    let mut kept = families;
    for (family, subsumer) in kept.iter_mut().zip(subsumers) {
        family.subsumed_by = subsumer;
    }

    // Fold shapes built from the same set of steps. Six permutations of
    // `Inspect`, `Modify` and `Shell` — plus the four-step and five-step shapes
    // over the same three — are one observation about which steps co-occur, not
    // ten leads. The best-supported member represents the set, says how many
    // siblings it stands for, and carries the longest of them so the longer
    // pipelines are not hidden. That many near-equal variants exist is itself
    // the finding: at these lengths, *which* steps co-occur says more than the
    // order they occur in.
    let keys: Vec<Vec<String>> = kept
        .iter()
        .map(|family| {
            let mut distinct: Vec<String> = family.marks.clone();
            distinct.sort();
            distinct.dedup();
            distinct
        })
        .collect();
    // A subsumed shape never represents a fold: it is already standing behind a
    // longer one.
    let eligible_representative: Vec<bool> = kept
        .iter()
        .map(|family| family.subsumed_by.is_none() && family.quarantine.is_empty())
        .collect();
    struct Fold {
        representative: String,
        variants: usize,
        max_sessions: usize,
        longest: Option<(usize, usize, Vec<String>)>,
    }
    let mut folds: BTreeMap<Vec<String>, Fold> = BTreeMap::new();
    for (index, key) in keys.iter().enumerate() {
        if !eligible_representative[index] {
            continue;
        }
        let fold = folds.entry(key.clone()).or_insert_with(|| Fold {
            representative: kept[index].id.clone(),
            variants: 0,
            max_sessions: 0,
            longest: None,
        });
        if fold.representative == kept[index].id {
            continue;
        }
        fold.variants += 1;
        fold.max_sessions = fold.max_sessions.max(kept[index].sessions);
        let candidate = (
            kept[index].k,
            kept[index].sessions,
            kept[index].pipeline.clone(),
        );
        if fold
            .longest
            .as_ref()
            .is_none_or(|best| (candidate.0, candidate.1) > (best.0, best.1))
        {
            fold.longest = Some(candidate);
        }
    }
    for (index, key) in keys.iter().enumerate() {
        let Some(fold) = folds.get(key) else {
            continue;
        };
        if fold.representative == kept[index].id {
            kept[index].variants = fold.variants;
            kept[index].variants_max_sessions = fold.max_sessions;
            if let Some((_, sessions, pipeline)) = &fold.longest
                && pipeline.len() > kept[index].k
            {
                kept[index].longest_variant = Some(pipeline.clone());
                kept[index].longest_variant_sessions = *sessions;
            }
        } else {
            kept[index].variant_of = Some(fold.representative.clone());
        }
    }

    let degeneracy = identities
        .iter()
        .zip(identical)
        .map(|(identity, count)| Degeneracy {
            identity: identity.clone(),
            identical: count,
            replicates,
            fraction: if replicates == 0 {
                0.0
            } else {
                count as f64 / replicates as f64
            },
        })
        .collect();

    let mean_length = if sequences.is_empty() {
        0.0
    } else {
        sequences.iter().map(EventSequence::len).sum::<usize>() as f64 / sequences.len() as f64
    };

    ProjectionFacts {
        projection,
        pairs,
        candidates: candidates.len(),
        exact_candidates,
        families: kept,
        null,
        degeneracy,
        mean_length,
    }
}

fn session_set(family: &Family) -> Vec<&str> {
    family
        .support
        .iter()
        .filter(|support| support.occurrences > 0)
        .map(|support| support.identity.as_str())
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn build_family(
    projection: Projection,
    marks: &[String],
    discovered_by_pairs: usize,
    best_r1: Option<f64>,
    identities: &[String],
    sequences: &[EventSequence<'_>],
    labels: &[Vec<String>],
    actions: Option<&[&[Action]]>,
    null: &[NullDistribution],
) -> Family {
    let k = marks.len();

    let mut support: Vec<SessionSupport> = Vec::new();
    let mut occurrences = 0usize;
    let mut tool_sequences: BTreeMap<Vec<String>, usize> = BTreeMap::new();
    for (slot, identity) in identities.iter().enumerate() {
        let starts = occurrences_of(&labels[slot], marks);
        occurrences += starts.len();
        let receipts = starts
            .iter()
            .take(3)
            .map(|start| Occurrence {
                start: *start,
                first_sequence: sequences[slot].events[*start].sequence,
                last_sequence: sequences[slot].events[*start + k - 1].sequence,
            })
            .collect();
        if let Some(actions) = actions {
            for start in &starts {
                let names: Vec<String> = actions[slot][*start..*start + k]
                    .iter()
                    .map(|action| action.tool_name.clone().unwrap_or_else(|| "—".to_owned()))
                    .collect();
                *tool_sequences.entry(names).or_insert(0) += 1;
            }
        }
        support.push(SessionSupport {
            identity: identity.clone(),
            occurrences: starts.len(),
            receipts,
        });
    }
    let sessions = support.iter().filter(|entry| entry.occurrences > 0).count();

    let mut ranked_tools: Vec<(Vec<String>, usize)> = tool_sequences.into_iter().collect();
    ranked_tools.sort_by(|left, right| right.1.cmp(&left.1).then(left.0.cmp(&right.0)));
    let (tool_sequence, tool_sequence_share) = match ranked_tools.first() {
        Some((names, count)) => (Some(names.clone()), *count),
        None => (None, 0),
    };

    let pipeline: Vec<String> = marks
        .iter()
        .map(|label| display_from_canonical(projection, label))
        .collect();
    let mut distinct: Vec<&String> = Vec::new();
    for mark in marks {
        if !distinct.contains(&mark) {
            distinct.push(mark);
        }
    }
    let top = distinct
        .iter()
        .map(|mark| marks.iter().filter(|other| other == mark).count())
        .max()
        .unwrap_or(0);

    let mut quarantine = Vec::new();
    if distinct.len() <= 2 {
        quarantine.push(Quarantine::LowDiversity);
    }
    if top * 3 >= k * 2 {
        quarantine.push(Quarantine::MarkDominance);
    }
    if projection == Projection::Raw {
        if alternates_request_outcome(marks) {
            quarantine.push(Quarantine::RequestOutcomeAlternation);
        }
        if single_tool(marks) {
            quarantine.push(Quarantine::SingleTool);
        }
    }

    let calibration = null
        .iter()
        .find(|distribution| distribution.k == k)
        .map(|distribution| {
            let tail = distribution.tail(sessions);
            FamilyCalibration {
                replicates: distribution.replicates,
                exceedances: distribution.exceedances(sessions),
                tail,
                exceptional: tail <= TAIL_THRESHOLD,
            }
        });

    let strength = if !quarantine.is_empty() {
        Strength::ProbablyRecorderGrammar
    } else if calibration
        .as_ref()
        .is_some_and(|calibration| calibration.exceptional)
    {
        Strength::PromisingBeyondLocalGrammar
    } else if sessions * 2 >= identities.len() {
        Strength::DescriptivelyCommon
    } else {
        Strength::WeakLead
    };

    Family {
        id: String::new(),
        subsumed_by: None,
        variant_of: None,
        variants: 0,
        variants_max_sessions: 0,
        longest_variant: None,
        longest_variant_sessions: 0,
        name: family_name(&pipeline),
        projection,
        k,
        marks: marks.to_vec(),
        pipeline,
        tool_sequence,
        tool_sequence_share,
        sessions,
        eligible: identities.len(),
        occurrences,
        support,
        distinct_marks: distinct.len(),
        quarantine,
        discovered_by_pairs,
        best_r1,
        calibration,
        strength,
    }
}

/// Render a canonical mark label for a human.
///
/// A canonical label is [`Mark::label`], which prefixes the schema-tagged event
/// kind. The raw projection's kind *is* its vocabulary and is kept; the workflow
/// projection's kind is redundant with the outcome suffix its category already
/// carries, so only the category survives into a pipeline.
fn display_from_canonical(projection: Projection, label: &str) -> String {
    match projection {
        Projection::Raw => label
            .split_once(':')
            .map(|(_, rest)| rest.replace('/', " "))
            .unwrap_or_else(|| label.to_owned()),
        Projection::Workflow => label
            .split_once('/')
            .map(|(_, category)| category.to_owned())
            .unwrap_or_else(|| label.to_owned()),
    }
}

/// Whether a raw window alternates opening-side and outcome-side records.
fn alternates_request_outcome(marks: &[String]) -> bool {
    let opening =
        |label: &str| label.contains("tool_requested") || label.contains("observed_tool_started");
    let outcome = |label: &str| {
        label.contains("tool_succeeded")
            || label.contains("tool_failed")
            || label.contains("tool_denied")
            || label.contains("observed_tool_finished")
    };
    marks.windows(2).all(|pair| {
        (opening(&pair[0]) && outcome(&pair[1])) || (outcome(&pair[0]) && opening(&pair[1]))
    })
}

/// Whether every raw mark in a window carries the same delivered tool name.
fn single_tool(marks: &[String]) -> bool {
    let name = |label: &str| label.split_once('/').map(|(_, name)| name.to_owned());
    let first = name(&marks[0]);
    first.is_some() && marks.iter().all(|label| name(label) == first)
}

// ---------------------------------------------------------------------------
// The report
// ---------------------------------------------------------------------------

/// How many leads the report ranks. Fewer are printed when fewer deserve it.
pub const MAX_LEADS: usize = 5;

/// How many quarantined shapes the background section prints.
pub const MAX_BACKGROUND: usize = 6;

fn percent(part: usize, whole: usize) -> String {
    if whole == 0 {
        return "0%".to_owned();
    }
    format!("{}%", (part as f64 * 100.0 / whole as f64).round() as i64)
}

fn pipeline_line(steps: &[String]) -> String {
    steps.join(" → ")
}

fn family_by_id<'a>(facts: &'a Facts, id: &str) -> Option<&'a Family> {
    facts
        .workflow
        .families
        .iter()
        .chain(&facts.raw.families)
        .find(|family| family.id == id)
}

fn support_line(family: &Family) -> String {
    let mut shown: Vec<String> = family
        .support
        .iter()
        .filter(|support| support.occurrences > 0)
        .take(3)
        .map(|support| {
            let receipts: Vec<String> = support
                .receipts
                .iter()
                .filter_map(|occurrence| {
                    occurrence
                        .first_sequence
                        .zip(occurrence.last_sequence)
                        .map(|(first, last)| format!("{first}–{last}"))
                })
                .collect();
            if receipts.is_empty() {
                format!("`{}`", support.identity)
            } else {
                format!("`{}` at sequence {}", support.identity, receipts.join(", "))
            }
        })
        .collect();
    let remaining = family
        .support
        .iter()
        .filter(|support| support.occurrences > 0)
        .count()
        .saturating_sub(shown.len());
    if remaining > 0 {
        shown.push(format!("and {remaining} more"));
    }
    shown.join("; ")
}

/// Why a lead may be worth a look, derived from its own shape and numbers.
fn why_investigate(family: &Family) -> String {
    let mut reasons: Vec<String> = Vec::new();
    if family.sessions * 2 >= family.eligible {
        reasons.push(format!(
            "it turns up in most of the corpus ({} of {} sessions)",
            family.sessions, family.eligible
        ));
    } else {
        reasons.push(format!(
            "it turns up in a minority of the corpus ({} of {} sessions), so it may belong to one \
             kind of work rather than to the agent's habits",
            family.sessions, family.eligible
        ));
    }
    if family.occurrences >= family.sessions * 3 {
        reasons.push(format!(
            "and it repeats within sessions — {} non-overlapping occurrences across {} of them",
            family.occurrences, family.sessions
        ));
    }
    if family.pipeline.iter().any(|step| step.contains("(failed)")) {
        reasons.push(
            "it contains a step the recorder saw fail, so it may be a recovery shape".to_owned(),
        );
    }
    if family
        .pipeline
        .iter()
        .any(|step| step.starts_with("Verify"))
    {
        reasons.push(
            "it contains a verification step, which is where a workflow either closes or loops"
                .to_owned(),
        );
    }
    reasons.join("; ")
}

/// The most important alternative explanation, derived from the family's facts.
fn confound(family: &Family, facts: &Facts) -> String {
    let dominant = facts
        .by_category
        .iter()
        .max_by_key(|(_, count)| *count)
        .map(|(category, count)| {
            (
                *category,
                *count,
                facts.by_category.iter().map(|(_, c)| *c).sum::<usize>(),
            )
        });
    if let Some((category, count, total)) = dominant
        && total > 0
        && family
            .pipeline
            .iter()
            .filter(|step| step.starts_with(category.as_str()))
            .count()
            * 2
            > family.k
        && count * 3 >= total
    {
        return format!(
            "`{}` is {} of every action in this corpus, so a shape mostly made of it is close to \
             what any ordering of these actions would produce",
            category.as_str(),
            percent(count, total)
        );
    }
    match &family.calibration {
        Some(calibration) if !calibration.exceptional => format!(
            "the detector considers it ordinary: {} of {} first-order null corpora produced a shape \
             of this length reaching the same session coverage or better",
            calibration.exceedances, calibration.replicates
        ),
        Some(_) => {
            "the shape survives the calibration, but the corpus is small and these sessions \
                    come from one project, so a coincidence across a handful of sessions is not \
                    ruled out"
                .to_owned()
        }
        None => "no calibration was run for this span length".to_owned(),
    }
}

/// Render `report.md` from a facts document, and from nothing else.
pub fn render_report(facts: &Facts) -> String {
    let mut out = String::new();
    let workflow = &facts.workflow;
    let leads: Vec<&Family> = facts
        .leads
        .iter()
        .filter_map(|id| family_by_id(facts, id))
        .take(MAX_LEADS)
        .collect();
    let exceptional: Vec<&Family> = facts
        .exceptional
        .iter()
        .filter_map(|id| family_by_id(facts, id))
        .collect();

    out.push_str(&format!(
        "# Corpus field report — `{}`\n\n",
        facts.corpus_label
    ));
    out.push_str(
        "*Derived, disposable, and local. Rebuildable from the recordings; safe to delete. \
         **Not** redacted, not sanitized, and not safe to share — it is exactly as sensitive as the \
         recordings behind it.*\n\n",
    );

    // -- 30 seconds ---------------------------------------------------------
    out.push_str("## The 30-second version\n\n");
    let total_actions: usize = facts.sessions.iter().map(|session| session.actions).sum();
    out.push_str(&format!(
        "- {} recording files were found. {} were analysed; {} were set aside and are listed in the \
         manifest with a reason.\n",
        facts.discovered, facts.included, facts.skipped
    ));
    out.push_str(&format!(
        "- Those {} sessions contain {} observed tool actions. Every percentage below is out of {} \
         sessions unless it says otherwise.\n",
        facts.eligible_sessions, total_actions, facts.eligible_sessions
    ));
    if leads.is_empty() {
        out.push_str(
            "- **No recurring workflow shape survived the report's own filters.** Everything the \
             search found was either confined to a single session or was recorder grammar.\n",
        );
    } else {
        out.push_str(&format!(
            "- The strongest recurring shape is **{}** — `{}` — in {} of {} sessions ({}).\n",
            leads[0].name,
            pipeline_line(&leads[0].pipeline),
            leads[0].sessions,
            leads[0].eligible,
            percent(leads[0].sessions, leads[0].eligible)
        ));
    }
    if exceptional.is_empty() {
        out.push_str(
            "- **Nothing here is statistically unusual.** Once each session's immediate \
             step-to-step grammar is held fixed, every shape the search found is the kind of thing \
             that grammar produces on its own. Read the leads below as *descriptions of what this \
             agent does*, not as discoveries.\n",
        );
    } else {
        out.push_str(&format!(
            "- **{} shape(s) survived the calibration** — they recur across more sessions than \
             each session's own step-to-step grammar accounts for.\n",
            exceptional.len()
        ));
    }
    out.push('\n');

    // -- Corpus at a glance --------------------------------------------------
    out.push_str("## Corpus at a glance\n\n");
    out.push_str(
        "| session | records | truncated | observed events | actions | most common tool |\n\
         |---|---:|---|---:|---:|---|\n",
    );
    for session in &facts.sessions {
        let top = session
            .by_tool
            .first()
            .map(|(name, count)| format!("{name} ({count})"))
            .unwrap_or_else(|| "—".to_owned());
        out.push_str(&format!(
            "| `{}` | {} | {} | {} | {} | {} |\n",
            session.identity,
            session.records,
            if session.truncated { "yes" } else { "no" },
            session.raw_events,
            session.actions,
            top
        ));
    }
    out.push('\n');
    if facts.truncated_included == 0 {
        out.push_str(
            "None of the analysed recordings ends mid-record. A truncated one would have been \
             included on its valid prefix and marked in the table above.\n\n",
        );
    } else {
        out.push_str(&format!(
            "{} of the analysed recordings end mid-record. Their valid prefix was used, and they \
             are marked in the table above; nothing is claimed about what the missing tail would \
             have said.\n\n",
            facts.truncated_included
        ));
    }

    out.push_str("**What was set aside, and why.** ");
    if facts.set_aside.is_empty() {
        out.push_str("Nothing. Every file discovered was analysed.\n\n");
    } else {
        out.push_str(&format!(
            "{} of the {} files discovered contributed nothing. None of them disappeared \
             silently:\n\n",
            facts.set_aside.len(),
            facts.discovered
        ));
        for (identity, reason) in &facts.set_aside {
            out.push_str(&format!("- `{}` — {}.\n", identity, reason.explain()));
        }
        out.push('\n');
    }
    out.push_str("A recording had to clear all of these to be counted:\n\n");
    for rule in &facts.eligibility {
        out.push_str(&format!("- {rule}\n"));
    }
    out.push_str(&format!(
        "\n**{} sessions cleared them, and {} is the denominator of every prevalence figure in \
         this report.**\n\n",
        facts.eligible_sessions, facts.eligible_sessions
    ));

    let total_actions_here: usize = facts.by_outcome.iter().map(|(_, count)| *count).sum();
    let outcome_line: Vec<String> = facts
        .by_outcome
        .iter()
        .filter(|(_, count)| *count > 0)
        .map(|(outcome, count)| format!("{count} {}", outcome.as_str()))
        .collect();
    out.push_str(&format!(
        "**What became of those actions.** Of {} actions: {}. A failed or denied step is kept as a \
         distinct kind of step, so a shape containing one is a different shape from the same steps \
         all succeeding.\n\n",
        total_actions_here,
        outcome_line.join(", ")
    ));

    out.push_str(&format!(
        "**The two channels stayed apart.** The corpus holds {} `reported_intent` records — what \
         the agent said about its own work. **None of them was read.** {} correlation id(s) were \
         cited only by reported intent and contributed no action at all, because a claim with no \
         observation beside it is not an observation.\n\n",
        facts.reported_intent_records, facts.reported_intent_only_groups
    ));

    let total_categorized: usize = facts.by_category.iter().map(|(_, count)| *count).sum();
    out.push_str(
        "**What the actions are, by category.** A category is this analyser's shorthand \
                  for a delivered tool name. It is *not* the agent's intent and *not* an observed \
                  fact about what a command did.\n\n",
    );
    out.push_str("| category | actions | share | assigned by |\n|---|---:|---:|---|\n");
    for (category, count) in &facts.by_category {
        if *count == 0 {
            continue;
        }
        out.push_str(&format!(
            "| `{}` | {} | {} | {} |\n",
            category.as_str(),
            count,
            percent(*count, total_categorized),
            category.rule()
        ));
    }
    let unused: Vec<&str> = facts
        .by_category
        .iter()
        .filter(|(_, count)| *count == 0)
        .map(|(category, _)| category.as_str())
        .collect();
    if !unused.is_empty() {
        out.push_str(&format!(
            "\nCategories in the vocabulary that nothing in this corpus reached: {}.\n",
            unused
                .iter()
                .map(|name| format!("`{name}`"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    out.push_str(
        "\nOne consequence worth holding onto: **a single tool can land in several categories.** \
         `Bash` becomes `Verify`, `VersionControl`, `Inspect` or `Shell` depending on the leading \
         program of its command, so a pipeline step reading `Inspect` may well be a `Bash` call in \
         the tool sequence beside it. That is the categorisation working, not a mismatch.\n\n",
    );
    out.push_str("| delivered tool | actions |\n|---|---:|\n");
    for (name, count) in facts.by_tool.iter().take(12) {
        out.push_str(&format!("| `{name}` | {count} |\n"));
    }
    out.push('\n');

    // -- Leads ---------------------------------------------------------------
    out.push_str("## Top investigation leads\n\n");
    if leads.is_empty() {
        out.push_str(
            "None. Every shape the search found across sessions was quarantined as recorder \
             grammar, or appeared in only one session. That is a real result about this corpus and \
             not a failure of the run.\n\n",
        );
    } else {
        out.push_str(&format!(
            "Ranked by how many sessions hold them. {} shown of {}.\n\nShapes built from the same \
             *kinds* of step are folded into one lead — six orderings of `Inspect`, `Modify` and \
             `Shell`, plus the longer shapes over those same three, are one observation about what \
             co-occurs, not ten. Each lead says how many variants it stands for and shows the \
             longest of them, and a large variant count is itself informative: it means the order \
             of these steps is carrying little.\n\n",
            leads.len(),
            facts.leads.len()
        ));
        for (rank, family) in leads.iter().enumerate() {
            out.push_str(&format!("### {}. {}\n\n", rank + 1, family.name));
            out.push_str(&format!("`{}`\n\n", pipeline_line(&family.pipeline)));
            if let Some(tools) = &family.tool_sequence {
                out.push_str(&format!(
                    "Most often that is the tool sequence `{}` ({} of {} occurrences).\n\n",
                    tools.join(" → "),
                    family.tool_sequence_share,
                    family.occurrences
                ));
            }
            out.push_str(&format!(
                "- **{} of {} sessions ({})**, {} non-overlapping occurrences in total.\n",
                family.sessions,
                family.eligible,
                percent(family.sessions, family.eligible),
                family.occurrences
            ));
            if family.variants > 0 {
                let longest = match &family.longest_variant {
                    Some(longest) => format!(
                        "; the longest is `{}`, in {} of {} sessions",
                        pipeline_line(longest),
                        family.longest_variant_sessions,
                        family.eligible
                    ),
                    None => String::new(),
                };
                out.push_str(&format!(
                    "- Stands for {} other shape(s) over the same kinds of step, the best in {} of \
                     {} sessions{}.\n",
                    family.variants, family.variants_max_sessions, family.eligible, longest
                ));
            }
            out.push_str(&format!("- Strength: **{}**.\n", family.strength.as_str()));
            out.push_str(&format!(
                "- Why it may be worth a look: {}.\n",
                why_investigate(family)
            ));
            out.push_str(&format!(
                "- The main alternative explanation: {}.\n",
                confound(family, facts)
            ));
            out.push_str(&format!("- Go and look: {}.\n\n", support_line(family)));
        }
    }

    // -- Background ----------------------------------------------------------
    out.push_str("## Common background grammar we should not mistake for motifs\n\n");
    out.push_str(
        "These recur too, and some of them recur in every session. They are quarantined because \
         they describe the recorder or a single dominant tool rather than a workflow. Nothing here \
         is a finding.\n\n",
    );
    let background: Vec<&Family> = facts
        .background
        .iter()
        .filter_map(|id| family_by_id(facts, id))
        .take(MAX_BACKGROUND)
        .collect();
    if background.is_empty() {
        out.push_str("Nothing was quarantined in this corpus.\n\n");
    } else {
        for family in background {
            let reasons: Vec<&str> = family
                .quarantine
                .iter()
                .map(|flag| flag.explain())
                .collect();
            out.push_str(&format!(
                "- **{}** (`{}` projection, {} of {} sessions): `{}` — set aside because {}.\n",
                family.name,
                family.projection.as_str(),
                family.sessions,
                family.eligible,
                pipeline_line(&family.pipeline),
                reasons.join(", and ")
            ));
        }
        out.push('\n');
    }
    out.push_str(&format!(
        "The raw event projection — one mark per record, the recorder's own vocabulary — is run \
         alongside as a control. It found {} exact cross-session shapes over {} session pairs; {} \
         of them were quarantined. That is what the instrument's own grammar looks like.\n\n",
        facts.raw.families.len(),
        facts.raw.pairs,
        facts
            .raw
            .families
            .iter()
            .filter(|family| !family.quarantine.is_empty())
            .count()
    ));

    // -- What the calibration did not find ------------------------------------
    out.push_str("## What the calibrated matcher did not find\n\n");
    out.push_str(
        "Each session was reshuffled so that its own immediate step-to-step transitions were kept \
         **exactly** — same steps, same counts, same first and last step, same pairwise \
         transitions — while any longer-range repetition was destroyed. The whole search was then \
         re-run on those reshuffled corpora, and the best shape it found there was recorded. A real \
         shape has to beat that.\n\n",
    );
    for distribution in &workflow.null {
        let observed = workflow
            .families
            .iter()
            .filter(|family| family.k == distribution.k)
            .map(|family| family.sessions)
            .max()
            .unwrap_or(0);
        out.push_str(&format!(
            "- At {} steps: the best real shape reaches **{} of {} sessions**; the reshuffled \
             corpora reached a median of {} and a maximum of {} over {} attempts.\n",
            distribution.k,
            observed,
            facts.eligible_sessions,
            distribution.median,
            distribution.max,
            distribution.replicates
        ));
    }
    let saturated: Vec<usize> = workflow
        .null
        .iter()
        .filter(|distribution| distribution.median >= facts.eligible_sessions)
        .map(|distribution| distribution.k)
        .collect();
    if !saturated.is_empty() {
        out.push_str(&format!(
            "\n**At {} step(s) the test has no room to work.** The reshuffled corpora already put \
             *some* shape in all {} sessions, more than half the time. A measure that is already at \
             its ceiling cannot separate anything, so a real shape at that length would have to be \
             one the reshuffling essentially never produces. Read those lengths as descriptive only.\n",
            saturated
                .iter()
                .map(|k| k.to_string())
                .collect::<Vec<_>>()
                .join(" and "),
            facts.eligible_sessions
        ));
    }
    out.push('\n');
    if exceptional.is_empty() {
        out.push_str(
            "**No shape cleared that bar.** Nothing in this corpus recurs across sessions more \
             than each session's own local grammar already explains. That is not evidence that \
             agents have no habits; it is evidence that *this* corpus, at *these* lengths, cannot \
             tell a habit apart from the local grammar it is made of.\n\n",
        );
    } else {
        for family in &exceptional {
            out.push_str(&format!(
                "- **{}** (`{}`) cleared the bar at {} of {} sessions.\n",
                family.name,
                family.projection.as_str(),
                family.sessions,
                family.eligible
            ));
        }
        out.push('\n');
    }

    // -- Other observations ----------------------------------------------------
    out.push_str("## Other things the corpus says\n\n");
    out.push_str(
        "Derived mechanically from the same counts, and descriptive: none of this is calibrated \
         against anything.\n\n",
    );

    let corpus_total: usize = facts.by_category.iter().map(|(_, count)| *count).sum();
    out.push_str(
        "**Session shapes differ from one another.** For each session, the category furthest from \
         its corpus-wide share:\n\n| session | actions | stands out on | in this session | across \
         the corpus |\n|---|---:|---|---:|---:|\n",
    );
    for session in &facts.sessions {
        let total: usize = session.by_category.iter().map(|(_, count)| *count).sum();
        if total == 0 {
            continue;
        }
        let standout = session
            .by_category
            .iter()
            .map(|(category, count)| {
                let mine = *count as f64 / total as f64;
                let theirs = facts
                    .by_category
                    .iter()
                    .find(|(other, _)| other == category)
                    .map(|(_, count)| *count as f64 / corpus_total.max(1) as f64)
                    .unwrap_or(0.0);
                (*category, mine, theirs, (mine - theirs).abs())
            })
            .max_by(|left, right| {
                left.3
                    .partial_cmp(&right.3)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then(right.0.cmp(&left.0))
            });
        // Below eight points a "standout" is arithmetic noise, and printing one
        // would invent a session archetype out of rounding.
        match standout {
            Some((category, mine, theirs, delta)) if delta >= 0.08 => out.push_str(&format!(
                "| `{}` | {} | `{}` | {:.0}% | {:.0}% |\n",
                session.identity,
                session.actions,
                category.as_str(),
                mine * 100.0,
                theirs * 100.0
            )),
            _ => out.push_str(&format!(
                "| `{}` | {} | — nothing by more than 8 points — | | |\n",
                session.identity, session.actions
            )),
        }
    }
    out.push_str(
        "\nA session whose share of one category is far from the corpus's was doing a different \
         kind of work from one whose share is far the other way. That is a fact about these ten \
         sessions and not a taxonomy of sessions in general, and a session with no entry is one \
         whose mix is close to the corpus's throughout.\n\n",
    );

    let unpaired: usize = facts
        .sessions
        .iter()
        .map(|session| session.unpaired_actions)
        .sum();
    out.push_str(&format!(
        "**{} action(s) are not a clean request-and-outcome pair** — a request with no outcome \
         record, an outcome with no request, or records that disagree. They are kept as actions \
         with their shape recorded rather than dropped or completed with a guess.\n\n",
        unpaired
    ));

    let failure_leads: Vec<&Family> = facts
        .workflow
        .families
        .iter()
        .filter(|family| {
            family.quarantine.is_empty()
                && family
                    .pipeline
                    .iter()
                    .any(|step| step.contains("(failed)") || step.contains("(denied)"))
        })
        .take(3)
        .collect();
    if failure_leads.is_empty() {
        out.push_str(
            "**No recurring shape contains a step the recorder saw fail or be denied.** With so \
             few failures in the corpus, a failure-recovery shape would have had very little to \
             recur from; this is an absence in the search's results, not evidence that agents do \
             not recover from failures.\n\n",
        );
    } else {
        out.push_str("**Shapes containing a failed or denied step:**\n\n");
        for family in failure_leads {
            out.push_str(&format!(
                "- `{}` — {} of {} sessions, {} occurrences.\n",
                pipeline_line(&family.pipeline),
                family.sessions,
                family.eligible,
                family.occurrences
            ));
        }
        out.push('\n');
    }

    // -- Limitations ----------------------------------------------------------
    out.push_str("## Limitations, in plain English\n\n");
    out.push_str(&format!(
        "1. **The corpus is small.** {} sessions, one project. Anything here is an envelope, not a \
         distribution, and a second project could look nothing like it.\n",
        facts.eligible_sessions
    ));
    out.push_str(
        "2. **A category is a label this analyser applied, not something the agent said or the \
         machine saw.** `Inspect` means \"a tool or leading program in the Inspect rule\", and \
         nothing more.\n",
    );
    out.push_str(
        "3. **Shell commands are classified by their leading program name only.** A command that \
         starts with a read-only utility is called `Inspect` even if a later stage of its pipeline \
         writes something. The command text was read to decide the label and then discarded; it is \
         in no output.\n",
    );
    out.push_str(
        "4. **An action is a correlation, not an execution.** A request record placed beside an \
         outcome record is two records that share an id. It is not a measured duration, not \
         containment, and not proof that anything ran between them.\n",
    );
    out.push_str(
        "5. **Nothing here reads reported intent.** What the agent said it was doing is a separate \
         channel and was not merged into any action. Correlation ids cited only by reported intent \
         contribute no action at all, and are counted separately.\n",
    );
    out.push_str(
        "6. **Prevalence counts sessions, not occurrences.** A long session that repeats a shape \
         forty times still counts once. The occurrence totals are printed beside the session counts \
         so a shape carried by one busy session is visible as one.\n",
    );
    let worst = workflow.degeneracy.iter().max_by(|left, right| {
        left.fraction
            .partial_cmp(&right.fraction)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    if let Some(worst) = worst
        && worst.fraction > 0.01
    {
        out.push_str(&format!(
            "7. **The reshuffling is not always free to move.** For session `{}`, {} of {} \
             reshuffles returned the session unchanged, so for that session the comparison is \
             partly against itself and the bar it sets is too high.\n",
            worst.identity, worst.identical, worst.replicates
        ));
    }
    out.push_str(
        "8. **This is an exploratory engineering round.** No criterion here was fixed in advance, \
         no verdict was declared before the numbers arrived, and nothing in this report is an \
         experimental result.\n\n",
    );

    // -- Reproduce ------------------------------------------------------------
    out.push_str("## How to reproduce this report\n\n");
    out.push_str("```text\n");
    out.push_str(&format!(
        "cargo run --release --example corpus-report -- \\\n    --recordings <DIR> --label {} \
         --out <OUTDIR>\n",
        facts.corpus_label
    ));
    out.push_str("```\n\n");
    out.push_str(&format!(
        "The same directory and the same configuration produce the same `facts.json`, \
         `manifest.json` and `report.md`, byte for byte. `manifest.json` fingerprints every input \
         file, so a later run can tell whether the corpus itself moved. To re-render the prose from \
         a stored analysis without re-analysing anything:\n\n```text\ncargo run --release --example \
         corpus-report -- --render-from <OUTDIR>/facts.json --out <OUTDIR>\n```\n\nAnalyser \
         `{}` version {}, vocabulary version {}, witnessglass {}. Null: {}. Statistic: {}.\n\n",
        facts.analyzer.name,
        facts.analyzer.version,
        facts.analyzer.vocabulary_version,
        facts.analyzer.witnessglass_version,
        facts.configuration.null,
        facts.configuration.statistic
    ));

    // -- A/B --------------------------------------------------------------------
    out.push_str("## What an A/B corpus comparison would mean\n\n");
    out.push_str(
        "`facts.json` is built to be diffed. Analyse a second directory with its own label, then:\n\n\
         ```text\ncargo run --release --example corpus-report -- \\\n    --compare <A>/facts.json \
         <B>/facts.json --out <OUTDIR>\n```\n\nThe result groups shapes into **gained** (present in \
         B, absent in A), **lost**, **strengthened** (present in both, more sessions in B), \
         **weakened**, and **unchanged**.\n\nWhat a difference would and would not mean:\n\n\
         - A shape gained in B is a difference between two corpora. It is *not* evidence that \
         anything changed in the agent, the project, or the tooling — corpora differ in length, in \
         task, and in how many sessions they hold.\n\
         - Two corpora with different session counts have different denominators, and a shape can \
         move from 3 of 4 to 6 of 12 and be weaker while its raw count doubled. The comparison \
         reports both numbers for that reason.\n\
         - Only shapes that exist in at least one side's search results appear. A shape absent from \
         both is not reported as absent, because the search was never asked about it.\n\n",
    );

    // -- Appendix ----------------------------------------------------------------
    out.push_str("## Technical appendix\n\n");
    out.push_str(&format!(
        "Search: `cross_pairs` over every unordered session pair at span lengths {:?}, ranked by \
         alignment total, deduplicated to {} non-overlapping winners per pair per length, scored \
         with {}. Workflow projection: {} pairs, {} candidates retained, {} of them exact. Raw \
         projection: {} pairs, {} candidates, {} exact. Mean sequence length {:.1} actions \
         (workflow) and {:.1} events (raw).\n\n",
        facts.configuration.span_ladder,
        facts.configuration.keep_per_pair,
        facts.configuration.statistic,
        workflow.pairs,
        workflow.candidates,
        workflow.exact_candidates,
        facts.raw.pairs,
        facts.raw.candidates,
        facts.raw.exact_candidates,
        workflow.mean_length,
        facts.raw.mean_length
    ));
    out.push_str(
        "The twenty best-supported workflow shapes, then the ten best-supported raw-event shapes. \
         `pairs` is how many session pairs the search independently produced the shape from.\n\n",
    );
    out.push_str("| shape | proj | k | sessions | occurrences | pairs | best R1 | tail |\n|---|---|---:|---:|---:|---:|---:|---:|\n");
    for family in workflow
        .families
        .iter()
        .take(20)
        .chain(facts.raw.families.iter().take(10))
    {
        out.push_str(&format!(
            "| `{}` | {} | {} | {} | {} | {} | {} | {} |\n",
            pipeline_line(&family.pipeline),
            family.projection.as_str(),
            family.k,
            family.sessions,
            family.occurrences,
            family.discovered_by_pairs,
            family
                .best_r1
                .map(|value| format!("{value:.4}"))
                .unwrap_or_else(|| "—".to_owned()),
            family
                .calibration
                .as_ref()
                .map(|calibration| format!("{:.3}", calibration.tail))
                .unwrap_or_else(|| "—".to_owned())
        ));
    }
    out.push('\n');
    out.push_str(&format!(
        "The tail is `(1 + exceedances) / ({} + 1)` where an exceedance is a reshuffled corpus \
         whose best shape of the same length reached the same session count or better. It is \
         family-wise by construction, because the null takes its own maximum over every shape it \
         could have found. It calibrates cross-session prevalence of an exact shape — **not** \
         sprint:19's `T`, and no conclusion about `T` transfers to it.\n",
        facts.configuration.replicates
    ));

    out
}

// ---------------------------------------------------------------------------
// Comparison
// ---------------------------------------------------------------------------

/// How one shape moved between two corpora.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Movement {
    /// Present in B, absent from A.
    Gained,
    /// Present in A, absent from B.
    Lost,
    /// In both, and B holds it in more sessions.
    Strengthened,
    /// In both, and B holds it in fewer sessions.
    Weakened,
    /// In both, at the same session count.
    Unchanged,
}

impl Movement {
    /// The heading this movement is printed under.
    pub fn heading(self) -> &'static str {
        match self {
            Movement::Gained => "Gained",
            Movement::Lost => "Lost",
            Movement::Strengthened => "Strengthened",
            Movement::Weakened => "Weakened",
            Movement::Unchanged => "Unchanged",
        }
    }
}

/// Compare two facts documents and render `comparison.md`.
///
/// Compares the workflow projection only, keyed on the exact mark sequence, so
/// two shapes are the same shape when they are literally the same steps in the
/// same order.
pub fn render_comparison(before: &Facts, after: &Facts) -> String {
    /// One side's view of a shape.
    struct Side {
        sessions: usize,
        eligible: usize,
        occurrences: usize,
        name: String,
        pipeline: Vec<String>,
    }
    let index = |facts: &Facts| -> BTreeMap<Vec<String>, Side> {
        facts
            .workflow
            .families
            .iter()
            .map(|family| {
                (
                    family.marks.clone(),
                    Side {
                        sessions: family.sessions,
                        eligible: family.eligible,
                        occurrences: family.occurrences,
                        name: family.name.clone(),
                        pipeline: family.pipeline.clone(),
                    },
                )
            })
            .collect()
    };
    let (left, right) = (index(before), index(after));

    let mut keys: Vec<&Vec<String>> = left.keys().chain(right.keys()).collect();
    keys.sort();
    keys.dedup();

    /// One rendered line, with the keys it is ordered by: total session support
    /// descending, then the shape itself, so the biggest change is the first
    /// thing read rather than whichever shape sorted first alphabetically.
    struct Entry {
        support: usize,
        marks: Vec<String>,
        line: String,
    }
    let mut buckets: BTreeMap<&'static str, Vec<Entry>> = BTreeMap::new();
    for key in keys {
        let a = left.get(key);
        let b = right.get(key);
        let (movement, line) = match (a, b) {
            (None, Some(b)) => (
                Movement::Gained,
                format!(
                    "- **{}** — `{}` — {} of {} sessions in `{}` ({} occurrences); not found in \
                     `{}`.",
                    b.name,
                    pipeline_line(&b.pipeline),
                    b.sessions,
                    b.eligible,
                    after.corpus_label,
                    b.occurrences,
                    before.corpus_label
                ),
            ),
            (Some(a), None) => (
                Movement::Lost,
                format!(
                    "- **{}** — `{}` — {} of {} sessions in `{}`; not found in `{}`.",
                    a.name,
                    pipeline_line(&a.pipeline),
                    a.sessions,
                    a.eligible,
                    before.corpus_label,
                    after.corpus_label
                ),
            ),
            (Some(a), Some(b)) => {
                let movement = match b.sessions.cmp(&a.sessions) {
                    std::cmp::Ordering::Greater => Movement::Strengthened,
                    std::cmp::Ordering::Less => Movement::Weakened,
                    std::cmp::Ordering::Equal => Movement::Unchanged,
                };
                (
                    movement,
                    format!(
                        "- **{}** — `{}` — {} of {} sessions in `{}`, {} of {} in `{}`.",
                        b.name,
                        pipeline_line(&b.pipeline),
                        a.sessions,
                        a.eligible,
                        before.corpus_label,
                        b.sessions,
                        b.eligible,
                        after.corpus_label
                    ),
                )
            }
            (None, None) => continue,
        };
        buckets.entry(movement.heading()).or_default().push(Entry {
            support: b.map(|side| side.sessions).unwrap_or(0)
                + a.map(|side| side.sessions).unwrap_or(0),
            marks: key.clone(),
            line,
        });
    }
    for lines in buckets.values_mut() {
        lines.sort_by(|left, right| {
            right
                .support
                .cmp(&left.support)
                .then(left.marks.cmp(&right.marks))
        });
    }

    let mut out = String::new();
    out.push_str(&format!(
        "# Corpus comparison — `{}` against `{}`\n\n",
        after.corpus_label, before.corpus_label
    ));
    out.push_str(
        "*Derived, disposable, and local. Not redacted and not safe to share.*\n\n\
         Shapes are matched on their exact step sequence in the workflow projection. Two corpora \
         with different session counts have different denominators, so both are printed; a shape \
         can gain sessions and still be a smaller share of its corpus.\n\n",
    );
    out.push_str(&format!(
        "`{}`: {} eligible sessions, {} shapes. `{}`: {} eligible sessions, {} shapes.\n\n",
        before.corpus_label,
        before.eligible_sessions,
        before.workflow.families.len(),
        after.corpus_label,
        after.eligible_sessions,
        after.workflow.families.len()
    ));
    for movement in [
        Movement::Gained,
        Movement::Strengthened,
        Movement::Weakened,
        Movement::Lost,
        Movement::Unchanged,
    ] {
        let lines = buckets.get(movement.heading());
        out.push_str(&format!("## {}\n\n", movement.heading()));
        match lines {
            Some(lines) if !lines.is_empty() => {
                for entry in lines.iter().take(20) {
                    out.push_str(&entry.line);
                    out.push('\n');
                }
                if lines.len() > 20 {
                    out.push_str(&format!("- …and {} more.\n", lines.len() - 20));
                }
                out.push('\n');
            }
            _ => out.push_str("None.\n\n"),
        }
    }
    out.push_str(
        "A difference between two corpora is a difference between two corpora. It is not evidence \
         that anything changed in the agent, the project, or the tooling.\n",
    );
    out
}
