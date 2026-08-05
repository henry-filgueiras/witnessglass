//! Conditional match surprisal: the same permutation null, asked a conditional
//! question, and evaluated exactly.
//!
//! **Disposable.** sprint:13, task:23. A challenger to the statistic sprint:11
//! built and sprint:12 partly broke. It lives in its own module so that deleting
//! the challenge is deleting one file, and so that nothing in it can be mistaken
//! for the incumbent machinery, which it does not touch.
//!
//! # The defect it is built against
//!
//! [`crate::experiment::event_sequence::null_ensemble`] permutes **both** sides
//! independently, so the event being scored is *"two independently permuted
//! spans agree at least as well as the observed pair did"*. For an
//! exact-agreement candidate that probability is
//!
//! ```text
//! P = Σ over ordered sequences s of  P_A(s) · P_B(s)  ≈  c^L
//! ```
//!
//! in the two recordings' marginal collision rate `c`, and **it does not mention
//! the observed span**. Two spans of the same length in the same pair of
//! recordings have the same null tail whatever marks they contain. That is why
//! sprint:12's Family E — a boundary repeating a mark the core already carries,
//! against one seen nowhere else — came out at a median of `−0.003`.
//!
//! # The one change
//!
//! Hold one side's observed span **fixed** and permute only the other. The event
//! being scored then names the observed content, so the content enters the
//! probability.
//!
//! Let `a[0..L)` be one side's observed marks, `b[0..L)` the other's, and
//! `k = #{i : a[i] == b[i]}`. Under the null the permuted span is an ordered
//! sample without replacement of length `L` from that recording's whole mark
//! multiset. The statistic is
//!
//! ```text
//! S(A→B) = − ln P( a nulled B-span agrees with the observed a in ≥ k positions )
//! S      = ½ [ S(A→B) + S(B→A) ]                                        in nats
//! ```
//!
//! # Exact, not sampled
//!
//! For a set `T` of positions, the probability that a nulled span carries `a`'s
//! marks at exactly those positions is
//!
//! ```text
//! f(T) = ∏_m (c_m)_{t_m} / (N)_{|T|}
//! ```
//!
//! with `(x)_n` the falling factorial, `c_m` the mark's count in the recording,
//! `t_m` its count among `{a[i] : i ∈ T}`, and `N` the recording's length.
//! Jordan's formula then gives the tail exactly:
//!
//! ```text
//! P(≥ k) = Σ_{j ≥ k} (−1)^{j−k} · C(j−1, k−1) · Σ_{|T| = j} f(T)
//! ```
//!
//! At the span lengths this experiment uses that is at most a few hundred
//! subsets of arithmetic. There is no ensemble, no realization count, and no
//! Monte-Carlo floor to saturate against — which matters, because task:23 §1.2
//! records that the information this statistic is after sits at around `1e-8`.
//!
//! # What it is not
//!
//! No weight is chosen anywhere: mark counts enter through an exact probability
//! under a null that already existed. No threshold, no window, no free
//! parameter. It uses only event counts and positional agreement, so it is
//! defined over any sequence of timestamped categorical events and mentions
//! nothing about tools, agents, or this project.
//!
//! # Two costs, stated before it was run
//!
//! * **Undefined across indels.** A positional agreement count needs a
//!   positional correspondence, so spans of different lengths have no `k` and
//!   the statistic returns `None`. Callers count those rather than dropping them.
//! * **Blind to timing.** The incumbent's statistic is about the combined
//!   distance; this one is about categorical agreement alone. That is a genuine
//!   narrowing and this round does not repair it.

use std::collections::BTreeMap;

use super::event_sequence::EventSequence;

/// Longest span this evaluates. Above it the subset enumeration stops being
/// cheap, and nothing in this experiment reaches it.
pub const MAX_SPAN: usize = 12;

/// The falling factorial `(x)_n = x(x−1)…(x−n+1)`.
///
/// Zero when `n` exceeds `x`, which is the honest answer: a recording holding
/// two copies of a mark cannot place three.
fn falling(x: usize, n: usize) -> f64 {
    if n > x {
        return 0.0;
    }
    (0..n).map(|index| (x - index) as f64).product()
}

fn binomial(n: usize, k: usize) -> f64 {
    if k > n {
        return 0.0;
    }
    let mut result = 1.0;
    for index in 0..k {
        result = result * (n - index) as f64 / (index + 1) as f64;
    }
    result
}

/// Mark counts over a whole sequence, by label.
pub fn counts(sequence: &EventSequence<'_>) -> BTreeMap<String, usize> {
    let mut totals = BTreeMap::new();
    for event in &sequence.events {
        *totals.entry(event.mark.label()).or_insert(0) += 1;
    }
    totals
}

/// The marks of one span, as labels.
pub fn span_marks(sequence: &EventSequence<'_>, span: (usize, usize)) -> Option<Vec<String>> {
    let events = sequence.window(span.0, span.1.checked_sub(span.0)?)?;
    Some(events.iter().map(|event| event.mark.label()).collect())
}

/// `P(a nulled span of length `target.len()` drawn from `population` agrees with
/// `target` in at least `k` positions)`.
///
/// Exact. `None` when the span is longer than [`MAX_SPAN`] or the population is
/// too small to hold it.
pub fn agreement_tail(
    target: &[String],
    population: &BTreeMap<String, usize>,
    total: usize,
    k: usize,
) -> Option<f64> {
    let span = target.len();
    if span == 0 || span > MAX_SPAN || total < span || k > span {
        return None;
    }
    // Agreeing in at least zero positions is certain, and Jordan's formula's
    // `C(j−1, k−1)` is not defined there. Found by the brute-force test, which is
    // the only caller that reaches it.
    if k == 0 {
        return Some(1.0);
    }

    // `sums[j]` is Σ over position sets of size j of f(T).
    let mut sums = vec![0.0f64; span + 1];
    for mask in 0u32..(1u32 << span) {
        let size = mask.count_ones() as usize;
        if size < k {
            continue;
        }
        let mut wanted: BTreeMap<&str, usize> = BTreeMap::new();
        for (index, mark) in target.iter().enumerate() {
            if mask & (1 << index) != 0 {
                *wanted.entry(mark.as_str()).or_insert(0) += 1;
            }
        }
        let numerator: f64 = wanted
            .iter()
            .map(|(mark, needed)| falling(population.get(*mark).copied().unwrap_or(0), *needed))
            .product();
        if numerator == 0.0 {
            continue;
        }
        sums[size] += numerator / falling(total, size);
    }

    // Jordan's formula for the probability of at least k of the events.
    let mut tail = 0.0f64;
    for (j, sum) in sums.iter().enumerate().take(span + 1).skip(k) {
        let sign = if (j - k).is_multiple_of(2) { 1.0 } else { -1.0 };
        tail += sign * binomial(j - 1, k - 1) * sum;
    }
    // Arithmetic can leave a tail a hair outside `[0, 1]`; clamping keeps the
    // logarithm defined without inventing a value.
    Some(tail.clamp(f64::MIN_POSITIVE, 1.0))
}

/// One direction's surprisal, and what it was computed from.
#[derive(Debug, Clone, PartialEq)]
pub struct Direction {
    /// Positions at which the two observed spans agree.
    pub agreements: usize,
    /// Span length.
    pub span: usize,
    /// `P(≥ agreements)` under the conditional null.
    pub tail: f64,
    /// `− ln tail`, in nats.
    pub surprisal: f64,
}

/// The symmetric conditional match surprisal of one candidate, in nats.
///
/// `None` when the two spans have different lengths — a positional agreement
/// count needs a positional correspondence — or when either side is longer than
/// [`MAX_SPAN`].
pub fn surprisal(
    first: &EventSequence<'_>,
    span_a: (usize, usize),
    second: &EventSequence<'_>,
    span_b: (usize, usize),
) -> Option<f64> {
    let (a, b) = (span_marks(first, span_a)?, span_marks(second, span_b)?);
    if a.len() != b.len() {
        return None;
    }
    let agreements = a
        .iter()
        .zip(&b)
        .filter(|(left, right)| left == right)
        .count();

    let (a_counts, b_counts) = (counts(first), counts(second));
    // A→B: how surprising is it that a permutation of B reproduced what A
    // actually held, at least as well as B did. And symmetrically.
    let forward = agreement_tail(&a, &b_counts, second.len(), agreements)?;
    let backward = agreement_tail(&b, &a_counts, first.len(), agreements)?;
    Some(0.5 * (-forward.ln() + -backward.ln()))
}

/// The same, keeping both directions' working, for inspection.
pub fn detailed(
    first: &EventSequence<'_>,
    span_a: (usize, usize),
    second: &EventSequence<'_>,
    span_b: (usize, usize),
) -> Option<(Direction, Direction)> {
    let (a, b) = (span_marks(first, span_a)?, span_marks(second, span_b)?);
    if a.len() != b.len() {
        return None;
    }
    let agreements = a
        .iter()
        .zip(&b)
        .filter(|(left, right)| left == right)
        .count();
    let (a_counts, b_counts) = (counts(first), counts(second));
    let forward = agreement_tail(&a, &b_counts, second.len(), agreements)?;
    let backward = agreement_tail(&b, &a_counts, first.len(), agreements)?;
    Some((
        Direction {
            agreements,
            span: a.len(),
            tail: forward,
            surprisal: -forward.ln(),
        },
        Direction {
            agreements,
            span: b.len(),
            tail: backward,
            surprisal: -backward.ln(),
        },
    ))
}
