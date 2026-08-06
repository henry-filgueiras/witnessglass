//! **Disposable research experiment.** sprint:19, task:29.
//!
//! Search-aware null calibration. The question is not whether R1 is large on
//! observed data — it is whether the **complete search procedure** produces
//! more exceptional maxima on observed sequences than the same procedure
//! produces on order-null replicates.
//!
//! # The one rule this module exists to keep
//!
//! [`complete_search`] is the only place `T` is computed, and **both** paths
//! call it. The observed path calls it on the projected sequences; the null
//! path calls it on order-null replicates of those sequences. Every
//! data-dependent stage — window enumeration, alignment, ranking,
//! deduplication, R1 readout — reruns inside each replicate.
//!
//! task:29 §PHASE 0 D2 records why this matters: the existing
//! [`super::event_sequence::null_evidence`] holds candidate boundaries fixed
//! and permutes only identity. That answers a narrower question, and reusing it
//! here would have measured the wrong thing while looking correct. It is not
//! called from this module, and a test asserts so.
//!
//! # What the search actually ranks by
//!
//! task:29 §PHASE 0 D1: `cross_pairs` sorts by `alignment.total`, not by R1.
//! R1 never drives selection. `T` is therefore the R1 value **at the winner the
//! alignment search chose**, and every claim this module supports is about that
//! pipeline rather than about R1 in isolation.

use std::collections::BTreeMap;

use serde::Serialize;

use super::event_sequence::{
    ChannelScope, EventSequence, Mark, MarkedEvent, ORDER_NULL_SEED, cross_pairs,
    dedupe_overlapping, null_seed, order_null_seeded,
};
use super::identifiability::Observation;
use super::repair::{Candidate, candidate};

/// The frozen sprint:9 ladder. Not a parameter of this round.
pub const LADDER: [usize; 5] = [3, 4, 6, 8, 12];

/// What `dedupe_overlapping` keeps, unchanged from sprint:16 and sprint:17.
pub const KEEP: usize = 5;

/// task:29 §PHASE 6's top-k, fixed before execution at the number the
/// deduplication already keeps so that no new parameter enters.
pub const TOP_K: usize = KEEP;

/// Null replicates. task:29 §PHASE 4 fixes this at 999 from a measured 0.097 s
/// complete search, giving a finest resolvable tail of 1/1000 against
/// thresholds of 0.01.
pub const REPLICATES: usize = 999;

/// The preregistered exceptional-tail threshold.
pub const TAIL_THRESHOLD: f64 = 0.01;

fn r1() -> &'static Candidate {
    candidate("R1 pooled sum").expect("the statistic under calibration must remain")
}

/// What one complete search produced at one span length.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SearchOutcome {
    /// The span length searched.
    pub k: usize,
    /// `T` — the maximum R1 over the deduplicated winner set. `None` when the
    /// search admitted no candidate, which is reported rather than scored zero.
    pub t: Option<f64>,
    /// The winner set's R1 values, descending. At most [`TOP_K`].
    pub top: Vec<f64>,
    /// Window pairs enumerated before ranking, so the two paths' search spaces
    /// can be shown to be the same size.
    pub considered: usize,
    /// Candidates surviving deduplication.
    pub kept: usize,
}

/// **The only definition of `T` in this round.** Both paths call this.
///
/// Runs the complete pipeline task:29 §PHASE 1 derived from the machinery:
/// enumerate every window pair at `k`, rank by alignment total ascending,
/// deduplicate greedily to [`KEEP`], then read R1 off the survivors.
pub fn complete_search(
    first: &EventSequence<'_>,
    second: &EventSequence<'_>,
    k: usize,
) -> SearchOutcome {
    let Some(ranked) = cross_pairs(first, second, k, usize::MAX) else {
        return SearchOutcome {
            k,
            t: None,
            top: Vec::new(),
            considered: 0,
            kept: 0,
        };
    };
    let considered = ranked.len();
    let kept = dedupe_overlapping(&ranked, KEEP);

    let mut scores: Vec<f64> = kept
        .iter()
        .filter_map(|candidate| {
            let (wa, wb) = (&candidate.comparison.a, &candidate.comparison.b);
            let observation = Observation::of(
                first,
                (wa.start, wa.start + wa.k),
                second,
                (wb.start, wb.start + wb.k),
            )?;
            (r1().score)(&observation)
        })
        .collect();
    scores.sort_by(|left, right| right.partial_cmp(left).unwrap_or(std::cmp::Ordering::Equal));

    SearchOutcome {
        k,
        t: scores.first().copied(),
        top: scores.iter().copied().take(TOP_K).collect(),
        considered,
        kept: kept.len(),
    }
}

/// The whole ladder, in one pass.
pub fn complete_search_ladder(
    first: &EventSequence<'_>,
    second: &EventSequence<'_>,
) -> Vec<SearchOutcome> {
    LADDER
        .iter()
        .map(|k| complete_search(first, second, *k))
        .collect()
}

/// One specimen's calibration: the observed `T` against the distribution of `T`
/// produced by complete searches on null replicates.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Calibration {
    /// Opaque specimen label. Session prefixes only — decision:8 forbids more.
    pub specimen: String,
    /// The span length.
    pub k: usize,
    /// `T` on the observed sequences.
    pub observed: Option<f64>,
    /// Replicates whose complete search yielded a defined `T`.
    pub realizations: usize,
    /// Null median.
    pub null_median: f64,
    /// Null quantiles at 0.05, 0.25, 0.50, 0.75, 0.95, 0.99.
    pub null_quantiles: Vec<f64>,
    /// The largest `T` any null search produced.
    pub null_max: f64,
    /// `count(T_null >= T_observed)`.
    pub exceedances: usize,
    /// `count(T_null < T_observed)`. sprint:20 reports the observed value's
    /// percentile within each null distribution, and a percentile needs the
    /// other side of the same count.
    pub below: usize,
    /// `(1 + exceedances) / (B + 1)` — a Monte Carlo null tail estimate under
    /// this null and this search, and nothing else.
    pub tail: f64,
    /// Whether `tail <= TAIL_THRESHOLD`.
    pub exceptional: bool,
    /// Window pairs the observed search enumerated.
    pub observed_considered: usize,
    /// Mean window pairs the null searches enumerated. Equal to the observed
    /// count when the null preserves length, which it does; reported so that is
    /// visible rather than assumed.
    pub null_considered_mean: f64,
    /// task:29 §PHASE 6: per-rank exceedance counts over the top-k order
    /// statistics. Descriptive; no verdict branch reads it.
    pub top_k_exceedances: Vec<usize>,
    /// Every `T_null` this calibration produced, ascending.
    ///
    /// Held so a paired plot can draw two null distributions on one axis, and
    /// **not serialized**: a document carrying one array per specimen per null
    /// would be megabytes of duplicated numbers, and the caller that wants them
    /// has them here.
    #[serde(skip)]
    pub samples: Vec<f64>,
}

/// Calibrate one pair at one span length.
///
/// Generates `replicates` independent order nulls of **both** sequences with the
/// existing deterministic seed scheme and reruns the complete search on each.
pub fn calibrate(
    specimen: &str,
    first: &EventSequence<'_>,
    second: &EventSequence<'_>,
    k: usize,
    replicates: usize,
) -> Calibration {
    calibrate_with(specimen, first, second, k, replicates, order_null_seeded)
}

/// The same calibration, with the null construction supplied by the caller.
///
/// **sprint:20's one extension to this module, and it extends nothing else.**
/// The construction may only produce a replacement [`EventSequence`] from a
/// sequence and a seed; every data-dependent stage of the search still reruns
/// inside every replicate, through the same [`complete_search`] both paths call.
/// [`calibrate`] is this function with sprint:19's order null passed in, so the
/// two rounds cannot drift apart in anything but the null.
pub fn calibrate_with<'a, F>(
    specimen: &str,
    first: &EventSequence<'a>,
    second: &EventSequence<'a>,
    k: usize,
    replicates: usize,
    construct: F,
) -> Calibration
where
    F: Fn(&EventSequence<'a>, u64) -> EventSequence<'a>,
{
    let observed = complete_search(first, second, k);

    let mut null_t: Vec<f64> = Vec::with_capacity(replicates);
    let mut considered: Vec<usize> = Vec::with_capacity(replicates);
    let mut null_tops: Vec<Vec<f64>> = Vec::with_capacity(replicates);

    for index in 0..replicates {
        let left = construct(first, null_seed(index, 0));
        let right = construct(second, null_seed(index, 1));
        // The complete search, rerun. Not a rescoring of observed boundaries.
        let outcome = complete_search(&left, &right, k);
        if let Some(value) = outcome.t {
            null_t.push(value);
        }
        considered.push(outcome.considered);
        null_tops.push(outcome.top);
    }

    null_t.sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));

    let exceedances = match observed.t {
        Some(value) => null_t.iter().filter(|null| **null >= value).count(),
        None => null_t.len(),
    };
    let below = match observed.t {
        Some(value) => null_t.iter().filter(|null| **null < value).count(),
        None => 0,
    };
    let tail = (1.0 + exceedances as f64) / (replicates as f64 + 1.0);

    let top_k_exceedances = (0..TOP_K)
        .map(|rank| {
            let Some(observed_rank) = observed.top.get(rank).copied() else {
                return 0;
            };
            null_tops
                .iter()
                .filter(|top| {
                    top.get(rank)
                        .copied()
                        .is_some_and(|null| null >= observed_rank)
                })
                .count()
        })
        .collect();

    Calibration {
        specimen: specimen.to_owned(),
        k,
        observed: observed.t,
        realizations: null_t.len(),
        null_median: quantile(&null_t, 0.50),
        null_quantiles: [0.05, 0.25, 0.50, 0.75, 0.95, 0.99]
            .iter()
            .map(|q| quantile(&null_t, *q))
            .collect(),
        null_max: null_t.last().copied().unwrap_or(f64::NAN),
        exceedances,
        below,
        tail,
        exceptional: tail <= TAIL_THRESHOLD,
        observed_considered: observed.considered,
        null_considered_mean: if considered.is_empty() {
            f64::NAN
        } else {
            considered.iter().sum::<usize>() as f64 / considered.len() as f64
        },
        top_k_exceedances,
        samples: null_t,
    }
}

/// Nearest-rank quantile of an already-sorted slice.
fn quantile(sorted: &[f64], q: f64) -> f64 {
    if sorted.is_empty() {
        return f64::NAN;
    }
    let rank = (q * (sorted.len() as f64 - 1.0)).round() as usize;
    sorted[rank.min(sorted.len() - 1)]
}

// ---------------------------------------------------------------------------
// PHASE 3 — the selection effect, on this project's own machinery
// ---------------------------------------------------------------------------

/// task:29 §PHASE 3's demonstration.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SelectionEffect {
    /// R1 at candidate pairs sampled uniformly from the admissible grid.
    pub arbitrary: Vec<f64>,
    /// `T` from complete searches on independently generated null specimens.
    pub selected: Vec<f64>,
    /// Median of each.
    pub arbitrary_median: f64,
    /// The selected median.
    pub selected_median: f64,
    /// The 0.99 quantile of the arbitrary distribution.
    pub arbitrary_q99: f64,
    /// `selected_median − arbitrary_q99`.
    pub margin: f64,
    /// Whether the preregistered ordering held.
    pub confirmed: bool,
}

/// Sample R1 at uniformly drawn admissible candidate pairs, without searching.
fn arbitrary_scores(
    first: &EventSequence<'_>,
    second: &EventSequence<'_>,
    k: usize,
    draws: usize,
    seed: u64,
) -> Vec<f64> {
    let (a_windows, b_windows) = (first.window_count(k), second.window_count(k));
    if a_windows == 0 || b_windows == 0 {
        return Vec::new();
    }
    // The same LCG shape the null uses, so no new generator enters the round.
    let mut state = seed | 1;
    let mut next = move || {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        state >> 33
    };
    (0..draws)
        .filter_map(|_| {
            let a_start = (next() as usize) % a_windows;
            let b_start = (next() as usize) % b_windows;
            let observation = Observation::of(
                first,
                (a_start, a_start + k),
                second,
                (b_start, b_start + k),
            )?;
            (r1().score)(&observation)
        })
        .collect()
}

/// Run the demonstration at the preregistered parameters.
pub fn selection_effect(
    first: &EventSequence<'_>,
    second: &EventSequence<'_>,
    k: usize,
    draws: usize,
    searches: usize,
) -> SelectionEffect {
    let mut arbitrary = arbitrary_scores(first, second, k, draws, 0x00C0_FFEE_5E1E_C700);
    arbitrary.sort_by(|l, r| l.partial_cmp(r).unwrap_or(std::cmp::Ordering::Equal));

    let mut selected: Vec<f64> = (0..searches)
        .filter_map(|index| {
            let left = order_null_seeded(first, null_seed(index, 0));
            let right = order_null_seeded(second, null_seed(index, 1));
            complete_search(&left, &right, k).t
        })
        .collect();
    selected.sort_by(|l, r| l.partial_cmp(r).unwrap_or(std::cmp::Ordering::Equal));

    let arbitrary_median = quantile(&arbitrary, 0.50);
    let selected_median = quantile(&selected, 0.50);
    let arbitrary_q99 = quantile(&arbitrary, 0.99);

    SelectionEffect {
        arbitrary_median,
        selected_median,
        arbitrary_q99,
        margin: selected_median - arbitrary_q99,
        confirmed: selected_median > arbitrary_median,
        arbitrary,
        selected,
    }
}

// ---------------------------------------------------------------------------
// PHASE 7 — null adequacy
// ---------------------------------------------------------------------------

/// A domain-neutral sequence summary: a name and the measure that computes it.
type Measure = (&'static str, fn(&EventSequence<'_>) -> f64);

/// Three domain-neutral categorical-sequence summaries, observed against null.
///
/// None reads a prompt, a command, a response, a path, or a file's contents.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StructuralSummary {
    /// Which summary.
    pub name: &'static str,
    /// The observed value.
    pub observed: f64,
    /// The null median.
    pub null_median: f64,
    /// The null distribution's range.
    pub null_min: f64,
    /// The other end of it.
    pub null_max: f64,
    /// Replicates whose value met or exceeded the observed one.
    pub exceedances: usize,
    /// Whether the observed value lies outside the whole null range, which
    /// task:29 §PHASE 7 makes the trigger for recording the limitation.
    pub outside_null_range: bool,
}

/// Fraction of adjacent positions carrying the same mark.
fn immediate_repetition(sequence: &EventSequence<'_>) -> f64 {
    let marks: Vec<_> = sequence.events.iter().map(|event| event.mark).collect();
    if marks.len() < 2 {
        return f64::NAN;
    }
    let repeats = marks.windows(2).filter(|pair| pair[0] == pair[1]).count();
    repeats as f64 / (marks.len() - 1) as f64
}

/// Mean length of maximal runs of one mark.
fn mean_run_length(sequence: &EventSequence<'_>) -> f64 {
    let marks: Vec<_> = sequence.events.iter().map(|event| event.mark).collect();
    if marks.is_empty() {
        return f64::NAN;
    }
    let mut runs = 1usize;
    for pair in marks.windows(2) {
        if pair[0] != pair[1] {
            runs += 1;
        }
    }
    marks.len() as f64 / runs as f64
}

/// Bigram entropy in nats, normalized by the entropy the marginals alone would
/// produce. Below 1 means order carries information the marginals do not.
fn transition_entropy_ratio(sequence: &EventSequence<'_>) -> f64 {
    let marks: Vec<_> = sequence.events.iter().map(|event| event.mark).collect();
    if marks.len() < 2 {
        return f64::NAN;
    }
    // Keyed by the stable label: `Mark` is deliberately not `Ord`, and this
    // needs an ordering only to count, never to rank anything.
    let mut bigrams: BTreeMap<(String, String), usize> = BTreeMap::new();
    let mut unigrams: BTreeMap<String, usize> = BTreeMap::new();
    for pair in marks.windows(2) {
        *bigrams
            .entry((pair[0].label(), pair[1].label()))
            .or_insert(0) += 1;
    }
    for mark in &marks {
        *unigrams.entry(mark.label()).or_insert(0) += 1;
    }
    let pairs = (marks.len() - 1) as f64;
    fn entropy<'b>(counts: impl Iterator<Item = &'b usize>, total: f64) -> f64 {
        counts
            .map(|count| {
                let p = *count as f64 / total;
                -p * p.ln()
            })
            .sum::<f64>()
    }
    let observed = entropy(bigrams.values(), pairs);
    // The marginal-product entropy of a bigram is twice the unigram entropy.
    let marginal = 2.0 * entropy(unigrams.values(), marks.len() as f64);
    if marginal == 0.0 {
        f64::NAN
    } else {
        observed / marginal
    }
}

/// Every summary for one sequence, against `replicates` order nulls of it.
pub fn structural_summaries(
    sequence: &EventSequence<'_>,
    replicates: usize,
) -> Vec<StructuralSummary> {
    structural_summaries_with(sequence, replicates, order_null_seeded)
}

/// The same three summaries against a null construction the caller supplies.
///
/// sprint:20 asks the adequacy question sprint:19 answered for the order null of
/// two further constructions, and asks it with the *same* three measures, so the
/// answers are comparable. The measures themselves do not move.
pub fn structural_summaries_with<'a, F>(
    sequence: &EventSequence<'a>,
    replicates: usize,
    construct: F,
) -> Vec<StructuralSummary>
where
    F: Fn(&EventSequence<'a>, u64) -> EventSequence<'a>,
{
    let measures: [Measure; 3] = [
        ("immediate repetition rate", immediate_repetition),
        ("mean run length", mean_run_length),
        ("transition entropy ratio", transition_entropy_ratio),
    ];

    measures
        .iter()
        .map(|(name, measure)| {
            let observed = measure(sequence);
            let mut nulls: Vec<f64> = (0..replicates)
                .map(|index| measure(&construct(sequence, null_seed(index, 0))))
                .collect();
            nulls.sort_by(|l, r| l.partial_cmp(r).unwrap_or(std::cmp::Ordering::Equal));
            let min = nulls.first().copied().unwrap_or(f64::NAN);
            let max = nulls.last().copied().unwrap_or(f64::NAN);
            StructuralSummary {
                name,
                observed,
                null_median: quantile(&nulls, 0.50),
                null_min: min,
                null_max: max,
                exceedances: nulls.iter().filter(|null| **null >= observed).count(),
                outside_null_range: observed < min || observed > max,
            }
        })
        .collect()
}

/// The seed the whole round derives from, re-exported so a report can state it
/// without reaching into [`super::event_sequence`].
pub const ROUND_SEED: u64 = ORDER_NULL_SEED;

// ---------------------------------------------------------------------------
// PHASE 4 — controlled fixtures
// ---------------------------------------------------------------------------

/// The 12-symbol synthetic alphabet the controls draw from.
///
/// Obviously synthetic, per `CLAUDE.md` §5: the tool names are `synthetic-00`
/// and friends, and no fixture here derives from any recording.
///
/// Public so sprint:20's first-order controls draw from the same alphabet
/// rather than inventing a second one.
pub const SYNTHETIC_TOOLS: [&str; 12] = [
    "synthetic-00",
    "synthetic-01",
    "synthetic-02",
    "synthetic-03",
    "synthetic-04",
    "synthetic-05",
    "synthetic-06",
    "synthetic-07",
    "synthetic-08",
    "synthetic-09",
    "synthetic-10",
    "synthetic-11",
];

/// The fixed categorical distribution both controls draw marks from, as
/// cumulative weights over [`SYNTHETIC_TOOLS`]. Deliberately uneven, so the
/// pooled rarity weights are not all equal.
///
/// Public for the same reason as [`SYNTHETIC_TOOLS`]: sprint:20's first-order
/// controls weight their own transitions with this vector rather than adding a
/// second uneven distribution to the round.
pub const SYNTHETIC_WEIGHTS: [u32; 12] = [22, 18, 14, 11, 9, 7, 6, 5, 4, 2, 1, 1];

/// The figure the positive control plants, as indices into
/// [`SYNTHETIC_TOOLS`]. Twelve marks, matching the ladder's longest span, and
/// drawn from the same alphabet as the background so the marginals shift as
/// little as planting permits.
const PLANTED_FIGURE: [usize; 12] = [3, 7, 1, 9, 3, 0, 5, 2, 7, 1, 4, 6];

/// Where the figure is planted in each control sequence.
const PLANT_SITES_A: [usize; 3] = [10, 60, 120];
const PLANT_SITES_B: [usize; 3] = [8, 40, 70];

/// Generate one synthetic sequence. `plant_at` is empty for the negative
/// control and carries the sites for the positive one.
///
/// Uses the same LCG the null uses, so no new generator enters the round.
fn synthetic_sequence<'a>(
    length: usize,
    seed: u64,
    plant_at: &[usize],
    session_id: &'a str,
) -> EventSequence<'a> {
    use super::super::inspection::{EventKind, ExaminedScope, Receipts, RecordCount, V2Kind};

    let total: u32 = SYNTHETIC_WEIGHTS.iter().sum();
    let mut state = seed | 1;
    let mut next = move || {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        state >> 33
    };

    let mut indices: Vec<usize> = Vec::with_capacity(length);
    for _ in 0..length {
        let draw = (next() as u32) % total;
        let mut running = 0u32;
        let mut chosen = SYNTHETIC_WEIGHTS.len() - 1;
        for (index, weight) in SYNTHETIC_WEIGHTS.iter().enumerate() {
            running += weight;
            if draw < running {
                chosen = index;
                break;
            }
        }
        indices.push(chosen);
    }

    // Plant the figure, overwriting the background at each site.
    for site in plant_at {
        for (offset, symbol) in PLANTED_FIGURE.iter().enumerate() {
            if let Some(slot) = indices.get_mut(site + offset) {
                *slot = *symbol;
            }
        }
    }

    let mut events = Vec::with_capacity(length);
    let mut offset_ms = 0u64;
    for (position, index) in indices.into_iter().enumerate() {
        let gap = 500 + (next() % 4_500);
        if position > 0 {
            offset_ms = offset_ms.saturating_add(gap);
        }
        events.push(MarkedEvent {
            sequence: None,
            mark: Mark {
                kind: EventKind::V2(V2Kind::ToolRequested),
                tool_name: Some(SYNTHETIC_TOOLS[index]),
            },
            offset_ms,
            gap_from_previous_ms: if position == 0 { None } else { Some(gap) },
        });
    }

    let scope = ExaminedScope::CompleteRecording { records: length };
    EventSequence {
        channels: ChannelScope::Observed,
        events,
        origin: jiff::Timestamp::UNIX_EPOCH,
        filtered_out: 0,
        clamped_gaps: 0,
        // A hand-built sequence has no records, so it has no receipts. That is
        // the absence `CLAUDE.md` §6 requires be carried rather than filled.
        non_monotonic: RecordCount {
            records: Receipts::default(),
            scope,
        },
        scope,
        session_id: Some(session_id),
    }
}

/// A control specimen: two synthetic sequences and the marginals they produced.
pub struct Control<'a> {
    /// The first sequence.
    pub first: EventSequence<'a>,
    /// The second.
    pub second: EventSequence<'a>,
}

/// task:29 §PHASE 4's negative control: two sequences generated **according to
/// the order-null hypothesis**, so observed and null are draws from one law.
pub fn negative_control<'a>() -> Control<'a> {
    Control {
        first: synthetic_sequence(160, 0x0E6A_71BE_0000_0001, &[], "negctl-a"),
        second: synthetic_sequence(90, 0x0E6A_71BE_0000_0002, &[], "negctl-b"),
    }
}

/// The positive control: the same generator with one figure planted three times
/// in each sequence, its marks drawn from the same alphabet as the background.
pub fn positive_control<'a>() -> Control<'a> {
    Control {
        first: synthetic_sequence(160, 0x0E6A_71BE_0000_0001, &PLANT_SITES_A, "posctl-a"),
        second: synthetic_sequence(90, 0x0E6A_71BE_0000_0002, &PLANT_SITES_B, "posctl-b"),
    }
}

/// A sequence's mark counts, so a control's marginal shift is visible rather
/// than asserted.
pub fn marginals(sequence: &EventSequence<'_>) -> Vec<(String, usize)> {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for event in &sequence.events {
        *counts.entry(event.mark.label()).or_insert(0) += 1;
    }
    let mut rows: Vec<(String, usize)> = counts.into_iter().collect();
    rows.sort_by(|left, right| right.1.cmp(&left.1).then(left.0.cmp(&right.0)));
    rows
}
