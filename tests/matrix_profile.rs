//! The sprint:6 Matrix Profile experiment.
//!
//! **Disposable.** Deleted with the experiment, along with the feature and the
//! dependency it gates.
//!
//! **What these tests are for.** Two things, and the first matters more than it
//! looks. `motif-rs` is a third-party library this project has not written and
//! will not maintain, and the risk is less that it is wrong than that *we* have
//! misunderstood it. So its treatment of trivial matches and exclusion zones,
//! constant subsequences, normalization, and index-to-time conversion is pinned
//! here against vectors small enough to check by hand. If a future version
//! changes any of those conventions, these fail loudly rather than silently
//! changing what an experiment means.
//!
//! Second, the fixture results themselves, and the preregistered criteria from
//! task:16.
//!
//! **What they do not check.** Changepoint detection, multivariate methods, or
//! any claim that a matched region means the same behaviour recurred. A
//! z-normalized distance is blind to amplitude and says nothing about intent.

#![cfg(feature = "experiment-matrix-profile")]

use motif_rs::{EuclideanEngine, MatrixProfileConfig};
use witnessglass::experiment::matrix_profile::{LADDER_MS, deterministic_shuffle, scan};

const EPSILON: f64 = 1e-9;

fn profile_of(ts: &[f64], m: usize) -> motif_rs::MatrixProfile {
    EuclideanEngine::new(MatrixProfileConfig::new(m)).compute(ts)
}

// ---------------------------------------------------------------------------
// What the library does, pinned against hand-checkable vectors
// ---------------------------------------------------------------------------

#[test]
fn an_exact_repeat_is_found_at_distance_zero_with_the_right_neighbour() {
    // [1,2,3,4] appears at index 0 and index 4. Nothing subtle.
    let ts = [1.0, 2.0, 3.0, 4.0, 1.0, 2.0, 3.0, 4.0];
    let mp = profile_of(&ts, 4);

    assert_eq!(mp.profile.len(), ts.len() - 4 + 1);
    assert!(mp.profile[0].abs() < EPSILON, "got {}", mp.profile[0]);
    assert_eq!(mp.profile_index[0], 4);
    assert!(mp.profile[4].abs() < EPSILON);
}

#[test]
fn the_exclusion_zone_is_ceil_m_over_four_and_no_neighbour_falls_inside_it() {
    // stumpy's convention, and the thing most likely to be misread. A neighbour
    // inside the zone would be a trivial match: a subsequence resembling itself
    // shifted by one sample.
    for m in [4usize, 8, 16, 32] {
        let config = MatrixProfileConfig::new(m);
        assert_eq!(config.exclusion_zone(), m.div_ceil(4), "m={m}");
    }

    let ts = [1.0, 2.0, 3.0, 4.0, 1.0, 2.0, 3.0, 4.0];
    let mp = profile_of(&ts, 4);
    assert_eq!(mp.exclusion_zone, 1);
    for (index, neighbour) in mp.profile_index.iter().enumerate() {
        assert!(
            index.abs_diff(*neighbour) > mp.exclusion_zone,
            "index {index} matched {neighbour}, inside the exclusion zone"
        );
    }
}

#[test]
fn two_constant_subsequences_are_at_distance_zero_which_is_the_whole_problem() {
    // The convention that decides this experiment. In a signal that is 78-94%
    // empty, most subsequences are all zeros, every pair of them is at distance
    // exactly 0, and an unmasked matrix profile is therefore a statement about
    // emptiness.
    let ts = [5.0; 8];
    let mp = profile_of(&ts, 4);
    assert!(
        mp.profile.iter().all(|d| d.abs() < EPSILON),
        "every constant pair is a perfect match: {:?}",
        mp.profile
    );

    // And zeros are constant like any other value.
    let zeros = [0.0; 8];
    let mp = profile_of(&zeros, 4);
    assert!(mp.profile.iter().all(|d| d.abs() < EPSILON));
}

#[test]
fn one_constant_subsequence_yields_the_sqrt_two_m_sentinel_which_is_not_a_match() {
    let ts = [1.0, 2.0, 3.0, 4.0, 9.0, 9.0, 9.0, 9.0];
    let mp = profile_of(&ts, 4);
    let sentinel = (2.0 * 4.0f64).sqrt();
    assert!(
        (mp.profile[4] - sentinel).abs() < EPSILON,
        "expected the sqrt(2m) sentinel {sentinel}, got {}",
        mp.profile[4]
    );
    // It is large but not the maximum, so it can outrank a real match in a
    // discord search. That is why the experiment masks it.
    assert!(sentinel < 2.0 * (4.0f64).sqrt());
}

#[test]
fn the_distance_is_blind_to_both_offset_and_amplitude() {
    // [1,2,3,4] and [10,20,30,40] are a perfect match: z-normalization removes
    // the mean and divides by the standard deviation, so a burst of one record
    // per bucket and a burst of ten are identical in shape.
    let ts = [1.0, 2.0, 3.0, 4.0, 0.0, 10.0, 20.0, 30.0, 40.0];
    let mp = profile_of(&ts, 4);
    assert!(
        mp.profile[0].abs() < EPSILON,
        "amplitude-scaled copies should be identical, got {}",
        mp.profile[0]
    );

    // Offset alone, stated separately so neither invariance is inferred from the
    // other.
    let shifted: Vec<f64> = ts.iter().map(|value| value + 1000.0).collect();
    let mp_shifted = profile_of(&shifted, 4);
    for (a, b) in mp.profile.iter().zip(mp_shifted.profile.iter()) {
        assert!((a - b).abs() < EPSILON);
    }
}

#[test]
fn a_subsequence_index_converts_to_the_span_it_actually_covers() {
    // Index i covers base samples [i, i+m), so at a 500 ms base it spans
    // [i*500, (i+m)*500). Off by one here would relocate every finding.
    let mut column = vec![0.0; 64];
    column[10] = 1.0;
    column[11] = 2.0;
    column[40] = 1.0;
    column[41] = 2.0;

    let found = scan(&column, 500, 4_000, 3).expect("the series holds an 8-sample window");
    assert_eq!(found.m, 8);
    assert_eq!(found.subsequences, 64 - 8 + 1);
    let pair = found.best().expect("a masked motif survives");
    assert_eq!(pair.a.end_ms - pair.a.start_ms, 4_000);
    assert_eq!(pair.a.start_ms, pair.a.index as u64 * 500);
    assert_eq!(pair.b.start_ms, pair.b.index as u64 * 500);
    assert_eq!(pair.lag_samples, 30, "the two bursts are 30 samples apart");
    assert!(
        pair.distance < EPSILON,
        "identical bursts: {}",
        pair.distance
    );
}

// ---------------------------------------------------------------------------
// What this experiment adds on top: the mask, and the null
// ---------------------------------------------------------------------------

#[test]
fn the_unmasked_answer_is_about_emptiness_and_the_masked_one_is_not() {
    // The predicted-in-advance shape of every result in this round, in miniature:
    // a mostly-empty series with two identical bursts. Unmasked, the library
    // correctly reports two stretches of nothing at distance 0. Masked, the
    // bursts appear.
    let mut column = vec![0.0; 80];
    for (offset, value) in [(0, 1.0), (1, 3.0), (2, 2.0), (3, 1.0)] {
        column[20 + offset] = value;
        column[60 + offset] = value;
    }

    let found = scan(&column, 500, 4_000, 5).expect("scan succeeds");

    let raw = found.raw_top.expect("the library reports a top motif");
    assert!(raw.distance.abs() < EPSILON);
    assert!(
        raw.a_constant && raw.b_constant,
        "unmasked, the best match is two constant stretches — that is the representation, \
         not a finding"
    );

    let masked = found.best().expect("a masked motif survives");
    assert!(!masked.a_constant && !masked.b_constant);
    assert!(masked.distance < EPSILON, "got {}", masked.distance);
    assert_eq!(masked.lag_samples, 40);

    // And the mask removed most of the candidate set, which is a fact the result
    // has to carry rather than hide.
    // 73 candidate subsequences; only the 22 that overlap one of the two bursts
    // are non-constant.
    assert_eq!(found.subsequences, 73);
    assert_eq!(found.constant_subsequences, 51);
    assert!(
        found.constant_fraction() > 0.65,
        "constant fraction {}",
        found.constant_fraction()
    );
}

#[test]
fn no_masked_motif_ever_pairs_with_a_constant_subsequence() {
    // Otherwise the sqrt(2m) sentinel could be reported as a match.
    let mut column = vec![0.0; 200];
    for index in (0..200).step_by(17) {
        column[index] = 1.0;
    }
    for window_ms in LADDER_MS {
        let Some(found) = scan(&column, 500, window_ms, 5) else {
            continue;
        };
        for pair in &found.masked_top {
            assert!(
                !pair.a_constant && !pair.b_constant,
                "window {window_ms}: a masked motif touched a constant subsequence"
            );
        }
    }
}

#[test]
fn the_null_preserves_every_value_and_destroys_every_ordering() {
    let column: Vec<f64> = (0..500)
        .map(|i| if i % 23 == 0 { 2.0 } else { 0.0 })
        .collect();
    let shuffled = deterministic_shuffle(&column);

    assert_eq!(shuffled.len(), column.len());
    let mut a = column.clone();
    let mut b = shuffled.clone();
    a.sort_by(|x, y| x.partial_cmp(y).unwrap());
    b.sort_by(|x, y| x.partial_cmp(y).unwrap());
    assert_eq!(a, b, "the multiset — and so the density — is untouched");
    assert_ne!(shuffled, column, "the ordering is not");

    // Deterministic, because a null that moves between runs is not a null.
    assert_eq!(deterministic_shuffle(&column), shuffled);
}

#[test]
fn a_series_too_short_for_a_window_produces_no_scan_rather_than_a_guess() {
    assert!(scan(&[0.0; 4], 500, 128_000, 5).is_none());
    assert!(scan(&[], 500, 8_000, 5).is_none());
    // One sample is not a window either.
    assert!(scan(&[1.0, 2.0], 500, 500, 5).is_none());
}

// ---------------------------------------------------------------------------
// The input representation, and the hazard found by running the experiment
// ---------------------------------------------------------------------------

#[test]
fn on_raw_counts_an_empty_window_is_exactly_constant() {
    // The requirement. An all-zero window has a standard deviation of exactly
    // zero, so the library detects it and the mask removes it.
    let mut column = vec![0.0; 200];
    column[50] = 1.0;
    column[150] = 1.0;

    let found = scan(&column, 500, 8_000, 3).expect("scan succeeds");
    // Independently counted: windows of 16 that touch neither non-zero bucket.
    let expected = (0..=200 - 16)
        .filter(|start| !(*start..start + 16).any(|i| column[i] != 0.0))
        .count();
    assert_eq!(
        found.constant_subsequences, expected,
        "every all-zero window must be detected as constant"
    );
}

#[test]
fn globally_z_scoring_before_the_detector_hides_empty_windows_and_must_not_be_done() {
    // Documents the hazard this round found by disbelieving an obviously wrong
    // intermediate: a "perfect motif" between two regions of the legible oracle
    // holding no records at all.
    //
    // The metric z-normalizes each subsequence itself, so a global affine
    // transform changes no distance in exact arithmetic. It does change the
    // arithmetic: an empty bucket stops being exactly zero, the rolling variance
    // of an all-empty window is then computed by cancellation, and the result is
    // far enough from zero to defeat the constant test — after which
    // z-normalization amplifies the rounding error into a full-amplitude shape.
    let mut column = vec![0.0; 400];
    for index in [100, 250] {
        column[index] = 1.0;
    }
    let mean = column.iter().sum::<f64>() / column.len() as f64;
    let variance =
        column.iter().map(|v| (v - mean) * (v - mean)).sum::<f64>() / column.len() as f64;
    let z_scored: Vec<f64> = column
        .iter()
        .map(|v| (v - mean) / variance.sqrt())
        .collect();

    let raw = scan(&column, 500, 16_000, 3).expect("scan succeeds");
    let scored = scan(&z_scored, 500, 16_000, 3).expect("scan succeeds");

    assert!(
        raw.constant_subsequences > scored.constant_subsequences,
        "if these ever agree, re-read the module header before trusting either: \
         raw {} vs z-scored {}",
        raw.constant_subsequences,
        scored.constant_subsequences
    );
    // And the consequence, which is worse than a missed mask: on the z-scored
    // input the detector reports a flawless motif whose two windows contain no
    // records at all. Occupancy is counted against the *original* counts, since
    // in the z-scored column an empty bucket is no longer the value zero.
    let ghost = scored
        .best()
        .expect("a masked motif survives on the z-scored input");
    let records_in = |span: &witnessglass::experiment::matrix_profile::Span| {
        column[span.index..(span.index + scored.m).min(column.len())]
            .iter()
            .filter(|value| **value != 0.0)
            .count()
    };
    assert!(
        ghost.distance < 1e-6,
        "a flawless match: {}",
        ghost.distance
    );
    assert_eq!(
        (records_in(&ghost.a), records_in(&ghost.b)),
        (0, 0),
        "the z-scored input's best 'motif' is two windows holding no records"
    );

    // The raw input refuses to make that mistake.
    let honest = raw.best().expect("a masked motif survives on raw counts");
    assert!(honest.a_occupancy > 0 && honest.b_occupancy > 0);
}

// ---------------------------------------------------------------------------
// The fixtures, against the criteria task:16 preregistered
// ---------------------------------------------------------------------------

const LEGIBLE: &str = "fixtures/synthetic-behavioral-oracle.ndjson";
const SPARSE: &str = "fixtures/synthetic-behavioral-oracle-sparse.ndjson";

/// Every window of the ladder for one dimension of one fixture, on raw counts.
macro_rules! ladder_over {
    ($path:expr, $label:expr) => {{
        let bytes = std::fs::read($path).expect("the fixture is readable");
        let replay = witnessglass::replay_bytes(&bytes).expect("the fixture replays");
        let inspection = witnessglass::inspection::inspect(&replay);
        let signal = witnessglass::experiment::signal::project(
            &inspection,
            witnessglass::experiment::signal::BucketWidth::from_ms(500).expect("non-zero"),
        )
        .expect("the fixture has records");
        let index = signal
            .dimension_index($label)
            .unwrap_or_else(|| panic!("{} has no dimension {}", $path, $label));
        let column = signal.column(index).expect("the column exists");
        LADDER_MS
            .iter()
            .map(|window_ms| scan(&column, 500, *window_ms, 5))
            .collect::<Vec<_>>()
    }};
}

#[test]
fn the_top_motif_in_a_sparse_dimension_is_a_pair_of_lone_events_not_a_repeated_figure() {
    // The round's central representation finding. Two windows each holding a
    // single non-zero bucket at the same relative offset are identical after
    // z-normalization and score exactly 0, whatever surrounds them. In these
    // dimensions that is what every top motif is.
    for (path, label) in [
        (LEGIBLE, "channel:reported"),
        (LEGIBLE, "tool_name:SyntheticSearcher"),
        (SPARSE, "channel:reported"),
        (SPARSE, "tool_name:SparseSyntheticSearcher"),
    ] {
        let ladder = ladder_over!(path, label);
        for window in ladder.iter().flatten() {
            let Some(best) = window.best() else { continue };
            assert!(
                best.a_occupancy <= 4 && best.b_occupancy <= 4,
                "{path} {label} at {} ms: occupancy {}/{} — if this ever rises, the detector \
                 has started matching figures rather than lone events",
                window.window_ms,
                best.a_occupancy,
                best.b_occupancy
            );
        }
        // And where the ladder reports a flawless match, it is those lone events.
        // The tolerance is 1e-6 rather than exact: the library's own validation
        // records ~4e-8 of accumulation in the distance recurrence.
        let at_16s = ladder[2].as_ref().expect("16 s scan exists");
        let best = at_16s.best().expect("a motif survives");
        assert!(best.distance < 1e-6, "{path} {label}: {}", best.distance);
        assert!(best.a_occupancy <= 2 && best.b_occupancy <= 2);
    }
}

#[test]
fn the_known_recurrence_is_recovered_by_overlap_and_missed_by_the_preregistered_criterion() {
    // task:16 asked whether a span *starts* inside each region. That is too tight
    // by up to a whole window: a window containing a region's first instance can
    // begin before the region does. Both criteria are reported, and the strict
    // one is kept as written rather than quietly replaced.
    let legible = ladder_over!(LEGIBLE, "channel:reported");
    let a = (60_000u64, 90_000u64);
    let b = (210_000u64, 240_000u64);

    // 16 s: the recurrence is rank 1 by overlap, and invisible to the strict rule.
    let at_16s = legible[2].as_ref().expect("16 s scan exists");
    assert_eq!(at_16s.overlapping(a, b).map(|(rank, _)| rank), Some(0));
    assert!(at_16s.linking(a, b).is_none());

    // The two shortest windows recover it under neither criterion.
    for scanned in legible.iter().take(2) {
        let window = scanned.as_ref().expect("scan exists");
        assert!(
            window.overlapping(a, b).is_none(),
            "window {} ms unexpectedly linked the regions",
            window.window_ms
        );
    }
}

#[test]
fn separation_from_the_null_is_absent_at_short_windows_and_appears_only_at_long_ones() {
    // The preregistered comparison metric, and the reason a distance of zero
    // proves nothing on its own: the shuffled null reaches zero too, until the
    // window grows long enough that coincidental alignment becomes impossible.
    let sparse = ladder_over!(SPARSE, "channel:reported");

    for scanned in sparse.iter().take(4) {
        let window = scanned.as_ref().expect("scan exists");
        assert_eq!(
            window.separation.map(|s| s.abs() < 1e-6),
            Some(true),
            "window {} ms: the null matched the signal exactly, so the match carries no \
             information",
            window.window_ms
        );
    }
    let longest = sparse[5].as_ref().expect("128 s scan exists");
    assert!(
        longest.separation.unwrap_or(0.0) > 0.2,
        "at 128 s the shuffle can no longer produce a lone-event window: {:?}",
        longest.separation
    );
    // And even there the match itself is still a pair of lone events.
    assert!(longest.best().expect("a motif").a_occupancy <= 2);
}
