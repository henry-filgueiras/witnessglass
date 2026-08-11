//! sprint:22, task:32 — the fixed-budget FewRS retrospective assay.
//!
//! Two things are load-bearing here and everything below defends one of them:
//! the budget is *derived* rather than asserted, and the decision rule is
//! *strict*. A third is defended by construction: this round adds no null, no
//! search and no statistic, so several tests exist to prove the FewRS path is
//! the frozen sprint:19 path with a different replicate count.
//!
//! Replicate counts here are deliberately tiny. The preregistered `m = 459` is
//! for the experiment; these check machinery, and `scripts/check.sh` runs on
//! every change.

use witnessglass::experiment::calibration::{self, LADDER, negative_control, positive_control};
use witnessglass::experiment::event_sequence::{null_seed, order_null_seeded};
use witnessglass::experiment::fewrs::{
    self, ALPHA, BUDGET, BudgetError, Cell, Classification, HISTORICAL_REPLICATES, RunDescriptor,
    SPRINT_19_ORDER_NULL_GRID, STRONG_THRESHOLD, assay, classify, fewrs_budget, fewrs_certifies,
    historical_cell, refuting_nulls,
};

/// Cheap replicate counts for machinery checks. Neither resolves anything the
/// experiment claims; both only have to exercise a code path.
const TINY: usize = 15;

// ---------------------------------------------------------------------------
// PHASE 1 — the budget
// ---------------------------------------------------------------------------

/// The one number the commission fixes, and the derivation that produces it.
#[test]
fn the_budget_at_the_frozen_alpha_is_459() {
    assert_eq!(ALPHA, 0.01, "sprint:19's frozen significance level");
    assert_eq!(
        fewrs_budget(0.01),
        Ok(459),
        "ceil(ln(1/0.01) / ln(1/0.99)) = ceil(458.2106) = 459"
    );
}

/// The constant exists only because `f64::ln` is not `const`. It must not drift
/// from the function that justifies it.
#[test]
fn the_module_constant_is_the_derivation_and_not_a_transcription() {
    assert_eq!(
        Ok(BUDGET),
        fewrs_budget(ALPHA),
        "BUDGET must be exactly what fewrs_budget(ALPHA) computes"
    );
}

/// The formula is defined on the open unit interval and nowhere else. Both
/// endpoints diverge — `ln(1/0)` at one end, `ln(1/0)` in the denominator at the
/// other — so both are refused rather than saturated.
#[test]
fn an_alpha_outside_the_open_unit_interval_is_rejected() {
    for bad in [
        0.0,
        1.0,
        -0.01,
        1.5,
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
    ] {
        assert_eq!(
            fewrs_budget(bad),
            Err(BudgetError::AlphaOutsideOpenUnitInterval),
            "alpha = {bad} must be refused, not silently coerced"
        );
    }
}

/// A looser alpha buys a smaller budget, which is the whole shape of the claim
/// and the reason the paper's examples quote 8-to-64 searches.
#[test]
fn a_looser_alpha_derives_a_smaller_budget() {
    assert_eq!(fewrs_budget(0.05), Ok(59));
    assert!(
        fewrs_budget(0.05).unwrap() < fewrs_budget(0.01).unwrap(),
        "the budget must be monotone in the direction the formula implies"
    );
}

// ---------------------------------------------------------------------------
// PHASE 2 — the decision rule
// ---------------------------------------------------------------------------

/// Strictly above every null certifies.
#[test]
fn a_strictly_larger_observation_certifies() {
    assert!(fewrs_certifies(Some(10.0), &[1.0, 5.0, 9.9999]));
}

/// **The tie rule, and it is the one most easily got wrong.** An observation
/// equal to the null maximum does not certify. No epsilon is introduced: an
/// epsilon would be a threshold chosen after seeing data.
#[test]
fn equality_with_the_null_maximum_does_not_certify() {
    assert!(
        !fewrs_certifies(Some(9.0), &[1.0, 9.0, 5.0]),
        "a tie with the maximum must not certify"
    );
    assert!(
        !fewrs_certifies(Some(9.0), &[9.0]),
        "a tie against a single null must not certify either"
    );
}

/// One null above the observation is enough to refuse, wherever it sits.
#[test]
fn a_single_null_above_the_observation_does_not_certify() {
    assert!(!fewrs_certifies(Some(9.0), &[1.0, 2.0, 9.5]));
    assert!(!fewrs_certifies(Some(9.0), &[9.5, 1.0, 2.0]));
}

/// The rule reads values and never positions.
#[test]
fn the_order_of_the_null_values_does_not_change_the_answer() {
    let ascending = [1.0, 2.0, 3.0, 8.5];
    let descending = [8.5, 3.0, 2.0, 1.0];
    let shuffled = [3.0, 8.5, 1.0, 2.0];
    for observed in [8.4, 8.5, 8.6, 100.0] {
        let first = fewrs_certifies(Some(observed), &ascending);
        assert_eq!(first, fewrs_certifies(Some(observed), &descending));
        assert_eq!(first, fewrs_certifies(Some(observed), &shuffled));
        assert_eq!(
            refuting_nulls(Some(observed), &ascending),
            refuting_nulls(Some(observed), &shuffled),
        );
    }
}

/// An undefined `T` is reported as undefined, never certified.
#[test]
fn an_undefined_observation_does_not_certify() {
    assert!(!fewrs_certifies(None, &[1.0, 2.0]));
    assert!(!fewrs_certifies(None, &[]));
}

/// **No evidence is not evidence.** An empty null set would otherwise certify
/// vacuously under "every null is below the observation", which is exactly the
/// branch a search that admitted no candidate would reach.
#[test]
fn an_empty_null_set_does_not_certify() {
    assert!(!fewrs_certifies(Some(1e9), &[]));
}

// ---------------------------------------------------------------------------
// PHASE 3 — the complete search stays inside every replicate
// ---------------------------------------------------------------------------

/// **The claim this round stands on, and it is sprint:19's claim unchanged.**
///
/// The null maximum a FewRS cell reports must be reproducible by running the
/// whole `complete_search` — enumeration, alignment ranking, deduplication, R1
/// readout — independently on `order_null_seeded` replicates at the existing
/// seeds. Recomputed here from the seeds outward rather than read back out of
/// the calibration, so a FewRS path that quietly rescored fixed boundaries, or
/// preselected candidates, would fail.
#[test]
fn the_null_maximum_is_a_complete_search_rerun_inside_every_replicate() {
    let control = negative_control();
    let k = 6;
    let cell = assay("recompute", &control.first, &control.second, k, TINY);

    let mut expected = f64::NEG_INFINITY;
    for index in 0..TINY {
        let left = order_null_seeded(&control.first, null_seed(index, 0));
        let right = order_null_seeded(&control.second, null_seed(index, 1));
        if let Some(t) = calibration::complete_search(&left, &right, k).t {
            expected = expected.max(t);
        }
    }

    assert!(
        (cell.null_max - expected).abs() < 1e-12,
        "the cell's null max {} must be the maximum of {TINY} complete searches, {expected}",
        cell.null_max
    );
    assert_eq!(
        cell.certified,
        cell.observed.expect("a defined T") > expected,
        "certification must be exactly the strict comparison against that maximum"
    );
}

/// The observed statistic is the frozen one, not a FewRS-specific recomputation.
#[test]
fn the_observed_statistic_is_the_unchanged_complete_search() {
    let control = positive_control();
    for k in LADDER {
        let cell = assay("obs", &control.first, &control.second, k, TINY);
        assert_eq!(
            cell.observed,
            calibration::complete_search(&control.first, &control.second, k).t,
            "at k={k} the assay must read T from complete_search and nowhere else"
        );
    }
}

/// The budget is spent, in full, and the seed range is recorded for audit.
#[test]
fn the_run_spends_the_planned_budget_and_records_its_seed_range() {
    let control = negative_control();
    let cell = assay("budget", &control.first, &control.second, 4, TINY);
    assert_eq!(cell.planned_budget, TINY);
    assert_eq!(cell.null_searches, TINY, "no cell stops early");
    assert_eq!(cell.null_datasets, TINY * 2, "one realization per side");
    assert!(cell.realizations <= TINY);
    assert_eq!(cell.seed_range, format!("null_seed(0..{TINY}, {{0,1}})"));
}

/// **task:32 §PHASE 0 D9, asserted rather than argued.** The replicates a
/// smaller budget consumes are a *prefix* of the ones a larger budget consumes,
/// so the maximum is monotone in the budget and a cell that certified at 999
/// certifies at 459 with certainty.
#[test]
fn a_smaller_budget_consumes_a_prefix_of_a_larger_ones_replicates() {
    let control = negative_control();
    let k = 4;
    let short_run = calibration::calibrate("prefix", &control.first, &control.second, k, TINY);
    let long_run = calibration::calibrate("prefix", &control.first, &control.second, k, TINY * 3);

    // `samples` comes back sorted, so a prefix in replicate order shows up as a
    // sub-multiset here. Counting occurrences is what proves that.
    for value in &short_run.samples {
        let in_short = short_run.samples.iter().filter(|v| *v == value).count();
        let in_long = long_run.samples.iter().filter(|v| *v == value).count();
        assert!(
            in_long >= in_short,
            "the short run produced a null value the long run never did: {value}"
        );
    }
    assert!(
        long_run.null_max >= short_run.null_max,
        "the null maximum must be monotone in the budget: {} then {}",
        short_run.null_max,
        long_run.null_max
    );
}

// ---------------------------------------------------------------------------
// PHASE 6 — the frozen reference grid
// ---------------------------------------------------------------------------

/// The transcription of task:29 §9 must still say what sprint:19 published. A
/// slip in a table of thirty numbers is otherwise invisible.
#[test]
fn the_frozen_grid_still_reproduces_sprint_19s_published_counts() {
    assert_eq!(SPRINT_19_ORDER_NULL_GRID.len(), 30);
    assert_eq!(HISTORICAL_REPLICATES, 999);

    let exceptional = SPRINT_19_ORDER_NULL_GRID
        .iter()
        .filter(|row| row.exceptional())
        .count();
    assert_eq!(
        exceptional, 23,
        "task:29 §9 published 23 of 30 at p-hat <= 0.01"
    );

    let certified = SPRINT_19_ORDER_NULL_GRID
        .iter()
        .filter(|row| row.certified())
        .count();
    assert_eq!(
        certified, 13,
        "the strict maximum rule applied to the same 999 replicates gives 13 of 30 — task:32 \
         §PHASE 0 D8, and the reason the two agreement rates are both reported"
    );

    // Every cell of the frozen ladder and no other.
    for row in &SPRINT_19_ORDER_NULL_GRID {
        assert!(
            LADDER.contains(&row.k),
            "k={} is off the frozen ladder",
            row.k
        );
    }
}

/// `exceedances = 1000 * p_hat - 1`, exactly, for a tail that is a ratio of
/// integers over 1000.
#[test]
fn the_exceedance_count_is_recovered_exactly_from_the_published_tail() {
    let cell = historical_cell("57f18ff9 x f5c18299", 12).expect("a published cell");
    assert_eq!(cell.tail, 0.001);
    assert_eq!(cell.exceedances(), 0);
    assert!(cell.certified() && cell.exceptional());

    let cell = historical_cell("8b68dece x 57f18ff9", 3).expect("a published cell");
    assert_eq!(cell.exceedances(), 156);
    assert!(!cell.certified() && !cell.exceptional());

    // The rule boundary: p-hat = 0.007 is 6 exceedances — exceptional under
    // sprint:19's rule, not certified under this round's.
    let cell = historical_cell("8b68dece x 57f18ff9", 12).expect("a published cell");
    assert_eq!(cell.exceedances(), 6);
    assert!(
        cell.exceptional(),
        "6 exceedances of 999 gives p-hat = 0.007"
    );
    assert!(!cell.certified(), "but a null did reach the observation");
}

/// A control is not one of the thirty, and must not silently borrow a verdict.
#[test]
fn a_cell_outside_the_frozen_grid_carries_no_historical_comparison() {
    assert!(historical_cell("negative control", 12).is_none());
    assert!(historical_cell("8b68dece x 57f18ff9", 5).is_none());

    let control = negative_control();
    let cell = assay("negative control", &control.first, &control.second, 3, TINY);
    assert!(cell.historical_tail.is_none());
    assert!(cell.agrees_with_historical_tail_rule.is_none());
    assert!(cell.agrees_with_historical_max_rule.is_none());
}

// ---------------------------------------------------------------------------
// PHASE 9 — the classification, gated by maintenance:3's envelope
// ---------------------------------------------------------------------------

/// A cell carrying the frozen protocol's metadata, cheap enough to clone.
///
/// The envelope is a pure function of cell *metadata* — budget, seed range, null
/// mode, identity — so the statistics behind these cells are irrelevant and one
/// real `assay` at a one-replicate budget supplies them all. Overwriting the
/// budget fields is what makes it a frozen-protocol template; a test that needed
/// real 459-replicate statistics would be testing the assay, not the gate.
fn frozen_template() -> Cell {
    let control = negative_control();
    let mut cell = assay("template", &control.first, &control.second, 3, 1);
    cell.planned_budget = BUDGET;
    cell.null_searches = BUDGET;
    cell.null_datasets = BUDGET * 2;
    cell.seed_range = fewrs::seed_range(BUDGET);
    cell
}

/// One cell at a given identity, with its historical join set the way the runner
/// would have set it.
fn cell_at(template: &Cell, specimen: &str, k: usize) -> Cell {
    let mut cell = template.clone();
    cell.specimen = specimen.to_owned();
    cell.k = k;
    cell.historical_tail = historical_cell(specimen, k).map(|row| row.tail);
    cell
}

/// The ten control cells §PHASE 4 requires.
fn frozen_controls(template: &Cell) -> Vec<Cell> {
    let mut cells = Vec::new();
    for label in fewrs::EXPECTED_CONTROLS {
        for k in LADDER {
            cells.push(cell_at(template, label, k));
        }
    }
    cells
}

/// The thirty observational cells, exactly the frozen grid.
fn frozen_grid(template: &Cell) -> Vec<Cell> {
    SPRINT_19_ORDER_NULL_GRID
        .iter()
        .map(|row| cell_at(template, row.specimen, row.k))
        .collect()
}

fn frozen_specimens() -> Vec<String> {
    fewrs::EXPECTED_SPECIMENS
        .iter()
        .map(|name| (*name).to_owned())
        .collect()
}

/// **The gate maintenance:3 exists for.** The frozen envelope is classifiable
/// and the partition still tiles inside it.
#[test]
fn the_frozen_envelope_is_classifiable_and_the_partition_tiles() {
    assert_eq!(STRONG_THRESHOLD, 15);
    let template = frozen_template();
    let controls = frozen_controls(&template);
    let cells = frozen_grid(&template);
    let specimens = frozen_specimens();
    let envelope = fewrs::envelope(&RunDescriptor {
        alpha: ALPHA,
        budget: BUDGET,
        specimens: &specimens,
        controls: &controls,
        cells: &cells,
    });

    assert!(
        envelope.protocol_established() && envelope.grid_complete(),
        "the frozen run must be eligible; reasons {:?}",
        envelope.reasons()
    );
    assert_eq!(classify(&envelope, true, 15), Classification::Strong);
    assert_eq!(classify(&envelope, true, 30), Classification::Strong);
    assert_eq!(classify(&envelope, true, 14), Classification::WeakMixed);
    assert_eq!(classify(&envelope, true, 0), Classification::WeakMixed);
    assert!(classify(&envelope, true, 17).is_preregistered());
}

/// **An eligible assay whose control fails is FALSIFIED, not merely
/// diagnostic.** The frozen protocol stops before the observational stage when a
/// control fails, so the empty grid is obedience and must not be read as a
/// broken envelope.
#[test]
fn an_eligible_assay_with_a_failed_control_is_falsified_rather_than_diagnostic() {
    let template = frozen_template();
    let controls = frozen_controls(&template);
    let envelope = fewrs::envelope(&RunDescriptor {
        alpha: ALPHA,
        budget: BUDGET,
        specimens: &[],
        controls: &controls,
        cells: &[],
    });

    assert!(envelope.protocol_established(), "the protocol is intact");
    assert!(
        !envelope.grid_complete(),
        "and the grid is empty, as it should be"
    );
    assert_eq!(
        classify(&envelope, false, 0),
        Classification::Falsification,
        "a failed control on an intact protocol is a preregistered outcome"
    );
}

/// Every altered run below must land on `DIAGNOSTIC / UNCLASSIFIED`, and must
/// say which condition it broke.
fn assert_unclassified(label: &str, envelope: &fewrs::Envelope, expect: &str) {
    assert_eq!(
        classify(envelope, true, 30),
        Classification::DiagnosticUnclassified,
        "{label}: a 30-certified count must not reach STRONG outside the envelope"
    );
    let reasons: Vec<String> = envelope.reasons().iter().map(|r| r.to_string()).collect();
    assert!(
        reasons.iter().any(|reason| reason.contains(expect)),
        "{label}: reasons {reasons:?} should mention {expect:?}"
    );
}

/// No observational corpus: the protocol is intact but the grid is not there.
#[test]
fn a_run_with_no_corpus_is_unclassified() {
    let template = frozen_template();
    let controls = frozen_controls(&template);
    let envelope = fewrs::envelope(&RunDescriptor {
        alpha: ALPHA,
        budget: BUDGET,
        specimens: &[],
        controls: &controls,
        cells: &[],
    });
    assert_unclassified("no corpus", &envelope, "unique observational cells");
}

/// **The defect that motivated the gate.** `--replicates 99` printed `STRONG`.
#[test]
fn a_run_at_99_replicates_is_unclassified() {
    let control = negative_control();
    let mut template = assay("template", &control.first, &control.second, 3, 1);
    template.planned_budget = 99;
    template.null_searches = 99;
    template.seed_range = fewrs::seed_range(99);
    let controls = frozen_controls(&template);
    let cells = frozen_grid(&template);
    let specimens = frozen_specimens();
    let envelope = fewrs::envelope(&RunDescriptor {
        alpha: ALPHA,
        budget: 99,
        specimens: &specimens,
        controls: &controls,
        cells: &cells,
    });
    assert!(!envelope.protocol_established());
    assert_unclassified("99 replicates", &envelope, "budget is 99");
    let reasons: Vec<String> = envelope.reasons().iter().map(|r| r.to_string()).collect();
    assert!(
        reasons.iter().any(|reason| reason.contains("seed range")),
        "the seed prefix is a separate condition and must also fire: {reasons:?}"
    );
}

/// Thirty cells is not the test. **Thirty of the right cells is.**
#[test]
fn thirty_cells_with_the_wrong_identities_are_unclassified() {
    let template = frozen_template();
    let controls = frozen_controls(&template);
    let specimens = frozen_specimens();
    // Same count, same span lengths, pairs that were never in the grid.
    let cells: Vec<Cell> = SPRINT_19_ORDER_NULL_GRID
        .iter()
        .map(|row| {
            let mut cell = cell_at(&template, row.specimen, row.k);
            cell.specimen = format!("aaaaaaaa x {}", &row.specimen[11..]);
            cell.historical_tail = None;
            cell
        })
        .collect();
    assert_eq!(cells.len(), 30, "the count alone would have passed");
    let envelope = fewrs::envelope(&RunDescriptor {
        alpha: ALPHA,
        budget: BUDGET,
        specimens: &specimens,
        controls: &controls,
        cells: &cells,
    });
    assert_unclassified("wrong identities", &envelope, "not the frozen 30 cells");
}

/// A missing, an extra, and a duplicated cell, each on its own.
#[test]
fn a_missing_extra_or_duplicate_cell_is_unclassified() {
    let template = frozen_template();
    let controls = frozen_controls(&template);
    let specimens = frozen_specimens();
    let descriptor = |cells: &[Cell]| {
        fewrs::envelope(&RunDescriptor {
            alpha: ALPHA,
            budget: BUDGET,
            specimens: &specimens,
            controls: &controls,
            cells,
        })
    };

    let mut missing = frozen_grid(&template);
    missing.pop();
    assert_unclassified("missing", &descriptor(&missing), "not the frozen 30 cells");

    let mut extra = frozen_grid(&template);
    extra.push(cell_at(&template, "8b68dece x 57f18ff9", 5));
    assert_unclassified("extra", &descriptor(&extra), "not the frozen 30 cells");

    let mut duplicated = frozen_grid(&template);
    duplicated.push(duplicated[0].clone());
    assert_unclassified("duplicate", &descriptor(&duplicated), "duplicated cells");
}

/// A cell that cannot be joined to the published grid is named as such.
#[test]
fn a_cell_unmatched_to_the_historical_grid_is_unclassified() {
    let template = frozen_template();
    let controls = frozen_controls(&template);
    let specimens = frozen_specimens();
    let mut cells = frozen_grid(&template);
    // Right identity, no historical join — the shape a widened grid would take.
    cells[7].historical_tail = None;
    let envelope = fewrs::envelope(&RunDescriptor {
        alpha: ALPHA,
        budget: BUDGET,
        specimens: &specimens,
        controls: &controls,
        cells: &cells,
    });
    assert_unclassified(
        "unmatched",
        &envelope,
        "no match in sprint:19's published grid",
    );
}

/// The specimen identity set is compared, not counted.
#[test]
fn a_wrong_specimen_set_is_unclassified() {
    let template = frozen_template();
    let controls = frozen_controls(&template);
    let cells = frozen_grid(&template);

    for specimens in [
        vec!["8b68dece", "57f18ff9", "f5c18299"],
        vec!["8b68dece", "57f18ff9", "f5c18299", "7d95c414", "c3afa0ca"],
        vec!["8b68dece", "57f18ff9", "f5c18299", "c3afa0ca"],
    ] {
        let owned: Vec<String> = specimens.iter().map(|s| (*s).to_owned()).collect();
        let envelope = fewrs::envelope(&RunDescriptor {
            alpha: ALPHA,
            budget: BUDGET,
            specimens: &owned,
            controls: &controls,
            cells: &cells,
        });
        assert_unclassified("specimen set", &envelope, "decision:8's four");
    }
}

/// A different null construction or seed schedule breaks the protocol, and the
/// cell reports it from the path it took rather than from a caller's claim.
#[test]
fn a_different_null_or_seed_schedule_is_unclassified() {
    let template = frozen_template();
    let specimens = frozen_specimens();

    let mut controls = frozen_controls(&template);
    controls[0].null_mode = "doublet (doublet_null_seeded)";
    let envelope = fewrs::envelope(&RunDescriptor {
        alpha: ALPHA,
        budget: BUDGET,
        specimens: &specimens,
        controls: &controls,
        cells: &frozen_grid(&template),
    });
    assert_unclassified("null mode", &envelope, "a null other than");

    let controls = frozen_controls(&template);
    let mut cells = frozen_grid(&template);
    cells[3].seed_range = "null_seed(540..999, {0,1})".to_owned();
    let envelope = fewrs::envelope(&RunDescriptor {
        alpha: ALPHA,
        budget: BUDGET,
        specimens: &specimens,
        controls: &controls,
        cells: &cells,
    });
    assert_unclassified("seed schedule", &envelope, "seed range is not");
}

/// A run that never executed the controls cannot be classified, whatever its
/// grid looks like.
#[test]
fn a_run_that_bypassed_the_controls_is_unclassified() {
    let template = frozen_template();
    let specimens = frozen_specimens();
    let mut controls = frozen_controls(&template);
    controls.retain(|cell| cell.specimen != "positive control");
    let envelope = fewrs::envelope(&RunDescriptor {
        alpha: ALPHA,
        budget: BUDGET,
        specimens: &specimens,
        controls: &controls,
        cells: &frozen_grid(&template),
    });
    assert_unclassified("no controls", &envelope, "controls not executed");
}

/// The cost figures are the commission's, computed rather than quoted.
#[test]
fn the_cost_accounting_reports_the_searches_actually_performed() {
    let control = negative_control();
    let cells: Vec<_> = LADDER
        .into_iter()
        .map(|k| assay("cost", &control.first, &control.second, k, TINY))
        .collect();
    let cost = fewrs::cost(&cells);

    assert_eq!(cost.null_searches, TINY * LADDER.len());
    assert_eq!(cost.null_datasets, TINY * LADDER.len() * 2);
    assert_eq!(cost.reference_null_searches, 999 * LADDER.len());
    assert_eq!(
        cost.searches_avoided,
        cost.reference_null_searches - cost.null_searches
    );
    assert!(
        cost.null_candidate_evaluations > 0,
        "existing instrumentation counts enumerated window pairs; no new counter was added"
    );

    // The headline ratio at the preregistered budget, from the same arithmetic.
    let at_budget = 999.0 / BUDGET as f64;
    assert!(
        (at_budget - 2.176).abs() < 0.001,
        "999/459 = {at_budget}, not the 8-to-64-search figure looser alphas suggest"
    );
}

/// The early-stopping counterfactual is an expectation, is `None` when nothing
/// refutes, and is never presented as a measured index.
#[test]
fn the_early_stopping_counterfactual_is_absent_when_nothing_refutes() {
    assert_eq!(fewrs::expected_early_stop(459, 0), None);
    assert_eq!(fewrs::expected_early_stop(459, 1), Some(230.0));
    assert_eq!(fewrs::expected_early_stop(459, 459), Some(1.0));

    let control = positive_control();
    let cell = assay("early", &control.first, &control.second, 12, TINY);
    if cell.certified {
        assert_eq!(cell.refuting_nulls, 0);
        assert!(cell.expected_early_stop.is_none());
    }
}

/// The smallest budget at which the negative control's rule is a real assertion
/// rather than a coin toss, and the reason it is not [`TINY`].
///
/// **This test first ran at `TINY = 15` and failed at `k = 8`** — not because the
/// machinery is wrong but because 15 replicates cannot resolve a cell whose
/// sprint:19 exceedance rate is `34/999 ≈ 0.034`: the chance that none of 15
/// replicates reaches the observation is about `0.6`, so "does not certify" was
/// a coin toss dressed as an assertion. That is the same unreachable-criterion
/// class `tests/calibration.rs` records from sprint:18, met again here. The fix
/// is to give the test a budget that resolves the rule, **never** to weaken the
/// rule to fit the budget: at 60 replicates the expected exceedance count is
/// about 2 and the measured refuting count at `k = 8` is 1.
const RESOLVING: usize = 60;

/// The negative control is generated *according to the order-null hypothesis*,
/// so it must not certify — the same rule §PHASE 4 preregisters, checked here at
/// the smallest budget that resolves it.
#[test]
fn the_negative_control_does_not_certify_at_any_ladder_length() {
    let control = negative_control();
    let flagged: Vec<usize> = LADDER
        .into_iter()
        .filter(|k| assay("neg", &control.first, &control.second, *k, RESOLVING).certified)
        .collect();
    assert!(
        flagged.is_empty(),
        "a specimen drawn from the null must not certify; certified at k={flagged:?}"
    );
}

/// The positive control plants a 12-mark figure three times in each sequence and
/// must certify at the planted length.
#[test]
fn the_positive_control_certifies_at_the_planted_length() {
    let control = positive_control();
    let cell = assay("pos", &control.first, &control.second, 12, TINY);
    assert!(
        cell.certified,
        "T = {:?} against a null maximum of {}",
        cell.observed, cell.null_max
    );
    assert_eq!(cell.refuting_nulls, 0);
}

// ---------------------------------------------------------------------------
// The output contract — maintenance:3
// ---------------------------------------------------------------------------
//
// These run the real example binary. Testing `serde_json::to_string` on the
// document would prove nothing: the defect maintenance:3 repaired was that the
// *human report* also went to stdout, so only capturing the process's stdout can
// show it is gone, and only the process path can catch a stray `println!` added
// later.

/// A line only the example's own `--help` prints.
const EXAMPLE_BANNER: &str = "event-motif — a disposable sprint:8 experiment";

/// The flag under test, as its own `--help` advertises it. A build that predates
/// sprint:22 prints the banner and not this.
const EXAMPLE_MODE: &str = "--fewrs";

/// The built, **runnable** `event-motif` example, found beside this test binary.
///
/// `CARGO_BIN_EXE_*` covers `[[bin]]` targets and not examples, so the directory
/// is derived from `current_exe()`: a test runs from `<profile>/deps/`, and cargo
/// puts examples in `<profile>/examples/`. That keeps debug and release runs
/// pointing at their own build rather than at a hard-coded `target/debug`.
///
/// **Picking by name, by modification time, or by banner is not enough**, and
/// finding that out is what this comment is for. Two distinct wrong builds sit in
/// this directory:
///
/// - Under `--all-targets` — which `scripts/check.sh` uses — cargo compiles each
///   example a *second* time as a libtest harness. It answers `--fewrs` with
///   `error: Unrecognized option: 'fewrs'` and exit 101.
/// - Builds from earlier sessions persist. One predating sprint:22 prints the
///   example banner happily and then rejects `--fewrs` as an unexpected
///   argument.
///
/// So candidates are selected by the **capability under test**: run `--help` and
/// keep a build whose own usage advertises the mode these tests exercise. Newest
/// first among those, so a current build wins over a current-but-older one.
fn event_motif_binary() -> std::path::PathBuf {
    let exe = std::env::current_exe().expect("the test binary knows its own path");
    let profile = exe
        .parent()
        .and_then(std::path::Path::parent)
        .expect("a test binary lives at <profile>/deps/<name>");
    let directory = profile.join("examples");
    let entries = std::fs::read_dir(&directory).unwrap_or_else(|err| {
        panic!(
            "{} should exist — `cargo test` builds examples by default: {err}",
            directory.display()
        )
    });

    let mut candidates: Vec<(std::time::SystemTime, std::path::PathBuf)> = Vec::new();
    for entry in entries {
        let entry = entry.expect("a readable directory entry");
        let name = entry.file_name().to_string_lossy().into_owned();
        // Cargo emits `event_motif-<hash>` plus `.d` and `.o` siblings.
        if !name.starts_with("event_motif-") || name.contains('.') {
            continue;
        }
        let modified = entry
            .metadata()
            .and_then(|meta| meta.modified())
            .expect("a modification time");
        candidates.push((modified, entry.path()));
    }
    // Newest first, and by path when two share a timestamp, so a failure names
    // the same build twice running.
    candidates.sort_by(|left, right| right.0.cmp(&left.0).then(left.1.cmp(&right.1)));

    for (_, candidate) in &candidates {
        let Ok(output) = std::process::Command::new(candidate).arg("--help").output() else {
            continue;
        };
        let help = String::from_utf8_lossy(&output.stdout);
        if help.contains(EXAMPLE_BANNER) && help.contains(EXAMPLE_MODE) {
            return candidate.clone();
        }
    }
    panic!(
        "no runnable event-motif example under {} advertising {EXAMPLE_MODE} (found {} \
         candidate(s); stale builds and libtest-harness builds are skipped by design)",
        directory.display(),
        candidates.len()
    )
}

/// A controls-only diagnostic run, cheap enough for the suite. Two replicates is
/// deliberately **not** the frozen budget: the run is diagnostic by construction,
/// which is what makes it exercise both repairs at once.
fn run_fewrs(json: bool) -> std::process::Output {
    let mut command = std::process::Command::new(event_motif_binary());
    command.args(["--fewrs", "--replicates", "2"]);
    if json {
        command.arg("--json");
    }
    command
        .output()
        .expect("the example should run to completion")
}

/// **The regression.** With `--json`, stdout is one JSON document and nothing
/// else — no banner, no table, no trailing prose.
#[test]
fn json_output_is_a_single_parseable_document_on_stdout() {
    let output = run_fewrs(true);
    assert!(output.status.success(), "the example should exit 0");

    let stdout = String::from_utf8(output.stdout).expect("stdout is UTF-8");
    assert!(
        stdout.trim_start().starts_with('{'),
        "stdout must open with the document, not a banner: {:?}",
        stdout.chars().take(120).collect::<String>()
    );

    // The whole stream, not a prefix of it: `from_str` refuses trailing content,
    // which is exactly the defect — the old path appended JSON *after* a report.
    let document: serde_json::Value =
        serde_json::from_str(&stdout).expect("the entire stdout stream must parse as one document");
    assert_eq!(document["label"], "fewrs");
    assert_eq!(document["budget"], 2);

    // And the human report is not lost — it moved to stderr.
    let stderr = String::from_utf8(output.stderr).expect("stderr is UTF-8");
    assert!(
        stderr.contains("the sprint:22 fixed-budget FewRS retrospective assay"),
        "the report must remain visible on stderr"
    );
    // Report-only markers. The document's own `role` field mentions the round,
    // so the marker has to be something only the human rendering emits.
    for marker in ["event-motif", "§PHASE 4", "control rules —", "null max"] {
        assert!(
            !stdout.contains(marker),
            "no part of the report may reach stdout; found {marker:?}"
        );
    }
}

/// Without `--json` the human rendering is unchanged and stdout carries it.
#[test]
fn without_json_the_human_report_still_goes_to_stdout() {
    let output = run_fewrs(false);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is UTF-8");
    assert!(stdout.contains("the sprint:22 fixed-budget FewRS retrospective assay"));
    assert!(stdout.contains("classification — task:32"));
    assert!(
        serde_json::from_str::<serde_json::Value>(&stdout).is_err(),
        "the human rendering is not JSON, and is not claimed to be"
    );
}

/// **The two renderings must not disagree.** Same run, same classification, same
/// eligibility, same reasons — one of them machine-readable and one of them not.
#[test]
fn json_and_human_output_agree_about_eligibility_classification_and_reasons() {
    let json_run = run_fewrs(true);
    let human_run = run_fewrs(false);

    let document: serde_json::Value =
        serde_json::from_str(&String::from_utf8(json_run.stdout).expect("UTF-8"))
            .expect("one document");
    let json_report = String::from_utf8(json_run.stderr).expect("UTF-8");
    let human_report = String::from_utf8(human_run.stdout).expect("UTF-8");

    let label = document["classification"]
        .as_str()
        .expect("a classification label");
    assert_eq!(
        label,
        Classification::DiagnosticUnclassified.label(),
        "a two-replicate controls-only run is a diagnostic"
    );
    assert_eq!(document["classification_is_preregistered"], false);
    assert_eq!(document["eligibility"]["protocol_established"], false);
    assert_eq!(document["eligibility"]["grid_complete"], false);

    for report in [&json_report, &human_report] {
        assert!(
            report.contains(label),
            "both renderings name the classification"
        );
        assert!(
            !report.contains("STRONG needs >="),
            "an unclassified run must never be shown against the frozen threshold"
        );
    }

    let reasons = document["eligibility"]["reasons_text"]
        .as_array()
        .expect("machine-readable reasons");
    assert!(!reasons.is_empty(), "an unclassified run must say why");
    for reason in reasons {
        let text = reason.as_str().expect("a reason line");
        for report in [&json_report, &human_report] {
            assert!(
                report.contains(text),
                "both renderings must carry the reason {text:?}"
            );
        }
    }

    // The typed reasons and their rendered text describe the same conditions.
    let conditions: Vec<&str> = document["eligibility"]["protocol_reasons"]
        .as_array()
        .expect("typed protocol reasons")
        .iter()
        .map(|reason| reason["condition"].as_str().expect("a tagged condition"))
        .collect();
    assert!(
        conditions.contains(&"budget_not_derived"),
        "a two-replicate run breaks the budget condition: {conditions:?}"
    );
}
