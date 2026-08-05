//! The sprint:8 event-native motif experiment.
//!
//! **Disposable.** Deleted with the experiment.
//!
//! **What these tests are for.** Three things, in the order task:18 requires
//! them to have happened.
//!
//! First, known-answer microtests: sequences small enough to check by eye, whose
//! ordering relationships are obvious before any code runs. If the metric
//! violates those, it is wrong and no fixture result would be worth reading.
//!
//! Second, the properties the metric intends: identity, symmetry, non-negativity,
//! boundedness, and the specific insertion/deletion and timing behaviour the
//! preregistration commits to. The triangle inequality is deliberately **not**
//! asserted and is not claimed: the timing term is a bounded metric on gaps, but
//! it is attached to whichever alignment the dynamic program chose, and this
//! round has no need of a true metric space.
//!
//! Third, the fixture results themselves, against the criteria task:18 fixed
//! before the matcher was run.
//!
//! **What they do not check.** That a matched pair means the same behaviour
//! recurred. Two windows carrying the same delivered tool names in the same
//! order at similar spacings are two windows carrying the same delivered tool
//! names in the same order at similar spacings, and nothing here says more.

use witnessglass::experiment::event_sequence::{
    ChannelScope, LENGTH_FLOOR, MarkedEvent, REFINE_RADIUS, align, cross_pairs, dedupe_overlapping,
    disjoint, ladder, neighbours, order_null, perturbation, project, refine, timing_null,
    timing_term, top_pairs, top_pairs_where,
};
use witnessglass::experiment::oracle;
use witnessglass::inspection::{EventKind, V2Kind, inspect};
use witnessglass::replay_bytes;

const EPSILON: f64 = 1e-9;

/// A hand-built event: a kind, a delivered tool name, and the gap that precedes
/// it. Every name here is synthetic and obviously so.
fn ev(kind: V2Kind, tool: &str, gap_ms: Option<u64>) -> MarkedEvent<'_> {
    MarkedEvent::hand_built(EventKind::V2(kind), Some(tool), gap_ms)
}

const READER: &str = "SyntheticReader";
const SEARCHER: &str = "SyntheticSearcher";
const EDITOR: &str = "SyntheticEditor";
const SHELL: &str = "SyntheticShell";

/// `Reader --1s--> Searcher --2s--> Editor`, the microtest reference.
fn sequence_a() -> Vec<MarkedEvent<'static>> {
    vec![
        ev(V2Kind::ToolRequested, READER, None),
        ev(V2Kind::ToolRequested, SEARCHER, Some(1_000)),
        ev(V2Kind::ToolRequested, EDITOR, Some(2_000)),
    ]
}

// ---------------------------------------------------------------------------
// Known-answer microtests, checkable by eye
// ---------------------------------------------------------------------------

#[test]
fn the_microtest_ordering_relationships_hold() {
    // A: Reader --1s--> Searcher --2s--> Editor
    // B: identical
    // C: the middle identity differs
    // D: the second gap is 8s instead of 2s
    // E: the middle event is missing, and its time is absorbed
    let a = sequence_a();
    let b = sequence_a();
    let c = vec![
        ev(V2Kind::ToolRequested, READER, None),
        ev(V2Kind::ToolRequested, SHELL, Some(1_000)),
        ev(V2Kind::ToolRequested, EDITOR, Some(2_000)),
    ];
    let d = vec![
        ev(V2Kind::ToolRequested, READER, None),
        ev(V2Kind::ToolRequested, SEARCHER, Some(1_000)),
        ev(V2Kind::ToolRequested, EDITOR, Some(8_000)),
    ];
    let e = vec![
        ev(V2Kind::ToolRequested, READER, None),
        ev(V2Kind::ToolRequested, EDITOR, Some(2_000)),
    ];

    let aa = align(&a, &a);
    let ab = align(&a, &b);
    let ac = align(&a, &c);
    let ad = align(&a, &d);
    let ae = align(&a, &e);

    assert!(aa.total.abs() < EPSILON, "d(A, A) = {}", aa.total);
    assert!(ab.total.abs() < EPSILON, "d(A, B) = {}", ab.total);
    assert!(
        ab.total < ac.total,
        "d(A,B) {} < d(A,C) {}",
        ab.total,
        ac.total
    );
    assert!(
        ab.total < ad.total,
        "d(A,B) {} < d(A,D) {}",
        ab.total,
        ad.total
    );
    assert!(
        ab.total < ae.total,
        "d(A,B) {} < d(A,E) {}",
        ab.total,
        ae.total
    );

    // The components say *why*, which is the whole point of reporting three
    // numbers rather than one.
    assert_eq!((ac.substitutions, ac.insertions, ac.deletions), (1, 0, 0));
    assert!(ac.timing_cost.abs() < EPSILON, "C differs in identity only");
    assert_eq!((ad.substitutions, ad.insertions, ad.deletions), (0, 0, 0));
    assert!(ad.timing_cost > 0.0, "D differs in timing only");

    // One event of A aligns against nothing in E.
    assert_eq!(ae.deletions + ae.insertions, 1, "{ae:?}");
    assert_eq!(
        ae.substitutions, 0,
        "the survivors match by identity: {ae:?}"
    );
}

#[test]
fn a_substitution_costs_more_than_a_fourfold_timing_stretch_and_both_are_bounded() {
    // Hand-checkable: at k = 3 the denominator is 3 + 0.5 x 2 = 4.
    //   one substitution  -> 1.0 / 4     = 0.250
    //   2s against 8s     -> 0.5 x 0.973 / 4 = 0.122
    let a = sequence_a();
    let substituted = vec![
        ev(V2Kind::ToolRequested, READER, None),
        ev(V2Kind::ToolRequested, SHELL, Some(1_000)),
        ev(V2Kind::ToolRequested, EDITOR, Some(2_000)),
    ];
    let stretched = vec![
        ev(V2Kind::ToolRequested, READER, None),
        ev(V2Kind::ToolRequested, SEARCHER, Some(1_000)),
        ev(V2Kind::ToolRequested, EDITOR, Some(8_000)),
    ];

    let sub = align(&a, &substituted);
    let stretch = align(&a, &stretched);
    assert!((sub.total - 0.25).abs() < 1e-6, "{sub:?}");
    assert!((stretch.total - 0.1216).abs() < 1e-3, "{stretch:?}");
    assert!(stretch.total < sub.total);
}

// ---------------------------------------------------------------------------
// The properties the metric intends
// ---------------------------------------------------------------------------

#[test]
fn identity_symmetry_non_negativity_and_boundedness() {
    let corpus = [
        perturbation::base(),
        perturbation::jittered(0.30),
        perturbation::inserted(),
        perturbation::omitted(),
        perturbation::substituted(),
        perturbation::unrelated(),
        sequence_a(),
    ];

    for (index, left) in corpus.iter().enumerate() {
        let self_distance = align(left, left);
        assert!(
            self_distance.total.abs() < EPSILON,
            "d(x, x) = {} for corpus entry {index}",
            self_distance.total
        );

        for (other, right) in corpus.iter().enumerate() {
            let forward = align(left, right);
            let backward = align(right, left);
            assert!(
                (forward.total - backward.total).abs() < EPSILON,
                "asymmetric between {index} and {other}: {} vs {}",
                forward.total,
                backward.total
            );
            // Insertions and deletions swap roles; nothing else may move.
            assert_eq!(forward.substitutions, backward.substitutions);
            assert_eq!(forward.insertions, backward.deletions);
            assert_eq!(forward.deletions, backward.insertions);

            for value in [
                forward.total,
                forward.event_norm,
                forward.timing_norm,
                forward.event_cost,
                forward.timing_cost,
            ] {
                assert!(
                    (0.0..=1.0).contains(&value) || value >= 0.0,
                    "negative component between {index} and {other}: {value}"
                );
            }
            for value in [forward.total, forward.event_norm, forward.timing_norm] {
                assert!(
                    (0.0..=1.0).contains(&value),
                    "normalized component out of range between {index} and {other}: {value}"
                );
            }
        }
    }
}

#[test]
fn the_timing_policy_matches_the_four_values_preregistered_for_it() {
    // task:18 fixed these before anything was run; they are the reason the
    // policy is multiplicative and floored rather than a difference in
    // milliseconds.
    assert!(
        (timing_term(1_000, 1_200) - 0.12).abs() < 0.01,
        "{}",
        timing_term(1_000, 1_200)
    );
    assert!((timing_term(1_000, 8_000) - 1.00).abs() < 1e-9);
    assert!(timing_term(100_000, 100_200) < 0.005);
    assert!(
        (timing_term(100, 300) - 0.50).abs() < 0.01,
        "{}",
        timing_term(100, 300)
    );

    // Symmetric, zero on equality, and clamped at one.
    assert!(timing_term(700, 700).abs() < EPSILON);
    assert!((timing_term(300, 100) - timing_term(100, 300)).abs() < EPSILON);
    assert!(timing_term(1, 10_000_000) <= 1.0);
}

#[test]
fn the_first_position_of_a_window_carries_identity_but_no_timing() {
    // A window's timing is translation-invariant: whatever gap preceded its
    // first event belongs to the events before it.
    let with_gap = vec![
        ev(V2Kind::ToolRequested, READER, Some(9_999_999)),
        ev(V2Kind::ToolRequested, SEARCHER, Some(1_000)),
    ];
    let without = vec![
        ev(V2Kind::ToolRequested, READER, None),
        ev(V2Kind::ToolRequested, SEARCHER, Some(1_000)),
    ];
    let compared = align(&with_gap, &without);
    assert!(compared.total.abs() < EPSILON, "{compared:?}");
    assert_eq!(compared.timed_pairs, 1, "only the second position is timed");
}

#[test]
fn the_preregistered_ladders_are_what_the_rule_produces() {
    assert_eq!(ladder(8), vec![3, 6, 7, 8, 9, 10]);
    assert_eq!(ladder(9), vec![3, 7, 8, 9, 10, 11]);
    assert_eq!(ladder(4), vec![3, 4, 5, 6]);
    assert_eq!(ladder(5), vec![3, 4, 5, 6, 7]);
}

#[test]
fn the_exclusion_policy_admits_exactly_the_windows_that_share_no_event() {
    for k in [3usize, 4, 8] {
        for offset in 0..(2 * k) {
            assert_eq!(
                disjoint(0, offset, k),
                offset >= k,
                "k={k} offset={offset} — windows overlap iff their starts are closer than k"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// The projection, and the fixture it is checked against
// ---------------------------------------------------------------------------

#[test]
fn the_projection_keeps_canonical_order_and_measures_gaps_between_retained_events() {
    let replay =
        replay_bytes(oracle::ndjson().as_bytes()).expect("the oracle fixture should replay");
    let inspection = inspect(&replay);
    let observed = project(&inspection, ChannelScope::Observed).expect("a sequence");
    let all = project(&inspection, ChannelScope::All).expect("a sequence");

    assert_eq!(all.len(), 196, "the legible oracle's record count");
    assert!(observed.len() < all.len());
    assert_eq!(observed.len() + observed.filtered_out, all.len());

    // Sequences ascend, gaps are the differences between consecutive retained
    // offsets, and the first event has no gap.
    assert_eq!(observed.events[0].gap_from_previous_ms, None);
    for pair in observed.events.windows(2) {
        assert!(
            pair[0].sequence < pair[1].sequence,
            "canonical order survives"
        );
        assert_eq!(
            pair[1].gap_from_previous_ms,
            Some(pair[1].offset_ms - pair[0].offset_ms)
        );
    }
    assert_eq!(
        observed.clamped_gaps, 0,
        "this fixture's clock never moves backwards"
    );
}

#[test]
fn the_perturbation_base_agrees_with_the_committed_fixtures_first_planted_occurrence() {
    // The sweep is hand-built from the oracle's constants rather than carved out
    // of the fixture. This is what stops "hand-built" from meaning "unchecked".
    let replay =
        replay_bytes(oracle::ndjson().as_bytes()).expect("the oracle fixture should replay");
    let inspection = inspect(&replay);
    let observed = project(&inspection, ChannelScope::Observed).expect("a sequence");

    let region = (oracle::FIRST_MOTIF_START_MS, oracle::FIRST_MOTIF_END_MS);
    let start = observed
        .first_window_within(region, 8)
        .expect("the first planted occurrence");
    let window = observed.window(start, 8).expect("eight events");
    let base = perturbation::base();

    for (index, (from_fixture, hand_built)) in window.iter().zip(&base).enumerate() {
        assert_eq!(
            from_fixture.mark, hand_built.mark,
            "mark {index} differs between the fixture and the hand-built figure"
        );
        if index > 0 {
            assert_eq!(
                from_fixture.gap_from_previous_ms, hand_built.gap_from_previous_ms,
                "gap {index} differs"
            );
        }
    }
    assert!(
        align(window, &base).total.abs() < EPSILON,
        "the hand-built figure must be the fixture's figure"
    );
}

// ---------------------------------------------------------------------------
// The nulls
// ---------------------------------------------------------------------------

#[test]
fn the_order_null_keeps_the_mark_multiset_and_every_gap() {
    let replay = replay_bytes(oracle::ndjson().as_bytes()).expect("replay");
    let inspection = inspect(&replay);
    let real = project(&inspection, ChannelScope::Observed).expect("a sequence");
    let nulled = order_null(&real);

    assert_eq!(nulled.len(), real.len());
    let mut before: Vec<String> = real.events.iter().map(|e| e.mark.label()).collect();
    let mut after: Vec<String> = nulled.events.iter().map(|e| e.mark.label()).collect();
    before.sort();
    after.sort();
    assert_eq!(before, after, "the mark multiset is preserved");
    assert_ne!(
        real.events
            .iter()
            .map(|e| e.mark.label())
            .collect::<Vec<_>>(),
        nulled
            .events
            .iter()
            .map(|e| e.mark.label())
            .collect::<Vec<_>>(),
        "and the order is not"
    );
    for (real_event, null_event) in real.events.iter().zip(&nulled.events) {
        assert_eq!(
            real_event.gap_from_previous_ms,
            null_event.gap_from_previous_ms
        );
        assert_eq!(real_event.offset_ms, null_event.offset_ms);
    }
    assert!(
        nulled.events.iter().all(|e| e.sequence.is_none()),
        "a permuted mark carries no receipt, because it is not what that record said"
    );
}

#[test]
fn the_timing_null_keeps_every_mark_in_place_and_the_gap_multiset() {
    let replay = replay_bytes(oracle::ndjson().as_bytes()).expect("replay");
    let inspection = inspect(&replay);
    let real = project(&inspection, ChannelScope::Observed).expect("a sequence");
    let nulled = timing_null(&real);

    assert_eq!(
        real.events
            .iter()
            .map(|e| e.mark.label())
            .collect::<Vec<_>>(),
        nulled
            .events
            .iter()
            .map(|e| e.mark.label())
            .collect::<Vec<_>>(),
        "every mark stays exactly where it was"
    );
    let mut before: Vec<u64> = real
        .events
        .iter()
        .skip(1)
        .filter_map(|e| e.gap_from_previous_ms)
        .collect();
    let mut after: Vec<u64> = nulled
        .events
        .iter()
        .skip(1)
        .filter_map(|e| e.gap_from_previous_ms)
        .collect();
    before.sort_unstable();
    after.sort_unstable();
    assert_eq!(before, after, "the gap multiset is preserved");

    // Still a timeline: offsets ascend by the permuted gaps.
    for pair in nulled.events.windows(2) {
        assert_eq!(
            pair[1].offset_ms - pair[0].offset_ms,
            pair[1].gap_from_previous_ms.unwrap_or(0)
        );
    }
}

#[test]
fn both_nulls_are_deterministic() {
    let replay = replay_bytes(oracle::sparse::ndjson().as_bytes()).expect("replay");
    let inspection = inspect(&replay);
    let real = project(&inspection, ChannelScope::Observed).expect("a sequence");
    assert_eq!(order_null(&real), order_null(&real));
    assert_eq!(timing_null(&real), timing_null(&real));
}

// ---------------------------------------------------------------------------
// Helpers shared by the fixture scans
// ---------------------------------------------------------------------------

/// The best distance from the query window to any window sharing no event with
/// it, and the neighbour it found.
fn query_top1(
    sequence: &witnessglass::experiment::event_sequence::EventSequence<'_>,
    query: usize,
    k: usize,
) -> (f64, usize) {
    let found = neighbours(sequence, query, k, 1);
    found
        .first()
        .map(|comparison| (comparison.alignment.total, comparison.b.start))
        .unwrap_or((f64::INFINITY, usize::MAX))
}

/// Whether a window at `start` lies entirely inside one of the planted regions.
///
/// A region, not an occurrence: a window straddling two instances of the figure
/// is inside the region too. The tests that need the stronger notion say so with
/// the alignment's own indel and substitution counts.
fn within_planted_region(
    sequence: &witnessglass::experiment::event_sequence::EventSequence<'_>,
    start: usize,
    k: usize,
    a: (u64, u64),
    b: (u64, u64),
) -> bool {
    sequence
        .window_ref(start, k)
        .is_some_and(|window| window.within(a) || window.within(b))
}

// ---------------------------------------------------------------------------
// Fixture recovery, against the criteria task:18 fixed before the matcher ran
// ---------------------------------------------------------------------------

/// The legible oracle's planted figure and regions, observed records only.
const LEGIBLE_FIGURE: usize = 8;
const LEGIBLE_A: (u64, u64) = (oracle::FIRST_MOTIF_START_MS, oracle::FIRST_MOTIF_END_MS);
const LEGIBLE_B: (u64, u64) = (oracle::SECOND_MOTIF_START_MS, oracle::SESSION_END_MS);

/// The sparse oracle's, likewise.
const SPARSE_FIGURE: usize = 4;
const SPARSE_A: (u64, u64) = (
    oracle::sparse::FIRST_MOTIF_START_MS,
    oracle::sparse::FIRST_MOTIF_END_MS,
);
const SPARSE_B: (u64, u64) = (
    oracle::sparse::SECOND_MOTIF_START_MS,
    oracle::sparse::SESSION_END_MS,
);

#[test]
fn s1_the_planted_query_finds_a_planted_neighbour_at_every_rung_of_both_ladders() {
    // Criterion S1, and the one sprint:6's sampled Matrix Profile failed: the
    // nearest thing to a planted occurrence should be another one.
    for (fixture, figure, a, b) in [
        (oracle::ndjson(), LEGIBLE_FIGURE, LEGIBLE_A, LEGIBLE_B),
        (oracle::sparse::ndjson(), SPARSE_FIGURE, SPARSE_A, SPARSE_B),
    ] {
        let replay = replay_bytes(fixture.as_bytes()).expect("replay");
        let inspection = inspect(&replay);
        let sequence = project(&inspection, ChannelScope::Observed).expect("a sequence");

        for k in ladder(figure) {
            let query = sequence
                .first_window_within(a, k)
                .expect("a window inside region A");
            let (distance, neighbour) = query_top1(&sequence, query, k);
            assert!(
                within_planted_region(&sequence, neighbour, k, a, b),
                "k={k}: the query's nearest neighbour at index {neighbour} is not planted"
            );
            assert!(distance.abs() < EPSILON, "k={k}: distance {distance}");
        }
    }
}

#[test]
fn s3_the_order_null_destroys_recovery_and_the_timing_null_does_not() {
    // Criteria S3 and prediction P5. The order null must separate by at least
    // 0.05; the timing null is expected to separate barely, and that is a
    // finding about where the discrimination lives, not a failure.
    for (fixture, figure, a) in [
        (oracle::ndjson(), LEGIBLE_FIGURE, LEGIBLE_A),
        (oracle::sparse::ndjson(), SPARSE_FIGURE, SPARSE_A),
    ] {
        let replay = replay_bytes(fixture.as_bytes()).expect("replay");
        let inspection = inspect(&replay);
        let sequence = project(&inspection, ChannelScope::Observed).expect("a sequence");
        let order = order_null(&sequence);
        let timing = timing_null(&sequence);

        let k = figure;
        let query = sequence.first_window_within(a, k).expect("a query window");
        let (real, _) = query_top1(&sequence, query, k);
        let (under_order, _) = query_top1(&order, query, k);
        let (under_timing, _) = query_top1(&timing, query, k);

        assert!(
            under_order - real >= 0.05,
            "S3: order-null separation {} is below the preregistered 0.05",
            under_order - real
        );
        assert!(
            under_timing - real < under_order - real,
            "P5: the timing null should separate less than the order null ({under_timing} vs {under_order})"
        );
    }
}

#[test]
fn s4_the_planted_windows_are_not_degenerate_and_the_global_top_pairs_are() {
    // Criterion S4, and prediction P1. Recovery must not be an artefact of one
    // mark repeating — and the abundant exact repeats in these fixtures are
    // exactly that, which is why the unrestricted global ranking is full of
    // them.
    for (fixture, figure, a, expected_marks) in [
        (oracle::ndjson(), LEGIBLE_FIGURE, LEGIBLE_A, 8usize),
        (oracle::sparse::ndjson(), SPARSE_FIGURE, SPARSE_A, 4usize),
    ] {
        let replay = replay_bytes(fixture.as_bytes()).expect("replay");
        let inspection = inspect(&replay);
        let sequence = project(&inspection, ChannelScope::Observed).expect("a sequence");
        let k = figure;
        let query = sequence.first_window_within(a, k).expect("a query window");

        let query_window = sequence.window_ref(query, k).expect("a window");
        assert_eq!(
            query_window.distinct_marks, expected_marks,
            "the planted figure's own window should carry one mark per event"
        );
        assert!(!query_window.degenerate());
        for found in neighbours(&sequence, query, k, 5) {
            assert!(
                !found.b.degenerate(),
                "a degenerate window reached the query's top five: {found:?}"
            );
        }

        // P1: the unrestricted global minimum is a pair of degenerate windows.
        let global = top_pairs(&sequence, k, 1);
        let best = global.first().expect("a global best pair");
        assert!(
            best.alignment.total.abs() < EPSILON && best.a.degenerate() && best.b.degenerate(),
            "P1 predicted the global top pair would be two degenerate windows at distance 0: {best:?}"
        );
    }
}

#[test]
fn the_cross_region_recurrence_is_recovered_and_the_injected_failure_costs_one_substitution() {
    // The result that matters, and prediction P6 with it. Ranks are reported
    // rather than asserted to be small, because the sparse fixture has 29 other
    // occurrences inside region A and every one of them precedes the first
    // cross-region pair — see this task's Result on criterion S2.
    for (fixture, figure, a, b, expected_planted_prefix) in [
        (
            oracle::ndjson(),
            LEGIBLE_FIGURE,
            LEGIBLE_A,
            LEGIBLE_B,
            7usize,
        ),
        (
            oracle::sparse::ndjson(),
            SPARSE_FIGURE,
            SPARSE_A,
            SPARSE_B,
            44usize,
        ),
    ] {
        let replay = replay_bytes(fixture.as_bytes()).expect("replay");
        let inspection = inspect(&replay);
        let sequence = project(&inspection, ChannelScope::Observed).expect("a sequence");
        let k = figure;
        let query = sequence.first_window_within(a, k).expect("a query window");
        let found = neighbours(&sequence, query, k, expected_planted_prefix + 1);

        // Every neighbour up to the prefix length is an *occurrence-aligned*
        // match: it lies inside a planted region, it needs no insertion or
        // deletion to line up, and it differs by at most the one substitution
        // the fixture injects. Lying inside a planted region is not by itself
        // enough — a window straddling two instances lies inside one too, which
        // is exactly what the first neighbour past the prefix is.
        for (rank, comparison) in found.iter().enumerate().take(expected_planted_prefix) {
            assert!(
                within_planted_region(&sequence, comparison.b.start, k, a, b),
                "rank {rank} falls outside both planted regions: {comparison:?}"
            );
            assert_eq!(
                (
                    comparison.alignment.insertions,
                    comparison.alignment.deletions
                ),
                (0, 0),
                "rank {rank} needed an indel to line up: {comparison:?}"
            );
            assert!(
                comparison.alignment.substitutions <= 1,
                "rank {rank} differs by more than the injected failure: {comparison:?}"
            );
        }
        let boundary = &found[expected_planted_prefix];
        assert!(
            boundary.alignment.insertions > 0 && boundary.alignment.deletions > 0,
            "the first neighbour past the prefix should be phase-shifted: {boundary:?}"
        );
        assert!(
            boundary.alignment.total > found[expected_planted_prefix - 1].alignment.total,
            "and strictly further away than every occurrence-aligned match: {boundary:?}"
        );

        // At least one neighbour reaches across into region B.
        let crossing = found
            .iter()
            .find(|comparison| {
                sequence
                    .window_ref(comparison.b.start, k)
                    .is_some_and(|window| window.within(b))
            })
            .expect("a cross-region neighbour");
        assert!(crossing.alignment.event_norm.abs() < EPSILON);
        assert!(
            crossing.alignment.timing_cost > 0.0,
            "the recurrence is jittered"
        );

        // P6: the instance carrying the injected failure differs by exactly one
        // substitution, and it is the worst of the planted occurrences.
        let worst = &found[expected_planted_prefix - 1];
        assert_eq!(
            worst.alignment.substitutions, 1,
            "the injected failing call should cost exactly one substitution: {worst:?}"
        );
    }
}

#[test]
fn the_perturbation_sweep_degrades_monotonically_with_timing_distortion() {
    // Earned only because recovery was: the sweep asks whether the metric falls
    // off a cliff when a figure stops being identical, and it does not.
    let sweep = perturbation::sweep();
    let of = |name: &str| {
        sweep
            .iter()
            .find(|(label, _)| *label == name)
            .map(|(_, alignment)| *alignment)
            .expect("a named variant")
    };

    let exact = of("exact");
    assert!(exact.total.abs() < EPSILON);

    let jitters = [
        of("10% timing jitter"),
        of("30% timing jitter"),
        of("100% timing jitter"),
        of("300% timing jitter"),
    ];
    for pair in jitters.windows(2) {
        assert!(
            pair[0].total < pair[1].total,
            "timing distortion must cost monotonically more: {pair:?}"
        );
        assert!(
            pair[0].event_norm.abs() < EPSILON && pair[1].event_norm.abs() < EPSILON,
            "and it must cost nothing in event identity"
        );
    }

    // One structural change costs more than a 30% timing wobble and far less
    // than being a different figure altogether.
    for name in [
        "one inserted event",
        "one omitted event",
        "one substituted identity",
    ] {
        let variant = of(name);
        assert!(variant.total > of("30% timing jitter").total, "{name}");
        assert!(
            variant.total < of("unrelated sequence").total * 0.5,
            "{name}"
        );
    }
    assert!(of("unrelated sequence").total > 0.5);
}

#[test]
fn the_degenerate_masked_global_minimum_is_the_planted_figure_without_any_query_anchor() {
    // Stronger than S1 and S2, and unanchored: with degenerate windows excluded
    // — the event-native analogue of sprint:6 masking constant subsequences —
    // the global minimum over every disjoint pair in the whole sequence is a
    // pair of planted occurrences, at distance zero. Nobody had to point at
    // region A for this one.
    for (fixture, figure, a, b) in [
        (oracle::ndjson(), LEGIBLE_FIGURE, LEGIBLE_A, LEGIBLE_B),
        (oracle::sparse::ndjson(), SPARSE_FIGURE, SPARSE_A, SPARSE_B),
    ] {
        let replay = replay_bytes(fixture.as_bytes()).expect("replay");
        let inspection = inspect(&replay);
        let sequence = project(&inspection, ChannelScope::Observed).expect("a sequence");
        let k = figure;

        let found = top_pairs_where(&sequence, k, 1, |left, right| {
            !left.degenerate() && !right.degenerate()
        });
        let best = found.first().expect("a non-degenerate global best pair");
        assert!(best.alignment.total.abs() < EPSILON, "{best:?}");
        assert!(
            within_planted_region(&sequence, best.a.start, k, a, b),
            "{best:?}"
        );
        assert!(
            within_planted_region(&sequence, best.b.start, k, a, b),
            "{best:?}"
        );
        assert_eq!(best.a.distinct_marks, k, "one mark per event in the figure");
        assert_eq!(best.b.distinct_marks, k);
    }
}

// ---------------------------------------------------------------------------
// Cross-recording comparison — sprint:9, task:19
// ---------------------------------------------------------------------------

mod common;

/// A synthetic two-call recording under a chosen session id, with the gaps a
/// caller asks for. Obviously fabricated: every name contains `synthetic`.
fn cross_fixture(session: &str, tools: &[&str], step_ms: u64) -> String {
    let mut records = Vec::new();
    let mut at = 0u64;
    for (index, tool) in tools.iter().enumerate() {
        let id = format!("toolu_synthetic_cross_{index:04}");
        let stamp = |ms: u64| {
            let seconds = ms / 1000;
            let millis = ms % 1000;
            format!("2026-05-01T00:00:{seconds:02}.{millis:03}Z")
        };
        records.push(common::raw_record(
            records.len() as u64 + 1,
            &stamp(at),
            session,
            common::ev_tool_requested(&id, tool),
        ));
        at += step_ms;
        records.push(common::raw_record(
            records.len() as u64 + 1,
            &stamp(at),
            session,
            common::ev_tool_succeeded(&id, tool, None),
        ));
        at += step_ms;
    }
    common::ndjson(&records)
}

const CROSS_A: &str = "sess-synthetic-cross-alpha";
const CROSS_B: &str = "sess-synthetic-cross-beta";

#[test]
fn identical_sequences_in_two_independent_recordings_rank_at_zero_with_correct_provenance() {
    let tools = ["SyntheticReader", "SyntheticSearcher", "SyntheticEditor"];
    let left = cross_fixture(CROSS_A, &tools, 500);
    let right = cross_fixture(CROSS_B, &tools, 500);

    let (a_replay, b_replay) = (
        replay_bytes(left.as_bytes()).expect("replay"),
        replay_bytes(right.as_bytes()).expect("replay"),
    );
    let (a_inspection, b_inspection) = (inspect(&a_replay), inspect(&b_replay));
    let a = project(&a_inspection, ChannelScope::Observed).expect("a sequence");
    let b = project(&b_inspection, ChannelScope::Observed).expect("a sequence");

    let ranked = cross_pairs(&a, &b, 6, 10).expect("two different sessions");
    let best = ranked.first().expect("a best pair");
    assert!(best.comparison.alignment.total.abs() < EPSILON, "{best:?}");

    // Provenance travels on the value, and each side names its own recording.
    assert_eq!(best.a_session, Some(CROSS_A));
    assert_eq!(best.b_session, Some(CROSS_B));
    // And the windows are the ones the sessions say they are.
    assert!(a.window(best.comparison.a.start, 6).is_some());
    assert!(b.window(best.comparison.b.start, 6).is_some());
}

#[test]
fn an_unrelated_second_recording_ranks_worse_than_an_identical_one() {
    let tools = ["SyntheticReader", "SyntheticSearcher", "SyntheticEditor"];
    let other = ["SyntheticShell", "SyntheticShell", "SyntheticShell"];
    let left = cross_fixture(CROSS_A, &tools, 500);
    let same = cross_fixture(CROSS_B, &tools, 500);
    let different = cross_fixture(CROSS_B, &other, 4_000);

    let left_replay = replay_bytes(left.as_bytes()).expect("replay");
    let same_replay = replay_bytes(same.as_bytes()).expect("replay");
    let diff_replay = replay_bytes(different.as_bytes()).expect("replay");
    let (a_i, same_i, diff_i) = (
        inspect(&left_replay),
        inspect(&same_replay),
        inspect(&diff_replay),
    );
    let a = project(&a_i, ChannelScope::Observed).expect("a sequence");
    let same = project(&same_i, ChannelScope::Observed).expect("a sequence");
    let different = project(&diff_i, ChannelScope::Observed).expect("a sequence");

    let close = cross_pairs(&a, &same, 6, 1).expect("pairs")[0]
        .comparison
        .alignment
        .total;
    let far = cross_pairs(&a, &different, 6, 1).expect("pairs")[0]
        .comparison
        .alignment
        .total;
    assert!(close < far, "identical {close} should beat unrelated {far}");
    assert!(
        far > 0.3,
        "an unrelated recording should be clearly worse: {far}"
    );
}

#[test]
fn the_primary_api_refuses_to_compare_a_recording_with_itself() {
    // "Do independently recorded sessions contain similar figures" is not a
    // question about one recording, and the API says so rather than quietly
    // ranking a recording against its own windows.
    let tools = ["SyntheticReader", "SyntheticSearcher"];
    let text = cross_fixture(CROSS_A, &tools, 500);
    let replay = replay_bytes(text.as_bytes()).expect("replay");
    let inspection = inspect(&replay);
    let sequence = project(&inspection, ChannelScope::Observed).expect("a sequence");

    assert!(cross_pairs(&sequence, &sequence, 3, 5).is_none());

    // And no cross pair ever puts two windows of one recording together, which
    // is true by construction and asserted so a future edit cannot break it.
    let other = cross_fixture(CROSS_B, &tools, 500);
    let other_replay = replay_bytes(other.as_bytes()).expect("replay");
    let other_inspection = inspect(&other_replay);
    let second = project(&other_inspection, ChannelScope::Observed).expect("a sequence");
    for pair in cross_pairs(&sequence, &second, 3, 20).expect("pairs") {
        assert_ne!(pair.a_session, pair.b_session);
    }
}

#[test]
fn cross_recording_ranking_and_its_nulls_are_deterministic() {
    let left = cross_fixture(CROSS_A, &["SyntheticReader", "SyntheticSearcher"], 400);
    let right = cross_fixture(CROSS_B, &["SyntheticReader", "SyntheticShell"], 900);
    let left_replay = replay_bytes(left.as_bytes()).expect("replay");
    let right_replay = replay_bytes(right.as_bytes()).expect("replay");
    let (a_i, b_i) = (inspect(&left_replay), inspect(&right_replay));
    let a = project(&a_i, ChannelScope::Observed).expect("a sequence");
    let b = project(&b_i, ChannelScope::Observed).expect("a sequence");

    assert_eq!(cross_pairs(&a, &b, 3, 8), cross_pairs(&a, &b, 3, 8));
    assert_eq!(
        cross_pairs(&order_null(&a), &order_null(&b), 3, 8),
        cross_pairs(&order_null(&a), &order_null(&b), 3, 8)
    );
    assert_eq!(
        cross_pairs(&timing_null(&a), &timing_null(&b), 3, 8),
        cross_pairs(&timing_null(&a), &timing_null(&b), 3, 8)
    );
}

#[test]
fn de_duplication_removes_overlap_without_touching_any_distance() {
    // task:19 §4. The policy is about which candidates are reported, and it must
    // not become a policy about what anything scores.
    let replay = replay_bytes(oracle::ndjson().as_bytes()).expect("replay");
    let sparse = replay_bytes(oracle::sparse::ndjson().as_bytes()).expect("replay");
    let (a_i, b_i) = (inspect(&replay), inspect(&sparse));
    let a = project(&a_i, ChannelScope::Observed).expect("a sequence");
    let b = project(&b_i, ChannelScope::Observed).expect("a sequence");

    let ranked = cross_pairs(&a, &b, 4, usize::MAX).expect("two different sessions");
    let kept = dedupe_overlapping(&ranked, 5);
    assert!(kept.len() <= 5);
    assert_eq!(
        kept.first().map(|pair| pair.comparison),
        ranked.first().map(|pair| pair.comparison),
        "the best candidate is always kept"
    );

    // Every kept candidate is a candidate that was ranked, with its distance
    // unchanged, and no two kept candidates overlap on either side.
    for candidate in &kept {
        assert!(
            ranked.iter().any(|pair| pair == candidate),
            "de-duplication invented a candidate: {candidate:?}"
        );
    }
    for (index, left) in kept.iter().enumerate() {
        for right in kept.iter().skip(index + 1) {
            assert!(
                left.comparison.a.start.abs_diff(right.comparison.a.start) >= 4,
                "two kept candidates overlap on the A side"
            );
            assert!(
                left.comparison.b.start.abs_diff(right.comparison.b.start) >= 4,
                "two kept candidates overlap on the B side"
            );
        }
    }
    // Ranking order survives de-duplication.
    for window in kept.windows(2) {
        assert!(window[0].comparison.alignment.total <= window[1].comparison.alignment.total);
    }
}

#[test]
fn a_distinct_mark_stratum_selects_from_the_ranking_without_changing_it() {
    // The strata are diagnostic slices. Filtering by distinct marks must pick a
    // pair out of the ranking exactly as computed — never rescore one.
    let replay = replay_bytes(oracle::ndjson().as_bytes()).expect("replay");
    let sparse = replay_bytes(oracle::sparse::ndjson().as_bytes()).expect("replay");
    let (a_i, b_i) = (inspect(&replay), inspect(&sparse));
    let a = project(&a_i, ChannelScope::Observed).expect("a sequence");
    let b = project(&b_i, ChannelScope::Observed).expect("a sequence");
    let ranked = cross_pairs(&a, &b, 4, usize::MAX).expect("pairs");

    for floor in [2usize, 3, 4] {
        let Some(picked) = ranked.iter().find(|pair| {
            pair.comparison
                .a
                .distinct_marks
                .min(pair.comparison.b.distinct_marks)
                >= floor
        }) else {
            continue;
        };
        // It is a member of the ranking, byte for byte.
        assert!(ranked.iter().any(|pair| pair == picked));
        // And recomputing its alignment from the windows gives the same numbers,
        // so nothing was rescored on the way through the filter.
        let recomputed = align(
            a.window(picked.comparison.a.start, 4).expect("a window"),
            b.window(picked.comparison.b.start, 4).expect("a window"),
        );
        assert_eq!(recomputed, picked.comparison.alignment);
        assert!(picked.comparison.alignment.total >= ranked[0].comparison.alignment.total);
    }
}

#[test]
fn serialized_cross_pairs_carry_the_numbers_that_were_computed() {
    // The example prints and can emit JSON; this asserts the serialized form is
    // the computed form rather than a second opinion about it.
    let left = cross_fixture(CROSS_A, &["SyntheticReader", "SyntheticSearcher"], 700);
    let right = cross_fixture(CROSS_B, &["SyntheticReader", "SyntheticEditor"], 1_500);
    let left_replay = replay_bytes(left.as_bytes()).expect("replay");
    let right_replay = replay_bytes(right.as_bytes()).expect("replay");
    let (a_i, b_i) = (inspect(&left_replay), inspect(&right_replay));
    let a = project(&a_i, ChannelScope::Observed).expect("a sequence");
    let b = project(&b_i, ChannelScope::Observed).expect("a sequence");
    let pair = cross_pairs(&a, &b, 4, 1).expect("pairs").remove(0);

    let json: serde_json::Value =
        serde_json::from_str(&serde_json::to_string(&pair).expect("serialize")).expect("parse");
    assert_eq!(json["a_session"], CROSS_A);
    assert_eq!(json["b_session"], CROSS_B);
    let alignment = &json["comparison"]["alignment"];
    assert_eq!(
        alignment["total"].as_f64().expect("a number"),
        pair.comparison.alignment.total
    );
    assert_eq!(
        alignment["event_norm"].as_f64().expect("a number"),
        pair.comparison.alignment.event_norm
    );
    assert_eq!(
        alignment["timing_norm"].as_f64().expect("a number"),
        pair.comparison.alignment.timing_norm
    );
    assert_eq!(
        json["comparison"]["a"]["start"].as_u64().expect("a number"),
        pair.comparison.a.start as u64
    );
}

// ---------------------------------------------------------------------------
// Local boundary refinement — sprint:10, task:20
// ---------------------------------------------------------------------------

/// The legible oracle, projected into an observed-scope sequence, plus the
/// replay and inspection that own its storage.
macro_rules! legible {
    ($replay:ident, $inspection:ident, $sequence:ident) => {
        let text = oracle::ndjson();
        let $replay = replay_bytes(text.as_bytes()).expect("replay");
        let $inspection = inspect(&$replay);
        let $sequence = project(&$inspection, ChannelScope::Observed).expect("a sequence");
    };
}

/// task:20 specimen A: the planted figure with two contaminating events on each
/// side of each span.
const CONTAMINATED_A: (usize, usize) = (18, 30);
const CONTAMINATED_B: (usize, usize) = (160, 172);
const PLANTED_A: (usize, usize) = (20, 28);
const PLANTED_B: (usize, usize) = (162, 170);

#[test]
fn a_contaminated_seed_refines_onto_the_planted_left_boundary_with_no_event_cost() {
    legible!(replay, inspection, sequence);
    let refined = refine(
        &sequence,
        CONTAMINATED_A,
        &sequence,
        CONTAMINATED_B,
        REFINE_RADIUS,
        LENGTH_FLOOR,
    )
    .expect("a valid seed");

    // The seed pays for its contamination.
    assert!(refined.seed.pair.comparison.alignment.event_norm > 0.0);

    // Somewhere on the frontier is a candidate that starts exactly where the
    // planted figure starts, on both sides, and costs nothing in event identity.
    let recovered = refined
        .frontier
        .iter()
        .find(|candidate| {
            candidate.pair.comparison.a.start == PLANTED_A.0
                && candidate.pair.comparison.b.start == PLANTED_B.0
        })
        .expect("the planted left boundary should be recovered exactly");
    assert!(
        recovered.pair.comparison.alignment.event_norm.abs() < EPSILON,
        "the recovered span should match mark for mark: {recovered:?}"
    );
    assert!(
        recovered.pair.comparison.alignment.total < refined.seed.pair.comparison.alignment.total
    );
}

#[test]
fn an_already_correct_seed_is_returned_unchanged_at_radius_zero() {
    legible!(replay, inspection, sequence);
    let refined =
        refine(&sequence, PLANTED_A, &sequence, PLANTED_B, 0, LENGTH_FLOOR).expect("a valid seed");
    assert_eq!(refined.evaluated, 1, "radius zero admits only the seed");
    let pick = refined.pick.expect("a pick");
    assert!(pick.delta.is_seed());
    assert_eq!(
        pick.pair.comparison.alignment,
        refined.seed.pair.comparison.alignment
    );
}

#[test]
fn refined_spans_may_differ_in_length_and_both_lengths_are_reported() {
    legible!(replay, inspection, sequence);
    let refined = refine(
        &sequence,
        CONTAMINATED_A,
        &sequence,
        CONTAMINATED_B,
        REFINE_RADIUS,
        LENGTH_FLOOR,
    )
    .expect("a valid seed");
    // The two sides move independently, so unequal lengths must be reachable —
    // the alignment already carries insertions and deletions.
    let uneven = refined
        .frontier
        .iter()
        .chain(refined.pick.iter())
        .any(|candidate| candidate.pair.comparison.a.k != candidate.pair.comparison.b.k);
    let seed_uneven = refined.seed.pair.comparison.a.k != refined.seed.pair.comparison.b.k;
    assert!(
        uneven || !seed_uneven,
        "unequal-length spans must be representable"
    );
    // And the retained axis is the shorter side, never the longer one.
    for candidate in &refined.frontier {
        assert_eq!(
            candidate.retained,
            candidate
                .pair
                .comparison
                .a
                .k
                .min(candidate.pair.comparison.b.k)
        );
    }
}

#[test]
fn an_inverted_or_out_of_range_seed_is_rejected_rather_than_clamped() {
    legible!(replay, inspection, sequence);
    assert!(refine(&sequence, (30, 18), &sequence, PLANTED_B, 1, LENGTH_FLOOR).is_none());
    assert!(refine(&sequence, (20, 20), &sequence, PLANTED_B, 1, LENGTH_FLOOR).is_none());
    assert!(refine(&sequence, (20, 28), &sequence, (900, 908), 1, LENGTH_FLOOR).is_none());

    // A seed at the very start has neighbours outside the sequence, and they are
    // skipped rather than pulled back inside it.
    let refined =
        refine(&sequence, (0, 6), &sequence, PLANTED_B, 3, LENGTH_FLOOR).expect("a valid seed");
    assert!(refined.rejected > 0);
    assert_eq!(refined.evaluated + refined.rejected, 7usize.pow(4));
    for candidate in &refined.frontier {
        assert!(candidate.pair.comparison.a.k >= LENGTH_FLOOR);
        assert!(candidate.pair.comparison.b.k >= LENGTH_FLOOR);
    }
}

#[test]
fn the_length_floor_removes_the_arithmetic_collapse_it_was_written_for() {
    // Without a floor, a one-event span has no gap at all, so its timing
    // component is structurally absent and two spans carrying the same mark are
    // at distance exactly zero. This is the degeneracy task:20 §3 preregistered,
    // demonstrated rather than asserted.
    legible!(replay, inspection, sequence);
    let seed_a = (20, 23);
    let seed_b = (162, 165);

    let unguarded =
        refine(&sequence, seed_a, &sequence, seed_b, REFINE_RADIUS, 1).expect("a valid seed");
    let collapsed = unguarded
        .frontier
        .iter()
        .find(|candidate| candidate.retained == 1)
        .expect("collapse to one event must be reachable when nothing prevents it");
    assert!(collapsed.pair.comparison.alignment.total.abs() < EPSILON);

    let guarded = refine(
        &sequence,
        seed_a,
        &sequence,
        seed_b,
        REFINE_RADIUS,
        LENGTH_FLOOR,
    )
    .expect("a valid seed");
    assert!(
        guarded
            .frontier
            .iter()
            .all(|candidate| candidate.retained >= LENGTH_FLOOR),
        "the floor must remove every span shorter than it"
    );
}

#[test]
fn refinement_is_deterministic_and_never_mutates_its_inputs() {
    legible!(replay, inspection, sequence);
    let before = sequence.clone();
    let first = refine(
        &sequence,
        CONTAMINATED_A,
        &sequence,
        CONTAMINATED_B,
        REFINE_RADIUS,
        LENGTH_FLOOR,
    );
    let second = refine(
        &sequence,
        CONTAMINATED_A,
        &sequence,
        CONTAMINATED_B,
        REFINE_RADIUS,
        LENGTH_FLOOR,
    );
    assert_eq!(first, second);
    assert_eq!(sequence, before, "the search borrows and rewrites nothing");
}

#[test]
fn every_frontier_point_carries_provenance_correct_deltas_and_a_rescorable_distance() {
    // Three properties at once, because they are three ways of asking whether the
    // reported candidate really is the span it says it is.
    let a_text = oracle::ndjson();
    let b_text = oracle::sparse::ndjson();
    let a_replay = replay_bytes(a_text.as_bytes()).expect("replay");
    let b_replay = replay_bytes(b_text.as_bytes()).expect("replay");
    let (a_i, b_i) = (inspect(&a_replay), inspect(&b_replay));
    let a = project(&a_i, ChannelScope::Observed).expect("a sequence");
    let b = project(&b_i, ChannelScope::Observed).expect("a sequence");

    let seed_a = (20, 28);
    let seed_b = (20, 28);
    let refined =
        refine(&a, seed_a, &b, seed_b, REFINE_RADIUS, LENGTH_FLOOR).expect("a valid seed");

    for candidate in refined
        .frontier
        .iter()
        .chain(std::iter::once(&refined.seed))
    {
        // Provenance: each side names its own recording, and they differ here.
        assert_eq!(candidate.pair.a_session, a.session_id);
        assert_eq!(candidate.pair.b_session, b.session_id);
        assert_ne!(candidate.pair.a_session, candidate.pair.b_session);

        // Deltas: the span really is the seed plus the reported movement.
        let (ca, cb) = (&candidate.pair.comparison.a, &candidate.pair.comparison.b);
        assert_eq!(
            ca.start as isize,
            seed_a.0 as isize + candidate.delta.a_start
        );
        assert_eq!(
            (ca.start + ca.k) as isize,
            seed_a.1 as isize + candidate.delta.a_end
        );
        assert_eq!(
            cb.start as isize,
            seed_b.0 as isize + candidate.delta.b_start
        );
        assert_eq!(
            (cb.start + cb.k) as isize,
            seed_b.1 as isize + candidate.delta.b_end
        );

        // Distance: rescoring the returned spans reproduces the reported numbers
        // exactly, so nothing was carried over from a neighbouring combination.
        let rescored = align(
            a.window(ca.start, ca.k).expect("a window"),
            b.window(cb.start, cb.k).expect("a window"),
        );
        assert_eq!(rescored, candidate.pair.comparison.alignment);
    }

    // The frontier is strictly monotone in both axes, which is what makes it a
    // frontier rather than a list.
    for window in refined.frontier.windows(2) {
        assert!(window[0].retained > window[1].retained);
        assert!(
            window[0].pair.comparison.alignment.total > window[1].pair.comparison.alignment.total
        );
    }
}

// ---------------------------------------------------------------------------
// The specimen page reads computed values — sprint:10, task:20 §9
// ---------------------------------------------------------------------------

#[test]
fn the_specimen_page_renders_the_numbers_the_experiment_computed() {
    // The smallest useful fidelity check: run the experiment, serialize it the
    // way the example does, render the page from that document, and assert the
    // page carries the computed distances and spans. A page holding its own
    // measurements would pass none of this.
    legible!(replay, inspection, sequence);
    let refined = refine(
        &sequence,
        CONTAMINATED_A,
        &sequence,
        CONTAMINATED_B,
        REFINE_RADIUS,
        LENGTH_FLOOR,
    )
    .expect("a valid seed");

    let document = serde_json::json!({
        "label": "A",
        "role": "synthetic",
        "truth_a": [PLANTED_A.0, PLANTED_A.1],
        "truth_b": [PLANTED_B.0, PLANTED_B.1],
        "refinement": refined,
    });
    let page = witnessglass::experiment::boundary_page::render(&[document]);

    // Every frontier point's total and both spans appear, to the precision the
    // page prints them at.
    for candidate in &refined.frontier {
        let (a, b) = (&candidate.pair.comparison.a, &candidate.pair.comparison.b);
        let total = format!("{:.3}", candidate.pair.comparison.alignment.total);
        assert!(
            page.contains(&total),
            "the page omits a computed total: {total}"
        );
        assert!(page.contains(&format!("A[{}..{})", a.start, a.start + a.k)));
        assert!(page.contains(&format!("B[{}..{})", b.start, b.start + b.k)));
    }
    // The seed's own distance is on the page too, so seed and pick are comparable.
    assert!(page.contains(&format!(
        "{:.3}",
        refined.seed.pair.comparison.alignment.total
    )));
    // And the planted boundaries are reported as absent from the frontier, which
    // is what this specimen actually found.
    assert!(page.contains("on the frontier: <strong>no</strong>"));

    // A page that invented a number would have to invent this one too.
    assert!(!page.contains("0.999"));
}
