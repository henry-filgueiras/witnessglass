//! sprint:19, task:29 — search-aware null calibration.
//!
//! The load-bearing claim of this round is that the observed and null paths run
//! the *same complete search*, rather than the null rescoring boundaries chosen
//! on observed data. Several tests below exist only to keep that true.
//!
//! Replicate counts here are deliberately small. The preregistered `B = 999` is
//! for the experiment; these check machinery, and `scripts/check.sh` has to stay
//! fast enough to run on every change.

use witnessglass::experiment::calibration::{
    self, LADDER, REPLICATES, TAIL_THRESHOLD, TOP_K, calibrate, complete_search, negative_control,
    positive_control, selection_effect, structural_summaries,
};
use witnessglass::experiment::event_sequence::{cross_pairs, null_seed, order_null_seeded};
use witnessglass::experiment::identifiability::Observation;
use witnessglass::experiment::repair;

/// A cheap replicate count for machinery checks that never assert the tail
/// threshold.
const SMALL: usize = 49;

/// The **smallest** replicate count at which the preregistered `TAIL_THRESHOLD`
/// of 0.01 is attainable at all: `(1 + 0)/(99 + 1) = 0.01`.
///
/// The first draft of this file asserted `exceptional` at `SMALL`, where the
/// best attainable tail is `1/50 = 0.02` and the assertion could never pass —
/// the same unreachable-criterion defect sprint:18 recorded as its eleventh,
/// this time caught by the suite instead of by a report. The fix is to give the
/// test a `B` that resolves the preregistered threshold, **never** to loosen the
/// threshold to fit the test.
const RESOLVING: usize = 99;

/// The preregistered constants must not drift.
#[test]
fn the_preregistered_parameters_are_the_ones_task_29_fixed() {
    assert_eq!(LADDER, [3, 4, 6, 8, 12], "the frozen sprint:9 ladder");
    assert_eq!(REPLICATES, 999, "B, fixed from a measured 0.097s search");
    assert_eq!(TAIL_THRESHOLD, 0.01);
    assert_eq!(TOP_K, 5, "the number dedupe_overlapping already keeps");
    assert_eq!(calibration::KEEP, 5);
}

/// **The claim this round stands on.** The null path re-searches; it does not
/// rescore the boundaries the observed search chose.
///
/// Built so the two are distinguishable: the observed search's winning windows
/// are scored under a null replicate and compared against what a *complete*
/// search of that same replicate finds. If the null path were boundary-fixed
/// the two would coincide.
#[test]
fn the_null_path_reruns_the_search_rather_than_rescoring_observed_boundaries() {
    let control = positive_control();
    let k = 12;

    // What the observed search selected.
    let ranked = cross_pairs(&control.first, &control.second, k, usize::MAX)
        .expect("the pair must be admissible");
    let observed_winner = ranked.first().expect("a ranking");
    let (wa, wb) = (&observed_winner.comparison.a, &observed_winner.comparison.b);

    let left = order_null_seeded(&control.first, null_seed(0, 0));
    let right = order_null_seeded(&control.second, null_seed(0, 1));

    // Boundary-fixed rescoring: the observed windows, read out of the null.
    let rescored = Observation::of(
        &left,
        (wa.start, wa.start + wa.k),
        &right,
        (wb.start, wb.start + wb.k),
    )
    .and_then(|observation| (repair::candidate("R1 pooled sum").expect("R1").score)(&observation))
    .expect("a rescore");

    // What the complete search finds on that same replicate.
    let searched = complete_search(&left, &right, k).t.expect("a null T");

    assert!(
        searched > rescored,
        "a complete search of the replicate must find a better winner than the \
         observed boundaries score there; got searched {searched} vs rescored {rescored}"
    );
}

/// The null preserves mark marginals exactly, so R1's per-mark weights are
/// identical on both paths and only *which* marks land in agreeing positions
/// moves. task:29 §PHASE 8 relies on this, so it is asserted rather than
/// assumed.
#[test]
fn the_null_leaves_every_pooled_weight_exactly_where_it_was() {
    let control = negative_control();
    let nulled = order_null_seeded(&control.first, null_seed(7, 0));

    assert_eq!(nulled.len(), control.first.len(), "length is preserved");

    let counts = |sequence: &witnessglass::experiment::event_sequence::EventSequence<'_>| {
        let mut counts: std::collections::BTreeMap<String, usize> = Default::default();
        for event in &sequence.events {
            *counts.entry(event.mark.label()).or_insert(0) += 1;
        }
        counts
    };
    assert_eq!(
        counts(&control.first),
        counts(&nulled),
        "the null is a permutation of the mark multiset, so every count survives"
    );
}

/// The two paths search spaces of the same size, because `window_count` depends
/// only on length and the null preserves length.
#[test]
fn both_paths_enumerate_the_same_number_of_candidate_pairs() {
    let control = negative_control();
    for k in LADDER {
        let result = calibrate("check", &control.first, &control.second, k, SMALL);
        assert!(
            (result.null_considered_mean - result.observed_considered as f64).abs() < 1e-9,
            "at k={k} the null searched {} pairs and the observed search {}",
            result.null_considered_mean,
            result.observed_considered
        );
    }
}

/// task:29 §PHASE 3 — the winner-selection effect, on this project's own
/// machinery, on a specimen generated from the null hypothesis itself.
#[test]
fn search_selected_maxima_are_drawn_from_a_different_distribution_than_arbitrary_candidates() {
    let control = negative_control();
    let effect = selection_effect(&control.first, &control.second, 8, 500, 60);

    assert!(
        effect.confirmed,
        "median(selected)={} must exceed median(arbitrary)={}",
        effect.selected_median, effect.arbitrary_median
    );
    // The effect is not marginal: selected maxima clear the arbitrary
    // distribution's 99th percentile.
    assert!(
        effect.margin > 0.0,
        "selected median {} vs arbitrary q99 {}",
        effect.selected_median,
        effect.arbitrary_q99
    );
}

/// The negative control is generated *according to the null hypothesis*, so its
/// observed `T` must look ordinary against its own null.
#[test]
fn the_negative_control_looks_ordinary_against_its_own_null() {
    let control = negative_control();
    let flagged: Vec<usize> = LADDER
        .into_iter()
        .filter(|k| calibrate("neg", &control.first, &control.second, *k, SMALL).exceptional)
        .collect();
    assert!(
        flagged.is_empty(),
        "a specimen drawn from the null must not be flagged; flagged at k={flagged:?}"
    );
}

/// The positive control plants a 12-mark figure three times in each sequence.
/// The search must recover it at the length it was planted at.
///
/// `REACHABLE` rather than [`SMALL`], and the reason is worth keeping: at
/// `B = 49` the smallest attainable tail is `1/50 = 0.02`, which cannot clear a
/// `0.01` threshold *whatever the data does*. This test first asserted
/// `exceptional` at `SMALL` and failed for that reason alone — the same
/// unreachable-criterion class as sprint:18's eleventh defect, caught here by
/// the machinery before it reached a report. `B = 99` gives `1/100 = 0.01`,
/// which the threshold admits.
const REACHABLE: usize = 99;

#[test]
fn the_positive_control_becomes_exceptional_at_the_planted_length() {
    assert!(
        1.0 / (REACHABLE as f64 + 1.0) <= TAIL_THRESHOLD,
        "the threshold must be attainable at this replicate count"
    );
    let control = positive_control();
    let result = calibrate("pos", &control.first, &control.second, 12, REACHABLE);
    assert_eq!(
        result.exceedances, 0,
        "no null search should reach the planted figure's score"
    );
    assert!(
        result.exceptional,
        "p-hat {} must clear the threshold",
        result.tail
    );

    // And it must beat the null by a margin, not squeak past it.
    assert!(
        result.observed.expect("a T") > result.null_max,
        "observed T {:?} must exceed every null T, the largest being {}",
        result.observed,
        result.null_max
    );
}

/// The two controls differ only by the planted figure, so the positive control
/// must score strictly higher at the planted length. If it did not, the fixture
/// would not be testing what it claims.
#[test]
fn planting_the_figure_is_what_separates_the_two_controls() {
    let negative = negative_control();
    let positive = positive_control();
    let without = complete_search(&negative.first, &negative.second, 12)
        .t
        .expect("a T");
    let with = complete_search(&positive.first, &positive.second, 12)
        .t
        .expect("a T");
    assert!(
        with > without,
        "planted {with} must exceed unplanted {without}"
    );
}

/// The tail estimate is exactly `(1 + exceedances) / (B + 1)`, and is never
/// described as anything but a Monte Carlo null tail.
#[test]
fn the_tail_estimate_is_the_preregistered_finite_sample_form() {
    let control = negative_control();
    let result = calibrate("form", &control.first, &control.second, 6, SMALL);
    let expected = (1.0 + result.exceedances as f64) / (SMALL as f64 + 1.0);
    assert!((result.tail - expected).abs() < 1e-12);
    assert!(
        result.tail >= 1.0 / (SMALL as f64 + 1.0),
        "bounded below by 1/(B+1)"
    );
    assert!(result.tail <= 1.0);
}

/// A tail threshold is attainable only when `1/(B+1) <= threshold`. Asserted
/// generally, so an unreachable assertion is caught by construction rather than
/// by a test happening to fail.
#[test]
fn a_replicate_count_can_only_assert_a_threshold_it_resolves() {
    let attainable = |b: usize| 1.0 / (b as f64 + 1.0) <= TAIL_THRESHOLD;
    assert!(
        !attainable(SMALL),
        "SMALL must NOT resolve the threshold, or the distinction is pointless"
    );
    assert!(
        attainable(RESOLVING),
        "RESOLVING must be the smallest that does"
    );
    assert!(!attainable(RESOLVING - 1), "and it must be the smallest");
    assert!(
        attainable(REPLICATES),
        "the preregistered B resolves it comfortably"
    );
}

/// Both thresholds are reachable at the preregistered `B`, on **both** sides.
/// sprint:18's eleventh defect was a reachability check applied to PASS rules
/// only; this checks the FAIL side too.
#[test]
fn both_sides_of_the_threshold_are_reachable_at_the_preregistered_replicate_count() {
    let best = 1.0 / (REPLICATES as f64 + 1.0);
    assert!(
        best <= TAIL_THRESHOLD,
        "0 exceedances gives {best}, which must clear the threshold"
    );
    let worst = (1.0 + REPLICATES as f64) / (REPLICATES as f64 + 1.0);
    assert!(
        worst > TAIL_THRESHOLD,
        "all exceedances gives {worst}, which must fail it"
    );
    // And the threshold is ten times coarser than the resolution floor.
    assert!(TAIL_THRESHOLD >= 10.0 * best);
}

/// task:29 §PHASE 6 — the top-k comparison is descriptive and reports one
/// exceedance count per rank.
#[test]
fn the_top_k_comparison_reports_one_exceedance_count_per_rank() {
    let control = positive_control();
    let result = calibrate("topk", &control.first, &control.second, 12, SMALL);
    assert_eq!(result.top_k_exceedances.len(), TOP_K);
    for count in &result.top_k_exceedances {
        assert!(*count <= SMALL, "an exceedance count cannot exceed B");
    }
}

/// task:29 §PHASE 7 — the three summaries are computed, and none reads anything
/// but marks.
#[test]
fn the_structural_summaries_are_the_three_preregistered_ones() {
    let control = negative_control();
    let summaries = structural_summaries(&control.first, 19);
    let names: Vec<&str> = summaries.iter().map(|summary| summary.name).collect();
    assert_eq!(
        names,
        vec![
            "immediate repetition rate",
            "mean run length",
            "transition entropy ratio"
        ]
    );
    // On a specimen drawn from the null, the summaries must sit inside the null
    // range — otherwise the check would flag everything and mean nothing.
    let outside: Vec<&str> = summaries
        .iter()
        .filter(|summary| summary.outside_null_range)
        .map(|summary| summary.name)
        .collect();
    assert!(
        outside.is_empty(),
        "an i.i.d. specimen must not look structured: {outside:?}"
    );
}

/// A specimen the search cannot admit yields an undefined `T`, reported as such
/// rather than scored zero.
#[test]
fn an_inadmissible_specimen_yields_an_undefined_t_rather_than_a_zero() {
    let control = negative_control();
    // cross_pairs refuses a pair sharing one session id.
    let outcome = complete_search(&control.first, &control.first, 8);
    assert_eq!(outcome.t, None, "a recording is not a cross-recording pair");
    assert_eq!(outcome.considered, 0);
    assert!(outcome.top.is_empty());
}

/// The fixtures are synthetic and obviously so, per `CLAUDE.md` §5.
#[test]
fn the_control_fixtures_are_obviously_synthetic_and_carry_no_receipts() {
    for control in [negative_control(), positive_control()] {
        for sequence in [&control.first, &control.second] {
            assert!(
                sequence.events.iter().all(|event| event.sequence.is_none()),
                "a hand-built event has no record, so it has no receipt"
            );
            assert!(
                sequence.events.iter().all(|event| event
                    .mark
                    .tool_name
                    .is_some_and(|name| name.starts_with("synthetic-"))),
                "every mark must announce itself as synthetic"
            );
        }
    }
}

/// The round changed nothing it was calibrating.
#[test]
fn the_round_adopted_nothing_and_changed_no_statistic() {
    assert!(
        !repair::candidate("R1 pooled sum").expect("R1").frozen,
        "R1 remains a proposal"
    );
    assert!(
        repair::candidate("S0 rarity_of_agreements")
            .expect("S0")
            .frozen
    );
    assert_eq!(repair::CANDIDATES.len(), 4, "no new score was invented");
    assert_eq!(
        witnessglass::experiment::envelope::UNDER_STUDY,
        "rarity_of_agreements"
    );
}

// ---------------------------------------------------------------------------
// The rendering — presentation only
// ---------------------------------------------------------------------------

/// The card must carry the null-adequacy limitation next to the separation it
/// bounds. task:29 §PHASE 11 is what stops an exceptional `T` reading as a
/// motif finding, and a page that showed the tails without it would let it.
#[test]
fn the_card_shows_the_calibration_limitation_beside_the_separation() {
    let control = negative_control();
    let summaries = structural_summaries(&control.first, 19);
    // Force the limitation branch with a summary that is outside its null range,
    // which is what every real specimen turned out to be.
    let mut outside = summaries.clone();
    outside[0].outside_null_range = true;

    let document = serde_json::json!({
        "replicates": 999,
        "selection_effect": {
            "arbitrary": vec![1.0f64, 2.0, 3.0],
            "selected": vec![8.0f64, 9.0, 10.0],
            "arbitrary_median": 2.0,
            "arbitrary_q99": 3.0,
            "selected_median": 9.0,
            "margin": 6.0,
            "confirmed": true,
        },
        "controls": [],
        "real": [],
        "structural": [{ "specimen": "test", "summaries": outside }],
        "verdict": "A NULL-SEPARATING",
    });
    let page = witnessglass::experiment::boundary_page::render(std::slice::from_ref(&document));

    assert!(page.contains("Nothing here is adopted"));
    assert!(
        page.contains("does not establish motif structure"),
        "the limitation must be stated wherever it holds"
    );
    assert!(
        page.contains("rescoring boundaries chosen on observed data"),
        "the card must say the null re-searches, which is the round's whole claim"
    );
    assert!(
        page.contains("observational specimens rather than ground truth"),
        "recordings are specimens, and the page must say so"
    );
    assert!(
        page.contains("<rect"),
        "both distributions must actually be drawn"
    );
}

/// An empty distribution renders as an absence rather than as an empty plot.
#[test]
fn an_absent_distribution_is_named_rather_than_drawn_as_nothing() {
    let document = serde_json::json!({
        "replicates": 0,
        "selection_effect": {
            "arbitrary": Vec::<f64>::new(),
            "selected": Vec::<f64>::new(),
            "arbitrary_median": f64::NAN,
            "arbitrary_q99": f64::NAN,
            "selected_median": f64::NAN,
            "margin": f64::NAN,
            "confirmed": false,
        },
        "controls": [], "real": [], "structural": [],
        "verdict": "C NULL / CALIBRATION INADEQUATE",
    });
    let page = witnessglass::experiment::boundary_page::render(std::slice::from_ref(&document));
    assert!(
        page.contains("That is an absence, not a zero"),
        "an absent distribution must be named"
    );
}
