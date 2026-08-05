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
    ChannelScope, MarkedEvent, align, disjoint, ladder, neighbours, order_null, perturbation,
    project, timing_null, timing_term, top_pairs, top_pairs_where,
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
