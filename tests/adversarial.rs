//! sprint:15's adversarial commissioning of `rarity_of_agreements`.
//!
//! **Disposable.** Deleted with the round.
//!
//! **What these tests are for.** Two things. They pin the minimized
//! counterexamples so that a later round cannot lose them, and they assert the
//! statistic under test is still the frozen one — because a commissioning result
//! is worthless if the thing commissioned has moved since.
//!
//! **They do not repair anything.** task:25 forbids repairing the statistic and
//! re-running the gauntlet built against its predecessor, so every counterexample
//! here is expected to reproduce exactly as it did when it was found.

use std::collections::BTreeMap;

use witnessglass::experiment::adversarial::{self, UNDER_TEST, Verdict};
use witnessglass::experiment::identifiability::{Observation, SCORERS};

const EPSILON: f64 = 1e-9;

fn observation(
    agreeing: &[(usize, usize)],
    disagreements: usize,
    a_total: usize,
    b_total: usize,
) -> Observation {
    let mut a = Vec::new();
    let mut b = Vec::new();
    let mut a_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut b_counts: BTreeMap<String, usize> = BTreeMap::new();
    for (index, (in_a, in_b)) in agreeing.iter().enumerate() {
        let mark = format!("m{index}");
        a.push(mark.clone());
        b.push(mark.clone());
        a_counts.insert(mark.clone(), *in_a);
        b_counts.insert(mark, *in_b);
    }
    for index in 0..disagreements {
        a.push(format!("dl{index}"));
        b.push(format!("dr{index}"));
        a_counts.insert(format!("dl{index}"), 1);
        b_counts.insert(format!("dr{index}"), 1);
    }
    Observation {
        a,
        b,
        a_counts,
        b_counts,
        a_total,
        b_total,
    }
}

fn rarity(observation: &Observation) -> f64 {
    let scorer = SCORERS
        .iter()
        .find(|scorer| scorer.name == UNDER_TEST)
        .expect("the statistic under test");
    (scorer.score)(observation).expect("a score")
}

#[test]
fn the_statistic_under_test_is_still_the_frozen_one() {
    // A commissioning result is about a specific function. If it moves, the
    // result stops being about anything.
    let scorer = SCORERS
        .iter()
        .find(|scorer| scorer.name == UNDER_TEST)
        .expect("the statistic under test must remain in the preregistered family");
    assert!(
        scorer.probe,
        "it is still flagged a probe: this round evaluates it and does not adopt it"
    );

    // Its value on a hand-computable case: three agreements on marks of count
    // 50, 50 and 7 in a corpus of 1000, plus one disagreement contributing
    // nothing.
    let expected = -(50.0f64 / 1000.0).ln() * 2.0 - (7.0f64 / 1000.0).ln();
    let measured = rarity(&observation(&[(50, 50), (50, 50), (7, 7)], 1, 1000, 1000));
    assert!(
        (measured - expected).abs() < EPSILON,
        "{measured} against {expected}"
    );
}

#[test]
fn counterexample_ag3b_one_sided_rarity_is_exactly_blind_to_the_other_side() {
    // The minimized AG3b witness. Two candidates identical in A; in B one mark
    // occurs once and the other five hundred times. An agreement on a mark that
    // is everywhere in B is easy to obtain and should count for less. The two
    // score *identically*, because the statistic never reads B's counts.
    let rare_in_both = observation(&[(1, 1)], 3, 1_000, 1_000);
    let ubiquitous_in_b = observation(&[(1, 500)], 3, 1_000, 1_000);
    let (left, right) = (rarity(&rare_in_both), rarity(&ubiquitous_in_b));
    assert!(
        (left - right).abs() < EPSILON,
        "expected exact blindness, got {left} against {right}"
    );
    assert!((left - 1000.0f64.ln()).abs() < EPSILON);
}

#[test]
fn counterexample_ag1_a_singleton_outscores_a_four_agreement_motif() {
    // Minimized: a corpus of 100, a motif of four agreements on marks of count
    // 50, and one accidental agreement on a mark seen once.
    let lone = rarity(&observation(&[(1, 1)], 3, 100, 100));
    let motif = rarity(&observation(
        &[(50, 50), (50, 50), (50, 50), (50, 50)],
        0,
        100,
        100,
    ));
    assert!(
        lone > motif,
        "the counterexample must reproduce: lone {lone} against motif {motif}"
    );
    assert!((lone - 100.0f64.ln()).abs() < EPSILON);
    assert!((motif - 4.0 * 2.0f64.ln()).abs() < EPSILON);

    // And the analytic boundary: the motif loses exactly when c > N^(3/4).
    for (total, count, expected_motif_wins) in [
        (100usize, 20usize, true),
        (100, 50, false),
        (1_000, 100, true),
        (1_000, 200, false),
    ] {
        let lone = rarity(&observation(&[(1, 1)], 3, total, total));
        let motif = rarity(&observation(&[(count, count); 4], 0, total, total));
        assert_eq!(
            motif > lone,
            expected_motif_wins,
            "N={total} c={count}: boundary is c = N^(3/4) = {:.1}",
            (total as f64).powf(0.75)
        );
    }
}

#[test]
fn counterexample_ag5_appending_unrelated_events_reorders_unchanged_candidates() {
    // Minimized: neither candidate's marks or counts change. Only the corpus
    // grows, with events carrying marks neither candidate contains.
    let sparse = |total: usize| rarity(&observation(&[(2, 2)], 2, total, total));
    let broad = |total: usize| rarity(&observation(&[(300, 300); 3], 0, total, total));

    assert!(
        sparse(1_000) > broad(1_000),
        "the sparse candidate leads at N=1000"
    );
    assert!(
        broad(11_000) > sparse(11_000),
        "and loses at N=11000, unchanged"
    );

    // The crossing is where (k_X − k_Y)·ln N + (Σln c_Y − Σln c_X) changes sign.
    let crossing = ((3.0 * 300.0f64.ln() - 2.0f64.ln()) / 2.0).exp();
    assert!(
        (3_600.0..3_800.0).contains(&crossing),
        "analytic crossing at {crossing}"
    );
}

#[test]
fn whole_corpus_duplication_is_exactly_invariant() {
    // The one transformation the statistic is scale-free under, and the reason
    // AG6a passes while AG6b does not: duplicating everything leaves every c/N
    // unchanged.
    let reference = rarity(&observation(&[(50, 50), (7, 7)], 1, 1_000, 1_000));
    for factor in [2usize, 4, 10] {
        let scaled = rarity(&observation(
            &[(50 * factor, 50 * factor), (7 * factor, 7 * factor)],
            1,
            1_000 * factor,
            1_000 * factor,
        ));
        assert!(
            (scaled - reference).abs() < 1e-9,
            "×{factor} moved the score: {scaled} against {reference}"
        );
    }
}

#[test]
fn the_adversarial_families_reproduce_their_recorded_verdicts() {
    // The round's result, pinned. Recorded as the verdict each family produced
    // when it was run, so a later change to the statistic shows up here rather
    // than silently rewriting a commissioning conclusion.
    let expected = [
        ("AG1 singleton vs motif", Verdict::Mixed),
        ("AG2 rarity explosion", Verdict::Mixed),
        ("AG3 rare disagreement", Verdict::Pass),
        ("AG3b one-sided rarity", Verdict::Fail),
        ("AG4 common but structural", Verdict::Mixed),
        ("AG5 vocabulary growth", Verdict::Mixed),
        ("AG6a whole-corpus duplication", Verdict::Pass),
        ("AG6b background duplication", Verdict::Mixed),
        ("AG7 sample-size stability", Verdict::Pass),
        ("AG8 coincidence vs repetition", Verdict::Mixed),
    ];
    let families = adversarial::families();
    assert_eq!(families.len(), expected.len());
    for (family, (name, verdict)) in families.iter().zip(expected) {
        assert_eq!(family.name, name);
        assert_eq!(family.verdict, verdict, "{name}");
    }
}
