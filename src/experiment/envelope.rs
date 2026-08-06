//! Where two known failure surfaces lie relative to the corpus we actually have.
//!
//! **Disposable.** sprint:16, task:26. An exposure study: `rarity_of_agreements`
//! is frozen and nothing here repairs, normalizes, symmetrizes, replaces, or
//! adopts it.
//!
//! # What this measures, and what it does not
//!
//! sprint:15 derived two failure surfaces analytically and confirmed both
//! against a sweep. It could not say whether real recordings sit near them. This
//! module measures the quantities that place a recording relative to each
//! surface, and nothing else.
//!
//! **The corpus is not ground truth.** These recordings have no known true motif
//! boundaries, so nothing here measures whether the statistic is *right*. It
//! measures only how close the recordings come to configurations where the
//! statistic is known to misorder — and a threshold tuned on this corpus and
//! then called validated by it would be the error sprint:12 exists to catch.
//!
//! # The accumulation surface
//!
//! Every agreement contributes `−ln(c/N)`, so a single agreement on a mark of
//! count `c₁` outscores `k` agreements each on marks of count `c` exactly when
//!
//! ```text
//! c^k / c₁ > N^{k−1}          and, for a singleton c₁ = 1,        c > N^{(k−1)/k}
//! ```
//!
//! Three measured quantities place a recording: `N`, the largest mark count, and
//! whether any mark occurs once.
//!
//! # The asymmetry surface
//!
//! `score(A,B)` reads only A's marginals, so
//!
//! ```text
//! score(A,B) − score(B,A) = Σ over agreeing marks of ln( (ĉ_B(m)/N_B) / (ĉ_A(m)/N_A) )
//! ```
//!
//! which is zero only when every agreeing mark holds the same relative frequency
//! in both recordings.

use std::collections::BTreeMap;

use serde::Serialize;

use super::event_sequence::EventSequence;
use super::identifiability::{Observation, SCORERS};

/// The statistic under study, by name in [`SCORERS`].
pub const UNDER_STUDY: &str = "rarity_of_agreements";

/// One mark's presence in a recording.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MarkFrequency {
    /// The delivered mark, verbatim.
    pub mark: String,
    /// How many events carry it.
    pub count: usize,
    /// Its empirical frequency.
    pub frequency: f64,
}

/// Everything the study measures about one recording.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Profile {
    /// Session id, truncated for reporting.
    pub session: String,
    /// Events in the scope the machinery uses.
    pub events: usize,
    /// Distinct marks.
    pub vocabulary: usize,
    /// Every mark, by descending count.
    pub frequencies: Vec<MarkFrequency>,
    /// Largest mark count, and the mark holding it.
    pub max_count: usize,
    /// Smallest nonzero mark count.
    pub min_count: usize,
    /// Marks occurring exactly once — the `c₁ = 1` the sharpest boundary needs.
    pub singletons: usize,
    /// Deciles of the frequency distribution, low to high.
    pub frequency_deciles: Vec<f64>,
}

/// Characterize one recording. Reads counts only; no content of any kind.
pub fn profile(sequence: &EventSequence<'_>) -> Profile {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for event in &sequence.events {
        *counts.entry(event.mark.label()).or_insert(0) += 1;
    }
    let events = sequence.len().max(1);
    let mut frequencies: Vec<MarkFrequency> = counts
        .into_iter()
        .map(|(mark, count)| MarkFrequency {
            mark,
            count,
            frequency: count as f64 / events as f64,
        })
        .collect();
    frequencies.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then(left.mark.cmp(&right.mark))
    });

    let mut ascending: Vec<f64> = frequencies.iter().map(|entry| entry.frequency).collect();
    ascending.sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));
    let decile = |fraction: f64| -> f64 {
        if ascending.is_empty() {
            return f64::NAN;
        }
        let position = fraction * (ascending.len() - 1) as f64;
        let lower = position.floor() as usize;
        let upper = position.ceil() as usize;
        if lower == upper {
            ascending[lower]
        } else {
            let weight = position - lower as f64;
            ascending[lower] * (1.0 - weight) + ascending[upper] * weight
        }
    };

    Profile {
        session: sequence
            .session_id
            .map(|id| id.chars().take(8).collect())
            .unwrap_or_else(|| "<none>".to_owned()),
        events: sequence.len(),
        vocabulary: frequencies.len(),
        max_count: frequencies.first().map(|entry| entry.count).unwrap_or(0),
        min_count: frequencies.last().map(|entry| entry.count).unwrap_or(0),
        singletons: frequencies.iter().filter(|entry| entry.count == 1).count(),
        frequency_deciles: (0..=10).map(|step| decile(step as f64 / 10.0)).collect(),
        frequencies,
    }
}

/// How close one recording sits to the accumulation surface, at one `k`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Approach {
    /// Agreements in the motif the singleton is competing against.
    pub k: usize,
    /// `N^{(k−1)/k}` — the count above which a `k`-agreement motif of marks that
    /// common loses to one singleton agreement.
    pub boundary: f64,
    /// The recording's largest mark count.
    pub max_count: usize,
    /// `max_count − boundary`. Positive means the recording holds a mark common
    /// enough to lose.
    pub absolute_margin: f64,
    /// `max_count / boundary`. Above one means the same, scale-free.
    pub relative_margin: f64,
    /// Marks at or above the boundary.
    pub marks_above: usize,
    /// Whether the recording also holds a singleton, which the sharpest form of
    /// the boundary requires.
    pub has_singleton: bool,
    /// Whether both ingredients are present, so an adversarially constructed
    /// candidate could cross. **Not** a claim that one was observed.
    pub constructible: bool,
}

/// Evaluate the accumulation surface for one recording across a range of `k`.
pub fn approaches(profile: &Profile, span_lengths: &[usize]) -> Vec<Approach> {
    let total = profile.events.max(1) as f64;
    span_lengths
        .iter()
        .filter(|k| **k >= 2)
        .map(|k| {
            let boundary = total.powf((*k as f64 - 1.0) / *k as f64);
            let marks_above = profile
                .frequencies
                .iter()
                .filter(|entry| entry.count as f64 >= boundary)
                .count();
            let has_singleton = profile.singletons > 0;
            Approach {
                k: *k,
                boundary,
                max_count: profile.max_count,
                absolute_margin: profile.max_count as f64 - boundary,
                relative_margin: profile.max_count as f64 / boundary,
                marks_above,
                has_singleton,
                constructible: marks_above > 0 && has_singleton,
            }
        })
        .collect()
}

/// One real candidate pair, scored both ways round.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AsymmetrySample {
    /// Which recordings, truncated.
    pub a_session: String,
    /// The other.
    pub b_session: String,
    /// Where the candidate came from, for provenance.
    pub origin: String,
    /// Span length.
    pub span: usize,
    /// Positions at which the two spans agree.
    pub agreements: usize,
    /// `score(A, B)`.
    pub forward: f64,
    /// `score(B, A)`.
    pub backward: f64,
    /// `|forward − backward|`.
    pub delta: f64,
}

/// The statistic a replay is conducted with.
pub type Stat = fn(&Observation) -> Option<f64>;

/// The statistic sprint:16 studied, for callers that want its original run.
pub fn under_study() -> Option<Stat> {
    SCORERS
        .iter()
        .find(|scorer| scorer.name == UNDER_STUDY)
        .map(|scorer| scorer.score)
}

/// Score one candidate both ways round.
///
/// Swapping means exchanging **both** the sequences and the spans, so the same
/// pair of windows is scored with the two recordings' roles reversed.
pub fn asymmetry_of(
    first: &EventSequence<'_>,
    span_a: (usize, usize),
    second: &EventSequence<'_>,
    span_b: (usize, usize),
    origin: &str,
) -> Option<AsymmetrySample> {
    asymmetry_with(under_study()?, first, span_a, second, span_b, origin)
}

/// The same measurement under any statistic. sprint:17, task:27 §E.
pub fn asymmetry_with(
    stat: Stat,
    first: &EventSequence<'_>,
    span_a: (usize, usize),
    second: &EventSequence<'_>,
    span_b: (usize, usize),
    origin: &str,
) -> Option<AsymmetrySample> {
    let forward_view = Observation::of(first, span_a, second, span_b)?;
    let backward_view = Observation::of(second, span_b, first, span_a)?;
    let forward = (stat)(&forward_view)?;
    let backward = (stat)(&backward_view)?;
    Some(AsymmetrySample {
        a_session: first
            .session_id
            .map(|id| id.chars().take(8).collect())
            .unwrap_or_else(|| "<none>".to_owned()),
        b_session: second
            .session_id
            .map(|id| id.chars().take(8).collect())
            .unwrap_or_else(|| "<none>".to_owned()),
        origin: origin.to_owned(),
        span: forward_view.a.len(),
        agreements: forward_view.agreeing().len(),
        forward,
        backward,
        delta: (forward - backward).abs(),
    })
}

/// Whether reversing the arguments reorders a set of candidates.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OrderingCheck {
    /// Which candidate set.
    pub origin: String,
    /// Candidates compared.
    pub candidates: usize,
    /// The highest-scoring candidate under `score(A,B)`, by index.
    pub forward_pick: usize,
    /// The highest-scoring candidate under `score(B,A)`.
    pub backward_pick: usize,
    /// Whether the designated pick moved.
    pub pick_changed: bool,
    /// Pairs of candidates whose relative order reversed.
    pub inversions: usize,
    /// Pairs compared.
    pub comparisons: usize,
}

/// Compare two rankings of the same candidate set.
pub fn ordering_check(origin: &str, samples: &[AsymmetrySample]) -> Option<OrderingCheck> {
    if samples.len() < 2 {
        return None;
    }
    let best = |select: fn(&AsymmetrySample) -> f64| -> usize {
        samples
            .iter()
            .enumerate()
            .max_by(|left, right| {
                select(left.1)
                    .partial_cmp(&select(right.1))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(index, _)| index)
            .unwrap_or(0)
    };
    let forward_pick = best(|sample| sample.forward);
    let backward_pick = best(|sample| sample.backward);

    let mut inversions = 0usize;
    let mut comparisons = 0usize;
    for (index, left) in samples.iter().enumerate() {
        for right in samples.iter().skip(index + 1) {
            comparisons += 1;
            let forward_order = left.forward.partial_cmp(&right.forward);
            let backward_order = left.backward.partial_cmp(&right.backward);
            if forward_order != backward_order {
                inversions += 1;
            }
        }
    }

    Some(OrderingCheck {
        origin: origin.to_owned(),
        candidates: samples.len(),
        forward_pick,
        backward_pick,
        pick_changed: forward_pick != backward_pick,
        inversions,
        comparisons,
    })
}

/// Quantiles of a sample, for reporting a distribution rather than a mean.
pub fn quantiles(values: &[f64]) -> Vec<f64> {
    if values.is_empty() {
        return Vec::new();
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));
    [0.0, 0.25, 0.5, 0.75, 0.9, 1.0]
        .iter()
        .map(|fraction| {
            let position = fraction * (sorted.len() - 1) as f64;
            let lower = position.floor() as usize;
            let upper = position.ceil() as usize;
            if lower == upper {
                sorted[lower]
            } else {
                let weight = position - lower as f64;
                sorted[lower] * (1.0 - weight) + sorted[upper] * weight
            }
        })
        .collect()
}

/// An observed crossing: a real candidate that outscores one with strictly more
/// agreements.
///
/// This is the **first** clause of task:26 §E's L1 test, and it is a different
/// claim from "the corpus holds parameter values from which one could be
/// constructed". A crossing here was produced by the unmodified machinery on
/// real recordings; nothing was built to provoke it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Crossing {
    /// Which candidate set.
    pub origin: String,
    /// Agreements of the candidate that scored higher.
    pub fewer_agreements: usize,
    /// Its score.
    pub fewer_score: f64,
    /// Agreements of the candidate it beat.
    pub more_agreements: usize,
    /// Its score.
    pub more_score: f64,
    /// How much the weaker-supported candidate won by, in nats.
    pub margin: f64,
}

/// Every accumulation crossing inside one candidate set.
///
/// Candidates within a set share a span length, so a difference in agreements is
/// a difference in how much of the span actually matched.
pub fn crossings(origin: &str, samples: &[AsymmetrySample]) -> Vec<Crossing> {
    let mut found = Vec::new();
    for left in samples {
        for right in samples {
            if left.agreements < right.agreements && left.forward > right.forward {
                found.push(Crossing {
                    origin: origin.to_owned(),
                    fewer_agreements: left.agreements,
                    fewer_score: left.forward,
                    more_agreements: right.agreements,
                    more_score: right.forward,
                    margin: left.forward - right.forward,
                });
            }
        }
    }
    found
}
