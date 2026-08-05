//! The sprint:12 adversarial gauntlet.
//!
//! **Disposable.** Deleted with the attack it implements.
//!
//! **What these tests are for.** The gauntlet's job is to be trustworthy about
//! what it built, because every family's verdict is only worth what its
//! construction is worth. sprint:12 learned that the hard way: the first draft
//! of the `noise` family drew its two marks independently and produced the
//! *same* mark on both sides in 20 of 60 trials, quietly turning a third of the
//! family into informative trials. These tests assert that each family builds
//! what it claims to, that the scoring rule is the preregistered one, and that
//! nothing here perturbs the machinery under attack.

use witnessglass::experiment::event_sequence::{ChannelScope, align, project};
use witnessglass::experiment::gauntlet::{self, Family, Verdict};
use witnessglass::inspection::inspect;
use witnessglass::replay_bytes;

const EPSILON: f64 = 1e-9;

fn trials_of(family: Family) -> Vec<gauntlet::Trial> {
    gauntlet::grid()
        .into_iter()
        .filter(|trial| trial.family == family)
        .collect()
}

#[test]
fn the_grid_is_the_preregistered_one_and_is_deterministic() {
    let first = gauntlet::grid();
    assert_eq!(
        first,
        gauntlet::grid(),
        "the grid must not move between calls"
    );
    assert_eq!(first.len(), 300);

    let count = |family: Family| first.iter().filter(|t| t.family == family).count();
    assert_eq!(count(Family::Informative), 60);
    assert_eq!(count(Family::Noise), 60);
    assert_eq!(count(Family::Common), 30);
    assert_eq!(count(Family::Rare), 30);
    assert_eq!(count(Family::Redundant), 30);
    assert_eq!(count(Family::Accidental), 30);
    assert_eq!(count(Family::Diluted), 40);
    assert_eq!(count(Family::Competing), 20);
}

#[test]
fn a_trial_reproduces_exactly() {
    let trial = trials_of(Family::Informative)[0];
    assert_eq!(gauntlet::run(&trial), gauntlet::run(&trial));
}

#[test]
fn the_noise_family_never_puts_the_same_mark_on_both_sides() {
    // The defect that made the first run of this gauntlet wrong. It is checked
    // over the whole family rather than sampled, because one collision in sixty
    // is what went unnoticed the first time.
    for trial in trials_of(Family::Noise) {
        let outcome = gauntlet::run(&trial).expect("a trial");
        let (a, b) = (
            outcome.a_marks.last().expect("a boundary mark"),
            outcome.b_marks.last().expect("a boundary mark"),
        );
        assert_ne!(a, b, "noise trial {:?} shares a boundary mark", trial.seed);
    }
}

#[test]
fn every_family_builds_the_boundary_it_claims_to() {
    let boundary = |family: Family| -> Vec<(String, String, Vec<String>, Vec<String>)> {
        trials_of(family)
            .into_iter()
            .filter_map(|trial| {
                let outcome = gauntlet::run(&trial)?;
                Some((
                    outcome.a_marks.last()?.clone(),
                    outcome.b_marks.last()?.clone(),
                    outcome.a_marks.clone(),
                    outcome.b_marks.clone(),
                ))
            })
            .collect()
    };

    for (a, b, a_marks, _) in boundary(Family::Informative) {
        assert_eq!(a, b, "an informative boundary is shared");
        assert!(
            a.contains("BoundaryRare"),
            "and is not a background mark: {a}"
        );
        // The core it extends is identical on both sides.
        assert!(a_marks.iter().filter(|m| m.contains("Core")).count() >= 3);
    }
    for (a, b, _, _) in boundary(Family::Rare) {
        assert_eq!(a, b);
        assert!(a.contains("BoundaryRare"));
    }
    for (a, b, _, _) in boundary(Family::Common) {
        assert_eq!(a, b, "common and rare differ only in background prevalence");
        assert!(a.contains("BoundaryRare"));
    }
    for (a, b, a_marks, _) in boundary(Family::Redundant) {
        assert_eq!(a, b);
        assert!(
            a_marks[..a_marks.len() - 1].contains(&a),
            "a redundant boundary must repeat a mark the core already carries: {a_marks:?}"
        );
    }
}

#[test]
fn common_and_rare_differ_only_in_background_prevalence() {
    // The round's central measurement rests on this: matched pairs whose raw
    // agreement is identical by construction. If the two families ever differ in
    // anything but prevalence, the comparison measures something else.
    let common = trials_of(Family::Common);
    let rare = trials_of(Family::Rare);
    for left in &common {
        let right = rare
            .iter()
            .find(|t| {
                t.core_len == left.core_len
                    && t.context_len == left.context_len
                    && t.seed == left.seed
                    && t.gap_ratio == left.gap_ratio
            })
            .expect("a matched rare trial");
        let (a, b) = (
            gauntlet::run(left).expect("a trial"),
            gauntlet::run(right).expect("a trial"),
        );
        assert_eq!(a.a_marks, b.a_marks, "the spans carry the same marks");
        assert_eq!(a.b_marks, b.b_marks);
        assert!(
            (a.delta_total - b.delta_total).abs() < 1e-6,
            "raw agreement must be identical by construction: {} vs {}",
            a.delta_total,
            b.delta_total
        );
    }
}

#[test]
fn the_common_family_actually_makes_its_boundary_mark_common() {
    // And the rare family actually makes it rare — otherwise the paired
    // comparison has no lever.
    let prevalence = |family: Family| -> f64 {
        let trial = trials_of(family)
            .into_iter()
            .find(|t| t.context_len == 40)
            .expect("a trial");
        // Rebuild the same specimen through the public path the gauntlet uses.
        let outcome = gauntlet::run(&trial).expect("a trial");
        let _ = outcome;
        // Prevalence is measured from the generated recording itself.
        let text = gauntlet::recording_for(&trial, 0).expect("a recording");
        let replay = replay_bytes(text.as_bytes()).expect("replay");
        let inspection = inspect(&replay);
        let sequence = project(&inspection, ChannelScope::Observed).expect("a sequence");
        let hits = sequence
            .events
            .iter()
            .filter(|event| {
                event
                    .mark
                    .tool_name
                    .is_some_and(|name| name.contains("BoundaryRare"))
            })
            .count();
        hits as f64 / sequence.len() as f64
    };

    let common = prevalence(Family::Common);
    let rare = prevalence(Family::Rare);
    assert!(
        common > 0.15,
        "the common boundary mark is not common: {common}"
    );
    assert!(rare < 0.05, "the rare boundary mark is not rare: {rare}");
    assert!(common > rare * 4.0);
}

#[test]
fn the_gauntlet_does_not_perturb_the_metric_it_attacks() {
    // A trial's reported raw distance must be exactly what `align` gives for the
    // same spans, computed independently here.
    let trial = trials_of(Family::Informative)[0];
    let outcome = gauntlet::run(&trial).expect("a trial");
    let a_text = gauntlet::recording_for(&trial, 0).expect("a recording");
    let b_text = gauntlet::recording_for(&trial, 1).expect("a recording");
    let a_replay = replay_bytes(a_text.as_bytes()).expect("replay");
    let b_replay = replay_bytes(b_text.as_bytes()).expect("replay");
    let (a_i, b_i) = (inspect(&a_replay), inspect(&b_replay));
    let a = project(&a_i, ChannelScope::Observed).expect("a sequence");
    let b = project(&b_i, ChannelScope::Observed).expect("a sequence");

    let start = trial.context_len;
    let core = align(
        a.window(start, trial.core_len).expect("a window"),
        b.window(start, trial.core_len).expect("a window"),
    );
    let expanded = align(
        a.window(start, trial.core_len + 1).expect("a window"),
        b.window(start, trial.core_len + 1).expect("a window"),
    );
    assert!((outcome.core_total - core.total).abs() < EPSILON);
    assert!((outcome.expanded_total - expanded.total).abs() < EPSILON);
    assert!((outcome.delta_total - (expanded.total - core.total)).abs() < EPSILON);
}

#[test]
fn the_scoring_rule_is_the_preregistered_one() {
    // Hand-checkable cases against task:22 §7, including the inverted rule the
    // noise family is scored by.
    let pass = gauntlet::score_values(&[1.0, 1.0, 1.0, -1.0], true);
    assert_eq!(pass, Verdict::Pass, "median positive, 3 of 4 agree");

    let mixed = gauntlet::score_values(&[1.0, 1.0, -1.0, -0.5], true);
    assert_eq!(mixed, Verdict::Mixed, "median positive, only half agree");

    let fail = gauntlet::score_values(&[-1.0, -1.0, 1.0, 1.0], true);
    assert_eq!(fail, Verdict::Fail, "median is zero");

    let wrong_sign = gauntlet::score_values(&[-1.0, -2.0, 1.0], true);
    assert_eq!(wrong_sign, Verdict::Fail);

    // Inverted: an absence of effect.
    let absent = gauntlet::score_values(&[-1.0, -1.0, -1.0, 0.5], false);
    assert_eq!(absent, Verdict::Pass);
    let present = gauntlet::score_values(&[1.0, 1.0, 2.0], false);
    assert_eq!(present, Verdict::Fail);
}

#[test]
fn the_report_page_renders_the_scorecard_the_gauntlet_computed() {
    // Fidelity: the page must carry the computed verdicts, fractions, and
    // medians, and must not invent a family.
    let (_, reports) = gauntlet::report();
    let document = serde_json::json!({
        "label": "gauntlet",
        "role": "controlled synthetic validation",
        "realizations": gauntlet::REALIZATIONS,
        "trials": 300,
        "families": reports,
    });
    let page = witnessglass::experiment::boundary_page::render(&[document]);

    for report in &reports {
        assert!(page.contains(report.family.label()), "family missing");
        assert!(page.contains(report.verdict.label()), "verdict missing");
        assert!(
            page.contains(&format!("{:.3}", report.expected_fraction)),
            "fraction {:.3} missing",
            report.expected_fraction
        );
        assert!(
            page.contains(&format!("{:+.3}", report.median)),
            "median {:+.3} missing",
            report.median
        );
    }
    // The interesting quadrant is drawn and the scatter has points.
    assert!(page.contains("class=\"interesting\""));
    assert!(page.contains("class=\"hit\"") || page.contains("class=\"miss\""));
    // A page inventing a family would have to invent this one.
    assert!(!page.contains("supercalifragilistic"));
}
