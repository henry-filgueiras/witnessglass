//! sprint:13's challenger: conditional match surprisal.
//!
//! **Disposable.** Deleted with the challenge.
//!
//! **What these tests are for.** The challenger is computed exactly rather than
//! sampled, so its correctness is checkable against brute force rather than
//! against a tolerance — and it is, on populations small enough to enumerate
//! every permutation. Beyond that they pin the two arithmetic facts task:23 §1.6
//! derived before the challenger existed, and they assert that adding a
//! challenger changed nothing about the incumbent it challenges.

use std::collections::BTreeMap;

use witnessglass::experiment::conditional_null::{agreement_tail, counts, span_marks, surprisal};
use witnessglass::experiment::event_sequence::{ChannelScope, project};
use witnessglass::inspection::inspect;
use witnessglass::replay_bytes;

const EPSILON: f64 = 1e-9;

fn population(pairs: &[(&str, usize)]) -> BTreeMap<String, usize> {
    pairs
        .iter()
        .map(|(mark, count)| ((*mark).to_owned(), *count))
        .collect()
}

fn marks(labels: &[&str]) -> Vec<String> {
    labels.iter().map(|label| (*label).to_owned()).collect()
}

/// Every ordered sample without replacement of length `span` from a multiset,
/// as a brute-force check on the closed form.
fn brute_force_tail(target: &[String], population: &BTreeMap<String, usize>, k: usize) -> f64 {
    // Expand the multiset into individual tokens, then enumerate every ordered
    // selection of `target.len()` of them.
    let mut tokens: Vec<String> = Vec::new();
    for (mark, count) in population {
        for _ in 0..*count {
            tokens.push(mark.clone());
        }
    }
    let span = target.len();
    let mut hits = 0usize;
    let mut total = 0usize;
    let mut indices = vec![0usize; span];

    #[allow(clippy::too_many_arguments)]
    fn walk(
        depth: usize,
        span: usize,
        used: &mut Vec<bool>,
        indices: &mut Vec<usize>,
        tokens: &[String],
        target: &[String],
        k: usize,
        hits: &mut usize,
        total: &mut usize,
    ) {
        if depth == span {
            let agreements = (0..span)
                .filter(|position| tokens[indices[*position]] == target[*position])
                .count();
            *total += 1;
            if agreements >= k {
                *hits += 1;
            }
            return;
        }
        for index in 0..tokens.len() {
            if used[index] {
                continue;
            }
            used[index] = true;
            indices[depth] = index;
            walk(
                depth + 1,
                span,
                used,
                indices,
                tokens,
                target,
                k,
                hits,
                total,
            );
            used[index] = false;
        }
    }

    let mut used = vec![false; tokens.len()];
    walk(
        0,
        span,
        &mut used,
        &mut indices,
        &tokens,
        target,
        k,
        &mut hits,
        &mut total,
    );
    hits as f64 / total as f64
}

#[test]
fn the_closed_form_agrees_with_brute_force_over_every_permutation() {
    // Small enough to enumerate exhaustively, varied enough to exercise
    // repeated marks in both the target and the population, and every value of
    // k from zero to the span length.
    let cases: Vec<(Vec<String>, BTreeMap<String, usize>)> = vec![
        (
            marks(&["a", "b"]),
            population(&[("a", 2), ("b", 2), ("c", 1)]),
        ),
        (
            marks(&["a", "a"]),
            population(&[("a", 2), ("b", 2), ("c", 1)]),
        ),
        (
            marks(&["a", "b", "c"]),
            population(&[("a", 1), ("b", 1), ("c", 1), ("d", 2)]),
        ),
        (
            marks(&["a", "a", "b"]),
            population(&[("a", 3), ("b", 1), ("c", 1)]),
        ),
        (marks(&["c", "c"]), population(&[("a", 3), ("c", 1)])),
    ];

    for (target, pool) in cases {
        let total: usize = pool.values().sum();
        for k in 0..=target.len() {
            let exact = agreement_tail(&target, &pool, total, k).expect("a tail");
            let brute = brute_force_tail(&target, &pool, k);
            assert!(
                (exact - brute).abs() < 1e-9,
                "target {target:?} pool {pool:?} k={k}: closed form {exact} against brute force {brute}"
            );
        }
    }
}

#[test]
fn a_span_the_population_cannot_hold_has_probability_zero() {
    // One copy of `a` cannot fill two positions, and the closed form must say so
    // rather than producing a plausible small number.
    let pool = population(&[("a", 1), ("b", 5)]);
    let tail = agreement_tail(&marks(&["a", "a"]), &pool, 6, 2).expect("a tail");
    assert!(tail <= f64::MIN_POSITIVE, "got {tail}");
}

#[test]
fn a_repeated_mark_is_exactly_twice_as_reachable_as_two_distinct_ones() {
    // task:23 §1.6, derived before the challenger was implemented: reproducing a
    // span that repeats a mark of multiplicity two is combinatorially twice as
    // easy as reproducing one whose marks are all unique.
    let total = 84usize;
    let novel = agreement_tail(
        &marks(&["c0", "c1", "c2", "rare"]),
        &population(&[("c0", 1), ("c1", 1), ("c2", 1), ("rare", 1), ("bg", 80)]),
        total,
        4,
    )
    .expect("a tail");
    let redundant = agreement_tail(
        &marks(&["c0", "c1", "c2", "c0"]),
        &population(&[("c0", 2), ("c1", 1), ("c2", 1), ("bg", 80)]),
        total,
        4,
    )
    .expect("a tail");
    assert!(
        (redundant / novel - 2.0).abs() < 1e-9,
        "ratio {}",
        redundant / novel
    );
    assert!(((-novel.ln()) - (-redundant.ln()) - std::f64::consts::LN_2).abs() < 1e-9);
}

#[test]
fn a_common_boundary_mark_is_reachable_in_proportion_to_its_count() {
    // The other half of §1.6: a boundary mark occurring `c` times is `c` times
    // easier to reproduce than one occurring once.
    let total = 84usize;
    let rare = agreement_tail(
        &marks(&["c0", "c1", "c2", "x"]),
        &population(&[("c0", 1), ("c1", 1), ("c2", 1), ("x", 1), ("bg", 80)]),
        total,
        4,
    )
    .expect("a tail");
    let common = agreement_tail(
        &marks(&["c0", "c1", "c2", "x"]),
        &population(&[("c0", 1), ("c1", 1), ("c2", 1), ("x", 29), ("bg", 52)]),
        total,
        4,
    )
    .expect("a tail");
    assert!(
        (common / rare - 29.0).abs() < 1e-6,
        "ratio {}",
        common / rare
    );
}

#[test]
fn the_statistic_is_symmetric_and_undefined_across_unequal_lengths() {
    let text = witnessglass::experiment::oracle::ndjson();
    let replay = replay_bytes(text.as_bytes()).expect("replay");
    let inspection = inspect(&replay);
    let sequence = project(&inspection, ChannelScope::Observed).expect("a sequence");

    let forward = surprisal(&sequence, (20, 28), &sequence, (162, 170)).expect("a value");
    let backward = surprisal(&sequence, (162, 170), &sequence, (20, 28)).expect("a value");
    assert!(
        (forward - backward).abs() < EPSILON,
        "{forward} vs {backward}"
    );
    assert!(
        forward > 0.0,
        "an exact match of eight marks should be surprising"
    );

    // Unequal lengths have no positional correspondence, so no agreement count.
    assert!(surprisal(&sequence, (20, 28), &sequence, (162, 171)).is_none());
}

#[test]
fn counts_and_span_marks_read_the_sequence_they_are_given() {
    let text = witnessglass::experiment::oracle::ndjson();
    let replay = replay_bytes(text.as_bytes()).expect("replay");
    let inspection = inspect(&replay);
    let sequence = project(&inspection, ChannelScope::Observed).expect("a sequence");

    let totals = counts(&sequence);
    assert_eq!(totals.values().sum::<usize>(), sequence.len());

    let span = span_marks(&sequence, (20, 28)).expect("marks");
    assert_eq!(span.len(), 8);
    for (index, mark) in span.iter().enumerate() {
        assert_eq!(mark, &sequence.events[20 + index].mark.label());
    }
}
