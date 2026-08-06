//! sprint:18, task:28 — the discriminating commissioning of R1 pooled sum.
//!
//! These tests bank task:28's derivations, including the two that go against
//! R1 and the criterion defect this round contributed. Each asserts a closed
//! form computed before the gauntlet existed, so a later change that quietly
//! invalidates one fails here rather than in a report.

use witnessglass::experiment::discriminating::{self, Outcome, Rule, Verdict, families, verdict};
use witnessglass::experiment::{adversarial, repair};

fn family(name: &str) -> discriminating::FamilyResult {
    families()
        .into_iter()
        .find(|entry| entry.name.starts_with(name))
        .unwrap_or_else(|| panic!("family `{name}` must remain preregistered"))
}

/// task:28 §PHASE 1 — the proof, exercised numerically. Requirement B applied
/// to k copies of a rare mark against k+1 of a common one forces the weight
/// ratio below (k+1)/k for every k, which drives it to 1.
#[test]
fn agreement_dominance_forces_the_weight_ratio_to_one() {
    let ratio_bound = |k: f64| (k + 1.0) / k;
    let mut previous = f64::INFINITY;
    for k in 1..=10_000u32 {
        let bound = ratio_bound(f64::from(k));
        assert!(bound < previous, "the admissible ratio must be decreasing");
        assert!(bound > 1.0, "and must stay above 1 at every finite k");
        previous = bound;
    }
    assert!(
        (previous - 1.0).abs() < 1e-3,
        "so in the limit only a constant weight satisfies agreement dominance"
    );
}

/// The same claim from the other side: at the frozen ladder's longest span,
/// buying agreement dominance costs almost all of the rarity weighting.
#[test]
fn agreement_dominance_at_the_frozen_span_admits_almost_no_rarity_weighting() {
    let k = 12.0f64;
    let admissible_weight_ratio = 1.0 + 1.0 / k;
    assert!((admissible_weight_ratio - 1.0833).abs() < 1e-4);
    // A weight ratio of w_max/w_min translates to a frequency ratio at N=1000.
    let frequency_ratio = (1000f64.ln() * (1.0 / k)).exp();
    assert!(
        frequency_ratio < 1.8,
        "a frequency ratio under {frequency_ratio:.3}x is not rarity weighting"
    );
}

/// F0 — the control. The fresh harness must reproduce sprint:17 §D4.
#[test]
fn the_control_family_confirms_the_two_statistics_are_identical_at_shared_marginals() {
    let f0 = family("F0");
    assert_eq!(f0.outcome, Outcome::Held);
    assert!(!f0.discriminating, "the control discriminates nothing");
    for point in &f0.points {
        assert_eq!(
            point.frozen, point.pooled,
            "S0 and R1 must be bit-identical at shared marginals"
        );
    }
}

/// F3 — the family that carries the round. S0 is blind to B's marginal and R1
/// is not, at exactly the predicted values.
#[test]
fn the_frozen_statistic_is_blind_to_bs_marginal_and_the_pooled_one_is_not() {
    let f3 = family("F3");
    assert_eq!(f3.outcome, Outcome::Held);
    assert_eq!(f3.rule, Rule::Pass);
    assert!(f3.discriminating);

    let frozen: Vec<f64> = f3.points.iter().map(|point| point.frozen).collect();
    assert!(
        frozen.windows(2).all(|pair| pair[0] == pair[1]),
        "S0 must be exactly flat across the sweep"
    );

    // The three values task:28 §PHASE 3 predicted, before any code existed.
    let pooled: Vec<f64> = f3.points.iter().map(|point| point.pooled).collect();
    for (observed, expected) in pooled.iter().zip([4.6052f64, 2.9004, 1.3665]) {
        assert!(
            (observed - expected).abs() < 1e-4,
            "R1 must hit {expected}, got {observed}"
        );
    }
    assert!(
        pooled.windows(2).all(|pair| pair[0] > pair[1]),
        "and must fall strictly as the mark gets commoner in B"
    );
}

/// F4 — pooling discards the direction of a marginal imbalance. Recorded as a
/// limitation because no preregistered clause adjudicates it.
#[test]
fn pooling_cannot_distinguish_which_recording_the_mark_is_common_in() {
    let f4 = family("F4");
    assert_eq!(f4.rule, Rule::Limitation);
    assert_eq!(f4.outcome, Outcome::Held);
    assert!(f4.precondition_held, "both sums must be held fixed");

    let splits = &f4.points[..3];
    let pooled: Vec<f64> = splits.iter().map(|point| point.pooled).collect();
    assert!(
        pooled
            .windows(2)
            .all(|pair| (pair[0] - pair[1]).abs() < 1e-12),
        "R1 is identical across every split of the same pooled total"
    );
    let frozen: Vec<f64> = splits.iter().map(|point| point.frozen).collect();
    let range = frozen.iter().copied().fold(f64::NEG_INFINITY, f64::max)
        - frozen.iter().copied().fold(f64::INFINITY, f64::min);
    assert!(
        range > 3.9,
        "while S0 ranges over {range:.4} nats across the same three"
    );
}

/// F5 — the pooled estimate is the length-weighted mean of the two relative
/// frequencies. **The mechanism holds at every closed-form point.**
#[test]
fn the_pooled_estimate_is_the_length_weighted_mean_of_the_two_frequencies() {
    let f5 = family("F5");
    let closed_form = &f5.points[..4];
    for point in closed_form {
        assert!(
            point.matched,
            "the weighted-mean form must hold at {}",
            point.params
        );
    }
    // Exchange invariance in value does not buy equal influence: the two
    // imbalanced configurations sit either side of the balanced one.
    assert!(f5.points[2].pooled > f5.points[0].pooled);
    assert!(f5.points[1].pooled < f5.points[0].pooled);
}

/// **The eleventh criterion defect, and it is this round's own.**
///
/// task:28 §PHASE 3 predicted F5's values as 1.3665 and 0.6941, then attached a
/// rule requiring the movement between them to exceed 1 nat. Their difference
/// is 0.6724. The rule fails on the values the same section predicts, and
/// §PHASE 4's reachability paragraph was applied to the PASS rules only.
///
/// The rule is **not** repaired here. This test asserts the inconsistency so it
/// stays visible.
#[test]
fn the_magnitude_gate_attached_to_f5_is_unreachable_on_its_own_predicted_values() {
    let f5 = family("F5");
    let movement = f5
        .points
        .last()
        .expect("the movement check must remain in the sweep");
    assert!(
        !movement.matched,
        "the gate must remain failing; repairing it after the fact is the defect"
    );
    assert!(
        (movement.pooled - 0.6724).abs() < 1e-3,
        "the observed movement is 0.6724, under the 1.0 the rule demanded"
    );
    // The mechanism the family exists to demonstrate is unaffected: across the
    // whole sweep R1 moves far more than the gate's threshold.
    let pooled: Vec<f64> = f5.points[..4].iter().map(|point| point.pooled).collect();
    let range = pooled.iter().copied().fold(f64::NEG_INFINITY, f64::max)
        - pooled.iter().copied().fold(f64::INFINITY, f64::min);
    assert!(
        range > 3.5,
        "the full sweep spans {range:.4} nats, which the gate never looked at"
    );
    // A broken limitation must not reject: only a PASS family can.
    assert_eq!(f5.rule, Rule::Limitation);
    assert_ne!(verdict(&families()), Verdict::Reject);
}

/// F6 — split by task:28 §PHASE 4 M5 because both statistics move on the A
/// side. Only the B side discriminates.
#[test]
fn one_sided_background_discriminates_on_bs_side_only() {
    let f6a = family("F6a");
    let f6b = family("F6b");
    assert_eq!(f6a.outcome, Outcome::Held);
    assert_eq!(f6b.outcome, Outcome::Held);
    assert!(!f6a.discriminating, "both statistics move on the A side");
    assert!(f6b.discriminating, "only R1 moves on the B side");

    for point in &f6a.points {
        assert!(point.frozen > 0.0, "S0 rises when A gains background");
        assert!(point.pooled > 0.0, "and so does R1");
        assert!(
            point.frozen > point.pooled,
            "S0 rises more, its denominator being the smaller one"
        );
    }
    for point in &f6b.points {
        assert_eq!(point.frozen, 0.0, "S0 cannot see B gaining background");
        assert!(point.pooled > 0.0, "R1 can");
    }
}

/// F7 — the crossover surface matches the analytic ordering at all 64 swept
/// points per rarity, which is the criterion §PHASE 4 M1 replaced "more
/// agreements must win" with.
#[test]
fn the_crossover_surface_matches_the_analytic_ordering_everywhere() {
    let f7 = family("F7");
    assert_eq!(f7.outcome, Outcome::Held);
    for point in &f7.points {
        assert_eq!(
            point.observed, 0.0,
            "no sign mismatch is permitted at {}",
            point.params
        );
    }
    // The predicted crossover ratios, from ln(1/p_c)/ln(1/p_r), recomputed here
    // rather than pasted: the middle one is exactly log10(2), and a literal
    // would be asserting a coincidence instead of the closed form.
    let observed: Vec<f64> = f7.points.iter().map(|point| point.pooled).collect();
    for (got, p_rare) in observed.iter().zip([0.001f64, 0.01, 0.05]) {
        let want = (1.0f64 / 0.25).ln() / (1.0f64 / p_rare).ln();
        assert!(
            (got - want).abs() < 1e-12,
            "crossover at p_rare={p_rare} must be {want}, got {got}"
        );
    }
}

/// F8 — R1 cannot distinguish five dependent repetitions from five independent
/// plantings. The independence assumption is load-bearing and false here.
#[test]
fn dependent_repetition_scores_identically_to_independent_planting() {
    let f8 = family("F8");
    assert_eq!(f8.rule, Rule::Limitation);
    assert_eq!(f8.outcome, Outcome::Held);
    assert!(f8.precondition_held, "both sides must be the same shape");
    let difference = f8.points.last().expect("the difference point").pooled;
    assert_eq!(
        difference, 0.0,
        "the two must be indistinguishable, which is the limitation"
    );
}

/// task:28 §PHASE 6 — the verdict, by precedence. B was predicted before the
/// run; asserting it here means a later change that would make R1 look
/// coherent has to face this test.
#[test]
fn the_verdict_is_useful_heuristic_and_not_coherent_survivor() {
    let all = families();
    assert_eq!(verdict(&all), Verdict::UsefulHeuristic);

    let broken_pass: Vec<&str> = all
        .iter()
        .filter(|entry| entry.rule == Rule::Pass && entry.outcome == Outcome::Broken)
        .map(|entry| entry.name)
        .collect();
    assert!(
        broken_pass.is_empty(),
        "no PASS family may fail, or the verdict is C: {broken_pass:?}"
    );

    let confirmed: Vec<&str> = all
        .iter()
        .filter(|entry| entry.rule == Rule::Limitation && entry.outcome == Outcome::Held)
        .map(|entry| entry.name)
        .collect();
    assert_eq!(
        confirmed.len(),
        3,
        "F4, F6a and F8 confirm; F5's gate does not. {confirmed:?}"
    );
}

/// The partition must tile: every family set maps to exactly one verdict, and
/// a broken PASS always wins.
#[test]
fn the_verdict_partition_tiles_by_precedence() {
    let mut all = families();
    assert_eq!(verdict(&all), Verdict::UsefulHeuristic);

    // Break a PASS family: rejection must take precedence over any limitation.
    let index = all
        .iter()
        .position(|entry| entry.rule == Rule::Pass)
        .expect("a PASS family must exist");
    all[index].outcome = Outcome::Broken;
    assert_eq!(verdict(&all), Verdict::Reject);

    // With no confirmed limitation and no broken PASS, the remaining state.
    let clean: Vec<discriminating::FamilyResult> = families()
        .into_iter()
        .filter(|entry| entry.rule == Rule::Pass)
        .collect();
    assert_eq!(verdict(&clean), Verdict::CoherentSurvivor);
}

/// The round changed no statistic and adopted nothing.
#[test]
fn the_round_froze_both_statistics_and_adopted_neither() {
    assert_eq!(adversarial::UNDER_TEST, "rarity_of_agreements");
    assert!(
        repair::candidate("S0 rarity_of_agreements")
            .expect("the incumbent")
            .frozen
    );
    assert!(
        !repair::candidate("R1 pooled sum")
            .expect("the candidate")
            .frozen,
        "R1 remains a proposal, not the incumbent"
    );
    assert_eq!(repair::CANDIDATES.len(), 4, "no R4 was created");
}

/// Every family names the exact quantity it computes, which decision:7 requires
/// and which the vague-wording ban in task:28 §PHASE 3 enforces.
#[test]
fn every_family_names_its_computed_quantity_and_avoids_vague_wording() {
    for entry in families() {
        assert!(
            !entry.quantity.is_empty(),
            "{} names no quantity",
            entry.name
        );
        assert!(!entry.semantic_expectation.is_empty());
        for banned in ["better", "stronger", "most exceptional", "more motif-like"] {
            assert!(
                !entry.quantity.to_lowercase().contains(banned),
                "{} uses the banned word `{banned}` in its quantity",
                entry.name
            );
        }
    }
}
