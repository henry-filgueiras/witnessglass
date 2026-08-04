//! Matrix Profile over one scalar behavioral dimension, at a fixed window.
//!
//! **Disposable.** See [`crate::experiment`]. sprint:6, task:16. Behind the
//! non-default `experiment-matrix-profile` feature, so a default build of the
//! product does not link a numerics stack for a research experiment.
//!
//! # What this is and is not
//!
//! A thin, honest wrapper around [`motif_rs`]. The distance computation, the
//! exclusion zone, the constant-subsequence convention, and the greedy motif and
//! discord extraction are all the library's; this module supplies the input,
//! converts indices to times, applies one documented mask, and computes one
//! preregistered comparison. It is not a detector abstraction, and there is
//! deliberately nothing here that a second detector could be plugged into.
//!
//! # The representation problem, stated before any result
//!
//! These signals are 78–94% empty. `motif_rs` documents — and a test in
//! `tests/matrix_profile.rs` confirms — that **two constant subsequences are at
//! distance exactly 0**, and that a constant matched against a non-constant one
//! yields the sentinel `√(2m)`. In a mostly-empty series the global minimum of
//! the matrix profile is therefore 0, attained by two stretches of nothing, and
//! the library's top motif is a statement about emptiness rather than about a
//! session.
//!
//! That is a property of the representation, not a defect in the library, and it
//! is not patched over. [`WindowScan`] carries **both** answers: the library's
//! unmasked top motif exactly as computed, and the top motifs after constant
//! subsequences are masked out — using the library's own `sigma_threshold`
//! rather than a second definition of "constant" invented here. The masked
//! fraction travels with them, because a result drawn from 6% of the candidate
//! subsequences is a different claim from one drawn from all of them.
//!
//! Entries whose *nearest neighbour* is constant are masked too. Their distance
//! is the `√(2m)` sentinel, which is not a match and must not be reported as a
//! discord.
//!
//! # Amplitude blindness
//!
//! z-normalized Euclidean distance is invariant to a constant offset **and to a
//! constant factor**. `[1,2,3,4]` and `[10,20,30,40]` are at distance 0, which a
//! test asserts against the library. Two bursts with the same shape and different
//! intensity are therefore a perfect match here. That may be what a reader wants;
//! it is emphatically not "the same behaviour recurred", and no output in this
//! experiment says so.
//!
//! # Feed it the raw counts, not sprint:4's z-scored column
//!
//! This is a correctness requirement, not a preference, and it was found by
//! running the experiment and disbelieving an obviously wrong intermediate: a
//! "perfect motif" reported between two regions of the legible oracle that
//! contain no records at all.
//!
//! Matrix Profile z-normalizes every subsequence independently, so a global
//! affine transform of the input changes no distance in exact arithmetic.
//! sprint:4's z-score is exactly such a transform, which makes raw counts and
//! z-scored counts equivalent inputs *mathematically*. They are not equivalent
//! numerically:
//!
//! * In raw counts an empty bucket is exactly `0`, so a window of empty buckets
//!   has a rolling standard deviation of exactly `0` and is detected as constant.
//! * In the z-scored column an empty bucket is `−μ/σ`, some value like
//!   `−1.3e-1`. The rolling variance of a window of identical such values is
//!   computed by cancellation and comes out at **`1.863e-9`**, not zero —
//!   measured on `kind:v2:reported_intent` at `m = 32`. That is six orders of
//!   magnitude above the `1e-15` threshold, so **275 of 309 genuinely empty
//!   windows were not detected as constant**.
//!
//! The second failure is worse than a missed mask. z-normalization then divides
//! those windows by `1.863e-9`, which amplifies pure floating-point noise into a
//! full-amplitude shape, and two amplified noise shapes match each other at
//! distance ~0. The detector reports a flawless motif between two stretches of
//! nothing and marks both as non-constant.
//!
//! So the caller passes unnormalized counts. This is **not** a change to
//! sprint:4's normalization policy, which is untouched and still applies to
//! everything that reads a `Normalized`; it is choosing the numerically sound
//! representative of an input the metric treats as identical either way.
//!
//! # What a window is, and what it is not
//!
//! The window is the length of the pattern being matched. It is not a Haar scale,
//! not a repetition period, and not a motif instance duration — task:16 separates
//! those three explicitly, because sprint:5's language did not always keep them
//! apart and reading a Haar cliff directly as a window length would be exactly
//! that conflation.

use motif_rs::{EuclideanEngine, MatrixProfileConfig, RollingStats, find_discords, find_motifs};
use serde::Serialize;

/// The preregistered window ladder, in milliseconds. task:16 derives each entry
/// and records what it is a control for; the list is fixed and this module does
/// not search around it.
pub const LADDER_MS: [u64; 6] = [2_000, 8_000, 16_000, 32_000, 64_000, 128_000];

/// A subsequence, located in the recording's own time.
///
/// Subsequence `index` covers base samples `[index, index + m)`, so it spans
/// `[index × bucket_ms, (index + m) × bucket_ms)` from the signal's origin. The
/// origin is `recorded_at` of the earliest record, which is descriptive metadata
/// and not the canonical order — sprint:4's caveat travels with every time here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Span {
    /// Subsequence index within the matrix profile.
    pub index: usize,
    /// Offset of the span's start from the signal origin, in milliseconds.
    pub start_ms: u64,
    /// Offset of the span's end from the signal origin, in milliseconds.
    pub end_ms: u64,
}

/// A motif: two subsequences and the distance between them.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct MotifPair {
    /// The subsequence the profile minimum was found at.
    pub a: Span,
    /// Its nearest neighbour.
    pub b: Span,
    /// z-normalized Euclidean distance. Zero means identical after removing
    /// offset and scale — including two stretches of nothing.
    pub distance: f64,
    /// Separation between the two indices, in samples.
    pub lag_samples: usize,
    /// Whether `a` is a constant subsequence.
    pub a_constant: bool,
    /// Whether `b` is a constant subsequence.
    pub b_constant: bool,
    /// Non-zero base samples inside span `a`.
    ///
    /// The diagnostic that decides how much a match is worth. Two windows each
    /// holding **one** non-zero bucket at the same relative offset are identical
    /// after z-normalization and score a distance of exactly 0, whatever else is
    /// or is not around them. A motif with `occupancy` of 1 is a pair of lone
    /// impulses that happen to line up, not a repeated figure, and reading it as
    /// the latter is the specific mistake this field exists to prevent.
    pub a_occupancy: usize,
    /// Non-zero base samples inside span `b`.
    pub b_occupancy: usize,
}

/// A discord: the subsequence furthest from its nearest neighbour.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct DiscordPoint {
    /// Where it is.
    pub at: Span,
    /// Distance to its nearest neighbour. Larger means less like anything else
    /// in the same series — within this window, this dimension, and this
    /// recording, and nothing beyond that.
    pub distance: f64,
}

/// Everything one (dimension, window) pair produced.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WindowScan {
    /// Window length in milliseconds.
    pub window_ms: u64,
    /// Window length in base samples.
    pub m: usize,
    /// Candidate subsequences: `series.len() - m + 1`.
    pub subsequences: usize,
    /// How many of them are constant by the library's `sigma_threshold`.
    pub constant_subsequences: usize,
    /// The library's trivial-match exclusion radius, `ceil(m / 4)`.
    pub exclusion_zone: usize,
    /// The maximum possible z-normalized Euclidean distance at this window,
    /// `2√m`. Every distance here is on that scale, which is what makes
    /// [`WindowScan::separation`] comparable across windows.
    pub max_distance: f64,
    /// The library's own top motif, unmasked and unmodified. Usually two
    /// constant subsequences at distance 0; that is the point of showing it.
    pub raw_top: Option<MotifPair>,
    /// Top motifs after constant subsequences and constant-partner entries are
    /// masked to infinity.
    pub masked_top: Vec<MotifPair>,
    /// Top discords, from the same masked profile.
    pub discords: Vec<DiscordPoint>,
    /// Best masked motif distance on a deterministically shuffled copy of the
    /// same series: the null.
    pub null_best_distance: Option<f64>,
    /// Constant subsequences in the shuffled copy. Shuffling breaks runs of
    /// zeros, so this is normally *lower* than the real series' count, which
    /// makes the null a harder comparison rather than a like-for-like one.
    pub null_constant_subsequences: usize,
    /// `(null_best − best) / 2√m`, the metric task:16 fixed before any result
    /// was seen. Positive means the real ordering admits matches the shuffled
    /// ordering cannot. `None` when either side found no masked motif.
    pub separation: Option<f64>,
}

impl WindowScan {
    /// Fraction of candidate subsequences that are constant.
    pub fn constant_fraction(&self) -> f64 {
        if self.subsequences == 0 {
            return 0.0;
        }
        self.constant_subsequences as f64 / self.subsequences as f64
    }

    /// The best masked motif, if any survived the mask.
    pub fn best(&self) -> Option<&MotifPair> {
        self.masked_top.first()
    }

    /// The rank and pair of the first masked motif linking one region to
    /// another, if any does.
    ///
    /// Regions are half-open millisecond intervals from the signal origin, and a
    /// pair links them when one span *starts* inside each. This is the
    /// cross-region criterion task:16 preregistered: recovering a match inside a
    /// perfectly periodic region proves the region is periodic, and recovering
    /// one that reaches across the intervening recording is the result that
    /// matters.
    pub fn linking(&self, first: (u64, u64), second: (u64, u64)) -> Option<(usize, MotifPair)> {
        let inside = |value: u64, range: (u64, u64)| value >= range.0 && value < range.1;
        self.matching(first, second, |span, range| inside(span.start_ms, range))
    }

    /// The same, with the criterion the preregistered one should have used.
    ///
    /// [`WindowScan::linking`] asks whether a span *starts* inside a region, and
    /// that is too tight by up to one whole window: a 32-sample window that
    /// contains a region's first instance can begin 31 samples before the region
    /// does, and it is still a match reaching into that region. The strict
    /// criterion was written into task:16 before the ladder was run and is kept
    /// exactly as written; this is the overlap criterion, reported beside it
    /// rather than in place of it, so the mis-specification stays visible.
    pub fn overlapping(&self, first: (u64, u64), second: (u64, u64)) -> Option<(usize, MotifPair)> {
        self.matching(first, second, |span, range| {
            span.start_ms < range.1 && span.end_ms > range.0
        })
    }

    fn matching(
        &self,
        first: (u64, u64),
        second: (u64, u64),
        hits: impl Fn(&Span, (u64, u64)) -> bool,
    ) -> Option<(usize, MotifPair)> {
        self.masked_top
            .iter()
            .enumerate()
            .find(|(_, pair)| {
                (hits(&pair.a, first) && hits(&pair.b, second))
                    || (hits(&pair.b, first) && hits(&pair.a, second))
            })
            .map(|(rank, pair)| (rank, *pair))
    }
}

/// Scan one dimension's column at one window.
///
/// **`column` must be the unnormalized counts.** The metric is invariant to a
/// global affine transform, so this changes no distance in exact arithmetic, and
/// it is the difference between detecting an empty window and amplifying its
/// rounding error into a false motif. See the module header for the measurement.
///
/// Returns `None` when the series is too short to hold two windows plus the
/// exclusion zone, which is an absence rather than an empty result: there is no
/// pair to compare, so there is no profile.
pub fn scan(column: &[f64], bucket_ms: u64, window_ms: u64, top_k: usize) -> Option<WindowScan> {
    let m = usize::try_from(window_ms / bucket_ms.max(1)).ok()?;
    if m < 2 || column.len() < m + 1 {
        return None;
    }

    let config = MatrixProfileConfig::new(m);
    let exclusion_zone = config.exclusion_zone();
    // `EuclideanEngine` is the library's alias for z-normalized Euclidean
    // distance. Named here so the choice of metric is visible at the call site.
    let profile = EuclideanEngine::new(config.clone());

    let computed = profile.compute(column);
    let constant = constant_flags(column, m, config.sigma_threshold);
    let subsequences = computed.profile.len();
    let constant_count = constant.iter().filter(|flag| **flag).count();

    let raw_top = find_motifs(&computed, 1).first().map(|motif| {
        pair(
            motif.idx_a,
            motif.idx_b,
            motif.distance,
            m,
            bucket_ms,
            &constant,
            column,
        )
    });

    // The one documented modification. Mask a subsequence that is constant, and
    // one whose nearest neighbour is constant — the latter's distance is the
    // library's `√(2m)` sentinel, which is not a match.
    let mut masked = computed.clone();
    for index in 0..subsequences {
        let partner = computed.profile_index[index];
        if constant[index] || constant.get(partner).copied().unwrap_or(false) {
            masked.profile[index] = f64::INFINITY;
        }
    }

    let masked_top: Vec<MotifPair> = find_motifs(&masked, top_k)
        .into_iter()
        .map(|motif| {
            pair(
                motif.idx_a,
                motif.idx_b,
                motif.distance,
                m,
                bucket_ms,
                &constant,
                column,
            )
        })
        .collect();
    let discords = find_discords(&masked, top_k)
        .into_iter()
        // A discord at distance zero is not anomalous; it is the greedy search
        // running out of finite entries after the real ones were excluded.
        .filter(|discord| discord.distance.is_finite() && discord.distance > 0.0)
        .map(|discord| DiscordPoint {
            at: span(discord.idx, m, bucket_ms),
            distance: discord.distance,
        })
        .collect();

    // The null: same values, same marginal density, no temporal order.
    let shuffled = deterministic_shuffle(column);
    let null_computed = profile.compute(&shuffled);
    let null_constant = constant_flags(&shuffled, m, config.sigma_threshold);
    let mut null_masked = null_computed.clone();
    for index in 0..null_computed.profile.len() {
        let partner = null_computed.profile_index[index];
        if null_constant[index] || null_constant.get(partner).copied().unwrap_or(false) {
            null_masked.profile[index] = f64::INFINITY;
        }
    }
    let null_best_distance = find_motifs(&null_masked, 1).first().map(|m| m.distance);

    let max_distance = 2.0 * (m as f64).sqrt();
    let separation = match (
        masked_top.first().map(|pair| pair.distance),
        null_best_distance,
    ) {
        (Some(best), Some(null)) => Some((null - best) / max_distance),
        _ => None,
    };

    Some(WindowScan {
        window_ms,
        m,
        subsequences,
        constant_subsequences: constant_count,
        exclusion_zone,
        max_distance,
        raw_top,
        masked_top,
        discords,
        null_best_distance,
        null_constant_subsequences: null_constant.iter().filter(|flag| **flag).count(),
        separation,
    })
}

/// Per-subsequence constancy, by the library's own rolling standard deviation
/// and threshold. Deliberately not a second definition of "constant".
fn constant_flags(series: &[f64], m: usize, sigma_threshold: f64) -> Vec<bool> {
    RollingStats::compute_with_threshold(series, m, sigma_threshold)
        .std
        .iter()
        .map(|deviation| *deviation <= sigma_threshold)
        .collect()
}

fn span(index: usize, m: usize, bucket_ms: u64) -> Span {
    Span {
        index,
        start_ms: index as u64 * bucket_ms,
        end_ms: (index + m) as u64 * bucket_ms,
    }
}

fn pair(
    idx_a: usize,
    idx_b: usize,
    distance: f64,
    m: usize,
    bucket_ms: u64,
    constant: &[bool],
    series: &[f64],
) -> MotifPair {
    let occupancy = |index: usize| {
        series
            .get(index..(index + m).min(series.len()))
            .map(|window| window.iter().filter(|value| **value != 0.0).count())
            .unwrap_or(0)
    };
    MotifPair {
        a: span(idx_a, m, bucket_ms),
        b: span(idx_b, m, bucket_ms),
        distance,
        lag_samples: idx_a.abs_diff(idx_b),
        a_constant: constant.get(idx_a).copied().unwrap_or(false),
        b_constant: constant.get(idx_b).copied().unwrap_or(false),
        a_occupancy: occupancy(idx_a),
        b_occupancy: occupancy(idx_b),
    }
}

/// A fixed-seed Fisher-Yates shuffle.
///
/// Preserves the exact multiset of bucket values — so the dimension's total
/// count, its sparsity, and its marginal distribution are untouched — and
/// destroys every temporal relationship between them. It answers one question:
/// what does a good motif distance look like when the ordering carries no
/// information?
///
/// Deterministic because an oracle whose null moves between runs is not an
/// oracle. The generator is the same one the synthetic fixtures use.
pub fn deterministic_shuffle(series: &[f64]) -> Vec<f64> {
    let mut out = series.to_vec();
    let mut state: u64 = 0x5245_4652_4149_4E01;
    for index in (1..out.len()).rev() {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let pick = ((state >> 33) as usize) % (index + 1);
        out.swap(index, pick);
    }
    out
}
