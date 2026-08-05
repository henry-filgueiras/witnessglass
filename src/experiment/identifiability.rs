//! The mark-only representation, and a preregistered family of functions over it.
//!
//! **Disposable.** sprint:14, task:24. A representation audit, not a statistic:
//! nothing here is offered as a repair or a scorer to adopt, and the module
//! exists to answer whether the information is present rather than to find a
//! function that uses it.
//!
//! # What a scorer can see
//!
//! ```text
//! R(candidate) = ( ā , b̄ , ĉ_A , ĉ_B , N_A , N_B )
//! ```
//!
//! the two spans' marks in order, each mark's count over the whole recording it
//! came from, and the two recording lengths — **up to a bijective relabelling of
//! the mark alphabet applied consistently to all four**. A mark is an opaque
//! label: a scorer may test two marks for equality and may look up a count, and
//! may do nothing else with one. Equivalently, `R` determines the equality
//! pattern of the two spans — which positions share a mark, within each and
//! across them — together with each occurring mark's count and the two lengths.
//!
//! [`Observation`] is that representation as a value, and every function in
//! [`SCORERS`] takes one. Nothing in this module can reach the timing policy,
//! a path, a payload, a channel, an adapter, a schema version, an agent, or the
//! text of a tool name beyond comparing it to another tool name.
//!
//! # The collision question, settled in task:24 §A.2
//!
//! Family E's arms are **not** `R`-identical: their equality patterns differ, and
//! [`Observation::tail_repeats_span`] separates them while testing only equality
//! and surviving any relabelling. So the distinction is identifiable and the
//! failures in sprint:12 and sprint:13 were not identifiability failures.
//!
//! A collision does exist for the stronger claim that *semantic* redundancy is a
//! function of `R`, and [`crate::experiment::identifiability::witness`] holds the
//! minimal one. It is a statement about the word, not about Family E.

use std::collections::BTreeMap;

use super::conditional_null;
use super::event_sequence::EventSequence;

/// One candidate, as the mark-only representation presents it.
///
/// Constructed from two sequences and two spans, and then closed: nothing a
/// scorer receives can reach back to the recording.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Observation {
    /// Span A's marks, in order.
    pub a: Vec<String>,
    /// Span B's marks, in order.
    pub b: Vec<String>,
    /// Every mark's count over the whole of recording A.
    pub a_counts: BTreeMap<String, usize>,
    /// Every mark's count over the whole of recording B.
    pub b_counts: BTreeMap<String, usize>,
    /// Recording A's length in events.
    pub a_total: usize,
    /// Recording B's length in events.
    pub b_total: usize,
}

impl Observation {
    /// Build the representation of one candidate.
    pub fn of(
        first: &EventSequence<'_>,
        span_a: (usize, usize),
        second: &EventSequence<'_>,
        span_b: (usize, usize),
    ) -> Option<Self> {
        Some(Self {
            a: conditional_null::span_marks(first, span_a)?,
            b: conditional_null::span_marks(second, span_b)?,
            a_counts: conditional_null::counts(first),
            b_counts: conditional_null::counts(second),
            a_total: first.len(),
            b_total: second.len(),
        })
    }

    /// Span length, when the two sides agree on one.
    ///
    /// `None` across an indel: positional questions need a positional
    /// correspondence, and every function here asks one.
    pub fn len(&self) -> Option<usize> {
        (self.a.len() == self.b.len()).then_some(self.a.len())
    }

    /// Whether the representation carries no span at all.
    pub fn is_empty(&self) -> bool {
        self.a.is_empty() || self.b.is_empty()
    }

    /// Positions where the two spans carry the same mark.
    pub fn agreeing(&self) -> Vec<usize> {
        self.a
            .iter()
            .zip(&self.b)
            .enumerate()
            .filter(|(_, (left, right))| left == right)
            .map(|(index, _)| index)
            .collect()
    }

    /// Whether span A's last mark occurs earlier in span A.
    ///
    /// task:24 §A.2's discriminator. It tests only mark equality, never mark
    /// identity, so it survives any relabelling — which is the whole point: it
    /// is a function of `R`, and it separates Family E's arms.
    pub fn tail_repeats_span(&self) -> Option<bool> {
        let (last, rest) = self.a.split_last()?;
        Some(rest.contains(last))
    }

    /// The equality pattern of the two spans: the only thing about the marks
    /// that survives relabelling.
    ///
    /// Row `i` of the within-span matrix, then the across-span vector. Used by
    /// the witness test to show two candidates are `R`-identical.
    pub fn equality_pattern(&self) -> (Vec<Vec<bool>>, Vec<bool>) {
        let within = self
            .a
            .iter()
            .map(|left| self.a.iter().map(|right| left == right).collect())
            .collect();
        let across = self
            .a
            .iter()
            .zip(&self.b)
            .map(|(left, right)| left == right)
            .collect();
        (within, across)
    }
}

/// One preregistered function of the representation.
pub struct Scorer {
    /// Its name in the matrix.
    pub name: &'static str,
    /// What it computes, in words.
    pub definition: &'static str,
    /// Whether task:24 §B.1 admits it only as a probe. A probe may appear in the
    /// matrix and may **not** be proposed as a statistic: functions 9 and 10 are
    /// inverse-frequency weightings, which sprints 12 and 13 forbid as repairs.
    pub probe: bool,
    /// Higher means more evidence. `None` where the candidate has no positional
    /// correspondence.
    pub score: fn(&Observation) -> Option<f64>,
}

fn distinct(marks: &[String]) -> usize {
    let mut seen: Vec<&str> = Vec::new();
    for mark in marks {
        if !seen.contains(&mark.as_str()) {
            seen.push(mark);
        }
    }
    seen.len()
}

fn agreements(observation: &Observation) -> Option<f64> {
    observation.len()?;
    Some(observation.agreeing().len() as f64)
}

fn agreement_rate(observation: &Observation) -> Option<f64> {
    let len = observation.len()?;
    (len > 0).then(|| observation.agreeing().len() as f64 / len as f64)
}

fn distinct_agreements(observation: &Observation) -> Option<f64> {
    observation.len()?;
    let marks: Vec<String> = observation
        .agreeing()
        .into_iter()
        .map(|index| observation.a[index].clone())
        .collect();
    Some(distinct(&marks) as f64)
}

fn distinct_agreement_rate(observation: &Observation) -> Option<f64> {
    let len = observation.len()?;
    let value = distinct_agreements(observation)?;
    (len > 0).then_some(value / len as f64)
}

fn span_distinct(observation: &Observation) -> Option<f64> {
    observation.len()?;
    Some(distinct(&observation.a) as f64)
}

/// Agreeing positions whose mark has not already occurred earlier in span A.
fn first_occurrence_agreements(observation: &Observation) -> Option<f64> {
    observation.len()?;
    Some(
        observation
            .agreeing()
            .into_iter()
            .filter(|index| !observation.a[..*index].contains(&observation.a[*index]))
            .count() as f64,
    )
}

fn negative_repeats(observation: &Observation) -> Option<f64> {
    let len = observation.len()?;
    Some(-((len - distinct(&observation.a)) as f64))
}

fn surprisal(observation: &Observation) -> Option<f64> {
    let agreeing = observation.agreeing().len();
    observation.len()?;
    let forward = conditional_null::agreement_tail(
        &observation.a,
        &observation.b_counts,
        observation.b_total,
        agreeing,
    )?;
    let backward = conditional_null::agreement_tail(
        &observation.b,
        &observation.a_counts,
        observation.a_total,
        agreeing,
    )?;
    Some(0.5 * (-forward.ln() + -backward.ln()))
}

/// `Σ over agreeing positions of −ln(count / N)`. **A probe.**
fn rarity_of_agreements(observation: &Observation) -> Option<f64> {
    observation.len()?;
    Some(
        observation
            .agreeing()
            .into_iter()
            .map(|index| {
                let count = observation
                    .a_counts
                    .get(&observation.a[index])
                    .copied()
                    .unwrap_or(1)
                    .max(1);
                -((count as f64 / observation.a_total.max(1) as f64).ln())
            })
            .sum(),
    )
}

/// The same, restricted to first occurrences within the span. **A probe.**
fn novel_rarity(observation: &Observation) -> Option<f64> {
    observation.len()?;
    Some(
        observation
            .agreeing()
            .into_iter()
            .filter(|index| !observation.a[..*index].contains(&observation.a[*index]))
            .map(|index| {
                let count = observation
                    .a_counts
                    .get(&observation.a[index])
                    .copied()
                    .unwrap_or(1)
                    .max(1);
                -((count as f64 / observation.a_total.max(1) as f64).ln())
            })
            .sum(),
    )
}

/// The ten functions task:24 §B.1 fixed, in that order. None was added, removed,
/// or edited after evaluation.
pub const SCORERS: [Scorer; 10] = [
    Scorer {
        name: "agreements",
        definition: "positions where the two spans carry the same mark",
        probe: false,
        score: agreements,
    },
    Scorer {
        name: "agreement_rate",
        definition: "agreements divided by span length",
        probe: false,
        score: agreement_rate,
    },
    Scorer {
        name: "distinct_agreements",
        definition: "distinct marks among agreeing positions",
        probe: false,
        score: distinct_agreements,
    },
    Scorer {
        name: "distinct_agreement_rate",
        definition: "distinct agreeing marks divided by span length",
        probe: false,
        score: distinct_agreement_rate,
    },
    Scorer {
        name: "span_distinct",
        definition: "distinct marks in span A, sprint:8's diagnostic as a score",
        probe: false,
        score: span_distinct,
    },
    Scorer {
        name: "first_occurrence_agreements",
        definition: "agreeing positions whose mark has not occurred earlier in the span",
        probe: false,
        score: first_occurrence_agreements,
    },
    Scorer {
        name: "negative_repeats",
        definition: "minus the number of repeated marks in span A",
        probe: false,
        score: negative_repeats,
    },
    Scorer {
        name: "surprisal",
        definition: "sprint:13's conditional match surprisal",
        probe: false,
        score: surprisal,
    },
    Scorer {
        name: "rarity_of_agreements",
        definition: "sum over agreeing positions of minus log relative count — a probe",
        probe: true,
        score: rarity_of_agreements,
    },
    Scorer {
        name: "novel_rarity",
        definition: "the same sum over first occurrences only — a probe",
        probe: true,
        score: novel_rarity,
    },
];

/// The minimal collision certificate for the claim that *semantic* redundancy is
/// a function of `R`.
///
/// task:24 §A.3. Two candidates whose representation is identical — same spans,
/// same counts, same lengths — and whose desired orderings are opposite:
///
/// * **P** a three-event core `(x, y, z)` extended by a repeat of `x` that adds
///   nothing;
/// * **Q** a four-event figure whose defining property is *returning to `x`*,
///   where the final `x` is the informative event.
///
/// No function of `R` can order these two differently, because no function can
/// distinguish inputs it cannot tell apart. This bounds what an `R`-based scorer
/// can mean by "redundant" — and it does **not** license changing Family E,
/// whose arms are separable because sprint:12 defined redundancy syntactically.
pub mod witness {
    use super::Observation;
    use std::collections::BTreeMap;

    fn labels(marks: &[&str]) -> Vec<String> {
        marks.iter().map(|mark| (*mark).to_owned()).collect()
    }

    fn population(pairs: &[(&str, usize)]) -> BTreeMap<String, usize> {
        pairs
            .iter()
            .map(|(mark, count)| ((*mark).to_owned(), *count))
            .collect()
    }

    /// One arm of the witness. Both arms return the identical value; the
    /// argument names which reading it is meant to stand for, and changes
    /// nothing, which is the point.
    pub fn arm(_reading: &str) -> Observation {
        Observation {
            a: labels(&["x", "y", "z", "x"]),
            b: labels(&["x", "y", "z", "x"]),
            a_counts: population(&[("x", 2), ("y", 1), ("z", 1), ("bg", 40)]),
            b_counts: population(&[("x", 2), ("y", 1), ("z", 1), ("bg", 40)]),
            a_total: 44,
            b_total: 44,
        }
    }

    /// The pair. Equal by construction, and a test asserts it.
    pub fn pair() -> (Observation, Observation) {
        (
            arm("a core plus a repeat that adds nothing"),
            arm("a figure whose defining property is returning to its first mark"),
        )
    }
}
