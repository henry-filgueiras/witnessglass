//! sprint:20, task:30 — the two first-order null constructions, and the claims
//! made about what they preserve.
//!
//! Every "preserved exactly" in this round is asserted here rather than stated
//! in prose. The distinction the round turns on is that one construction
//! preserves first-order transition counts *exactly* and the other preserves
//! them only in expectation, and a test that could not tell the two apart would
//! let the round call either one transition-preserving.

use std::collections::BTreeMap;

use witnessglass::experiment::calibration::SYNTHETIC_TOOLS;
use witnessglass::experiment::event_sequence::{EventSequence, null_seed, order_null_seeded};
use witnessglass::experiment::transition_null::{
    self, CONSTRUCTIONS, SUCCESSORS, degeneracy, doublet_null_seeded, fidelity,
    first_order_negative, first_order_positive, longest_shared_run, markov_null_seeded,
    repeated_ngrams, states, vocabulary_size,
};

/// Adjacent-pair counts over mark labels.
fn doublets(sequence: &EventSequence<'_>) -> BTreeMap<(String, String), usize> {
    let mut counts = BTreeMap::new();
    for pair in sequence.events.windows(2) {
        *counts
            .entry((pair[0].mark.label(), pair[1].mark.label()))
            .or_insert(0) += 1;
    }
    counts
}

/// Mark counts.
fn marginals(sequence: &EventSequence<'_>) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for event in &sequence.events {
        *counts.entry(event.mark.label()).or_insert(0) += 1;
    }
    counts
}

/// The state path in the controls' own 0..12 alphabet, rather than in
/// first-appearance order.
fn control_states(sequence: &EventSequence<'_>) -> Vec<usize> {
    sequence
        .events
        .iter()
        .map(|event| {
            let name = event.mark.tool_name.expect("a control mark carries a name");
            SYNTHETIC_TOOLS
                .iter()
                .position(|tool| *tool == name)
                .expect("a control mark is one of the synthetic tools")
        })
        .collect()
}

// ---------------------------------------------------------------------------
// The exact construction
// ---------------------------------------------------------------------------

/// **The claim the round's primary null stands on.** Every first-order
/// transition count survives, exactly, in every replicate.
#[test]
fn the_doublet_null_preserves_every_transition_count_exactly() {
    let control = first_order_negative();
    for sequence in [&control.first, &control.second] {
        let expected = doublets(sequence);
        for index in 0..40 {
            let replicate = doublet_null_seeded(sequence, null_seed(index, 0));
            assert_eq!(
                doublets(&replicate),
                expected,
                "replicate {index} moved a transition count"
            );
        }
    }
}

/// And the mark multiset, the length, and both endpoints with it.
#[test]
fn the_doublet_null_preserves_marginals_length_and_both_endpoints() {
    let control = first_order_positive();
    let sequence = &control.first;
    let expected = marginals(sequence);
    for index in 0..40 {
        let replicate = doublet_null_seeded(sequence, null_seed(index, 0));
        assert_eq!(replicate.len(), sequence.len(), "length");
        assert_eq!(marginals(&replicate), expected, "mark multiset");
        assert_eq!(
            replicate.events.first().map(|event| event.mark),
            sequence.events.first().map(|event| event.mark),
            "the first mark is held fixed"
        );
        assert_eq!(
            replicate.events.last().map(|event| event.mark),
            sequence.events.last().map(|event| event.mark),
            "and the last is determined by the counts"
        );
    }
}

/// Preserving everything first-order is worth nothing if the construction
/// cannot move the sequence at all, so it must be shown to move it.
#[test]
fn the_doublet_null_actually_reaches_other_sequences() {
    let control = first_order_negative();
    let measured = degeneracy(&control.first, 60, |sequence, index| {
        doublet_null_seeded(sequence, null_seed(index, 0))
    });
    assert_eq!(
        measured.identical, 0,
        "no replicate may be the input itself"
    );
    assert!(
        measured.distinct > 50,
        "the construction reached only {} distinct sequences",
        measured.distinct
    );
}

/// The timing skeleton stays attached to positions, exactly as sprint:19's order
/// null leaves it, so the only property that moved between the two rounds is the
/// mark process.
#[test]
fn both_constructions_leave_every_gap_and_offset_where_it_was() {
    let control = first_order_positive();
    for (name, construct) in CONSTRUCTIONS {
        let replicate = construct(&control.second, null_seed(3, 1));
        for (before, after) in control.second.events.iter().zip(&replicate.events) {
            assert_eq!(before.offset_ms, after.offset_ms, "{name} moved an offset");
            assert_eq!(
                before.gap_from_previous_ms, after.gap_from_previous_ms,
                "{name} moved a gap"
            );
        }
        assert!(
            replicate
                .events
                .iter()
                .all(|event| event.sequence.is_none()),
            "{name} must drop receipts: a mark that moved is not that record's"
        );
    }
}

// ---------------------------------------------------------------------------
// The in-expectation construction, and the difference between the two
// ---------------------------------------------------------------------------

/// The fitted-chain null preserves length and the initial state, and invents no
/// mark — but **not** the transition counts, and this test exists to keep the
/// round from describing it as though it did.
#[test]
fn the_markov_null_preserves_the_model_and_not_the_counts() {
    let control = first_order_negative();
    let sequence = &control.first;
    let expected = doublets(sequence);
    let mut moved = 0;
    for index in 0..40 {
        let replicate = markov_null_seeded(sequence, null_seed(index, 0));
        assert_eq!(replicate.len(), sequence.len(), "length is preserved");
        assert_eq!(
            replicate.events.first().map(|event| event.mark),
            sequence.events.first().map(|event| event.mark),
            "the initial state is held fixed"
        );
        let (vocabulary, _) = states(sequence);
        assert!(
            replicate
                .events
                .iter()
                .all(|event| vocabulary.contains(&event.mark)),
            "the chain cannot generate a mark the sequence never delivered"
        );
        if doublets(&replicate) != expected {
            moved += 1;
        }
    }
    assert_eq!(
        moved, 40,
        "every replicate of a resampled chain should move some transition count; \
         if none did, this construction is not the in-expectation one it is described as"
    );
}

/// The same distinction, read through the measurement the round uses to decide
/// which construction is transition-preserving.
#[test]
fn fidelity_separates_the_exact_construction_from_the_fitted_one() {
    let control = first_order_negative();
    let sequence = &control.second;
    for index in 0..20 {
        let exact = fidelity(
            sequence,
            &doublet_null_seeded(sequence, null_seed(index, 0)),
        );
        assert_eq!(exact.transition_tv, 0.0);
        assert_eq!(exact.max_state_tv, 0.0);
        assert_eq!(exact.absent_transitions, 0);
        assert_eq!(exact.marginal_tv, 0.0);
    }
    let fitted: Vec<_> = (0..20)
        .map(|index| fidelity(sequence, &markov_null_seeded(sequence, null_seed(index, 0))))
        .collect();
    assert!(
        fitted.iter().all(|measured| measured.transition_tv > 0.0),
        "a resampled chain does not reproduce its own transition frequencies"
    );
    assert!(
        fitted
            .iter()
            .any(|measured| measured.absent_transitions > 0),
        "and at these lengths it misses observed transitions entirely"
    );
}

/// sprint:19's null is the first construction in the table, unchanged, so the
/// two rounds are paired rather than merely adjacent.
#[test]
fn the_order_null_is_carried_forward_unchanged() {
    let control = first_order_negative();
    let (name, construct) = CONSTRUCTIONS[0];
    assert_eq!(name, "order");
    assert_eq!(
        construct(&control.first, null_seed(11, 0)),
        order_null_seeded(&control.first, null_seed(11, 0)),
        "the paired baseline must be sprint:19's own null, called the same way"
    );
}

// ---------------------------------------------------------------------------
// The controlled fixtures
// ---------------------------------------------------------------------------

/// **Planting creates no transition the background could not have produced.**
/// The whole point of choosing plant sites by entry and exit condition, and the
/// reason the positive control's first-order contamination is a frequency shift
/// and not a support change.
#[test]
fn the_planted_control_introduces_no_transition_outside_the_background_support() {
    for control in [first_order_negative(), first_order_positive()] {
        for sequence in [&control.first, &control.second] {
            for pair in control_states(sequence).windows(2) {
                assert!(
                    SUCCESSORS[pair[0]].contains(&pair[1]),
                    "transition {} -> {} is outside the background chain",
                    pair[0],
                    pair[1]
                );
            }
        }
    }
}

/// The two controls share one background walk and differ only inside the
/// planted windows, so the fixture pair isolates the plant.
#[test]
fn the_two_controls_differ_only_where_the_figure_was_planted() {
    let negative = first_order_negative();
    let positive = first_order_positive();
    let figure = transition_null::FIRST_ORDER_FIGURE.len();
    for (without, with, sites) in [
        (&negative.first, &positive.first, positive.planted.0.clone()),
        (
            &negative.second,
            &positive.second,
            positive.planted.1.clone(),
        ),
    ] {
        assert_eq!(sites.len(), 2, "each sequence carries two plants");
        let (left, right) = (control_states(without), control_states(with));
        assert_eq!(left.len(), right.len());
        for (position, (before, after)) in left.iter().zip(&right).enumerate() {
            if before == after {
                continue;
            }
            assert!(
                sites
                    .iter()
                    .any(|site| position >= *site && position < site + figure),
                "position {position} differs outside every planted window {sites:?}"
            );
        }
    }
}

/// The figure is a legal path of the background chain, so nothing about it is
/// available from the transition support alone.
#[test]
fn the_planted_figure_is_a_legal_path_of_the_background_chain() {
    for pair in transition_null::FIRST_ORDER_FIGURE.windows(2) {
        assert!(SUCCESSORS[pair[0]].contains(&pair[1]));
    }
    assert_eq!(
        transition_null::FIRST_ORDER_FIGURE.len(),
        12,
        "the ladder's longest span"
    );
}

/// The fixture must be able to discriminate at all: the plant puts a shared run
/// into the pair that the exact first-order null does not reproduce, while the
/// unplanted pair looks ordinary. Measured on a diagnostic that is **not** `T`.
#[test]
fn the_positive_fixture_plants_a_shared_run_the_exact_null_does_not_reproduce() {
    let negative = first_order_negative();
    let positive = first_order_positive();

    let planted = longest_shared_run(&positive.first, &positive.second);
    let unplanted = longest_shared_run(&negative.first, &negative.second);
    assert!(
        planted > unplanted,
        "planted {planted} must exceed unplanted {unplanted}"
    );

    let mut null_runs: Vec<usize> = (0..60)
        .map(|index| {
            longest_shared_run(
                &doublet_null_seeded(&positive.first, null_seed(index, 0)),
                &doublet_null_seeded(&positive.second, null_seed(index, 1)),
            )
        })
        .collect();
    null_runs.sort_unstable();
    assert!(
        planted > *null_runs.last().expect("a null run"),
        "the planted run {planted} must clear every null run, the longest being {:?}",
        null_runs.last()
    );
}

/// The fixtures are synthetic and obviously so, per `CLAUDE.md` §5, and carry no
/// receipts because they stand for no record.
#[test]
fn the_first_order_fixtures_are_obviously_synthetic() {
    for control in [first_order_negative(), first_order_positive()] {
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
            assert!(vocabulary_size(sequence) <= SYNTHETIC_TOOLS.len());
        }
    }
}

// ---------------------------------------------------------------------------
// The diagnostics themselves
// ---------------------------------------------------------------------------

/// The destroy-side counters count what they say they count.
#[test]
fn the_descriptive_counters_are_the_quantities_they_are_named_for() {
    let control = first_order_positive();
    // A figure planted twice contributes repeated n-grams; the order null, which
    // destroys adjacency outright, must reproduce far fewer of them.
    let observed = repeated_ngrams(&control.first, 4);
    let nulled = repeated_ngrams(&order_null_seeded(&control.first, null_seed(0, 0)), 4);
    assert!(
        observed > nulled,
        "observed {observed} repeated 4-grams against {nulled} under the order null"
    );

    // The longest shared run is symmetric and is a length, never a mark.
    assert_eq!(
        longest_shared_run(&control.first, &control.second),
        longest_shared_run(&control.second, &control.first)
    );
    assert!(longest_shared_run(&control.first, &control.first) == control.first.len());
}

/// A degenerate construction is reported as degenerate rather than silently
/// producing a null distribution that cannot move.
#[test]
fn degeneracy_reports_a_construction_that_returns_its_own_input() {
    let control = first_order_negative();
    let frozen = degeneracy(&control.second, 25, |sequence, _| sequence.clone());
    assert_eq!(frozen.identical, 25);
    assert_eq!(frozen.distinct, 1);
    assert!((frozen.identical_fraction - 1.0).abs() < 1e-12);
}

// ---------------------------------------------------------------------------
// The calibration, and the properties that make the two rounds paired
// ---------------------------------------------------------------------------

use witnessglass::experiment::calibration::{
    self, LADDER, TAIL_THRESHOLD, calibrate, calibrate_with, complete_search,
};

/// A cheap replicate count for machinery checks that assert no threshold.
const SMALL: usize = 49;

/// The smallest replicate count at which the preregistered threshold is
/// attainable: `(1 + 0)/(99 + 1) = 0.01`. sprint:19 learned this the hard way and
/// the lesson is applied here rather than rediscovered.
const RESOLVING: usize = 99;

/// sprint:19's `calibrate` is this round's `calibrate_with` called with the
/// order null, so nothing but the null can differ between the two rounds.
#[test]
fn the_paired_baseline_is_the_same_function_with_the_same_null() {
    let control = first_order_negative();
    assert_eq!(
        calibrate("pair", &control.first, &control.second, 6, 9),
        calibrate_with(
            "pair",
            &control.first,
            &control.second,
            6,
            9,
            order_null_seeded
        ),
        "calibrate must be calibrate_with(order_null_seeded), or the rounds are not paired"
    );
}

/// **The claim this round inherits and must keep.** The null path re-searches;
/// it does not rescore the boundaries the observed search chose.
#[test]
fn the_first_order_null_path_reruns_the_complete_search() {
    let control = first_order_positive();
    let k = 12;
    let observed = complete_search(&control.first, &control.second, k)
        .t
        .expect("an observed T");

    let left = doublet_null_seeded(&control.first, null_seed(0, 0));
    let right = doublet_null_seeded(&control.second, null_seed(0, 1));
    let searched = complete_search(&left, &right, k).t.expect("a null T");

    // The two are computed by one function, and the replicate's own winner is
    // found by searching the replicate rather than by reading observed spans.
    assert!(searched.is_finite() && observed.is_finite());
    assert!(
        searched != observed,
        "a complete search of a replicate that differs from the observation must not \
         return the observation's own score"
    );
}

/// `T` on the observed data cannot depend on which null it will be compared
/// against, and the paired table says so in every row.
#[test]
fn the_observed_statistic_is_identical_under_both_nulls() {
    let control = first_order_negative();
    for k in LADDER {
        let order = calibrate("t", &control.first, &control.second, k, 5);
        let first_order = calibrate_with(
            "t",
            &control.first,
            &control.second,
            k,
            5,
            doublet_null_seeded,
        );
        assert_eq!(order.observed, first_order.observed);
        assert_eq!(order.observed_considered, first_order.observed_considered);
    }
}

/// The two counts partition the realizations, so a percentile computed from one
/// agrees with a tail computed from the other.
#[test]
fn the_exceedance_and_below_counts_partition_the_null_distribution() {
    let control = first_order_positive();
    let row = calibrate_with(
        "split",
        &control.first,
        &control.second,
        8,
        SMALL,
        doublet_null_seeded,
    );
    assert_eq!(row.below + row.exceedances, row.realizations);
    assert_eq!(row.samples.len(), row.realizations);
    assert!(
        row.samples.windows(2).all(|pair| pair[0] <= pair[1]),
        "the retained samples must be ascending, since the plot bins them"
    );
}

/// The negative control is generated *by the first-order chain*, so its observed
/// `T` must look ordinary against a null that preserves that chain's counts.
#[test]
fn the_first_order_negative_control_looks_ordinary_against_its_own_null() {
    let control = first_order_negative();
    let flagged: Vec<usize> = LADDER
        .into_iter()
        .filter(|k| {
            calibrate_with(
                "neg",
                &control.first,
                &control.second,
                *k,
                SMALL,
                doublet_null_seeded,
            )
            .exceptional
        })
        .collect();
    assert!(
        flagged.is_empty(),
        "a specimen drawn from the null must not be flagged; flagged at k={flagged:?}"
    );
}

/// The positive control plants a figure whose recovery is not available from the
/// transition counts, and the search must recover it at the planted length.
#[test]
fn the_first_order_positive_control_becomes_exceptional_at_the_planted_length() {
    assert!(
        1.0 / (RESOLVING as f64 + 1.0) <= TAIL_THRESHOLD,
        "the threshold must be attainable at this replicate count"
    );
    let control = first_order_positive();
    let row = calibrate_with(
        "pos",
        &control.first,
        &control.second,
        12,
        RESOLVING,
        doublet_null_seeded,
    );
    assert!(
        row.exceptional,
        "p-hat {} must clear the threshold; T {:?} against a null max of {}",
        row.tail, row.observed, row.null_max
    );
}

/// The round adopted nothing and changed no statistic.
#[test]
fn sprint_20_adopted_nothing() {
    use witnessglass::experiment::repair;
    assert!(
        !repair::candidate("R1 pooled sum").expect("R1").frozen,
        "R1 remains a proposal"
    );
    assert_eq!(repair::CANDIDATES.len(), 4, "no new score was invented");
    assert_eq!(calibration::LADDER, [3, 4, 6, 8, 12]);
    assert_eq!(calibration::REPLICATES, 999);
    assert_eq!(calibration::TAIL_THRESHOLD, 0.01);
    assert_eq!(calibration::KEEP, 5);
    assert_eq!(calibration::TOP_K, 5);
}

// ---------------------------------------------------------------------------
// The rendering — presentation only
// ---------------------------------------------------------------------------

/// The card must draw both nulls against one observed value and must not let a
/// separation read as a motif finding.
#[test]
fn the_paired_card_draws_both_nulls_and_states_what_separation_would_earn() {
    let document = serde_json::json!({
        "replicates": 999,
        "controls": [],
        "degeneracy": [{
            "specimen": "aaaaaaaa",
            "degeneracy": { "replicates": 999, "identical": 371, "distinct": 600,
                            "identical_fraction": 0.371 },
        }],
        "paired": [{
            "specimen": "aaaaaaaa x bbbbbbbb", "k": 12, "observed": 20.0, "degenerate": false,
            "order": { "null_median": 8.0, "tail": 0.001, "exceptional": true },
            "first_order": { "null_median": 19.0, "tail": 0.40, "exceptional": false },
            "median_shift": 11.0,
        }],
        "plotted": [{
            "specimen": "aaaaaaaa x bbbbbbbb", "k": 12, "observed": 20.0,
            "order": vec![7.0f64, 8.0, 9.0],
            "first_order": vec![18.0f64, 19.0, 21.0],
        }],
        "summary": { "defined": 30, "eligible": 30, "order_separating": 23,
                     "first_order_separating": 2, "retained": 2 },
        "verdict": "B COLLAPSES UNDER THE FIRST-ORDER NULL",
    });
    let page = witnessglass::experiment::boundary_page::render(std::slice::from_ref(&document));

    assert!(page.contains("Nothing here is adopted"));
    assert!(
        page.contains("Only the null moved"),
        "the card must say what changed between the rounds"
    );
    assert!(
        page.contains("insufficient to explain what the search finds"),
        "what separation would earn must be stated wherever tails are shown"
    );
    assert!(
        page.contains("observational specimens rather"),
        "recordings are specimens, and the page must say so"
    );
    assert!(page.contains("<rect"), "both distributions must be drawn");
    assert!(
        page.contains("class=\"observed\""),
        "the observed T must be marked on the shared axis"
    );
    assert!(
        page.contains("Partial degeneracy"),
        "a null that is partly the observation itself must say so"
    );
    assert!(
        page.contains("lost"),
        "a cell that stopped separating is named"
    );
}

/// An undefined `T` is named rather than drawn at zero, and an empty
/// distribution is an absence rather than an empty plot.
#[test]
fn an_undefined_statistic_is_named_rather_than_plotted() {
    let document = serde_json::json!({
        "replicates": 999,
        "controls": [], "degeneracy": [], "paired": [],
        "plotted": [
            { "specimen": "cccccccc x dddddddd", "k": 12, "observed": serde_json::Value::Null,
              "order": vec![1.0f64, 2.0], "first_order": vec![1.5f64, 2.5] },
            { "specimen": "eeeeeeee x ffffffff", "k": 12, "observed": 3.0,
              "order": Vec::<f64>::new(), "first_order": Vec::<f64>::new() },
        ],
        "summary": {},
        "verdict": "C FIRST-ORDER NULL INADEQUATE",
    });
    let page = witnessglass::experiment::boundary_page::render(std::slice::from_ref(&document));
    assert!(
        page.contains("T undefined, so nothing is marked"),
        "an undefined T must be named, never drawn at zero"
    );
    assert!(
        page.contains("That is an absence, not a zero"),
        "an absent distribution must be named"
    );
}

/// The construction card must carry the measurement that motivated the change.
#[test]
fn the_construction_card_shows_why_the_null_was_replaced() {
    let document = serde_json::json!({
        "replicates": 199,
        "fidelity": [
            { "specimen": "aaaaaaaa", "construction": "doublet", "transition_tv": 0.0,
              "max_state_tv": 0.0, "absent_transitions": 0.0, "marginal_tv": 0.0 },
            { "specimen": "aaaaaaaa", "construction": "markov", "transition_tv": 0.29,
              "max_state_tv": 1.0, "absent_transitions": 7.0, "marginal_tv": 0.19 },
        ],
        "summaries": [{
            "specimen": "aaaaaaaa", "construction": "order",
            "summary": { "name": "immediate repetition rate", "observed": 0.0,
                         "null_median": 0.2857, "null_min": 0.1786, "null_max": 0.375,
                         "exceedances": 199, "outside_null_range": true },
        }],
        "shared_runs": [{ "pair": "aaaaaaaa x bbbbbbbb", "construction": "doublet",
                          "observed": 26.0, "null_median": 9.0, "null_min": 5.0,
                          "null_max": 18.0 }],
    });
    let page = witnessglass::experiment::boundary_page::render(std::slice::from_ref(&document));
    assert!(page.contains("no verdict is reached in it"));
    assert!(
        page.contains("preserved in expectation is not preserved"),
        "the distinction the round turns on must be stated where it is measured"
    );
    assert!(
        page.contains("outside null range"),
        "a destroyed nuisance property must be marked"
    );
    assert!(
        page.contains("It is a length, never a mark"),
        "the descriptive diagnostic must say what it is not"
    );
}
