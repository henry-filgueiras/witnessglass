//! Deriving an inspection view from a replayed recording.
//!
//! This is a projection in the sense `CLAUDE.md` §3 means: derived, disposable,
//! and fully rebuildable from the raw stream. It holds no fact the records
//! cannot regenerate, it borrows the records rather than owning copies of them,
//! and deleting it loses nothing. It never reads a file, never consults a clock,
//! and never mutates its input.
//!
//! decision:6 settles the boundary this module implements: raw replay is the
//! authority, this is the only correlation layer, and a rendering layer renders
//! what this supplies rather than reinterpreting raw NDJSON for itself.
//!
//! # Receipts
//!
//! Every derived entity here — a correlation, a cardinality classification, an
//! anomaly, an aggregate, a coverage summary, a timestamp extremum — carries the
//! raw sequence numbers that support it. A derived claim that cannot produce its
//! receipts is asserting rather than deriving, and this module has no way to
//! express one.
//!
//! # Negative claims
//!
//! An empty receipt list is not evidence of absence. "No `tool_failed` record"
//! means something different in a recording that ended cleanly and in one that
//! stops mid-record, and it means nothing at all about whether a tool failed
//! outside what was captured. So a count travels as a [`RecordCount`]: the
//! matching sequences *and* the [`ExaminedScope`] they were looked for in. Every
//! count in this module reads "records observed", never "events that occurred".
//!
//! # Order
//!
//! `sequence` remains the only total order. Nothing here sorts by timestamp.
//! Timestamp extrema are computed, because a reader wants to know when a
//! recording was written, but each extremum carries the sequence that supplies
//! it and no extremum establishes order, duration, overlap, or causality.
//! Output ordering is canonical record order throughout, and grouped values are
//! kept in first-appearance order rather than hash order, so two runs over one
//! recording produce byte-identical output.
//!
//! # Schema versions stay apart
//!
//! v1 and v2 are not flattened into a common lifecycle. A v1
//! `observed_tool_started` claims a witnessed beginning; a v2 `tool_requested`
//! claims only that a request existed. v1 knows succeeded and failed; v2 keeps
//! success, execution failure, and permission denial distinct. A v1
//! `tool_call_id` and a v2 `tool_use_id` are schema-specific correlation
//! mechanisms, and [`CorrelationId`] tags each with its version so two ids
//! spelled the same cannot compare equal across schemas.
//!
//! # Derived claims are not a raw channel
//!
//! [`Channel`] keeps its three raw values. `reported`, `observed`, and
//! `recorder` describe how a record reached the recording; a derived claim did
//! not reach the recording at all. Everything in this module is derived by
//! construction, so adding a fourth channel would only blur the raw provenance
//! the projection exists to carry forward.
//!
//! # Stability
//!
//! These types derive [`Serialize`] so a local rendering layer can consume them.
//! That representation is **internal and unstable**. It is not a public
//! interchange format, nothing outside this repository may depend on its shape,
//! and it carries no compatibility promise.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::record::{AnyRecord, Channel, v1, v2};
use crate::replay::{Replay, Tail};

/// A position in the canonical append chain.
pub type Sequence = u64;

/// Raw provenance channels, in a fixed order, so aggregate output does not
/// depend on which channels a recording happened to contain.
const CHANNELS: [Channel; 3] = [Channel::Reported, Channel::Observed, Channel::Recorder];

/// The raw sequence numbers supporting one derived claim.
///
/// Always ascending, because receipts are collected by scanning the records in
/// canonical order. On its own an empty list says nothing: see [`RecordCount`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Receipts(Vec<Sequence>);

impl Receipts {
    /// An empty receipt list.
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// The supporting sequences, ascending.
    pub fn sequences(&self) -> &[Sequence] {
        &self.0
    }

    /// How many raw records support the claim.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether no raw record supports the claim.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The earliest supporting sequence, if any.
    pub fn first(&self) -> Option<Sequence> {
        self.0.first().copied()
    }

    /// The latest supporting sequence, if any.
    pub fn last(&self) -> Option<Sequence> {
        self.0.last().copied()
    }

    /// Append a supporting sequence. Private, and only ever called while
    /// scanning records in canonical order, which is what keeps the list sorted.
    fn push(&mut self, sequence: Sequence) {
        debug_assert!(
            self.0.last().is_none_or(|&last| last < sequence),
            "receipts are collected in canonical order and must stay ascending"
        );
        self.0.push(sequence);
    }
}

/// What was examined to reach a conclusion, so that an absence can be read at
/// the strength the evidence actually supports.
///
/// This is the difference between "no matching record in this complete
/// recording" and "no matching record in the valid prefix of a recording that
/// stops mid-record". Neither is a claim about what happened outside the
/// recording, and nothing in this module makes one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExaminedScope {
    /// Every record of a recording whose final record was newline-terminated.
    /// Nothing is known to be missing from the end.
    CompleteRecording {
        /// How many records were examined.
        records: usize,
    },
    /// Every record of the valid prefix of a recording that stops mid-record.
    /// The fragment is not a record and was not examined; what it would have
    /// said is unknown.
    ValidPrefix {
        /// How many complete records were examined.
        records: usize,
        /// Byte offset where the unterminated fragment begins.
        fragment_byte_offset: u64,
        /// Length of the fragment in bytes.
        fragment_bytes: usize,
    },
}

impl ExaminedScope {
    /// How many records were examined.
    pub fn records(&self) -> usize {
        match self {
            ExaminedScope::CompleteRecording { records }
            | ExaminedScope::ValidPrefix { records, .. } => *records,
        }
    }

    /// Whether the recording examined stops mid-record.
    pub fn is_truncated(&self) -> bool {
        matches!(self, ExaminedScope::ValidPrefix { .. })
    }
}

/// A count of records, carrying both its receipts and the scope it was counted
/// in.
///
/// A positive count cites the records that produced it. A zero count cites the
/// scope it searched, which is the only thing that makes zero interpretable. It
/// means "no matching record was observed here" and never "this did not happen".
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RecordCount {
    /// Sequences of the matching records, ascending.
    pub records: Receipts,
    /// The population searched.
    pub scope: ExaminedScope,
}

impl RecordCount {
    /// How many records matched.
    pub fn count(&self) -> usize {
        self.records.len()
    }

    /// Whether no record matched within [`RecordCount::scope`].
    pub fn is_absent(&self) -> bool {
        self.records.is_empty()
    }
}

/// A value as delivered, with the records that delivered it.
///
/// Used where several records supply a field that ought to agree and might not.
/// Every delivered value is kept; none is chosen as canonical.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DeliveredValue<T> {
    /// The value exactly as the integration delivered it.
    pub value: T,
    /// Records that delivered it.
    pub receipts: Receipts,
}

/// One count within an aggregate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Tally<T> {
    /// What is being counted.
    pub value: T,
    /// The records counted, and the scope they were counted in. A zero tally is
    /// a first-class entry: it says the vocabulary has this kind and this
    /// recording contains no record of it.
    pub records: RecordCount,
}

/// The v1 event vocabulary, as a projection-side kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum V1Kind {
    /// Opening session boundary.
    SessionStarted,
    /// Closing session boundary.
    SessionEnded,
    /// Reported semantics.
    ReportedIntent,
    /// A claimed observation of a tool call *beginning*. v1 asserts this; v2
    /// deliberately does not.
    ObservedToolStarted,
    /// An observed end of a tool call, succeeded or failed. v1 has no denial.
    ObservedToolFinished,
}

impl V1Kind {
    /// Every v1 kind, so a zero count can be reported for one a recording never
    /// contained.
    pub const ALL: [V1Kind; 5] = [
        V1Kind::SessionStarted,
        V1Kind::SessionEnded,
        V1Kind::ReportedIntent,
        V1Kind::ObservedToolStarted,
        V1Kind::ObservedToolFinished,
    ];

    /// Stable name, as it appears in a record.
    pub fn as_str(self) -> &'static str {
        match self {
            V1Kind::SessionStarted => "session_started",
            V1Kind::SessionEnded => "session_ended",
            V1Kind::ReportedIntent => "reported_intent",
            V1Kind::ObservedToolStarted => "observed_tool_started",
            V1Kind::ObservedToolFinished => "observed_tool_finished",
        }
    }
}

/// The v2 event vocabulary, as a projection-side kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum V2Kind {
    /// Opening session boundary.
    SessionStarted,
    /// Closing session boundary.
    SessionEnded,
    /// Reported semantics.
    ReportedIntent,
    /// A request existed. Not evidence that anything executed.
    ToolRequested,
    /// A call executed successfully.
    ToolSucceeded,
    /// A call executed and failed.
    ToolFailed,
    /// A call was denied and did not execute.
    ToolDenied,
    /// A subagent started.
    SubagentStarted,
    /// A subagent stopped.
    SubagentStopped,
}

impl V2Kind {
    /// Every v2 kind, so a zero count can be reported for one a recording never
    /// contained — which is how "no failure record was observed" gets said
    /// without saying "nothing failed".
    pub const ALL: [V2Kind; 9] = [
        V2Kind::SessionStarted,
        V2Kind::SessionEnded,
        V2Kind::ReportedIntent,
        V2Kind::ToolRequested,
        V2Kind::ToolSucceeded,
        V2Kind::ToolFailed,
        V2Kind::ToolDenied,
        V2Kind::SubagentStarted,
        V2Kind::SubagentStopped,
    ];

    /// Stable name, as it appears in a record.
    pub fn as_str(self) -> &'static str {
        match self {
            V2Kind::SessionStarted => "session_started",
            V2Kind::SessionEnded => "session_ended",
            V2Kind::ReportedIntent => "reported_intent",
            V2Kind::ToolRequested => "tool_requested",
            V2Kind::ToolSucceeded => "tool_succeeded",
            V2Kind::ToolFailed => "tool_failed",
            V2Kind::ToolDenied => "tool_denied",
            V2Kind::SubagentStarted => "subagent_started",
            V2Kind::SubagentStopped => "subagent_stopped",
        }
    }
}

/// A schema-tagged event kind.
///
/// Tagged rather than flattened because the two vocabularies do not mean the
/// same things. A reader comparing kinds across versions has to unwrap the tag,
/// which is the point.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    /// A v1 kind.
    V1(V1Kind),
    /// A v2 kind.
    V2(V2Kind),
}

impl EventKind {
    /// Stable name, as it appears in a record. Two versions can share a name and
    /// still be different kinds, which is why this is not an identity.
    pub fn as_str(self) -> &'static str {
        match self {
            EventKind::V1(kind) => kind.as_str(),
            EventKind::V2(kind) => kind.as_str(),
        }
    }

    /// Schema version this kind belongs to.
    pub fn schema_version(self) -> u32 {
        match self {
            EventKind::V1(_) => 1,
            EventKind::V2(_) => 2,
        }
    }
}

/// A correlation identifier, tagged with the schema that defines it.
///
/// v1's `tool_call_id` and v2's `tool_use_id` are schema-specific correlation
/// mechanisms, not one normalized field. Tagging makes accidental cross-version
/// equivalence unrepresentable rather than merely discouraged: two ids spelled
/// identically under different schemas are different keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CorrelationId<'a> {
    /// A v1 `tool_call_id`.
    V1ToolCallId(&'a str),
    /// A v2 `tool_use_id`.
    V2ToolUseId(&'a str),
}

impl<'a> CorrelationId<'a> {
    /// The identifier as delivered, without its schema tag.
    pub fn as_str(self) -> &'a str {
        match self {
            CorrelationId::V1ToolCallId(id) | CorrelationId::V2ToolUseId(id) => id,
        }
    }
}

/// The current-agent identity a record was delivered with.
///
/// Absent identity is a fact about coverage and is preserved as one. It is never
/// filled in with "root", "main", or a synthetic id, and it is never taken from
/// a subagent lifecycle event, whose `agent_id` names the child the event is
/// *about* rather than the agent the event came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentAttribution<'a> {
    /// The integration supplied a current-agent identity for this record.
    Supplied {
        /// The delivered `context.agent_id`.
        agent_id: &'a str,
        /// The delivered `context.agent_type`, when supplied.
        agent_type: Option<&'a str>,
    },
    /// The envelope could carry a current-agent identity and none was supplied.
    /// This is not evidence of a root agent.
    NotSupplied {
        /// A delivered `context.agent_type` with no accompanying id, when that
        /// is what arrived. Kept because discarding delivered evidence is its
        /// own defect.
        agent_type: Option<&'a str>,
    },
    /// A v1 record. The v1 envelope has no causal context at all, so the
    /// question was never asked of this record — different from asking and
    /// getting nothing.
    NotRepresentable,
}

/// A parent identity, recorded only because a subagent event delivered one.
///
/// Never inferred from an `Agent` tool call, from sequence containment, from
/// matching agent types, or from timing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct SuppliedParent<'a> {
    /// The delivered `parent_agent_id`, when supplied.
    pub agent_id: Option<&'a str>,
    /// The delivered `parent_agent_type`, when supplied.
    pub agent_type: Option<&'a str>,
}

/// The child agent a subagent lifecycle record is *about*.
///
/// Structurally separate from [`AgentAttribution`]: this identifies the subject
/// of the event, not the agent it was delivered from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct SubjectAgent<'a> {
    /// The delivered child `agent_id`.
    pub agent_id: &'a str,
    /// The delivered child `agent_type`, when supplied.
    pub agent_type: Option<&'a str>,
    /// Parent identity, present only when the event itself delivered one.
    pub supplied_parent: Option<SuppliedParent<'a>>,
}

/// Field-level facts about a record's payload, extracted by Rust.
///
/// A rendering layer filters on these. It does not reinterpret raw event JSON to
/// rediscover them, because two implementations of what a recording says are two
/// opinions about what a recording says.
///
/// The presence flags mean "the delivered JSON value is not `null`". A field
/// that a schema always carries can still arrive null, and that is a different
/// fact from the field being absent from the vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct PayloadFacets<'a> {
    /// A requested input arrived: v2 `tool_requested`/`tool_denied`, or v1
    /// `arguments` on a claimed start.
    pub has_requested_input: bool,
    /// An effective input arrived: v2 `tool_succeeded`/`tool_failed`. v1 has no
    /// effective input, so this is always false for a v1 record.
    pub has_effective_input: bool,
    /// A tool response arrived: v2 `tool_succeeded`, or v1 `result`.
    pub has_response: bool,
    /// The delivered error text, on a v2 `tool_failed`.
    pub error: Option<&'a str>,
    /// The delivered `duration_ms`, when the integration supplied one. Absent
    /// means it did not supply one, not that the call took no time.
    pub duration_ms: Option<u64>,
    /// The delivered `interrupted` flag. Absent means the integration did not
    /// say either way, which is not the same as `false`.
    pub interrupted: Option<bool>,
    /// v1's succeeded/failed outcome vocabulary, kept out of v2's.
    pub v1_outcome: Option<v1::ToolOutcome>,
    /// The delivered session start source.
    pub session_source: Option<&'a str>,
    /// The delivered session end reason.
    pub session_reason: Option<&'a str>,
    /// What the agent said, on a reported intent. A claim, and stays one.
    pub reported_text: Option<&'a str>,
}

/// One entry of the canonical ledger: one raw record, plus the semantic metadata
/// a rendering layer needs to display and filter it.
///
/// The raw record is borrowed, not copied. The projection cannot rewrite raw
/// evidence because it does not own any.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LedgerEntry<'a> {
    /// Position in the canonical append chain. The only total order.
    pub sequence: Sequence,
    /// When the recorder wrote the record. Descriptive metadata; it establishes
    /// no order, duration, overlap, or causality.
    pub recorded_at: jiff::Timestamp,
    /// Raw provenance channel. Not a derived classification.
    pub channel: Channel,
    /// Integration that produced the event.
    pub adapter: &'a str,
    /// Capture point within that integration.
    pub mechanism: &'a str,
    /// Schema-tagged event kind.
    pub kind: EventKind,
    /// The schema-specific correlation identifier this record carries, if any.
    pub correlation: Option<CorrelationId<'a>>,
    /// Tool name exactly as delivered on this record.
    pub tool_name: Option<&'a str>,
    /// Identity of the agent this record was delivered from.
    pub current_agent: AgentAttribution<'a>,
    /// Identity of the child agent this record is *about*, on a subagent
    /// lifecycle record only.
    pub subject_agent: Option<SubjectAgent<'a>>,
    /// The delivered `context.prompt_id`, carried through as raw context.
    ///
    /// It groups nothing. dragon:3 is open: this identifier delimits no unit of
    /// work this project has established, so no projection may segment by it and
    /// no view may describe a recording as containing N turns. It survives here
    /// as a field, and appears in the coverage summary as a presence count.
    pub prompt_id: Option<&'a str>,
    /// Payload facts Rust extracted.
    pub facets: PayloadFacets<'a>,
    /// The raw record, borrowed and unmodified.
    pub record: &'a AnyRecord,
}

/// Observed tool-lifecycle evidence for one correlation id, kept in the
/// vocabulary of its own schema.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolEvidence {
    /// v1 evidence. `started` claims a witnessed *beginning* — a claim v2
    /// refuses to make — and the outcome vocabulary is succeeded/failed with no
    /// denial.
    V1 {
        /// `observed_tool_started` records.
        started: Receipts,
        /// `observed_tool_finished` records with outcome `succeeded`.
        finished_succeeded: Receipts,
        /// `observed_tool_finished` records with outcome `failed`.
        finished_failed: Receipts,
    },
    /// v2 evidence. A request is not an execution, and denial is not failure.
    V2 {
        /// `tool_requested` records. Evidence that a request existed, and
        /// nothing more.
        requested: Receipts,
        /// `tool_succeeded` records.
        succeeded: Receipts,
        /// `tool_failed` records: the call ran and something went wrong.
        failed: Receipts,
        /// `tool_denied` records: the call did not run.
        denied: Receipts,
    },
}

impl ToolEvidence {
    /// Records on the opening side: v2 `tool_requested`, or v1
    /// `observed_tool_started`. The count is comparable across versions; what
    /// the record *claims* is not, which is why the two stay in separate
    /// variants.
    pub fn opening(&self) -> &Receipts {
        match self {
            ToolEvidence::V1 { started, .. } => started,
            ToolEvidence::V2 { requested, .. } => requested,
        }
    }

    /// How many outcome records correlate to this id.
    pub fn outcome_count(&self) -> usize {
        match self {
            ToolEvidence::V1 {
                finished_succeeded,
                finished_failed,
                ..
            } => finished_succeeded.len() + finished_failed.len(),
            ToolEvidence::V2 {
                succeeded,
                failed,
                denied,
                ..
            } => succeeded.len() + failed.len() + denied.len(),
        }
    }

    /// The single outcome sequence, when exactly one outcome record exists.
    fn sole_outcome(&self) -> Option<Sequence> {
        if self.outcome_count() != 1 {
            return None;
        }
        match self {
            ToolEvidence::V1 {
                finished_succeeded,
                finished_failed,
                ..
            } => finished_succeeded
                .first()
                .or_else(|| finished_failed.first()),
            ToolEvidence::V2 {
                succeeded,
                failed,
                denied,
                ..
            } => succeeded
                .first()
                .or_else(|| failed.first())
                .or_else(|| denied.first()),
        }
    }

    /// How many semantically distinct outcome classes appear for this id. More
    /// than one is a disagreement about what became of the call.
    pub fn outcome_classes(&self) -> usize {
        match self {
            ToolEvidence::V1 {
                finished_succeeded,
                finished_failed,
                ..
            } => {
                usize::from(!finished_succeeded.is_empty())
                    + usize::from(!finished_failed.is_empty())
            }
            ToolEvidence::V2 {
                succeeded,
                failed,
                denied,
                ..
            } => {
                usize::from(!succeeded.is_empty())
                    + usize::from(!failed.is_empty())
                    + usize::from(!denied.is_empty())
            }
        }
    }

    /// Every outcome record's sequence, ascending.
    fn outcome_receipts(&self) -> Receipts {
        let mut all: Vec<Sequence> = match self {
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
        Receipts(all)
    }
}

/// How much evidence correlates to one id, and in what shape.
///
/// This classifies cardinality only. What an opening-side record *claims*
/// differs between schema versions and lives in [`ToolEvidence`]; nothing here
/// normalizes the two into one meaning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupShape {
    /// Only reported intent cites this id. No observed record does. A claim with
    /// no observation beside it.
    ReportedIntentOnly,
    /// An opening-side record with no outcome record in the examined scope. The
    /// recorder saw a request or a claimed start and did not see what became of
    /// it. Not "still running".
    OpeningWithoutOutcome,
    /// An outcome record with no opening-side record in the examined scope. The
    /// only evidence that the call existed, kept exactly as delivered.
    OutcomeWithoutOpening,
    /// Exactly one opening-side record and exactly one outcome record.
    ///
    /// This is the only shape that may be described as a paired lifecycle, and
    /// even here it is a correlation between two records — not an execution
    /// span, not a measured duration, and not a containment relationship.
    PairedLifecycle,
    /// More than one record on a side, or outcome records that disagree.
    /// Nothing is selected as canonical and nothing is greedily paired; see the
    /// recording's anomalies for which of those applies.
    Ambiguous,
}

/// Two canonical positions in the append chain.
///
/// Exposed only for the unambiguous one-to-one case. It is not elapsed time, not
/// execution duration, not nesting, and not causal containment. Records
/// appearing between these two positions are not thereby children of anything —
/// the recording does not distinguish a nested call from an adjacent one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct SequenceInterval {
    /// Sequence of the opening-side record.
    pub opening: Sequence,
    /// Sequence of the outcome record.
    pub outcome: Sequence,
}

/// Everything correlating to one schema-specific tool correlation id.
///
/// Correlation places evidence beside evidence. Reported intent stays a separate
/// record and a separate claim; it is never fused with the observed evidence it
/// sits next to, and no intent is reconstructed from a command, a tool name, a
/// payload description, a path, temporal proximity, or a result.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ToolGroup<'a> {
    /// The schema-tagged correlation id.
    pub id: CorrelationId<'a>,
    /// Earliest sequence of any record in the group. The group's canonical
    /// position.
    pub first_sequence: Sequence,
    /// Reported intent records citing this id. Claims, correlated and not merged.
    pub reported_intents: RecordCount,
    /// Observed evidence, in its own schema's vocabulary.
    pub evidence: ToolEvidence,
    /// Cardinality classification.
    pub shape: GroupShape,
    /// The two correlated positions, for [`GroupShape::PairedLifecycle`] only.
    pub paired_interval: Option<SequenceInterval>,
    /// Every tool name delivered for this id, with receipts. More than one entry
    /// is a disagreement between records; none is chosen as canonical.
    pub delivered_tool_names: Vec<DeliveredValue<&'a str>>,
    /// The scope every negative statement about this group was reached in.
    pub scope: ExaminedScope,
}

/// A child agent's lifecycle boundaries, as recorded.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SubagentLifecycle<'a> {
    /// The child agent id, as delivered.
    pub agent_id: &'a str,
    /// Earliest sequence of any boundary record naming this id.
    pub first_sequence: Sequence,
    /// `subagent_started` records naming this id.
    pub started: RecordCount,
    /// `subagent_stopped` records naming this id.
    pub stopped: RecordCount,
    /// Every `agent_type` delivered for this id, including absence, with
    /// receipts. More than one entry is a disagreement, exposed rather than
    /// resolved by taking the first.
    pub delivered_types: Vec<DeliveredValue<Option<&'a str>>>,
    /// Every parent identity delivered on a boundary record for this id. Empty
    /// means no boundary record supplied one, and no parent is inferred from
    /// anything else.
    pub supplied_parents: Vec<DeliveredValue<SuppliedParent<'a>>>,
}

/// Session boundary records, counted honestly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SessionBoundaries {
    /// `session_started` records.
    pub started: RecordCount,
    /// `session_ended` records.
    pub ended: RecordCount,
}

/// Current-agent attribution across the recording.
///
/// Counts records by the identity they were *delivered from*. The child ids
/// named by subagent lifecycle events are deliberately not folded in here: a
/// subagent boundary record is about a child and is not evidence that the record
/// came from it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CurrentAgentAggregate<'a> {
    /// Records per delivered `context.agent_id`, in first-appearance order.
    pub supplied: Vec<Tally<&'a str>>,
    /// Records whose envelope could carry a current-agent identity and did not.
    /// Explicitly unattributed. Not a root agent.
    pub not_supplied: RecordCount,
    /// v1 records, whose envelope has no causal context field at all.
    pub not_representable: RecordCount,
}

/// A field whose population and coverage are worth stating.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CoveredField {
    /// `duration_ms` on v2 `tool_succeeded` and `tool_failed`.
    V2DurationMs,
    /// `interrupted` on v2 `tool_failed`.
    V2Interrupted,
    /// Supplied parent identity on v2 subagent boundary records.
    V2SuppliedParentAgent,
    /// `context.prompt_id` on v2 records. A presence count only: dragon:3 is
    /// open and this identifier groups nothing.
    V2PromptId,
}

/// How often a field arrived, within a stated population.
///
/// Always a statement about records. "Absent" means the integration did not
/// supply the field on those records, never that the underlying quantity does
/// not exist.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FieldCoverage {
    /// Which field.
    pub field: CoveredField,
    /// The records the field could have arrived on.
    pub population: RecordCount,
    /// Records where it did arrive.
    pub present: RecordCount,
    /// Records in the population where it did not.
    pub absent: RecordCount,
}

/// Counts across the recording, by raw provenance and by delivered metadata.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Aggregates<'a> {
    /// The scope every count here was taken in.
    pub scope: ExaminedScope,
    /// Records per raw provenance channel, in a fixed channel order, including
    /// zero counts.
    pub by_channel: Vec<Tally<Channel>>,
    /// Records per schema-specific event kind, covering the recording's whole
    /// schema vocabulary including kinds it contains none of. Empty for a
    /// recording with no complete records, where no vocabulary is established.
    pub by_event_kind: Vec<Tally<EventKind>>,
    /// Records per adapter, in first-appearance order.
    pub by_adapter: Vec<Tally<&'a str>>,
    /// Records per capture mechanism, in first-appearance order.
    pub by_mechanism: Vec<Tally<&'a str>>,
}

/// A timestamp and the record that supplies it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct TimestampPoint {
    /// The recorded wall-clock value.
    pub recorded_at: jiff::Timestamp,
    /// The record it came from. Ties are resolved to the earliest such record in
    /// canonical order — a resolution about which record is cited, not about
    /// which came first.
    pub sequence: Sequence,
}

/// Descriptive timestamp extrema.
///
/// Computed by scanning, never by sorting. Timestamps establish no order, no
/// duration, no overlap, and no causality; these values describe when the
/// recorder wrote records, and each carries the sequence that supplies it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TimestampExtrema {
    /// The earliest recorded timestamp.
    pub earliest: TimestampPoint,
    /// The latest recorded timestamp.
    pub latest: TimestampPoint,
    /// Records whose timestamp is earlier than that of the record before them in
    /// append order. A clock that moves backwards mid-session leaves the
    /// recording's order perfectly intact; this is descriptive, not damage.
    pub non_monotonic: RecordCount,
}

/// Something the recording contains that a reader should not have to find.
///
/// An anomaly is evidence, not a parse failure. Corruption fails upstream in
/// replay; everything survivable inside a valid replay arrives here with its
/// receipts and the scope it was found in.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Anomaly<'a> {
    /// What was found.
    pub kind: AnomalyKind<'a>,
    /// The records supporting it. Empty only where the anomaly *is* an absence,
    /// in which case [`Anomaly::scope`] carries what was searched.
    pub receipts: Receipts,
    /// The population examined.
    pub scope: ExaminedScope,
}

/// The kinds of survivable irregularity this projection reports.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AnomalyKind<'a> {
    /// No `session_started` record in the examined scope.
    MissingSessionStart,
    /// No `session_ended` record in the examined scope.
    MissingSessionEnd,
    /// More than one `session_started` record.
    DuplicateSessionStart,
    /// More than one `session_ended` record.
    DuplicateSessionEnd,
    /// More than one opening-side record shares one correlation id. Not paired
    /// greedily with anything.
    DuplicateOpenings {
        /// The correlation id.
        id: CorrelationId<'a>,
    },
    /// More than one outcome record shares one correlation id.
    DuplicateOutcomes {
        /// The correlation id.
        id: CorrelationId<'a>,
    },
    /// Outcome records for one id disagree about what became of the call — a v2
    /// success beside a failure or a denial, or a v1 succeeded beside a failed.
    /// Both are kept; neither is chosen.
    ConflictingOutcomes {
        /// The correlation id.
        id: CorrelationId<'a>,
    },
    /// An opening-side record with no outcome record in the examined scope.
    OpeningWithoutOutcome {
        /// The correlation id.
        id: CorrelationId<'a>,
    },
    /// An outcome record with no opening-side record in the examined scope.
    OutcomeWithoutOpening {
        /// The correlation id.
        id: CorrelationId<'a>,
    },
    /// Reported intent citing an id no observed record carries.
    ReportedIntentWithoutObservedEvidence {
        /// The correlation id.
        id: CorrelationId<'a>,
    },
    /// Records for one id delivered different tool names. None is canonical.
    DivergentToolNames {
        /// The correlation id.
        id: CorrelationId<'a>,
    },
    /// A `subagent_stopped` whose child id no observed `subagent_started` named.
    SubagentStopWithoutStart {
        /// The child agent id, as delivered.
        agent_id: &'a str,
    },
    /// A `subagent_started` whose child id no observed `subagent_stopped` named.
    SubagentStartWithoutStop {
        /// The child agent id, as delivered.
        agent_id: &'a str,
    },
    /// One agent id arrived with more than one delivered `agent_type`. All
    /// delivered values are retained; the disagreement is exposed rather than
    /// resolved by taking the first.
    DivergentAgentTypes {
        /// The agent id, as delivered.
        agent_id: &'a str,
    },
}

/// A derived, disposable inspection view over one replayed recording.
///
/// Borrows the replay. Nothing here owns raw evidence, so nothing here can
/// rewrite it, and dropping the whole projection loses nothing that the raw
/// stream cannot regenerate.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Inspection<'a> {
    /// Schema version established by the recording's first record. `None` for a
    /// recording holding no complete records, where no vocabulary is
    /// established at all.
    pub schema_version: Option<u64>,
    /// The scope every negative claim in this projection was reached in.
    ///
    /// This also carries the recording's tail state: a complete recording and
    /// the valid prefix of a truncated one are different scopes, which is
    /// exactly the distinction an absence has to be read against. See
    /// [`Inspection::tail`] to recover the replay's own [`Tail`] from it.
    pub scope: ExaminedScope,
    /// The session the records belong to. `None` for a recording with no
    /// complete records.
    pub session_id: Option<&'a str>,
    /// The raw records, borrowed, in exact canonical append order.
    pub records: &'a [AnyRecord],
    /// One entry per raw record, in canonical append order, with the semantic
    /// metadata a rendering layer needs.
    pub ledger: Vec<LedgerEntry<'a>>,
    /// Session boundary counts.
    pub session_boundaries: SessionBoundaries,
    /// Tool evidence grouped by schema-specific correlation id, in canonical
    /// order of each group's earliest record.
    pub tool_groups: Vec<ToolGroup<'a>>,
    /// Subagent boundaries grouped by child agent id, in canonical order.
    pub subagents: Vec<SubagentLifecycle<'a>>,
    /// Current-agent attribution.
    pub current_agents: CurrentAgentAggregate<'a>,
    /// Channel, event-kind, adapter, and mechanism counts.
    pub aggregates: Aggregates<'a>,
    /// Field coverage summaries, in a fixed field order.
    pub coverage: Vec<FieldCoverage>,
    /// Descriptive timestamp extrema. `None` when there are no records.
    pub timestamps: Option<TimestampExtrema>,
    /// Survivable irregularities, in canonical order of their earliest receipt.
    pub anomalies: Vec<Anomaly<'a>>,
}

impl Inspection<'_> {
    /// How many raw records the projection covers.
    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    /// The replay's tail state, recovered from [`Inspection::scope`].
    ///
    /// Complete-versus-truncated survives the projection intact, and it survives
    /// as part of the scope rather than beside it, because an absence and the
    /// tail state it was found under are one fact, not two.
    pub fn tail(&self) -> Tail {
        match self.scope {
            ExaminedScope::CompleteRecording { .. } => Tail::Complete,
            ExaminedScope::ValidPrefix {
                fragment_byte_offset,
                fragment_bytes,
                ..
            } => Tail::Truncated {
                byte_offset: fragment_byte_offset,
                bytes: fragment_bytes,
            },
        }
    }
}

/// Counts keyed by a delivered value, kept in first-appearance order.
///
/// Deliberately not a hash map: output order must be canonical record order so
/// that two runs over one recording agree byte for byte.
struct FirstAppearance<K> {
    entries: Vec<(K, Receipts)>,
}

impl<K: PartialEq> FirstAppearance<K> {
    fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    fn record(&mut self, key: K, sequence: Sequence) {
        if let Some(entry) = self
            .entries
            .iter_mut()
            .find(|(existing, _)| *existing == key)
        {
            entry.1.push(sequence);
        } else {
            let mut receipts = Receipts::new();
            receipts.push(sequence);
            self.entries.push((key, receipts));
        }
    }

    fn into_entries(self) -> Vec<(K, Receipts)> {
        self.entries
    }
}

/// What is being accumulated for one correlation id while scanning.
struct GroupBuilder<'a> {
    first_sequence: Sequence,
    reported_intents: Receipts,
    evidence: ToolEvidence,
    tool_names: FirstAppearance<&'a str>,
}

/// What is being accumulated for one child agent id while scanning.
struct SubagentBuilder<'a> {
    first_sequence: Sequence,
    started: Receipts,
    stopped: Receipts,
    types: FirstAppearance<Option<&'a str>>,
    parents: FirstAppearance<SuppliedParent<'a>>,
}

/// Project a replayed recording into an inspection view.
///
/// Pure and deterministic: the same [`Replay`] always yields the same
/// projection. It reads no file, consults no clock, and takes its input by
/// shared reference, so it cannot mutate the records it derives from.
///
/// It cannot fail. Corruption is a replay failure and never reaches here; a
/// survivable irregularity inside a valid replay is evidence, and arrives in
/// [`Inspection::anomalies`] with its receipts.
pub fn inspect<'a>(replay: &'a Replay) -> Inspection<'a> {
    let records = replay.records.as_slice();
    let scope = match replay.tail {
        Tail::Complete => ExaminedScope::CompleteRecording {
            records: records.len(),
        },
        Tail::Truncated { byte_offset, bytes } => ExaminedScope::ValidPrefix {
            records: records.len(),
            fragment_byte_offset: byte_offset,
            fragment_bytes: bytes,
        },
    };
    let count = |receipts: Receipts| RecordCount {
        records: receipts,
        scope,
    };

    let mut ledger = Vec::with_capacity(records.len());
    let mut groups: BTreeMap<CorrelationId<'a>, GroupBuilder<'a>> = BTreeMap::new();
    let mut subagents: BTreeMap<&'a str, SubagentBuilder<'a>> = BTreeMap::new();
    let mut by_channel: [Receipts; 3] = [Receipts::new(), Receipts::new(), Receipts::new()];
    let mut by_kind: FirstAppearance<EventKind> = FirstAppearance::new();
    let mut by_adapter: FirstAppearance<&'a str> = FirstAppearance::new();
    let mut by_mechanism: FirstAppearance<&'a str> = FirstAppearance::new();
    let mut current_agents: FirstAppearance<&'a str> = FirstAppearance::new();
    let mut agent_types: BTreeMap<&'a str, FirstAppearance<Option<&'a str>>> = BTreeMap::new();
    let mut agent_not_supplied = Receipts::new();
    let mut agent_not_representable = Receipts::new();
    let mut session_started = Receipts::new();
    let mut session_ended = Receipts::new();
    let mut duration_population = Receipts::new();
    let mut duration_present = Receipts::new();
    let mut duration_absent = Receipts::new();
    let mut interrupted_population = Receipts::new();
    let mut interrupted_present = Receipts::new();
    let mut interrupted_absent = Receipts::new();
    let mut parent_population = Receipts::new();
    let mut parent_present = Receipts::new();
    let mut parent_absent = Receipts::new();
    let mut prompt_population = Receipts::new();
    let mut prompt_present = Receipts::new();
    let mut prompt_absent = Receipts::new();
    let mut non_monotonic = Receipts::new();
    let mut earliest: Option<TimestampPoint> = None;
    let mut latest: Option<TimestampPoint> = None;
    let mut previous_timestamp: Option<jiff::Timestamp> = None;

    for record in records {
        let sequence = record.sequence();
        let provenance = record.provenance();
        let recorded_at = record.recorded_at();

        by_channel[channel_index(provenance.channel)].push(sequence);
        by_adapter.record(provenance.adapter.as_str(), sequence);
        by_mechanism.record(provenance.mechanism.as_str(), sequence);

        // Extrema by scanning, never by sorting. A tie cites the earliest
        // record in canonical order, which decides which record is quoted and
        // nothing else.
        if earliest
            .as_ref()
            .is_none_or(|point| recorded_at < point.recorded_at)
        {
            earliest = Some(TimestampPoint {
                recorded_at,
                sequence,
            });
        }
        if latest
            .as_ref()
            .is_none_or(|point| recorded_at > point.recorded_at)
        {
            latest = Some(TimestampPoint {
                recorded_at,
                sequence,
            });
        }
        if previous_timestamp.is_some_and(|previous| recorded_at < previous) {
            non_monotonic.push(sequence);
        }
        previous_timestamp = Some(recorded_at);

        let (kind, correlation, tool_name, current_agent, subject_agent, prompt_id, facets) =
            match record {
                AnyRecord::V1(v1_record) => {
                    let parts = describe_v1(&v1_record.event);
                    agent_not_representable.push(sequence);
                    (
                        EventKind::V1(parts.kind),
                        parts.correlation,
                        parts.tool_name,
                        AgentAttribution::NotRepresentable,
                        None,
                        None,
                        parts.facets,
                    )
                }
                AnyRecord::V2(v2_record) => {
                    let parts = describe_v2(&v2_record.event);
                    let context = &v2_record.context;

                    // The current agent is the envelope's identity and only the
                    // envelope's identity. A subagent boundary event's agent_id
                    // names the child the event is about, and is filed as the
                    // subject below, where it cannot be mistaken for the emitter.
                    let attribution = match context.agent_id.as_deref() {
                        Some(agent_id) => {
                            current_agents.record(agent_id, sequence);
                            agent_types
                                .entry(agent_id)
                                .or_insert_with(FirstAppearance::new)
                                .record(context.agent_type.as_deref(), sequence);
                            AgentAttribution::Supplied {
                                agent_id,
                                agent_type: context.agent_type.as_deref(),
                            }
                        }
                        None => {
                            agent_not_supplied.push(sequence);
                            AgentAttribution::NotSupplied {
                                agent_type: context.agent_type.as_deref(),
                            }
                        }
                    };

                    prompt_population.push(sequence);
                    match context.prompt_id.as_deref() {
                        Some(_) => prompt_present.push(sequence),
                        None => prompt_absent.push(sequence),
                    }

                    match &v2_record.event {
                        v2::Event::ToolSucceeded(_) | v2::Event::ToolFailed(_) => {
                            duration_population.push(sequence);
                            match parts.facets.duration_ms {
                                Some(_) => duration_present.push(sequence),
                                None => duration_absent.push(sequence),
                            }
                        }
                        _ => {}
                    }
                    if matches!(&v2_record.event, v2::Event::ToolFailed(_)) {
                        interrupted_population.push(sequence);
                        match parts.facets.interrupted {
                            Some(_) => interrupted_present.push(sequence),
                            None => interrupted_absent.push(sequence),
                        }
                    }
                    if parts.subject.is_some() {
                        parent_population.push(sequence);
                    }

                    (
                        EventKind::V2(parts.kind),
                        parts.correlation,
                        parts.tool_name,
                        attribution,
                        parts.subject,
                        context.prompt_id.as_deref(),
                        parts.facets,
                    )
                }
            };

        by_kind.record(kind, sequence);

        match kind {
            EventKind::V1(V1Kind::SessionStarted) | EventKind::V2(V2Kind::SessionStarted) => {
                session_started.push(sequence);
            }
            EventKind::V1(V1Kind::SessionEnded) | EventKind::V2(V2Kind::SessionEnded) => {
                session_ended.push(sequence);
            }
            _ => {}
        }

        if let Some(id) = correlation {
            let group = groups.entry(id).or_insert_with(|| GroupBuilder {
                first_sequence: sequence,
                reported_intents: Receipts::new(),
                evidence: empty_evidence(id),
                tool_names: FirstAppearance::new(),
            });
            if let Some(name) = tool_name {
                group.tool_names.record(name, sequence);
            }
            file_evidence(group, kind, facets.v1_outcome, sequence);
        }

        if let Some(subject) = subject_agent {
            let entry = subagents
                .entry(subject.agent_id)
                .or_insert_with(|| SubagentBuilder {
                    first_sequence: sequence,
                    started: Receipts::new(),
                    stopped: Receipts::new(),
                    types: FirstAppearance::new(),
                    parents: FirstAppearance::new(),
                });
            match kind {
                EventKind::V2(V2Kind::SubagentStarted) => entry.started.push(sequence),
                EventKind::V2(V2Kind::SubagentStopped) => entry.stopped.push(sequence),
                _ => {}
            }
            entry.types.record(subject.agent_type, sequence);
            match subject.supplied_parent {
                Some(parent) => {
                    entry.parents.record(parent, sequence);
                    parent_present.push(sequence);
                }
                None => parent_absent.push(sequence),
            }
        }

        ledger.push(LedgerEntry {
            sequence,
            recorded_at,
            channel: provenance.channel,
            adapter: provenance.adapter.as_str(),
            mechanism: provenance.mechanism.as_str(),
            kind,
            correlation,
            tool_name,
            current_agent,
            subject_agent,
            prompt_id,
            facets,
            record,
        });
    }

    let mut anomalies: Vec<Anomaly<'a>> = Vec::new();

    if session_started.is_empty() {
        anomalies.push(Anomaly {
            kind: AnomalyKind::MissingSessionStart,
            receipts: Receipts::new(),
            scope,
        });
    } else if session_started.len() > 1 {
        anomalies.push(Anomaly {
            kind: AnomalyKind::DuplicateSessionStart,
            receipts: session_started.clone(),
            scope,
        });
    }
    if session_ended.is_empty() {
        anomalies.push(Anomaly {
            kind: AnomalyKind::MissingSessionEnd,
            receipts: Receipts::new(),
            scope,
        });
    } else if session_ended.len() > 1 {
        anomalies.push(Anomaly {
            kind: AnomalyKind::DuplicateSessionEnd,
            receipts: session_ended.clone(),
            scope,
        });
    }

    let mut tool_groups: Vec<ToolGroup<'a>> = Vec::with_capacity(groups.len());
    for (id, builder) in groups {
        let opening = builder.evidence.opening().clone();
        let outcomes = builder.evidence.outcome_receipts();
        let classes = builder.evidence.outcome_classes();

        let shape = if opening.is_empty() && outcomes.is_empty() {
            GroupShape::ReportedIntentOnly
        } else if opening.len() > 1 || outcomes.len() > 1 || classes > 1 {
            GroupShape::Ambiguous
        } else if outcomes.is_empty() {
            GroupShape::OpeningWithoutOutcome
        } else if opening.is_empty() {
            GroupShape::OutcomeWithoutOpening
        } else {
            GroupShape::PairedLifecycle
        };

        // Only the unambiguous one-to-one case gets an interval, and even then
        // it is two positions in the append chain.
        let paired_interval = match (shape, opening.first(), builder.evidence.sole_outcome()) {
            (GroupShape::PairedLifecycle, Some(opening), Some(outcome)) => {
                Some(SequenceInterval { opening, outcome })
            }
            _ => None,
        };

        if opening.len() > 1 {
            anomalies.push(Anomaly {
                kind: AnomalyKind::DuplicateOpenings { id },
                receipts: opening.clone(),
                scope,
            });
        }
        if outcomes.len() > 1 {
            anomalies.push(Anomaly {
                kind: AnomalyKind::DuplicateOutcomes { id },
                receipts: outcomes.clone(),
                scope,
            });
        }
        if classes > 1 {
            anomalies.push(Anomaly {
                kind: AnomalyKind::ConflictingOutcomes { id },
                receipts: outcomes.clone(),
                scope,
            });
        }
        // A missing half is reported on its own terms, independently of the
        // cardinality classification. Two requests and no outcome is both a
        // duplicate and a missing half, and reporting only the first would hide
        // the second behind the word "ambiguous".
        if outcomes.is_empty() && !opening.is_empty() {
            anomalies.push(Anomaly {
                kind: AnomalyKind::OpeningWithoutOutcome { id },
                receipts: opening.clone(),
                scope,
            });
        }
        if opening.is_empty() && !outcomes.is_empty() {
            anomalies.push(Anomaly {
                kind: AnomalyKind::OutcomeWithoutOpening { id },
                receipts: outcomes.clone(),
                scope,
            });
        }
        if opening.is_empty() && outcomes.is_empty() {
            anomalies.push(Anomaly {
                kind: AnomalyKind::ReportedIntentWithoutObservedEvidence { id },
                receipts: builder.reported_intents.clone(),
                scope,
            });
        }

        let delivered_tool_names: Vec<DeliveredValue<&'a str>> = builder
            .tool_names
            .into_entries()
            .into_iter()
            .map(|(value, receipts)| DeliveredValue { value, receipts })
            .collect();
        if delivered_tool_names.len() > 1 {
            let mut receipts: Vec<Sequence> = delivered_tool_names
                .iter()
                .flat_map(|delivered| delivered.receipts.sequences().iter().copied())
                .collect();
            receipts.sort_unstable();
            anomalies.push(Anomaly {
                kind: AnomalyKind::DivergentToolNames { id },
                receipts: Receipts(receipts),
                scope,
            });
        }

        tool_groups.push(ToolGroup {
            id,
            first_sequence: builder.first_sequence,
            reported_intents: count(builder.reported_intents),
            evidence: builder.evidence,
            shape,
            paired_interval,
            delivered_tool_names,
            scope,
        });
    }
    tool_groups.sort_by_key(|group| group.first_sequence);

    let mut subagent_lifecycles: Vec<SubagentLifecycle<'a>> = Vec::with_capacity(subagents.len());
    for (agent_id, builder) in subagents {
        if builder.started.is_empty() {
            anomalies.push(Anomaly {
                kind: AnomalyKind::SubagentStopWithoutStart { agent_id },
                receipts: builder.stopped.clone(),
                scope,
            });
        }
        if builder.stopped.is_empty() {
            anomalies.push(Anomaly {
                kind: AnomalyKind::SubagentStartWithoutStop { agent_id },
                receipts: builder.started.clone(),
                scope,
            });
        }
        let delivered_types: Vec<DeliveredValue<Option<&'a str>>> = builder
            .types
            .into_entries()
            .into_iter()
            .map(|(value, receipts)| DeliveredValue { value, receipts })
            .collect();
        let supplied_parents: Vec<DeliveredValue<SuppliedParent<'a>>> = builder
            .parents
            .into_entries()
            .into_iter()
            .map(|(value, receipts)| DeliveredValue { value, receipts })
            .collect();
        subagent_lifecycles.push(SubagentLifecycle {
            agent_id,
            first_sequence: builder.first_sequence,
            started: count(builder.started),
            stopped: count(builder.stopped),
            delivered_types,
            supplied_parents,
        });
    }
    subagent_lifecycles.sort_by_key(|lifecycle| lifecycle.first_sequence);

    // Already in first-appearance order, which is canonical record order.
    let supplied_agents: Vec<Tally<&'a str>> = current_agents
        .into_entries()
        .into_iter()
        .map(|(value, receipts)| Tally {
            value,
            records: count(receipts),
        })
        .collect();
    for (agent_id, types) in agent_types {
        let entries = types.into_entries();
        if entries.len() > 1 {
            let mut receipts: Vec<Sequence> = entries
                .iter()
                .flat_map(|(_, receipts)| receipts.sequences().iter().copied())
                .collect();
            receipts.sort_unstable();
            anomalies.push(Anomaly {
                kind: AnomalyKind::DivergentAgentTypes { agent_id },
                receipts: Receipts(receipts),
                scope,
            });
        }
    }

    // Anomalies are ordered by their earliest supporting record. An anomaly that
    // *is* an absence has no supporting record, so it sorts last, behind
    // everything the recording actually contains.
    anomalies.sort_by_key(|anomaly| {
        (
            anomaly.receipts.first().unwrap_or(Sequence::MAX),
            anomaly_order(&anomaly.kind),
        )
    });

    let by_event_kind = match replay.schema_version {
        Some(1) => vocabulary_tallies(
            V1Kind::ALL.iter().map(|kind| EventKind::V1(*kind)),
            &by_kind,
            scope,
        ),
        Some(2) => vocabulary_tallies(
            V2Kind::ALL.iter().map(|kind| EventKind::V2(*kind)),
            &by_kind,
            scope,
        ),
        // No complete record, so no vocabulary was established. Enumerating one
        // would be choosing a schema the recording never declared.
        _ => Vec::new(),
    };

    let aggregates = Aggregates {
        scope,
        by_channel: CHANNELS
            .iter()
            .enumerate()
            .map(|(index, channel)| Tally {
                value: *channel,
                records: count(by_channel[index].clone()),
            })
            .collect(),
        by_event_kind,
        by_adapter: by_adapter
            .into_entries()
            .into_iter()
            .map(|(value, receipts)| Tally {
                value,
                records: count(receipts),
            })
            .collect(),
        by_mechanism: by_mechanism
            .into_entries()
            .into_iter()
            .map(|(value, receipts)| Tally {
                value,
                records: count(receipts),
            })
            .collect(),
    };

    let coverage = vec![
        FieldCoverage {
            field: CoveredField::V2DurationMs,
            population: count(duration_population),
            present: count(duration_present),
            absent: count(duration_absent),
        },
        FieldCoverage {
            field: CoveredField::V2Interrupted,
            population: count(interrupted_population),
            present: count(interrupted_present),
            absent: count(interrupted_absent),
        },
        FieldCoverage {
            field: CoveredField::V2SuppliedParentAgent,
            population: count(parent_population),
            present: count(parent_present),
            absent: count(parent_absent),
        },
        FieldCoverage {
            field: CoveredField::V2PromptId,
            population: count(prompt_population),
            present: count(prompt_present),
            absent: count(prompt_absent),
        },
    ];

    let timestamps = match (earliest, latest) {
        (Some(earliest), Some(latest)) => Some(TimestampExtrema {
            earliest,
            latest,
            non_monotonic: count(non_monotonic),
        }),
        _ => None,
    };

    Inspection {
        schema_version: replay.schema_version,
        scope,
        session_id: records.first().map(AnyRecord::session_id),
        records,
        ledger,
        session_boundaries: SessionBoundaries {
            started: count(session_started),
            ended: count(session_ended),
        },
        tool_groups,
        subagents: subagent_lifecycles,
        current_agents: CurrentAgentAggregate {
            supplied: supplied_agents,
            not_supplied: count(agent_not_supplied),
            not_representable: count(agent_not_representable),
        },
        aggregates,
        coverage,
        timestamps,
        anomalies,
    }
}

/// Tally a schema's whole vocabulary, so a kind the recording contains none of
/// still appears, with a count of zero and the scope that zero was found in.
fn vocabulary_tallies(
    vocabulary: impl Iterator<Item = EventKind>,
    seen: &FirstAppearance<EventKind>,
    scope: ExaminedScope,
) -> Vec<Tally<EventKind>> {
    vocabulary
        .map(|kind| {
            let records = seen
                .entries
                .iter()
                .find(|(existing, _)| *existing == kind)
                .map(|(_, receipts)| receipts.clone())
                .unwrap_or_default();
            Tally {
                value: kind,
                records: RecordCount { records, scope },
            }
        })
        .collect()
}

/// Stable ordering for anomalies that share an earliest receipt, or have none.
fn anomaly_order(kind: &AnomalyKind<'_>) -> u8 {
    match kind {
        AnomalyKind::MissingSessionStart => 0,
        AnomalyKind::MissingSessionEnd => 1,
        AnomalyKind::DuplicateSessionStart => 2,
        AnomalyKind::DuplicateSessionEnd => 3,
        AnomalyKind::DuplicateOpenings { .. } => 4,
        AnomalyKind::DuplicateOutcomes { .. } => 5,
        AnomalyKind::ConflictingOutcomes { .. } => 6,
        AnomalyKind::OpeningWithoutOutcome { .. } => 7,
        AnomalyKind::OutcomeWithoutOpening { .. } => 8,
        AnomalyKind::ReportedIntentWithoutObservedEvidence { .. } => 9,
        AnomalyKind::DivergentToolNames { .. } => 10,
        AnomalyKind::SubagentStopWithoutStart { .. } => 11,
        AnomalyKind::SubagentStartWithoutStop { .. } => 12,
        AnomalyKind::DivergentAgentTypes { .. } => 13,
    }
}

fn channel_index(channel: Channel) -> usize {
    match channel {
        Channel::Reported => 0,
        Channel::Observed => 1,
        Channel::Recorder => 2,
    }
}

/// An empty evidence set in the vocabulary of the id's own schema.
fn empty_evidence(id: CorrelationId<'_>) -> ToolEvidence {
    match id {
        CorrelationId::V1ToolCallId(_) => ToolEvidence::V1 {
            started: Receipts::new(),
            finished_succeeded: Receipts::new(),
            finished_failed: Receipts::new(),
        },
        CorrelationId::V2ToolUseId(_) => ToolEvidence::V2 {
            requested: Receipts::new(),
            succeeded: Receipts::new(),
            failed: Receipts::new(),
            denied: Receipts::new(),
        },
    }
}

/// File one record into its group's evidence, in its own schema's vocabulary.
fn file_evidence(
    group: &mut GroupBuilder<'_>,
    kind: EventKind,
    v1_outcome: Option<v1::ToolOutcome>,
    sequence: Sequence,
) {
    match (&mut group.evidence, kind) {
        (ToolEvidence::V1 { started, .. }, EventKind::V1(V1Kind::ObservedToolStarted)) => {
            started.push(sequence);
        }
        (
            ToolEvidence::V1 {
                finished_succeeded,
                finished_failed,
                ..
            },
            EventKind::V1(V1Kind::ObservedToolFinished),
        ) => match v1_outcome {
            Some(v1::ToolOutcome::Succeeded) => finished_succeeded.push(sequence),
            Some(v1::ToolOutcome::Failed) => finished_failed.push(sequence),
            None => {}
        },
        (ToolEvidence::V2 { requested, .. }, EventKind::V2(V2Kind::ToolRequested)) => {
            requested.push(sequence);
        }
        (ToolEvidence::V2 { succeeded, .. }, EventKind::V2(V2Kind::ToolSucceeded)) => {
            succeeded.push(sequence);
        }
        (ToolEvidence::V2 { failed, .. }, EventKind::V2(V2Kind::ToolFailed)) => {
            failed.push(sequence);
        }
        (ToolEvidence::V2 { denied, .. }, EventKind::V2(V2Kind::ToolDenied)) => {
            denied.push(sequence);
        }
        (_, EventKind::V1(V1Kind::ReportedIntent) | EventKind::V2(V2Kind::ReportedIntent)) => {
            group.reported_intents.push(sequence);
        }
        _ => {}
    }
}

/// What one record contributes, read once so the scan does not match on the
/// event twice.
struct V1Parts<'a> {
    kind: V1Kind,
    correlation: Option<CorrelationId<'a>>,
    tool_name: Option<&'a str>,
    facets: PayloadFacets<'a>,
}

struct V2Parts<'a> {
    kind: V2Kind,
    correlation: Option<CorrelationId<'a>>,
    tool_name: Option<&'a str>,
    subject: Option<SubjectAgent<'a>>,
    facets: PayloadFacets<'a>,
}

const EMPTY_FACETS: PayloadFacets<'static> = PayloadFacets {
    has_requested_input: false,
    has_effective_input: false,
    has_response: false,
    error: None,
    duration_ms: None,
    interrupted: None,
    v1_outcome: None,
    session_source: None,
    session_reason: None,
    reported_text: None,
};

fn describe_v1(event: &v1::Event) -> V1Parts<'_> {
    match event {
        v1::Event::SessionStarted => V1Parts {
            kind: V1Kind::SessionStarted,
            correlation: None,
            tool_name: None,
            facets: EMPTY_FACETS,
        },
        v1::Event::SessionEnded => V1Parts {
            kind: V1Kind::SessionEnded,
            correlation: None,
            tool_name: None,
            facets: EMPTY_FACETS,
        },
        v1::Event::ReportedIntent(intent) => V1Parts {
            kind: V1Kind::ReportedIntent,
            correlation: intent
                .tool_call_id
                .as_deref()
                .map(CorrelationId::V1ToolCallId),
            tool_name: None,
            facets: PayloadFacets {
                reported_text: Some(intent.text.as_str()),
                ..EMPTY_FACETS
            },
        },
        v1::Event::ObservedToolStarted(started) => V1Parts {
            kind: V1Kind::ObservedToolStarted,
            correlation: Some(CorrelationId::V1ToolCallId(started.tool_call_id.as_str())),
            tool_name: Some(started.tool_name.as_str()),
            facets: PayloadFacets {
                has_requested_input: !started.arguments.is_null(),
                ..EMPTY_FACETS
            },
        },
        v1::Event::ObservedToolFinished(finished) => V1Parts {
            kind: V1Kind::ObservedToolFinished,
            correlation: Some(CorrelationId::V1ToolCallId(finished.tool_call_id.as_str())),
            tool_name: None,
            facets: PayloadFacets {
                has_response: !finished.result.is_null(),
                v1_outcome: Some(finished.outcome),
                ..EMPTY_FACETS
            },
        },
    }
}

fn describe_v2(event: &v2::Event) -> V2Parts<'_> {
    match event {
        v2::Event::SessionStarted(started) => V2Parts {
            kind: V2Kind::SessionStarted,
            correlation: None,
            tool_name: None,
            subject: None,
            facets: PayloadFacets {
                session_source: started.source.as_deref(),
                ..EMPTY_FACETS
            },
        },
        v2::Event::SessionEnded(ended) => V2Parts {
            kind: V2Kind::SessionEnded,
            correlation: None,
            tool_name: None,
            subject: None,
            facets: PayloadFacets {
                session_reason: ended.reason.as_deref(),
                ..EMPTY_FACETS
            },
        },
        v2::Event::ReportedIntent(intent) => V2Parts {
            kind: V2Kind::ReportedIntent,
            correlation: intent
                .tool_use_id
                .as_deref()
                .map(CorrelationId::V2ToolUseId),
            tool_name: None,
            subject: None,
            facets: PayloadFacets {
                reported_text: Some(intent.text.as_str()),
                ..EMPTY_FACETS
            },
        },
        v2::Event::ToolRequested(requested) => V2Parts {
            kind: V2Kind::ToolRequested,
            correlation: Some(CorrelationId::V2ToolUseId(requested.tool_use_id.as_str())),
            tool_name: Some(requested.tool_name.as_str()),
            subject: None,
            facets: PayloadFacets {
                has_requested_input: !requested.requested_input.is_null(),
                ..EMPTY_FACETS
            },
        },
        v2::Event::ToolSucceeded(succeeded) => V2Parts {
            kind: V2Kind::ToolSucceeded,
            correlation: Some(CorrelationId::V2ToolUseId(succeeded.tool_use_id.as_str())),
            tool_name: Some(succeeded.tool_name.as_str()),
            subject: None,
            facets: PayloadFacets {
                has_effective_input: !succeeded.effective_input.is_null(),
                has_response: !succeeded.response.is_null(),
                duration_ms: succeeded.duration_ms,
                ..EMPTY_FACETS
            },
        },
        v2::Event::ToolFailed(failed) => V2Parts {
            kind: V2Kind::ToolFailed,
            correlation: Some(CorrelationId::V2ToolUseId(failed.tool_use_id.as_str())),
            tool_name: Some(failed.tool_name.as_str()),
            subject: None,
            facets: PayloadFacets {
                has_effective_input: !failed.effective_input.is_null(),
                error: Some(failed.error.as_str()),
                duration_ms: failed.duration_ms,
                interrupted: failed.interrupted,
                ..EMPTY_FACETS
            },
        },
        v2::Event::ToolDenied(denied) => V2Parts {
            kind: V2Kind::ToolDenied,
            correlation: Some(CorrelationId::V2ToolUseId(denied.tool_use_id.as_str())),
            tool_name: Some(denied.tool_name.as_str()),
            subject: None,
            facets: PayloadFacets {
                has_requested_input: !denied.requested_input.is_null(),
                ..EMPTY_FACETS
            },
        },
        v2::Event::SubagentStarted(started) => V2Parts {
            kind: V2Kind::SubagentStarted,
            correlation: None,
            tool_name: None,
            subject: Some(SubjectAgent {
                agent_id: started.agent_id.as_str(),
                agent_type: started.agent_type.as_deref(),
                supplied_parent: supplied_parent(
                    started.parent_agent_id.as_deref(),
                    started.parent_agent_type.as_deref(),
                ),
            }),
            facets: EMPTY_FACETS,
        },
        v2::Event::SubagentStopped(stopped) => V2Parts {
            kind: V2Kind::SubagentStopped,
            correlation: None,
            tool_name: None,
            subject: Some(SubjectAgent {
                agent_id: stopped.agent_id.as_str(),
                agent_type: stopped.agent_type.as_deref(),
                supplied_parent: supplied_parent(
                    stopped.parent_agent_id.as_deref(),
                    stopped.parent_agent_type.as_deref(),
                ),
            }),
            facets: EMPTY_FACETS,
        },
    }
}

/// A parent identity exists only when the event delivered at least one of its
/// two fields. Nothing else creates one.
fn supplied_parent<'a>(
    agent_id: Option<&'a str>,
    agent_type: Option<&'a str>,
) -> Option<SuppliedParent<'a>> {
    if agent_id.is_none() && agent_type.is_none() {
        None
    } else {
        Some(SuppliedParent {
            agent_id,
            agent_type,
        })
    }
}
