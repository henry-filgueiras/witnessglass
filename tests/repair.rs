//! sprint:17, task:27 — the comparative repair experiment.
//!
//! These tests bank the derivations task:27 §D made *before* any candidate ran,
//! so that a later change which quietly invalidates one fails here rather than
//! in a report. They assert what was derived, including the results that went
//! against the candidates.

use std::collections::BTreeMap;

use witnessglass::experiment::adversarial;
use witnessglass::experiment::identifiability::Observation;
use witnessglass::experiment::repair::{
    CANDIDATES, Candidate, candidate, contract, contracts, crossing_witnesses,
};

fn named(name: &str) -> &'static Candidate {
    candidate(name).unwrap_or_else(|| panic!("candidate `{name}` must remain preregistered"))
}

fn s0() -> &'static Candidate {
    named("S0 rarity_of_agreements")
}
fn r1() -> &'static Candidate {
    named("R1 pooled sum")
}
fn r2() -> &'static Candidate {
    named("R2 pooled mean")
}
fn r3() -> &'static Candidate {
    named("R3 pooled density")
}

/// Build an observation directly, so a test can control both recordings'
/// marginals independently of any fixture.
fn observation(
    agreeing: &[(&str, usize, usize)],
    disagreements: usize,
    a_total: usize,
    b_total: usize,
) -> Observation {
    let mut a = Vec::new();
    let mut b = Vec::new();
    let mut a_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut b_counts: BTreeMap<String, usize> = BTreeMap::new();
    for (mark, in_a, in_b) in agreeing {
        a.push((*mark).to_owned());
        b.push((*mark).to_owned());
        a_counts.insert((*mark).to_owned(), *in_a);
        b_counts.insert((*mark).to_owned(), *in_b);
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

fn swapped(observation: &Observation) -> Observation {
    Observation {
        a: observation.b.clone(),
        b: observation.a.clone(),
        a_counts: observation.b_counts.clone(),
        b_counts: observation.a_counts.clone(),
        a_total: observation.b_total,
        b_total: observation.a_total,
    }
}

#[test]
fn the_preregistered_candidates_are_all_present_and_the_incumbent_is_marked() {
    assert_eq!(
        CANDIDATES.len(),
        4,
        "task:27 §C names one incumbent and three candidates"
    );
    assert!(s0().frozen, "S0 is the frozen incumbent");
    for entry in [r1(), r2(), r3()] {
        assert!(
            !entry.frozen,
            "{} is a proposal, not the incumbent",
            entry.name
        );
    }
    for entry in CANDIDATES.iter() {
        assert!(
            !entry.interpretation.is_empty(),
            "task:27 §G rejects a candidate justifiable only by what it passes, \
             so {} must carry an interpretation",
            entry.name
        );
    }
}

/// task:27 §B C1, and the defect sprint:16 measured. Not free for S0.
#[test]
fn every_candidate_is_exchange_invariant_and_the_incumbent_is_not() {
    for entry in [r1(), r2(), r3()] {
        let report = contract(entry);
        let c1 = &report.clauses[0];
        assert_eq!(c1.clause, "C1");
        assert!(
            c1.satisfied && c1.value == 0.0,
            "{} must satisfy C1 exactly; measured {}",
            entry.name,
            c1.value
        );
    }
    let incumbent = contract(s0());
    assert!(
        !incumbent.clauses[0].satisfied,
        "S0's asymmetry is the defect this round was commissioned against"
    );
}

/// task:27 §D1 — C1, C4 and C5 are free by construction and confer no
/// eligibility credit. The flag exists so a report cannot quietly count them.
#[test]
fn the_clauses_that_are_free_by_construction_are_marked_as_such() {
    let report = contract(r1());
    let free: Vec<&str> = report
        .clauses
        .iter()
        .filter(|clause| clause.free_by_construction)
        .map(|clause| clause.clause)
        .collect();
    assert_eq!(
        free,
        vec!["C1", "C4", "C5"],
        "task:27 §D1 names exactly these"
    );
}

/// task:27 §D4 — R1 is *numerically identical* to S0 whenever the two
/// recordings share marginals, which is why nine of ten families cannot
/// distinguish them.
#[test]
fn pooled_rarity_collapses_onto_the_incumbent_when_marginals_are_shared() {
    for case in [
        observation(&[("x", 1, 1)], 3, 1_000, 1_000),
        observation(&[("x", 50, 50), ("y", 50, 50)], 2, 1_000, 1_000),
        observation(&[("x", 7, 7)], 0, 44, 44),
    ] {
        let frozen = (s0().score)(&case).expect("a score");
        let pooled = (r1().score)(&case).expect("a score");
        assert!(
            (frozen - pooled).abs() < 1e-12,
            "§D4: R1 must equal S0 at shared marginals, got {frozen} and {pooled}"
        );
    }
}

/// The same derivation's other half: where marginals differ, R1 must *not*
/// equal S0, or it would repair nothing.
#[test]
fn pooled_rarity_departs_from_the_incumbent_exactly_where_marginals_differ() {
    let case = observation(&[("x", 1, 500)], 3, 1_000, 1_000);
    let frozen = (s0().score)(&case).expect("a score");
    let pooled = (r1().score)(&case).expect("a score");
    assert!(
        (frozen - pooled).abs() > 1.0,
        "a mark rare in A and ubiquitous in B must score differently under R1"
    );
    // And the incumbent is blind to the difference, which AG3b says in the
    // gauntlet and this says directly.
    let blind = observation(&[("x", 1, 1)], 3, 1_000, 1_000);
    assert!(
        ((s0().score)(&case).expect("a score") - (s0().score)(&blind).expect("a score")).abs()
            < 1e-12,
        "S0 never reads B's marginals"
    );
}

/// task:27 §D3 — at fixed span length R3 is a positive multiple of R1, so the
/// two induce the same order. This is the derivation the envelope replay was
/// set up to falsify; it holds, so it is banked.
#[test]
fn density_and_sum_induce_the_same_order_at_a_fixed_span_length() {
    let cases = [
        observation(&[("x", 1, 1)], 3, 1_000, 900),
        observation(&[("x", 50, 20), ("y", 3, 90)], 2, 1_000, 900),
        observation(&[("x", 9, 9), ("y", 9, 4), ("z", 200, 7)], 1, 1_000, 900),
    ];
    for case in &cases {
        assert_eq!(case.len(), Some(4), "the fixture holds span length fixed");
    }
    for left in &cases {
        for right in &cases {
            let sum_order = (r1().score)(left)
                .expect("a score")
                .partial_cmp(&(r1().score)(right).expect("a score"));
            let density_order = (r3().score)(left)
                .expect("a score")
                .partial_cmp(&(r3().score)(right).expect("a score"));
            assert_eq!(sum_order, density_order, "§D3: same order at fixed L");
        }
    }
}

/// task:27 §P7 — the mean violates agreement monotonicity, and the sum and the
/// density do not.
#[test]
fn the_mean_falls_when_a_below_average_agreement_is_added() {
    let before = observation(&[("x", 1, 1)], 3, 1_000, 1_000);
    let after = observation(&[("x", 1, 1), ("y", 500, 500)], 2, 1_000, 1_000);
    assert!(
        (r2().score)(&after).expect("a score") < (r2().score)(&before).expect("a score"),
        "R2 must fall, which is the cost of dividing by the agreement count"
    );
    for entry in [s0(), r1(), r3()] {
        assert!(
            (entry.score)(&after).expect("a score") >= (entry.score)(&before).expect("a score"),
            "{} adds a non-negative term to a fixed denominator",
            entry.name
        );
    }
    let report = contract(r2());
    let c2 = &report.clauses[1];
    assert_eq!(c2.clause, "C2");
    assert!(!c2.satisfied, "and the contract check must say so");
}

/// task:27 §D2, the crossing theorem, exhibited constructively: *every*
/// rarity-weighted candidate admits fewer agreements outscoring more.
///
/// This is the result that reframes accumulation as a consequence of C3 rather
/// than a defect of any one statistic, so no candidate is exempt.
#[test]
fn every_candidate_including_the_incumbent_admits_a_crossing() {
    for witness in crossing_witnesses(1, 500) {
        assert!(
            witness.crossed,
            "§D2 says {} must cross; it did not within k ≤ 24",
            witness.candidate
        );
        assert!(
            witness.fewer < witness.more,
            "a crossing means strictly fewer agreements winning"
        );
        assert!(witness.fewer_score > witness.more_score);
    }
}

/// The undeviated loss. R3 fails AG3 — "rarity that does not agree contributes
/// nothing" — because dividing by span length lets a *disagreeing* position
/// dilute the score. task:27 §D did not derive this, and §G rejects R3 for it.
///
/// Banked as a test so the loss cannot be forgotten if R3 is revisited.
#[test]
fn dividing_by_span_length_lets_a_disagreement_change_the_score() {
    let without = observation(&[("x", 20, 20), ("y", 20, 20)], 0, 1_000, 1_000);
    let with_rare = observation(&[("x", 20, 20), ("y", 20, 20)], 1, 1_000, 1_000);
    for entry in [s0(), r1(), r2()] {
        let delta = ((entry.score)(&with_rare).expect("a score")
            - (entry.score)(&without).expect("a score"))
        .abs();
        assert!(
            delta < 1e-12,
            "{} holds AG3's invariant exactly",
            entry.name
        );
    }
    let delta = ((r3().score)(&with_rare).expect("a score")
        - (r3().score)(&without).expect("a score"))
    .abs();
    assert!(delta > 1e-12, "R3 does not, and that is why it is rejected");
}

/// Running the ten families with the frozen statistic must reproduce sprint:15
/// exactly. The parameterization added in sprint:17 changed no construction,
/// and this is what says so.
#[test]
fn parameterizing_the_gauntlet_left_the_frozen_run_untouched() {
    let original = adversarial::families();
    let through_parameter = adversarial::families_with(adversarial::under_test());
    assert_eq!(original.len(), 10);
    for (left, right) in original.iter().zip(through_parameter.iter()) {
        assert_eq!(left.name, right.name);
        assert_eq!(left.verdict, right.verdict, "{} must not move", left.name);
        assert_eq!(left.boundary, right.boundary);
        assert_eq!(left.points.len(), right.points.len());
    }
}

/// task:27 §D5 — AG3b's first sweep point compares a case to itself under a
/// strict inequality, so no statistic whatever can pass it. The family is left
/// unmodified; this records the defect instead.
#[test]
fn the_one_sided_rarity_family_contains_a_point_no_statistic_can_pass() {
    for entry in CANDIDATES.iter() {
        let families = adversarial::families_with(entry.score);
        let ag3b = families
            .iter()
            .find(|family| family.name.starts_with("AG3b"))
            .expect("AG3b must remain");
        assert_eq!(
            ag3b.boundary.as_deref(),
            Some("count_B=1"),
            "{}'s only AG3b failure is the degenerate point",
            entry.name
        );
    }
}

/// R1 repairs AG3b everywhere the family can actually be passed.
#[test]
fn pooled_rarity_holds_every_reachable_point_of_the_one_sided_rarity_family() {
    let families = adversarial::families_with(r1().score);
    let ag3b = families
        .iter()
        .find(|family| family.name.starts_with("AG3b"))
        .expect("AG3b must remain");
    let failures: Vec<&str> = ag3b
        .points
        .iter()
        .filter(|point| !point.holds)
        .map(|point| point.params.as_str())
        .collect();
    assert_eq!(
        failures,
        vec!["count_B=1"],
        "every non-degenerate point must hold under R1"
    );
}

/// Nothing in this round adopted anything: the statistic sprint:16 studied and
/// the one sprint:15 tested are still the frozen one.
#[test]
fn the_round_adopted_nothing() {
    assert_eq!(adversarial::UNDER_TEST, "rarity_of_agreements");
    assert_eq!(
        witnessglass::experiment::envelope::UNDER_STUDY,
        "rarity_of_agreements"
    );
    assert!(s0().frozen);
}

/// Every clause is reported for every candidate, in the order task:27 §B fixed.
#[test]
fn the_contract_reports_all_six_clauses_in_the_preregistered_order() {
    for report in contracts() {
        let ids: Vec<&str> = report.clauses.iter().map(|clause| clause.clause).collect();
        assert_eq!(ids, vec!["C1", "C2", "C3", "C4", "C5", "C6"]);
        for clause in &report.clauses {
            assert!(
                !clause.quantity.is_empty(),
                "decision:7 requires each criterion to name its computed quantity"
            );
        }
    }
}

/// Pooling is symmetric in the two recordings before any statistic uses it.
#[test]
fn the_pooled_frequency_is_symmetric_in_the_two_recordings() {
    let case = observation(&[("x", 3, 90), ("y", 400, 2)], 1, 1_000, 7_000);
    for entry in [r1(), r2(), r3()] {
        let forward = (entry.score)(&case).expect("a score");
        let backward = (entry.score)(&swapped(&case)).expect("a score");
        assert!(
            (forward - backward).abs() < 1e-12,
            "{} must not depend on argument order",
            entry.name
        );
    }
}

// ---------------------------------------------------------------------------
// The rendering — presentation only
// ---------------------------------------------------------------------------

/// The card must not rank the candidates or total their ticks: task:27 §I
/// forbids selection by aggregate pass count, and a page that summed the
/// column would be making the selection the contract reserves for §I.
#[test]
fn the_repair_card_reports_clauses_without_totalling_them() {
    let document = serde_json::json!({
        "under_test": "rarity_of_agreements",
        "candidates": [{
            "name": "R1 pooled sum",
            "formula": "Σ_agreeing −ln p̂(m)",
            "interpretation": "total surprisal under the shared-source model",
            "frozen": false,
        }],
        "contract": contracts(),
        "crossing_witnesses": crossing_witnesses(1, 500),
        "family_matrix": [],
        "shared_marginal_points": 65,
        "total_family_points": 68,
        "envelope": [],
    });
    let page = witnessglass::experiment::boundary_page::render(std::slice::from_ref(&document));

    assert!(page.contains("Nothing here is adopted"));
    assert!(
        page.contains("confer no eligibility"),
        "the free clauses must be marked as conferring nothing"
    );
    for banned in ["score:", "total:", "passes:", "rank"] {
        assert!(
            !page.to_lowercase().contains(banned),
            "the card must not aggregate or rank; found `{banned}`"
        );
    }
}

/// task:27's third condition on a decision:5 projection: an absence is rendered
/// as an absence. A document with no envelope replay must say so rather than
/// show zeros.
#[test]
fn an_unreplayed_envelope_is_rendered_as_an_absence_not_a_zero() {
    let document = serde_json::json!({
        "under_test": "rarity_of_agreements",
        "candidates": [],
        "contract": [],
        "crossing_witnesses": [],
        "family_matrix": [],
        "shared_marginal_points": 0,
        "total_family_points": 0,
        "envelope": [],
    });
    let page = witnessglass::experiment::boundary_page::render(std::slice::from_ref(&document));
    assert!(
        page.contains("That is an absence, not a zero"),
        "an unreplayed envelope must be named, not implied by empty cells"
    );
}
