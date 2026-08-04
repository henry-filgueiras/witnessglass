//! A regular, evenly spaced, multivariate behavioral signal over one recording.
//!
//! **Disposable.** See [`crate::experiment`].
//!
//! # The shape of the problem
//!
//! Numerical methods want a matrix: `T` rows evenly spaced in time, `D` columns,
//! no gaps, no irregular sampling, normalized. A WitnessGlass recording is the
//! opposite of that. It is a set of irregular events on an append chain whose
//! canonical order is a counter, not a clock, and whose only clock is
//! deliberately declared to be descriptive metadata.
//!
//! Closing that gap costs two things. Both are paid explicitly here rather than
//! quietly.
//!
//! # Cost one: the time axis is `recorded_at`, which is not the order
//!
//! `sequence` is the canonical order and is not a duration. `recorded_at` is the
//! only quantity in a record with a unit of time, and `CLAUDE.md`, decision:3,
//! [`crate::replay`], and [`crate::inspection`] all say the same thing about it:
//! it says when the recorder wrote a record, it establishes no order, no
//! duration, no overlap, and no causality.
//!
//! A bucketed series has to have a clock, so this one uses `recorded_at`, and
//! the consequences are carried rather than hidden:
//!
//! * The axis is named in [`TimeAxis`] and in every rendering of it, so no reader
//!   can mistake it for the append chain.
//! * **Nothing is reordered.** Each record is assigned to the bucket its own
//!   timestamp falls in. Records are visited in canonical order and stay in it.
//! * A recording whose clock moved backwards produces buckets that disagree with
//!   its append chain. That is a real property of that recording, so
//!   [`TimeAxis::non_monotonic`] carries [`crate::inspection`]'s count of it,
//!   with receipts, straight through. It is not repaired.
//! * Sub-millisecond resolution survives: bucket assignment is computed in
//!   nanoseconds, so two records in the same millisecond are not forced together
//!   by the arithmetic that places them.
//!
//! A signal built on this axis is a signal about *when records were written*.
//! For a cooperative hook adapter that is close to when the events happened and
//! is not the same claim, and no function here makes the stronger one.
//!
//! # Cost two: most of the obvious dimensions are not licensed
//!
//! Every dimension is one of six shapes, and each is a count or a measurement of
//! something literally in a record:
//!
//! | Dimension | What licenses it |
//! |---|---|
//! | [`Dimension::Records`] | one row of the ledger is one raw record |
//! | [`Dimension::Channel`] | `provenance.channel`, a raw field with three values |
//! | [`Dimension::EventKind`] | the record's own `kind` tag, schema-tagged by [`crate::inspection`] so v1 and v2 never merge |
//! | [`Dimension::DeliveredToolName`] | `tool_name` **verbatim**, as the byte string the integration delivered |
//! | [`Dimension::DistinctCorrelationIds`] | distinct delivered `tool_use_id`/`tool_call_id` values, the identifiers decision:6 licenses |
//! | [`Dimension::RecordedResponseJsonBytes`] | the serialized size of the response value this recording holds |
//!
//! The dimension *set* is discovered from the recording rather than declared in
//! advance, which means two recordings generally have different signal shapes.
//! Comparing two recordings numerically would need an alignment step that does
//! not exist. That is a limitation, not an oversight.
//!
//! ## What was considered and refused
//!
//! These were on the table for this experiment and are deliberately absent. Each
//! is followed by the evidence it would have needed and does not have.
//!
//! * **"Shell activity", "read activity", "search activity", "edit activity".**
//!   Would require a mapping from a delivered tool-name string to a semantic
//!   category. No decision in this repository establishes one, and the strings
//!   are supplied by the integration rather than by this project. `Bash` is a
//!   name; "the agent ran a shell command" is an interpretation of that name.
//!   A per-tool-name dimension is a partition by a delivered value, which is a
//!   different and much weaker thing, and it is what this module builds. A
//!   reader who *knows* the integration may group columns afterwards; the
//!   substrate will not do it for them.
//! * **"Compiler or test activity".** The same problem with a sharper edge: it
//!   would require classifying the *contents* of a command string. `CLAUDE.md`
//!   §2 forbids exactly this promotion, and the task brief names it.
//! * **"Unique files touched".** Would require reading a path out of
//!   `requested_input`/`effective_input`, which every schema version stores
//!   uninterpreted on purpose, and would then present a tool-derived list as an
//!   account of what changed on disk. decision:5's third condition and
//!   `CLAUDE.md` §6's "filesystem effects the evidence does not establish" both
//!   name this one. First contact already measured the counterexample: files
//!   were changed by a shell command with no mutation event anywhere.
//! * **"Output volume".** Not directly observed. What is observed is the size of
//!   the response value *this recording holds*, after JSON normalization, which
//!   is a fact about the recording rather than about what a tool emitted.
//!   [`Dimension::RecordedResponseJsonBytes`] is that narrower quantity and is
//!   named for it.
//! * **`duration_ms` as a dimension.** Supplied on some completions and absent on
//!   others, and absent does not mean zero. Putting it in a matrix would require
//!   choosing a filler for the absences, which is the one thing decision:5 says
//!   is most likely to be broken by accident. [`crate::inspection`] already
//!   reports its coverage honestly; a signal column cannot.
//! * **Anything segmented by `prompt_id`.** dragon:3 is open.
//!
//! # Cost three, which turns out to be free: empty buckets
//!
//! A bucket with no records gets zeros, and zero is the honest value: the count
//! of records observed in that interval really is zero. Every count in this
//! module reads "records observed in this bucket within this recording", never
//! "nothing happened then", and [`BehavioralSignal::scope`] carries what was
//! examined so an absence can be read at the strength the evidence supports —
//! the valid prefix of a truncated recording is a different scope from a
//! complete one.
//!
//! # Normalization
//!
//! One policy: per-dimension z-score against the population mean and standard
//! deviation, with an explicit rule for a constant dimension. See
//! [`Normalized`] for the policy, the alternative that was measured and
//! rejected, and why.
//!
//! Normalization is derived and additive. [`BehavioralSignal::normalize`] takes
//! `&self` and returns a new value; the unnormalized counts are never touched,
//! and a caller always holds both.

use std::collections::BTreeSet;

use serde::Serialize;

use crate::inspection::{
    CorrelationId, EventKind, ExaminedScope, Inspection, RecordCount, Sequence,
};
use crate::record::{AnyRecord, Channel, v1, v2};

/// Default bucket width, in milliseconds.
///
/// 500 ms resolves the sub-second structure a hook adapter produces around one
/// tool call: the gap between a request record and its outcome record is tens to
/// hundreds of milliseconds, and a wider bucket collapses the two into one
/// number.
///
/// It is a bad width for looking at a whole session, and that is measured rather
/// than suspected. A real 234-record session spanning 17.5 minutes produces 2108
/// buckets at this width and has a record in 119 of them — 94% empty. The same
/// session at 5 s is 63% empty and at 30 s is 22% empty.
///
/// There is therefore no single correct width, only a width per question, which
/// is why this is a parameter and why the number here is a default rather than a
/// decision.
pub const DEFAULT_BUCKET_MS: u64 = 500;

/// Raw provenance channels, in a fixed order, so a signal's columns do not
/// depend on which channels a recording happened to contain.
const CHANNELS: [Channel; 3] = [Channel::Reported, Channel::Observed, Channel::Recorder];

/// A bucket width in milliseconds, known to be non-zero.
///
/// A newtype rather than a bare `u64` because zero is the one value that turns
/// the whole projection into a division by zero, and rejecting it once at the
/// boundary is cheaper than checking for it everywhere afterwards.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct BucketWidth(u64);

impl BucketWidth {
    /// A width in milliseconds. `None` for zero, which is not a width.
    pub fn from_ms(ms: u64) -> Option<Self> {
        (ms > 0).then_some(Self(ms))
    }

    /// The width in milliseconds.
    pub fn ms(self) -> u64 {
        self.0
    }

    /// The width in nanoseconds, which is what bucket assignment divides by.
    fn ns(self) -> i128 {
        i128::from(self.0) * 1_000_000
    }
}

impl Default for BucketWidth {
    fn default() -> Self {
        Self(DEFAULT_BUCKET_MS)
    }
}

/// One column of the signal.
///
/// Every variant names something literally present in a record. There is no
/// variant for a category, a role, an inferred effect, or a classification of a
/// delivered string, and the module header lists what that rules out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Dimension<'a> {
    /// Raw records in the bucket, whatever their channel or kind.
    Records,
    /// Records in the bucket whose raw provenance channel is this one. This is
    /// the reported/observed distinction surviving into the signal as two
    /// columns that are never summed together.
    Channel(Channel),
    /// Records in the bucket of this schema-tagged event kind. v1 and v2 kinds
    /// are distinct columns even where they share a name.
    EventKind(EventKind),
    /// Records in the bucket that delivered exactly this tool-name string.
    ///
    /// The string is the one the integration delivered, compared byte for byte.
    /// It is not normalized, lower-cased, stemmed, or grouped with anything.
    DeliveredToolName(&'a str),
    /// Distinct delivered correlation ids appearing on records in the bucket.
    ///
    /// A count of ids present, not of calls: two records of one call in one
    /// bucket contribute one. It says nothing about whether those calls started,
    /// finished, overlapped, or ran at all.
    DistinctCorrelationIds,
    /// Serialized bytes of the response values recorded in this bucket.
    ///
    /// A property of the recording, not of the tool. The schema stores a response
    /// uninterpreted but JSON-normalized, so this is the size of what was kept,
    /// which is not the size of what the tool emitted. Zero in a bucket means no
    /// record in it carried a response, which is not the same as a tool having
    /// produced no output.
    RecordedResponseJsonBytes,
}

impl Dimension<'_> {
    /// A stable label for output. Not an identifier and not parsed by anything.
    pub fn label(&self) -> String {
        match self {
            Dimension::Records => "records".to_owned(),
            Dimension::Channel(channel) => format!("channel:{}", channel.as_str()),
            Dimension::EventKind(kind) => {
                format!("kind:v{}:{}", kind.schema_version(), kind.as_str())
            }
            Dimension::DeliveredToolName(name) => format!("tool_name:{name}"),
            Dimension::DistinctCorrelationIds => "distinct_correlation_ids".to_owned(),
            Dimension::RecordedResponseJsonBytes => "recorded_response_json_bytes".to_owned(),
        }
    }
}

/// The time axis a signal was bucketed on, and what it costs.
///
/// Every field here is about `recorded_at`, which is descriptive metadata. The
/// type exists so that a caller holding a matrix of numbers also holds the
/// caveats attached to the axis those numbers are indexed by.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TimeAxis {
    /// Earliest `recorded_at` in the examined scope. Bucket 0 starts here, so
    /// bucket boundaries are aligned to the first record rather than to the
    /// epoch, and bucket 0 has no unobserved prefix.
    pub origin: jiff::Timestamp,
    /// The record supplying [`TimeAxis::origin`]. Ties resolve to the earliest
    /// such record in canonical order — a statement about which record is cited,
    /// not about which came first.
    pub origin_sequence: Sequence,
    /// Latest `recorded_at` in the examined scope.
    pub latest: jiff::Timestamp,
    /// The record supplying [`TimeAxis::latest`].
    pub latest_sequence: Sequence,
    /// Milliseconds from [`TimeAxis::origin`] to [`TimeAxis::latest`].
    ///
    /// The distance between two recorded timestamps. Not a session duration, not
    /// an execution time, and not a claim that anything ran for this long.
    pub span_ms: u64,
    /// Offset of the last record within the final bucket, in milliseconds.
    ///
    /// The final bucket is emitted at full width like every other bucket. This
    /// says how far into it the recording's evidence stops: after this offset the
    /// bucket contains no observation rather than an observed absence. Nothing is
    /// scaled, weighted, or extrapolated to compensate, and the value is zero
    /// when the last record lands exactly on a boundary.
    pub final_bucket_observed_ms: u64,
    /// Records whose timestamp precedes that of the record before them in append
    /// order, carried through from [`crate::inspection`] with its receipts.
    ///
    /// Non-zero means this recording's clock and its append chain disagree, so
    /// the bucket a record lands in and its position in canonical order are not
    /// consistent orderings of the same recording. The signal is still computed;
    /// the disagreement is reported rather than resolved.
    pub non_monotonic: RecordCount,
}

/// One evenly spaced bucket: one row of the matrix.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BehavioralSample {
    /// Index of this bucket, from zero.
    pub bucket: usize,
    /// Offset of this bucket's start from [`TimeAxis::origin`], in milliseconds.
    /// Always `bucket * bucket_ms`; buckets are contiguous and none is skipped.
    pub offset_ms: u64,
    /// Unnormalized values, parallel to [`BehavioralSignal::dimensions`].
    ///
    /// Counts are held as `f64` because that is what a downstream numerical
    /// method wants, and every value here is a small non-negative integer or a
    /// byte total, all of which are exact in `f64`.
    pub values: Vec<f64>,
    /// Sequence numbers of the records that produced this bucket's values,
    /// ascending.
    ///
    /// Receipts, in the sense decision:6 requires of a derived claim: every
    /// number in `values` can be traced back to these records and recomputed
    /// from them. An empty list is not evidence of absence on its own and must
    /// be read against [`BehavioralSignal::scope`].
    pub records: Vec<Sequence>,
}

/// A regular multivariate signal derived from one recording.
///
/// Borrows the projection it came from, which borrows the replay, which borrows
/// nothing it can write to. Nothing here owns raw evidence, so nothing here can
/// rewrite it, and dropping the whole signal loses nothing the raw stream cannot
/// regenerate.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BehavioralSignal<'a> {
    /// Bucket width this signal was computed at.
    pub bucket_ms: u64,
    /// The axis, and its caveats.
    pub axis: TimeAxis,
    /// Columns, in a deterministic order: total records, then the three raw
    /// channels, then the recording's whole event-kind vocabulary, then each
    /// delivered tool name in first-appearance order, then distinct correlation
    /// ids, then recorded response bytes.
    pub dimensions: Vec<Dimension<'a>>,
    /// Rows, contiguous and evenly spaced, one per bucket from zero. Never
    /// sparse: a bucket with no records is present with zeros.
    pub samples: Vec<BehavioralSample>,
    /// The population every count here was taken over. A complete recording and
    /// the valid prefix of a truncated one are different scopes.
    pub scope: ExaminedScope,
    /// Schema version established by the recording's first record.
    pub schema_version: Option<u64>,
    /// Session the records belong to.
    pub session_id: Option<&'a str>,
}

impl<'a> BehavioralSignal<'a> {
    /// How many buckets the signal holds.
    pub fn len(&self) -> usize {
        self.samples.len()
    }

    /// Whether the signal holds no buckets. Only reachable for a recording whose
    /// examined scope holds no records, where [`project`] returns `None` instead.
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    /// Index of the dimension with this label, if the signal has one.
    pub fn dimension_index(&self, label: &str) -> Option<usize> {
        self.dimensions
            .iter()
            .position(|dimension| dimension.label() == label)
    }

    /// One dimension's unnormalized values across every bucket, in bucket order.
    pub fn column(&self, dimension: usize) -> Option<Vec<f64>> {
        (dimension < self.dimensions.len())
            .then(|| self.samples.iter().map(|s| s.values[dimension]).collect())
    }

    /// Normalize, without touching this signal.
    ///
    /// See [`Normalized`] for the policy.
    pub fn normalize(&self) -> Normalized {
        Normalized::of(self)
    }
}

/// Project an inspected recording into an evenly spaced behavioral signal.
///
/// Pure and deterministic: the same [`Inspection`] and the same width always
/// yield the same signal. It reads no file, consults no clock, and takes its
/// input by shared reference.
///
/// Returns `None` when the examined scope holds no records. That is an absence
/// rendered as an absence: with no record there is no earliest timestamp, so
/// there is no origin, so there is no axis, and a zero-row matrix over an
/// invented axis would be a fabricated one.
pub fn project<'a>(
    inspection: &'a Inspection<'a>,
    width: BucketWidth,
) -> Option<BehavioralSignal<'a>> {
    let extrema = inspection.timestamps.as_ref()?;

    let origin_ns = extrema.earliest.recorded_at.as_nanosecond();
    let latest_ns = extrema.latest.recorded_at.as_nanosecond();
    let span_ns = latest_ns - origin_ns;
    let width_ns = width.ns();

    // The last record belongs to a bucket, so the count is inclusive of it.
    // `span_ns` is non-negative because `origin` is the minimum of the same set
    // `latest` is the maximum of.
    let bucket_count = usize::try_from(span_ns / width_ns).ok()? + 1;

    let dimensions = discover_dimensions(inspection);
    let mut accumulators: Vec<Accumulator> = (0..bucket_count)
        .map(|_| Accumulator::new(dimensions.len()))
        .collect();

    // Canonical order in, canonical order out. Each record is placed by its own
    // timestamp; none is moved, dropped, or resequenced to make the axis tidy.
    for entry in &inspection.ledger {
        let offset_ns = entry.recorded_at.as_nanosecond() - origin_ns;
        let bucket = usize::try_from(offset_ns / width_ns).unwrap_or(0);
        let Some(accumulator) = accumulators.get_mut(bucket) else {
            continue;
        };

        accumulator.records.push(entry.sequence);
        if let Some(id) = entry.correlation {
            accumulator.correlations.insert(id);
        }
        let response_bytes = recorded_response_json_bytes(entry.record);

        for (index, dimension) in dimensions.iter().enumerate() {
            let contribution = match dimension {
                Dimension::Records => 1.0,
                Dimension::Channel(channel) => f64::from(u8::from(entry.channel == *channel)),
                Dimension::EventKind(kind) => f64::from(u8::from(entry.kind == *kind)),
                Dimension::DeliveredToolName(name) => {
                    f64::from(u8::from(entry.tool_name == Some(*name)))
                }
                // Filled in after the pass, from the accumulated set.
                Dimension::DistinctCorrelationIds => 0.0,
                Dimension::RecordedResponseJsonBytes => response_bytes.unwrap_or(0) as f64,
            };
            accumulator.values[index] += contribution;
        }
    }

    let distinct_index = dimensions
        .iter()
        .position(|d| matches!(d, Dimension::DistinctCorrelationIds));

    let samples = accumulators
        .into_iter()
        .enumerate()
        .map(|(bucket, mut accumulator)| {
            if let Some(index) = distinct_index {
                accumulator.values[index] = accumulator.correlations.len() as f64;
            }
            BehavioralSample {
                bucket,
                offset_ms: bucket as u64 * width.ms(),
                values: accumulator.values,
                records: accumulator.records,
            }
        })
        .collect();

    let span_ms = u64::try_from(span_ns / 1_000_000).unwrap_or(0);

    Some(BehavioralSignal {
        bucket_ms: width.ms(),
        axis: TimeAxis {
            origin: extrema.earliest.recorded_at,
            origin_sequence: extrema.earliest.sequence,
            latest: extrema.latest.recorded_at,
            latest_sequence: extrema.latest.sequence,
            span_ms,
            final_bucket_observed_ms: span_ms % width.ms(),
            non_monotonic: extrema.non_monotonic.clone(),
        },
        dimensions,
        samples,
        scope: inspection.scope,
        schema_version: inspection.schema_version,
        session_id: inspection.session_id,
    })
}

/// What one bucket accumulates during the single pass.
struct Accumulator<'a> {
    values: Vec<f64>,
    records: Vec<Sequence>,
    correlations: BTreeSet<CorrelationId<'a>>,
}

impl Accumulator<'_> {
    fn new(dimensions: usize) -> Self {
        Self {
            values: vec![0.0; dimensions],
            records: Vec::new(),
            correlations: BTreeSet::new(),
        }
    }
}

/// Build the column set from what the recording actually contains.
///
/// Deterministic: fixed families in a fixed order, and within the one
/// recording-dependent family — delivered tool names — first-appearance order in
/// the canonical record order, never hash order.
fn discover_dimensions<'a>(inspection: &'a Inspection<'a>) -> Vec<Dimension<'a>> {
    let mut dimensions = vec![Dimension::Records];

    dimensions.extend(CHANNELS.map(Dimension::Channel));

    // The whole vocabulary of the recording's schema, including kinds it
    // contains none of, so a zero column is a stated absence rather than a
    // missing one. `inspection` already supplies it in a fixed order.
    dimensions.extend(
        inspection
            .aggregates
            .by_event_kind
            .iter()
            .map(|tally| Dimension::EventKind(tally.value)),
    );

    let mut names: Vec<&str> = Vec::new();
    for entry in &inspection.ledger {
        if let Some(name) = entry.tool_name
            && !names.contains(&name)
        {
            names.push(name);
        }
    }
    dimensions.extend(names.into_iter().map(Dimension::DeliveredToolName));

    dimensions.push(Dimension::DistinctCorrelationIds);
    dimensions.push(Dimension::RecordedResponseJsonBytes);
    dimensions
}

/// Serialized size of the response value this record holds, if it holds one.
///
/// `None` means the record's kind carries no response at all, which is why it
/// contributes nothing rather than zero bytes. The value is re-serialized from
/// what was parsed, so this measures the recording's normalized JSON and not the
/// bytes the integration delivered — the schema is explicit that a payload
/// survives semantically rather than byte for byte.
fn recorded_response_json_bytes(record: &AnyRecord) -> Option<usize> {
    let value = match record {
        AnyRecord::V2(record) => match &record.event {
            v2::Event::ToolSucceeded(succeeded) => &succeeded.response,
            _ => return None,
        },
        AnyRecord::V1(record) => match &record.event {
            v1::Event::ObservedToolFinished(finished) => &finished.result,
            _ => return None,
        },
    };
    // Serializing a `serde_json::Value` cannot fail: it holds no non-string map
    // key and no non-finite number. The arm exists so that an impossible failure
    // is reported as "not measured" rather than fabricated as zero bytes.
    serde_json::to_string(value).ok().map(|text| text.len())
}

/// Descriptive statistics for one dimension, over the whole signal.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DimensionStats {
    /// Sum across every bucket.
    pub sum: f64,
    /// Arithmetic mean across every bucket, empty buckets included.
    pub mean: f64,
    /// Population standard deviation — divided by `N`, not `N - 1`. The signal
    /// is the whole finite series being described, not a sample drawn from a
    /// larger one.
    pub stddev: f64,
    /// Smallest value across every bucket.
    pub min: f64,
    /// Largest value across every bucket.
    pub max: f64,
    /// Buckets whose value is non-zero. The complement is how sparse the
    /// dimension is, and it is what decided the normalization policy.
    pub nonzero_buckets: usize,
    /// Whether every bucket holds the same value, making the standard deviation
    /// zero and a z-score undefined.
    pub constant: bool,
}

/// A normalized view of a signal, derived from it and beside it.
///
/// # Policy
///
/// Per-dimension z-score against the population mean and standard deviation:
///
/// ```text
/// z[t, j] = (x[t, j] - mean[j]) / stddev[j]
/// ```
///
/// **Constant dimensions are defined to be zero.** When `stddev[j]` is exactly
/// zero every bucket holds the same value, the z-score is `0/0`, and there is no
/// non-arbitrary answer. This module writes `0.0` for every bucket of such a
/// dimension and sets [`DimensionStats::constant`], so the choice is visible
/// rather than inferred from a column of zeros that could equally have been a
/// column of genuine zeros. It is deterministic, it never produces `NaN` or
/// infinity, and it makes the dimension inert for any downstream distance
/// measure — which is the correct behaviour for a column carrying no variation.
///
/// # Why not median and MAD
///
/// Median/MAD is the standard robust alternative and it is the wrong choice for
/// *this* data, which is worth stating because the reverse is usually true.
///
/// These dimensions are counts of events in short buckets, so the overwhelming
/// majority of buckets are zero. The oracle fixture at 500 ms has 481 buckets
/// and every dimension is zero in more than 70% of them, most in more than 95%;
/// a real 234-record session measured at the same width was emptier still, with
/// records in 119 of 2108 buckets. When more than half of a series is zero the
/// median is zero, and the absolute deviations from it are the series itself, so
/// the MAD is zero too. Median/MAD therefore degenerates to a division by zero
/// on essentially *every* dimension at the widths this substrate is interesting
/// at, where mean and standard deviation degenerate only on the genuinely
/// constant ones. `tests/behavioral_signal.rs` measures the sparsity rather than
/// assuming it, so a fixture that stopped being sparse would force the argument
/// to be made again instead of inherited.
///
/// The crossover is real and worth knowing: widen the buckets far enough — 30 s
/// on that same session — and the majority of buckets become occupied, at which
/// point the argument above stops applying. It is an argument about this data at
/// these widths, not a general claim about robust statistics.
///
/// Robustness against outliers is also not what this substrate wants. A burst is
/// an outlier by construction, and it is the signal, not contamination. A
/// normalization that suppresses the tail suppresses the thing the next
/// experiment is looking for.
///
/// One policy is implemented. Adding a second because it exists would be a
/// configuration surface for a question nobody has asked.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Normalized {
    /// Per-dimension statistics, parallel to the signal's dimensions.
    pub stats: Vec<DimensionStats>,
    /// Normalized values, `[bucket][dimension]`, the same shape as the signal.
    pub values: Vec<Vec<f64>>,
}

impl Normalized {
    /// Normalize a signal without modifying it.
    fn of(signal: &BehavioralSignal<'_>) -> Self {
        let buckets = signal.samples.len();
        let width = signal.dimensions.len();

        let mut stats = Vec::with_capacity(width);
        for index in 0..width {
            let mut sum = 0.0;
            let mut min = f64::INFINITY;
            let mut max = f64::NEG_INFINITY;
            let mut nonzero = 0usize;
            for sample in &signal.samples {
                let value = sample.values[index];
                sum += value;
                min = min.min(value);
                max = max.max(value);
                if value != 0.0 {
                    nonzero += 1;
                }
            }
            let mean = if buckets == 0 {
                0.0
            } else {
                sum / buckets as f64
            };
            let variance = if buckets == 0 {
                0.0
            } else {
                signal
                    .samples
                    .iter()
                    .map(|sample| {
                        let deviation = sample.values[index] - mean;
                        deviation * deviation
                    })
                    .sum::<f64>()
                    / buckets as f64
            };
            let stddev = variance.sqrt();
            stats.push(DimensionStats {
                sum,
                mean,
                stddev,
                min: if buckets == 0 { 0.0 } else { min },
                max: if buckets == 0 { 0.0 } else { max },
                nonzero_buckets: nonzero,
                constant: stddev == 0.0,
            });
        }

        let values = signal
            .samples
            .iter()
            .map(|sample| {
                sample
                    .values
                    .iter()
                    .zip(&stats)
                    .map(|(value, stat)| {
                        if stat.constant {
                            0.0
                        } else {
                            (value - stat.mean) / stat.stddev
                        }
                    })
                    .collect()
            })
            .collect();

        Self { stats, values }
    }

    /// One dimension's normalized values across every bucket, in bucket order.
    pub fn column(&self, dimension: usize) -> Option<Vec<f64>> {
        self.stats.get(dimension)?;
        Some(self.values.iter().map(|row| row[dimension]).collect())
    }
}
