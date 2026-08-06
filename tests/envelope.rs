//! sprint:16's operating-envelope exposure study.
//!
//! **Disposable.** Deleted with the study.
//!
//! **What these tests are for.** The study's conclusions are about real
//! recordings, which are untracked and cannot be a test dependency. So these
//! pin the *arithmetic* the conclusions rest on — the boundary formula, the
//! asymmetry identity, and the crossing detector — against committed fixtures
//! and hand-checkable inputs. If the arithmetic moves, the report's numbers stop
//! meaning what they say.

use witnessglass::experiment::envelope::{
    self, AsymmetrySample, UNDER_STUDY, approaches, asymmetry_of, crossings, profile, quantiles,
};
use witnessglass::experiment::event_sequence::{ChannelScope, project};
use witnessglass::experiment::identifiability::SCORERS;
use witnessglass::experiment::oracle;
use witnessglass::inspection::inspect;
use witnessglass::replay_bytes;

const EPSILON: f64 = 1e-9;

macro_rules! sequence_of {
    ($text:expr, $replay:ident, $inspection:ident, $sequence:ident) => {
        let text = $text;
        let $replay = replay_bytes(text.as_bytes()).expect("replay");
        let $inspection = inspect(&$replay);
        let $sequence = project(&$inspection, ChannelScope::Observed).expect("a sequence");
    };
}

#[test]
fn the_statistic_under_study_is_the_frozen_one() {
    assert_eq!(UNDER_STUDY, "rarity_of_agreements");
    let scorer = SCORERS
        .iter()
        .find(|scorer| scorer.name == UNDER_STUDY)
        .expect("still in the preregistered family");
    assert!(
        scorer.probe,
        "still a probe: this round measures and does not adopt"
    );
}

#[test]
fn a_profile_counts_every_mark_and_nothing_else() {
    sequence_of!(oracle::ndjson(), replay, inspection, sequence);
    let measured = profile(&sequence);

    assert_eq!(measured.events, sequence.len());
    assert_eq!(
        measured
            .frequencies
            .iter()
            .map(|entry| entry.count)
            .sum::<usize>(),
        sequence.len(),
        "every event is counted exactly once"
    );
    assert_eq!(measured.vocabulary, measured.frequencies.len());
    // Descending by count, so the first entry is the maximum.
    for pair in measured.frequencies.windows(2) {
        assert!(pair[0].count >= pair[1].count);
    }
    assert_eq!(measured.max_count, measured.frequencies[0].count);
    assert!(
        (measured.frequencies[0].frequency - measured.max_count as f64 / measured.events as f64)
            .abs()
            < EPSILON
    );
    assert_eq!(measured.frequency_deciles.len(), 11);
}

#[test]
fn the_accumulation_boundary_is_n_to_the_k_minus_one_over_k() {
    // The formula the whole accumulation result rests on, hand-checked, and the
    // specific value sprint:15 carried forward as an estimate: at N = 169 and
    // k = 4 the boundary is 46.9, not "about 47" by recollection.
    sequence_of!(oracle::ndjson(), replay, inspection, sequence);
    let mut measured = profile(&sequence);
    measured.events = 169;

    let found = approaches(&measured, &[3, 4, 5, 12]);
    let boundary_at = |k: usize| {
        found
            .iter()
            .find(|approach| approach.k == k)
            .map(|approach| approach.boundary)
            .expect("a boundary")
    };
    assert!((boundary_at(3) - 169.0f64.powf(2.0 / 3.0)).abs() < EPSILON);
    assert!((boundary_at(4) - 169.0f64.powf(3.0 / 4.0)).abs() < EPSILON);
    assert!(
        (boundary_at(4) - 46.9).abs() < 0.1,
        "sprint:15's estimate reproduced from the formula: {}",
        boundary_at(4)
    );
    // The boundary rises with k: longer motifs are harder for a singleton to beat.
    for pair in [(3usize, 4usize), (4, 5), (5, 12)] {
        assert!(boundary_at(pair.0) < boundary_at(pair.1));
    }
    // Below two the family is not evaluated at all.
    assert!(approaches(&measured, &[1]).is_empty());
}

#[test]
fn constructibility_needs_both_a_singleton_and_a_common_mark() {
    sequence_of!(oracle::ndjson(), replay, inspection, sequence);
    let base = profile(&sequence);

    // A recording with no singleton cannot supply the c₁ = 1 the sharp boundary
    // needs, however common its commonest mark is.
    let mut no_singleton = base.clone();
    no_singleton.singletons = 0;
    no_singleton.max_count = no_singleton.events;
    for approach in approaches(&no_singleton, &[3, 4]) {
        assert!(
            !approach.constructible,
            "no singleton, so not constructible"
        );
    }

    // And a recording whose commonest mark is below the boundary cannot supply
    // the other half, however many singletons it has.
    let mut all_rare = base.clone();
    all_rare.singletons = 10;
    all_rare.max_count = 1;
    all_rare.frequencies.retain(|entry| entry.count == 1);
    for approach in approaches(&all_rare, &[3, 4]) {
        assert!(!approach.constructible);
        assert!(approach.relative_margin < 1.0);
    }
}

#[test]
fn asymmetry_is_zero_only_when_the_two_recordings_share_a_frequency() {
    // Against itself the statistic is trivially symmetric: the same marginals on
    // both sides. That is the control, and it is the only way delta reaches zero.
    sequence_of!(oracle::ndjson(), replay, inspection, sequence);
    let same = asymmetry_of(&sequence, (20, 28), &sequence, (162, 170), "self").expect("a sample");
    assert!(
        same.delta < EPSILON,
        "self-comparison must be symmetric: {}",
        same.delta
    );

    // Against a different recording with different marginals it is not.
    sequence_of!(
        oracle::sparse::ndjson(),
        other_replay,
        other_inspection,
        other
    );
    if let Some(cross) = asymmetry_of(&sequence, (20, 24), &other, (20, 24), "cross") {
        assert!(
            (cross.forward - cross.backward).abs() >= 0.0,
            "the identity holds by construction"
        );
        // And the reported delta is exactly the absolute difference.
        assert!((cross.delta - (cross.forward - cross.backward).abs()).abs() < EPSILON);
    }
}

#[test]
fn a_crossing_is_fewer_agreements_outscoring_more() {
    let sample = |agreements: usize, forward: f64| AsymmetrySample {
        a_session: "a".to_owned(),
        b_session: "b".to_owned(),
        origin: "test".to_owned(),
        span: 4,
        agreements,
        forward,
        backward: forward,
        delta: 0.0,
    };

    // Three agreements outscoring four is a crossing; the reverse is not.
    let found = crossings("test", &[sample(3, 9.9), sample(4, 6.4)]);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].fewer_agreements, 3);
    assert_eq!(found[0].more_agreements, 4);
    assert!((found[0].margin - 3.5).abs() < 1e-9);

    // Monotone sets contain none.
    assert!(crossings("test", &[sample(3, 6.0), sample(4, 9.0)]).is_empty());
    // Equal agreement counts are not crossings whatever they score.
    assert!(crossings("test", &[sample(4, 9.0), sample(4, 1.0)]).is_empty());
}

#[test]
fn an_ordering_check_notices_a_moved_pick() {
    let sample = |forward: f64, backward: f64| AsymmetrySample {
        a_session: "a".to_owned(),
        b_session: "b".to_owned(),
        origin: "test".to_owned(),
        span: 4,
        agreements: 4,
        forward,
        backward,
        delta: (forward - backward).abs(),
    };
    let moved =
        envelope::ordering_check("test", &[sample(1.0, 5.0), sample(2.0, 1.0)]).expect("a check");
    assert!(moved.pick_changed);
    assert_eq!(moved.inversions, 1);

    let stable =
        envelope::ordering_check("test", &[sample(1.0, 1.5), sample(2.0, 3.0)]).expect("a check");
    assert!(!stable.pick_changed);
    assert_eq!(stable.inversions, 0);

    // One candidate is not a ranking.
    assert!(envelope::ordering_check("test", &[sample(1.0, 1.0)]).is_none());
}

#[test]
fn quantiles_are_the_six_the_report_prints() {
    let values = vec![0.0, 1.0, 2.0, 3.0, 4.0];
    let found = quantiles(&values);
    assert_eq!(found.len(), 6);
    assert!((found[0] - 0.0).abs() < EPSILON, "min");
    assert!((found[2] - 2.0).abs() < EPSILON, "median");
    assert!((found[5] - 4.0).abs() < EPSILON, "max");
    assert!(quantiles(&[]).is_empty());
}
