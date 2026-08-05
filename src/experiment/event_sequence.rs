//! Events kept as events: an ordered sequence of marked events, and a small
//! alignment distance between short windows of them.
//!
//! **Disposable.** See [`crate::experiment`]. sprint:8, task:18. No dependency,
//! no feature gate, and nothing outside this module, `examples/event-motif.rs`,
//! and `tests/event_sequence.rs` refers to it.
//!
//! # Why this exists
//!
//! sprint:6 ran a univariate Matrix Profile over sprint:4's 500 ms raster and
//! found something sharper than the window question it set out to answer: in a
//! signal that is 78–94% empty, two windows each holding **one** non-empty
//! bucket at the same relative offset are identical after z-normalization and
//! score a distance of exactly zero, whatever surrounds them. Those pairs
//! occupied the top of every masked motif list at every window in every sparse
//! dimension of both fixtures and the real recording. The detector never matched
//! the figure the fixture was built to contain.
//!
//! That diagnosis points at the representation. This module is the smallest
//! alternative that tests it: keep the events, keep their order, keep the time
//! between them, and compare short windows directly instead of rasterizing them
//! into mostly-empty fixed-width buckets.
//!
//! # What a mark is allowed to be
//!
//! Two raw fields and nothing else:
//!
//! * the schema-tagged event kind [`crate::inspection`] assigns, so a v1 and a
//!   v2 kind that share a name are different marks;
//! * the tool-name string the integration delivered, **verbatim** — compared
//!   byte for byte, never normalized, lower-cased, stemmed, or grouped.
//!
//! If the recording knows `SyntheticReader`, a mark may know `SyntheticReader`.
//! It may not promote that to `filesystem_read`, `inspection`, or `research`.
//! `CLAUDE.md` §2 forbids that promotion and sprint:4 already refused it for the
//! sampled substrate; the refusal does not weaken because a second
//! representation would find it convenient.
//!
//! Absent from the mark, deliberately: paths, payload sizes, recorded response
//! bytes, edit deltas, correlation ids, `prompt_id`, agent identity, reported
//! text, inferred hierarchy, and `duration_ms`. Several of those are
//! mechanically derivable from raw evidence and would be legitimate facets for a
//! later round. They are excluded here so that the first question — whether
//! event identity and relative timing are enough — is answered on its own terms.
//!
//! # The time axis, with sprint:4's caveat attached
//!
//! Offsets and gaps are computed from `recorded_at`, which says when the
//! recorder wrote a record and establishes no order, duration, overlap, or
//! causality. The canonical order is `sequence`, and this module visits records
//! in it and never reorders them. A recording whose clock moved backwards
//! produces a gap this module clamps to zero and counts in
//! [`EventSequence::clamped_gaps`], beside [`crate::inspection`]'s own
//! non-monotonic count carried through with its receipts. Nothing is repaired.
//!
//! # What a window's timing is
//!
//! A window of `k` events carries `k − 1` **within-window** gaps: the first
//! event's gap points at an event outside the window and is not used. A window's
//! timing is therefore translation-invariant — it does not depend on when the
//! window starts — which is the property that lets two occurrences of the same
//! figure be compared at all.
//!
//! # What this is not
//!
//! Not a detector abstraction. There is no trait, no registry, and nothing a
//! second metric could be plugged into. Not a motif schema: nothing here is
//! serialized into a recording, and the `Serialize` implementations exist so an
//! example can print JSON. Not cross-recording: one sequence comes from one
//! recording, two recordings are never compared, and no state outlives a call.

use serde::Serialize;

use crate::inspection::{EventKind, ExaminedScope, Inspection, RecordCount, Sequence};
use crate::record::Channel;

/// Cost of aligning two events carrying different marks.
///
/// At most twice [`GAP`], so an alignment never prefers a deletion plus an
/// insertion over a substitution, and the largest event cost of aligning two
/// sequences is `max(len_a, len_b)`.
pub const SUBSTITUTION: f64 = 1.0;

/// Cost of an insertion or a deletion.
pub const GAP: f64 = 1.0;

/// Weight on the timing term of an aligned pair, relative to identity.
///
/// Half, so timing can shade a comparison and can never outvote which events
/// happened. task:18 fixes it before any fixture is run.
pub const TIMING_WEIGHT: f64 = 0.5;

/// Milliseconds added to both gaps before the ratio is taken.
///
/// Damps the sub-100 ms region where a cooperative hook adapter's own latency
/// lives. Without it, 1 ms against 10 ms would be a full-scale disagreement
/// about nothing.
pub const TIMING_FLOOR_MS: f64 = 100.0;

/// The gap ratio that scores a full timing penalty.
///
/// A fourfold difference is maximally different; anything larger is clamped.
pub const TIMING_RATIO_FULL: f64 = 4.0;

/// Which raw provenance channels a sequence retains.
///
/// A parameter rather than a policy, and task:18 makes the harder scope the
/// primary one: the planted figure in both fixtures opens with a
/// `reported_intent` record, and in the sparse fixture that mark appears nowhere
/// else at all. Recovering the figure with the reported channel present would be
/// a result about a rare marker rather than about sequence structure.
///
/// Neither scope promotes one channel into the other. Every mark keeps its own
/// kind, a reported mark is never equal to an observed one, and nothing here
/// describes a reported claim as an observed fact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChannelScope {
    /// Records whose raw provenance channel is `observed`.
    Observed,
    /// Every record, whatever its channel.
    All,
}

impl ChannelScope {
    /// Whether a channel is retained under this scope.
    pub fn retains(self, channel: Channel) -> bool {
        match self {
            ChannelScope::Observed => channel == Channel::Observed,
            ChannelScope::All => true,
        }
    }

    /// A stable label for output.
    pub fn label(self) -> &'static str {
        match self {
            ChannelScope::Observed => "observed",
            ChannelScope::All => "all",
        }
    }
}

/// The identity of one event: two raw fields, and nothing derived.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Mark<'a> {
    /// The record's own schema-tagged kind.
    pub kind: EventKind,
    /// The tool name this record delivered, exactly as delivered, or `None`
    /// where the record's kind carries no tool name.
    pub tool_name: Option<&'a str>,
}

impl Mark<'_> {
    /// A stable label for output. Not an identifier, and nothing parses it.
    pub fn label(&self) -> String {
        match self.tool_name {
            Some(name) => format!(
                "v{}:{}/{}",
                self.kind.schema_version(),
                self.kind.as_str(),
                name
            ),
            None => format!("v{}:{}", self.kind.schema_version(), self.kind.as_str()),
        }
    }
}

/// One event, marked and placed in time.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct MarkedEvent<'a> {
    /// The record this event came from, in the canonical append chain. `None`
    /// for a hand-built event, which has no receipt because it has no record —
    /// the perturbation sweep and the microtests construct such events, and a
    /// missing receipt is how they stay distinguishable from evidence.
    pub sequence: Option<Sequence>,
    /// Event identity.
    pub mark: Mark<'a>,
    /// Milliseconds from the sequence origin to this event's `recorded_at`.
    pub offset_ms: u64,
    /// Milliseconds from the previous retained event's `recorded_at`. `None` for
    /// the first event of a sequence, which has no predecessor to measure from.
    pub gap_from_previous_ms: Option<u64>,
}

impl<'a> MarkedEvent<'a> {
    /// A hand-built event, for microtests and the perturbation sweep.
    ///
    /// Carries no [`MarkedEvent::sequence`] because it stands for no record, and
    /// its `offset_ms` is left at zero: a hand-built sequence is compared by its
    /// gaps, which is all a window's timing ever uses.
    pub fn hand_built(kind: EventKind, tool_name: Option<&'a str>, gap_ms: Option<u64>) -> Self {
        Self {
            sequence: None,
            mark: Mark { kind, tool_name },
            offset_ms: 0,
            gap_from_previous_ms: gap_ms,
        }
    }
}

/// One recording's retained events, in canonical order.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EventSequence<'a> {
    /// Which channels were retained.
    pub channels: ChannelScope,
    /// The events, in canonical append order. Never reordered.
    pub events: Vec<MarkedEvent<'a>>,
    /// Earliest `recorded_at` in the examined scope, which every offset is
    /// measured from. Taken from the whole inspection rather than from the first
    /// retained event, so offsets agree with sprint:4's and sprint:6's.
    pub origin: jiff::Timestamp,
    /// Records the channel filter discarded.
    pub filtered_out: usize,
    /// Gaps that came out negative and were clamped to zero, because the clock
    /// moved backwards between two retained records. Reported, never repaired.
    pub clamped_gaps: usize,
    /// [`crate::inspection`]'s own count of non-monotonic records, with receipts.
    pub non_monotonic: RecordCount,
    /// The population every event here was drawn from.
    pub scope: ExaminedScope,
    /// Session the records belong to.
    pub session_id: Option<&'a str>,
}

impl<'a> EventSequence<'a> {
    /// How many events the sequence holds.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Whether the sequence holds no events.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Windows of `k` events available at this length.
    pub fn window_count(&self, k: usize) -> usize {
        if k == 0 || self.events.len() < k {
            0
        } else {
            self.events.len() - k + 1
        }
    }

    /// The `k` events starting at `start`, or `None` if they do not all exist.
    pub fn window(&self, start: usize, k: usize) -> Option<&[MarkedEvent<'a>]> {
        self.events.get(start..start.checked_add(k)?)
    }

    /// Where a window sits, and how degenerate it is.
    pub fn window_ref(&self, start: usize, k: usize) -> Option<WindowRef> {
        let events = self.window(start, k)?;
        let first = events.first()?;
        let last = events.last()?;
        let mut marks: Vec<Mark<'a>> = Vec::new();
        for event in events {
            if !marks.contains(&event.mark) {
                marks.push(event.mark);
            }
        }
        Some(WindowRef {
            start,
            k,
            start_ms: first.offset_ms,
            last_ms: last.offset_ms,
            distinct_marks: marks.len(),
            first_sequence: first.sequence,
            last_sequence: last.sequence,
        })
    }

    /// The first window of `k` events lying entirely inside a half-open
    /// millisecond region, measured from [`EventSequence::origin`].
    ///
    /// Regions are supplied by the caller. Nothing in this module knows what a
    /// fixture contains, and a region is never guessed from one.
    pub fn first_window_within(&self, region: (u64, u64), k: usize) -> Option<usize> {
        (0..self.window_count(k)).find(|start| {
            self.window_ref(*start, k)
                .is_some_and(|window| window.within(region))
        })
    }
}

/// Where one window sits in a sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct WindowRef {
    /// Index of the window's first event within the sequence.
    pub start: usize,
    /// Events in the window.
    pub k: usize,
    /// Offset of the first event, in milliseconds from the sequence origin.
    pub start_ms: u64,
    /// Offset of the **last** event, not of a window end: a window is a set of
    /// events, and it has no width beyond the events it holds.
    pub last_ms: u64,
    /// Distinct marks inside the window.
    ///
    /// The diagnostic that replaces sprint:6's occupancy column. A window whose
    /// events are all one mark, or which alternates two, is a degenerate figure
    /// however perfectly it repeats — matching it says as little as matching two
    /// lone impulses did. See [`WindowRef::degenerate`].
    pub distinct_marks: usize,
    /// Sequence of the window's first event, where it has one.
    pub first_sequence: Option<Sequence>,
    /// Sequence of the window's last event, where it has one.
    pub last_sequence: Option<Sequence>,
}

impl WindowRef {
    /// Milliseconds from the first event to the last. Not a duration: it is the
    /// distance between two recorder timestamps and nothing executed for it.
    pub fn extent_ms(&self) -> u64 {
        self.last_ms.saturating_sub(self.start_ms)
    }

    /// Whether every event lies inside a half-open millisecond region.
    pub fn within(&self, region: (u64, u64)) -> bool {
        self.start_ms >= region.0 && self.last_ms < region.1
    }

    /// Whether the window holds two marks or fewer.
    pub fn degenerate(&self) -> bool {
        self.distinct_marks <= 2
    }
}

/// A comparison of two windows, decomposed.
///
/// Every component is reported. task:18 requires that the three distances never
/// collapse into one number, because the round's question includes whether
/// timing helps, hurts, or does nothing — and a single scalar cannot say.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Alignment {
    /// Aligned pairs carrying the same mark.
    pub matches: usize,
    /// Aligned pairs carrying different marks.
    pub substitutions: usize,
    /// Events of the first sequence aligned against nothing.
    pub deletions: usize,
    /// Events of the second sequence aligned against nothing.
    pub insertions: usize,
    /// Aligned pairs where **both** events have a within-window predecessor, and
    /// which therefore contributed a timing term.
    pub timed_pairs: usize,
    /// `SUBSTITUTION × substitutions + GAP × (insertions + deletions)`.
    pub event_cost: f64,
    /// Sum of `TIMING_WEIGHT × t(g_a, g_b)` over timed pairs.
    pub timing_cost: f64,
    /// `event_cost / max(len_a, len_b)`, in `[0, 1]`.
    pub event_norm: f64,
    /// `timing_cost / (TIMING_WEIGHT × timed_pairs)`, in `[0, 1]`, or zero when
    /// no pair carried timing. [`Alignment::timed_pairs`] says which.
    pub timing_norm: f64,
    /// `(event_cost + timing_cost) / (L + TIMING_WEIGHT × (L − 1))` where
    /// `L = max(len_a, len_b)`, in `[0, 1]`. The ranking quantity.
    ///
    /// Its denominator is fixed by the lengths, so when every window in a scan
    /// holds the same number of events — which is what a fixed event-count
    /// ladder means — ranking by `total` is ranking by raw alignment cost, and
    /// the alignment the dynamic program minimized is the one this is read from.
    pub total: f64,
}

/// The timing term between two within-window gaps.
///
/// Bounded symmetric log-ratio, floored: `min(1, |ln((a + FLOOR)/(b + FLOOR))| /
/// ln RATIO_FULL)`. Multiplicative rather than absolute, so 1.0 s against 1.2 s
/// is a small disagreement, 1.0 s against 8.0 s is a total one, 100 s against
/// 100.2 s is nothing, and the same 200 ms between 0.1 s and 0.3 s is a lot.
pub fn timing_term(a_ms: u64, b_ms: u64) -> f64 {
    let a = a_ms as f64 + TIMING_FLOOR_MS;
    let b = b_ms as f64 + TIMING_FLOOR_MS;
    ((a / b).ln().abs() / TIMING_RATIO_FULL.ln()).min(1.0)
}

/// The within-window gap of position `index`, which the first position does not
/// have however the event was built.
fn within_window_gap(events: &[MarkedEvent<'_>], index: usize) -> Option<u64> {
    if index == 0 {
        None
    } else {
        events.get(index)?.gap_from_previous_ms
    }
}

/// Align two windows and report the decomposition.
///
/// A weighted global alignment: substitution [`SUBSTITUTION`], insertion and
/// deletion [`GAP`] each, and [`TIMING_WEIGHT`] times [`timing_term`] on every
/// aligned pair where both sides have a within-window predecessor. Deterministic
/// and symmetric in the quantities it reports, with insertions and deletions
/// exchanging roles when the arguments are exchanged.
pub fn align(a: &[MarkedEvent<'_>], b: &[MarkedEvent<'_>]) -> Alignment {
    let (la, lb) = (a.len(), b.len());

    // Cost of aligning a[i] against b[j]: identity, plus timing where both
    // positions have a within-window predecessor.
    let pair_cost = |i: usize, j: usize| -> (f64, f64) {
        let identity = if a[i].mark == b[j].mark {
            0.0
        } else {
            SUBSTITUTION
        };
        let timing = match (within_window_gap(a, i), within_window_gap(b, j)) {
            (Some(ga), Some(gb)) => TIMING_WEIGHT * timing_term(ga, gb),
            _ => 0.0,
        };
        (identity, timing)
    };

    let mut cost = vec![vec![0.0f64; lb + 1]; la + 1];
    for (i, row) in cost.iter_mut().enumerate() {
        row[0] = i as f64 * GAP;
    }
    for (j, entry) in cost[0].iter_mut().enumerate() {
        *entry = j as f64 * GAP;
    }
    for i in 1..=la {
        for j in 1..=lb {
            let (identity, timing) = pair_cost(i - 1, j - 1);
            let diagonal = cost[i - 1][j - 1] + identity + timing;
            let deletion = cost[i - 1][j] + GAP;
            let insertion = cost[i][j - 1] + GAP;
            cost[i][j] = diagonal.min(deletion).min(insertion);
        }
    }

    // Backtrack for the decomposition. The diagonal is preferred on a tie, so
    // the reported alignment is a single deterministic one rather than whichever
    // the arithmetic happened to reach first.
    let mut matches = 0usize;
    let mut substitutions = 0usize;
    let mut deletions = 0usize;
    let mut insertions = 0usize;
    let mut timed_pairs = 0usize;
    let mut timing_cost = 0.0f64;
    let (mut i, mut j) = (la, lb);
    const TOLERANCE: f64 = 1e-12;
    while i > 0 || j > 0 {
        if i > 0 && j > 0 {
            let (identity, timing) = pair_cost(i - 1, j - 1);
            if (cost[i][j] - (cost[i - 1][j - 1] + identity + timing)).abs() <= TOLERANCE {
                if identity == 0.0 {
                    matches += 1;
                } else {
                    substitutions += 1;
                }
                if within_window_gap(a, i - 1).is_some() && within_window_gap(b, j - 1).is_some() {
                    timed_pairs += 1;
                    timing_cost += timing;
                }
                i -= 1;
                j -= 1;
                continue;
            }
        }
        if i > 0 && (cost[i][j] - (cost[i - 1][j] + GAP)).abs() <= TOLERANCE {
            deletions += 1;
            i -= 1;
            continue;
        }
        // The only remaining move, and it is always available when `j > 0`.
        if j > 0 {
            insertions += 1;
            j -= 1;
            continue;
        }
        // Unreachable while the table is consistent; breaking keeps the
        // function total rather than looping if it ever is not.
        break;
    }

    let event_cost = SUBSTITUTION * substitutions as f64 + GAP * (insertions + deletions) as f64;
    let longest = la.max(lb);
    let event_norm = if longest == 0 {
        0.0
    } else {
        event_cost / longest as f64
    };
    let timing_norm = if timed_pairs == 0 {
        0.0
    } else {
        timing_cost / (TIMING_WEIGHT * timed_pairs as f64)
    };
    let denominator = longest as f64 + TIMING_WEIGHT * (longest.saturating_sub(1)) as f64;
    let total = if denominator == 0.0 {
        0.0
    } else {
        (event_cost + timing_cost) / denominator
    };

    Alignment {
        matches,
        substitutions,
        deletions,
        insertions,
        timed_pairs,
        event_cost,
        timing_cost,
        event_norm,
        timing_norm,
        total,
    }
}

/// Two windows and the alignment between them.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Comparison {
    /// The first window.
    pub a: WindowRef,
    /// The second window.
    pub b: WindowRef,
    /// The decomposition.
    pub alignment: Alignment,
}

/// Whether two windows of `k` events share no event.
///
/// The trivial-match exclusion policy, stated once. Stricter than the
/// `ceil(m/4)` exclusion zone sprint:6 inherited from `motif-rs`, and stricter
/// on purpose: a window overlapping itself by one event is not a second
/// occurrence of anything.
pub fn disjoint(start_a: usize, start_b: usize, k: usize) -> bool {
    start_a.abs_diff(start_b) >= k
}

/// Every window sharing no event with the one at `query`, best first.
///
/// Ties break on the neighbour's start index, so the order is total and does not
/// depend on iteration accidents.
pub fn neighbours(
    sequence: &EventSequence<'_>,
    query: usize,
    k: usize,
    top: usize,
) -> Vec<Comparison> {
    let Some(query_ref) = sequence.window_ref(query, k) else {
        return Vec::new();
    };
    let Some(query_events) = sequence.window(query, k) else {
        return Vec::new();
    };

    let mut found: Vec<Comparison> = (0..sequence.window_count(k))
        .filter(|start| disjoint(query, *start, k))
        .filter_map(|start| {
            let events = sequence.window(start, k)?;
            Some(Comparison {
                a: query_ref,
                b: sequence.window_ref(start, k)?,
                alignment: align(query_events, events),
            })
        })
        .collect();
    sort_comparisons(&mut found);
    found.truncate(top);
    found
}

/// Every disjoint pair of windows in the sequence, best first.
///
/// `O(n²)` comparisons of `O(k²)` each, which is a few tens of millions of
/// floating-point operations on the recordings this experiment runs against and
/// is not worth indexing around.
pub fn top_pairs(sequence: &EventSequence<'_>, k: usize, top: usize) -> Vec<Comparison> {
    top_pairs_where(sequence, k, top, |_, _| true)
}

/// The same, over the pairs a caller is willing to look at.
///
/// The one use this round has for it is sprint:6's lesson in a new shape.
/// Emptiness dominated the sampled representation's rankings; exact repetition
/// of a **degenerate** figure dominates this one, and for the same underlying
/// reason — the abundant thing is what a global minimum finds. sprint:6 answered
/// that by reporting the library's raw top motif *and* a masked one, with the
/// masked fraction travelling beside it. This is that, in events: a caller can
/// ask for the unrestricted ranking and for the ranking over pairs where neither
/// window is degenerate, and is expected to report both. The filter is a second
/// view of one ranking, never a replacement for it.
pub fn top_pairs_where(
    sequence: &EventSequence<'_>,
    k: usize,
    top: usize,
    keep: impl Fn(&WindowRef, &WindowRef) -> bool,
) -> Vec<Comparison> {
    let count = sequence.window_count(k);
    let mut found: Vec<Comparison> = Vec::new();
    for first in 0..count {
        let Some(a_events) = sequence.window(first, k) else {
            continue;
        };
        let Some(a) = sequence.window_ref(first, k) else {
            continue;
        };
        for second in (first + k)..count {
            let (Some(b_events), Some(b)) =
                (sequence.window(second, k), sequence.window_ref(second, k))
            else {
                continue;
            };
            if !keep(&a, &b) {
                continue;
            }
            found.push(Comparison {
                a,
                b,
                alignment: align(a_events, b_events),
            });
        }
    }
    sort_comparisons(&mut found);
    found.truncate(top);
    found
}

/// Best total first, then by window position, so the order is total.
fn sort_comparisons(found: &mut [Comparison]) {
    found.sort_by(|left, right| {
        left.alignment
            .total
            .partial_cmp(&right.alignment.total)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(left.a.start.cmp(&right.a.start))
            .then(left.b.start.cmp(&right.b.start))
    });
}

/// Project an inspected recording into an ordered sequence of marked events.
///
/// Pure and deterministic: the same [`Inspection`] and the same scope always
/// yield the same sequence. Reads no file, consults no clock, and borrows its
/// input.
///
/// Returns `None` when the examined scope holds no records — with no record
/// there is no earliest timestamp, so there is no origin, and a sequence over an
/// invented origin would be a fabricated one.
pub fn project<'a>(
    inspection: &'a Inspection<'a>,
    channels: ChannelScope,
) -> Option<EventSequence<'a>> {
    let extrema = inspection.timestamps.as_ref()?;
    let origin = extrema.earliest.recorded_at;
    let origin_ns = origin.as_nanosecond();

    let mut events: Vec<MarkedEvent<'a>> = Vec::new();
    let mut filtered_out = 0usize;
    let mut clamped_gaps = 0usize;
    let mut previous_ns: Option<i128> = None;

    // Canonical order in, canonical order out. Nothing is sorted by timestamp.
    for entry in &inspection.ledger {
        if !channels.retains(entry.channel) {
            filtered_out += 1;
            continue;
        }
        let at_ns = entry.recorded_at.as_nanosecond();
        let gap_from_previous_ms = previous_ns.map(|previous| {
            let delta_ns = at_ns - previous;
            if delta_ns < 0 {
                // The clock moved backwards between two retained records. The
                // recording's order is intact; its clock disagrees with it. The
                // disagreement is counted, not repaired, and a negative gap is
                // not a quantity this metric can carry.
                clamped_gaps += 1;
                0
            } else {
                u64::try_from(delta_ns / 1_000_000).unwrap_or(0)
            }
        });
        previous_ns = Some(at_ns);

        events.push(MarkedEvent {
            sequence: Some(entry.sequence),
            mark: Mark {
                kind: entry.kind,
                tool_name: entry.tool_name,
            },
            offset_ms: u64::try_from((at_ns - origin_ns) / 1_000_000).unwrap_or(0),
            gap_from_previous_ms,
        });
    }

    Some(EventSequence {
        channels,
        events,
        origin,
        filtered_out,
        clamped_gaps,
        non_monotonic: extrema.non_monotonic.clone(),
        scope: inspection.scope,
        session_id: inspection.session_id,
    })
}

/// The event-count ladder task:18 preregistered, for a figure of `n` events.
///
/// `{3} ∪ {n−2 … n+2}`, sorted, deduplicated, with anything below three dropped.
/// Three is the short control: the shortest window with two within-window gaps,
/// and short enough that a fragment of the figure should be indistinguishable
/// from a fragment of a baseline.
pub fn ladder(n: usize) -> Vec<usize> {
    let mut lengths: Vec<usize> = [3]
        .into_iter()
        .chain((n.saturating_sub(2))..=(n + 2))
        .filter(|k| *k >= 3)
        .collect();
    lengths.sort_unstable();
    lengths.dedup();
    lengths
}

/// A fixed linear congruential generator, so a null is reproducible.
///
/// The same shape as the fixtures' own and sprint:6's, with its own seed. A null
/// that moves between runs is not a control.
struct Lcg(u64);

impl Lcg {
    fn new(seed: u64) -> Self {
        Self(seed | 1)
    }

    fn next_below(&mut self, bound: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        if bound == 0 {
            0
        } else {
            ((self.0 >> 33) as usize) % bound
        }
    }

    /// A fixed-seed Fisher–Yates permutation of the indices `0..len`.
    fn permutation(&mut self, len: usize) -> Vec<usize> {
        let mut order: Vec<usize> = (0..len).collect();
        for index in (1..len).rev() {
            let pick = self.next_below(index + 1);
            order.swap(index, pick);
        }
        order
    }
}

/// The order null: the same marks, somewhere else.
///
/// Permutes the marks across the whole sequence with a fixed-seed Fisher–Yates
/// and leaves every gap and offset exactly where it is. The mark multiset and
/// the entire timing profile survive; which event happened when does not.
///
/// It answers one question: **does event order carry information?**
pub fn order_null<'a>(sequence: &EventSequence<'a>) -> EventSequence<'a> {
    let mut generator = Lcg::new(0x4F52_4445_524E_554C);
    let order = generator.permutation(sequence.events.len());
    let mut shuffled = sequence.clone();
    for (position, source) in order.into_iter().enumerate() {
        shuffled.events[position].mark = sequence.events[source].mark;
        // A permuted mark is no longer the mark that record carried, so the
        // receipt would be a false one.
        shuffled.events[position].sequence = None;
    }
    shuffled
}

/// The timing null: the same events, at the wrong times.
///
/// Permutes the gaps across the whole sequence with the same generator, leaves
/// every mark where it is, and recomputes offsets cumulatively so the result is
/// still a timeline rather than a set of contradictory timestamps. The event
/// sequence survives exactly; relative timing does not.
///
/// It answers one question: **does timing contribute anything beyond identity?**
pub fn timing_null<'a>(sequence: &EventSequence<'a>) -> EventSequence<'a> {
    let mut generator = Lcg::new(0x0054_494D_494E_4701);
    // The first event has no gap and keeps its position; the rest are permuted
    // among themselves.
    let gaps: Vec<u64> = sequence
        .events
        .iter()
        .skip(1)
        .map(|event| event.gap_from_previous_ms.unwrap_or(0))
        .collect();
    let order = generator.permutation(gaps.len());

    let mut shuffled = sequence.clone();
    let mut offset = sequence.events.first().map(|e| e.offset_ms).unwrap_or(0);
    for (position, source) in order.into_iter().enumerate() {
        let gap = gaps[source];
        offset = offset.saturating_add(gap);
        let event = &mut shuffled.events[position + 1];
        event.gap_from_previous_ms = Some(gap);
        event.offset_ms = offset;
        // The timestamps are no longer this record's, so neither is the receipt.
        event.sequence = None;
    }
    if let Some(first) = shuffled.events.first_mut() {
        first.sequence = None;
    }
    shuffled
}

/// The controlled perturbation sweep: one known figure, made progressively less
/// like itself.
///
/// **Only meaningful if basic recovery is earned**, which is why nothing here is
/// consulted by the fixture scans. It answers a different question: does the
/// distance degrade *gracefully*, or does it fall off a cliff the moment an
/// occurrence stops being identical?
///
/// The base figure is hand-built from the legible oracle's own generator
/// constants rather than carved out of the fixture, so the sweep cannot
/// accidentally perturb committed evidence. `tests/event_sequence.rs` asserts
/// that it agrees mark for mark and gap for gap with the window the ordinary
/// projection extracts from the committed fixture's first planted occurrence, so
/// "hand-built" does not mean "unverified".
pub mod perturbation {
    use super::{MarkedEvent, align};
    use crate::experiment::oracle;
    use crate::inspection::{EventKind, V2Kind};

    /// The legible oracle's planted figure, observed records only: four calls,
    /// each a request and a success, in a fixed tool order.
    ///
    /// Gaps are the differences between the generator's own offsets — the figure
    /// starts at `+100` and the offsets are 100, 300, 600, 900, 1200, 1500,
    /// 1800, 2200 — so the within-window gaps are 200, 300, 300, 300, 300, 300,
    /// 400 milliseconds.
    pub fn base() -> Vec<MarkedEvent<'static>> {
        let offsets = [100u64, 300, 600, 900, 1200, 1500, 1800, 2200];
        let steps = [
            (V2Kind::ToolRequested, oracle::TOOL_READER),
            (V2Kind::ToolSucceeded, oracle::TOOL_READER),
            (V2Kind::ToolRequested, oracle::TOOL_SEARCHER),
            (V2Kind::ToolSucceeded, oracle::TOOL_SEARCHER),
            (V2Kind::ToolRequested, oracle::TOOL_EDITOR),
            (V2Kind::ToolSucceeded, oracle::TOOL_EDITOR),
            (V2Kind::ToolRequested, oracle::TOOL_SHELL),
            (V2Kind::ToolSucceeded, oracle::TOOL_SHELL),
        ];
        steps
            .into_iter()
            .enumerate()
            .map(|(index, (kind, tool))| {
                let gap = (index > 0).then(|| offsets[index] - offsets[index - 1]);
                MarkedEvent::hand_built(EventKind::V2(kind), Some(tool), gap)
            })
            .collect()
    }

    /// Scale every within-window gap by `1 ± fraction`, alternating sign by
    /// position. Deterministic, and it moves every gap rather than one.
    pub fn jittered(fraction: f64) -> Vec<MarkedEvent<'static>> {
        base()
            .into_iter()
            .enumerate()
            .map(|(index, mut event)| {
                if let Some(gap) = event.gap_from_previous_ms {
                    let sign = if index.is_multiple_of(2) { 1.0 } else { -1.0 };
                    let scaled = (gap as f64 * (1.0 + sign * fraction)).round();
                    event.gap_from_previous_ms = Some(scaled.max(0.0) as u64);
                }
                event
            })
            .collect()
    }

    /// One extra event: a second reader request spliced in after the first call,
    /// carrying a gap of the same order as its neighbours.
    pub fn inserted() -> Vec<MarkedEvent<'static>> {
        let mut events = base();
        events.insert(
            2,
            MarkedEvent::hand_built(
                EventKind::V2(V2Kind::ToolRequested),
                Some(oracle::TOOL_READER),
                Some(300),
            ),
        );
        events
    }

    /// One omitted event: the searcher's success never arrives, and the next
    /// event's gap absorbs the time it would have taken.
    pub fn omitted() -> Vec<MarkedEvent<'static>> {
        let mut events = base();
        let removed = events.remove(3);
        if let (Some(gap), Some(next)) = (removed.gap_from_previous_ms, events.get_mut(3)) {
            next.gap_from_previous_ms = Some(next.gap_from_previous_ms.unwrap_or(0) + gap);
        }
        events
    }

    /// One substituted identity: the shell call fails instead of succeeding,
    /// which is the deviation the oracle's own recurrence injects.
    pub fn substituted() -> Vec<MarkedEvent<'static>> {
        let mut events = base();
        if let Some(last) = events.last_mut() {
            last.mark.kind = EventKind::V2(V2Kind::ToolFailed);
        }
        events
    }

    /// An unrelated sequence of the same length: the oracle's baseline figure,
    /// which is one tool name alternating request and success.
    pub fn unrelated() -> Vec<MarkedEvent<'static>> {
        (0..8usize)
            .map(|index| {
                let kind = if index.is_multiple_of(2) {
                    V2Kind::ToolRequested
                } else {
                    V2Kind::ToolSucceeded
                };
                let gap = match index {
                    0 => None,
                    i if i.is_multiple_of(2) => Some(5_880),
                    _ => Some(120),
                };
                MarkedEvent::hand_built(EventKind::V2(kind), Some(oracle::TOOL_READER), gap)
            })
            .collect()
    }

    /// The whole sweep, in the order it is meant to be read: named variants,
    /// each aligned against [`base`].
    pub fn sweep() -> Vec<(&'static str, super::Alignment)> {
        let reference = base();
        [
            ("exact", base()),
            ("10% timing jitter", jittered(0.10)),
            ("30% timing jitter", jittered(0.30)),
            ("100% timing jitter", jittered(1.00)),
            ("300% timing jitter", jittered(3.00)),
            ("one inserted event", inserted()),
            ("one omitted event", omitted()),
            ("one substituted identity", substituted()),
            ("unrelated sequence", unrelated()),
        ]
        .into_iter()
        .map(|(name, variant)| (name, align(&reference, &variant)))
        .collect()
    }
}
