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
