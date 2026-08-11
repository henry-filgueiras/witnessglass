//! **Disposable research experiment.** sprint:22, task:32.
//!
//! Few Random Searches (FewRS), reduced to the one thing task:32 asks of it: a
//! fixed replicate budget derived from `alpha`, and a strict maximum-null
//! decision rule applied to the *complete* search-aware statistic sprint:19
//! froze.
//!
//! # What this module is not
//!
//! Not a subsystem, not a framework, and not a multi-analysis procedure. There
//! is no null here, no search here, and no statistic here. Everything numerical
//! comes from [`super::calibration`]: [`assay`] calls
//! [`super::calibration::calibrate`], which reruns
//! [`super::calibration::complete_search`] inside every replicate over
//! `order_null_seeded` at the existing `null_seed` schedule. This module
//! contributes two scalar functions and a row type.
//!
//! # The guarantee, stated exactly, because it is smaller than the paper's
//!
//! task:32 §PHASE 0 D11. The published FewRS procedure compares every analysis
//! against the maximum statistic over **all analyses and all `m` resamples**,
//! and derives family-wise error control from that pooled maximum. This module
//! compares each cell against **its own** null maximum, because `T_k` at
//! different span lengths are R1 sums over different window sizes and are not on
//! a common scale — there is no pooled maximum to form, and task:32 §PHASE 11
//! forbids inventing one by normalization.
//!
//! What a certified cell therefore earns is an **exact conditional test at level
//! `1/(m+1) = 1/460`**, by exchangeability of the observed statistic with its own
//! 459 null statistics. It earns **no** family-wise guarantee across the 30
//! cells, and nothing here may be read as one.

use serde::Serialize;

use super::calibration::{self, Calibration};
use super::event_sequence::EventSequence;

/// WitnessGlass's frozen significance level — sprint:19's `TAIL_THRESHOLD`,
/// re-used rather than re-chosen.
pub const ALPHA: f64 = calibration::TAIL_THRESHOLD;

/// The FewRS budget at [`ALPHA`], as task:32 §PHASE 1 derives it.
///
/// A constant rather than a call because `f64::ln` is not `const`. A test pins
/// this to [`fewrs_budget`]'s own output, so the two cannot drift.
pub const BUDGET: usize = 459;

/// Why an `alpha` was refused. One variant, because there is one way to be
/// wrong: the formula is defined on the open interval and nowhere else.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum BudgetError {
    /// `alpha` was not finite, or not strictly inside `(0, 1)`.
    AlphaOutsideOpenUnitInterval,
}

impl std::fmt::Display for BudgetError {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlphaOutsideOpenUnitInterval => {
                out.write_str("alpha must be finite and strictly between 0 and 1")
            }
        }
    }
}

impl std::error::Error for BudgetError {}

/// The FewRS budget: `m = ceil( ln(1/alpha) / ln(1/(1-alpha)) )`.
///
/// At `alpha = 0.01` this is `ceil(4.6051701860 / 0.0100503359) = ceil(458.2106)
/// = 459`.
///
/// Both endpoints are refused: at `alpha = 0` the numerator diverges and at
/// `alpha = 1` the denominator does, so neither yields a budget.
pub fn fewrs_budget(alpha: f64) -> Result<usize, BudgetError> {
    if !alpha.is_finite() || alpha <= 0.0 || alpha >= 1.0 {
        return Err(BudgetError::AlphaOutsideOpenUnitInterval);
    }
    let ratio = (1.0 / alpha).ln() / (1.0 / (1.0 - alpha)).ln();
    Ok(ratio.ceil() as usize)
}

/// The decision rule, task:32 §PHASE 2. **Strict**, and deliberately so.
///
/// Certifies iff the observation is defined, at least one null statistic exists,
/// and **every** null statistic is strictly below the observation. A tie with the
/// null maximum does not certify. An empty null set does not certify: no evidence
/// is not evidence, and that branch is what would otherwise turn a search which
/// admitted no candidate into a certification.
///
/// Reads only values, never positions, so it is order-invariant by construction.
pub fn fewrs_certifies(observed: Option<f64>, nulls: &[f64]) -> bool {
    let Some(observed) = observed else {
        return false;
    };
    !nulls.is_empty() && nulls.iter().all(|null| *null < observed)
}

/// How many null statistics reached or exceeded the observation.
///
/// Zero exactly when [`fewrs_certifies`] holds on a non-empty null set. Order
/// invariant, like the rule itself.
pub fn refuting_nulls(observed: Option<f64>, nulls: &[f64]) -> usize {
    match observed {
        Some(observed) => nulls.iter().filter(|null| **null >= observed).count(),
        // An undefined observation is refuted by every replicate, which is the
        // same convention `calibration::calibrate` already uses.
        None => nulls.len(),
    }
}

/// task:32 §PHASE 7's early-stopping counterfactual, as an **expectation** and
/// labelled as one.
///
/// A non-certifying scalar assay can stop at the first replicate whose statistic
/// reaches the observation, because no later sample can reverse the failure. The
/// index at which that happens depends on the order the replicates ran in, and
/// `calibration::calibrate` returns its samples sorted — the order the maximum
/// needs, not the order the searches happened in. Rather than read a position out
/// of a sorted vector and call it a replicate index, this returns the expected
/// stop position under exchangeable replicate ordering, `(m + 1) / (r + 1)` for
/// `r` refuting nulls.
///
/// `None` when nothing refutes, where the full budget is spent by definition.
/// **No early-stopping path is executed anywhere in this round**: the fixed
/// budget is what produces the maximum-null comparisons the assay audits.
pub fn expected_early_stop(budget: usize, refuting: usize) -> Option<f64> {
    if refuting == 0 {
        return None;
    }
    Some((budget as f64 + 1.0) / (refuting as f64 + 1.0))
}

/// One `(specimen, k)` cell of the retrospective assay, with everything task:32
/// §PHASE 5 retains for audit.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Cell {
    /// Opaque specimen label. Session prefixes only — decision:8 forbids more.
    pub specimen: String,
    /// The span length.
    pub k: usize,
    /// `T_k` on the observed sequences. `None` when the search admitted nothing.
    pub observed: Option<f64>,
    /// The largest `T_k` any of the `null_searches` null searches produced.
    pub null_max: f64,
    /// The strict rule's answer.
    pub certified: bool,
    /// The budget the run planned to spend.
    pub planned_budget: usize,
    /// Complete null searches actually performed for this cell. Equal to
    /// `planned_budget`: no cell stops early, and reporting the two separately is
    /// what makes that visible rather than assumed.
    pub null_searches: usize,
    /// Null sequence realizations generated for this cell — two per replicate,
    /// one per side.
    pub null_datasets: usize,
    /// Replicates whose complete search yielded a defined `T_k`.
    pub realizations: usize,
    /// Window pairs one observed search enumerated.
    pub observed_considered: usize,
    /// Mean window pairs one null search enumerated. Existing instrumentation;
    /// no new counter was added for this round.
    pub null_considered_mean: f64,
    /// Null searches whose `T_k` reached or exceeded the observation. Zero
    /// exactly when `certified`.
    pub refuting_nulls: usize,
    /// task:32 §PHASE 7's early-stopping counterfactual, as an expectation under
    /// exchangeable replicate ordering. `None` when nothing refutes. Not a
    /// measured index, and no early-stopping path was run.
    pub expected_early_stop: Option<f64>,
    /// The frozen 999-replicate `p_hat`, when this cell is one of the 30.
    pub historical_tail: Option<f64>,
    /// The frozen 999-replicate exceedance count.
    pub historical_exceedances: Option<usize>,
    /// The frozen verdict under sprint:19's own rule, `p_hat <= 0.01`.
    pub historical_exceptional: Option<bool>,
    /// The frozen verdict under **this round's** rule, `exceedances == 0`.
    pub historical_certified: Option<bool>,
    /// `certified == historical_exceptional`. task:32 §PHASE 9's primary.
    pub agrees_with_historical_tail_rule: Option<bool>,
    /// `certified == historical_certified`. §PHASE 9's rule-matched secondary.
    pub agrees_with_historical_max_rule: Option<bool>,
    /// The seed-range identity this cell consumed, for audit.
    pub seed_range: String,
}

/// Run one cell at the FewRS budget.
///
/// Delegates every numerical stage to [`calibration::calibrate`], which is
/// sprint:19's frozen path: `order_null_seeded` replicates at `null_seed(i, 0)`
/// and `null_seed(i, 1)` for `i` in `0..budget`, with the entire complete search
/// rerun inside each one. Nothing here preselects a motif or a candidate.
pub fn assay(
    specimen: &str,
    first: &EventSequence<'_>,
    second: &EventSequence<'_>,
    k: usize,
    budget: usize,
) -> Cell {
    let calibration = calibration::calibrate(specimen, first, second, k, budget);
    cell_from(calibration, budget)
}

/// The same cell, built from a calibration the caller already has.
///
/// Split out so a test can prove the FewRS reading of a calibration is exactly
/// the strict rule applied to that calibration's own samples, without paying for
/// a second 459-replicate run.
pub fn cell_from(calibration: Calibration, budget: usize) -> Cell {
    let historical = historical_cell(&calibration.specimen, calibration.k);
    let certified = fewrs_certifies(calibration.observed, &calibration.samples);
    let refuting = refuting_nulls(calibration.observed, &calibration.samples);

    Cell {
        certified,
        planned_budget: budget,
        null_searches: budget,
        null_datasets: budget * 2,
        realizations: calibration.realizations,
        observed_considered: calibration.observed_considered,
        null_considered_mean: calibration.null_considered_mean,
        refuting_nulls: refuting,
        expected_early_stop: expected_early_stop(budget, refuting),
        historical_tail: historical.map(|row| row.tail),
        historical_exceedances: historical.map(HistoricalCell::exceedances),
        historical_exceptional: historical.map(HistoricalCell::exceptional),
        historical_certified: historical.map(HistoricalCell::certified),
        agrees_with_historical_tail_rule: historical.map(|row| certified == row.exceptional()),
        agrees_with_historical_max_rule: historical.map(|row| certified == row.certified()),
        seed_range: format!("null_seed(0..{budget}, {{0,1}})"),
        specimen: calibration.specimen,
        k: calibration.k,
        observed: calibration.observed,
        null_max: calibration.null_max,
    }
}

// ---------------------------------------------------------------------------
// PHASE 6 — the frozen reference grid
// ---------------------------------------------------------------------------

/// The replicate count sprint:19 and sprint:20 both spent.
pub const HISTORICAL_REPLICATES: usize = 999;

/// One published cell of sprint:19's order-null grid.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct HistoricalCell {
    /// The specimen label, in the form the runner builds from session prefixes.
    pub specimen: &'static str,
    /// The span length.
    pub k: usize,
    /// The published `p_hat = (1 + exceedances) / 1000`.
    pub tail: f64,
}

impl HistoricalCell {
    /// `exceedances = 1000 * p_hat - 1`, recovered exactly from the published
    /// tail. `p_hat` is a ratio of integers with a denominator of 1000, so the
    /// rounding here repairs binary representation and nothing else.
    pub fn exceedances(self) -> usize {
        ((self.tail * (HISTORICAL_REPLICATES as f64 + 1.0)).round() as usize).saturating_sub(1)
    }

    /// sprint:19's own verdict for this cell: `p_hat <= 0.01`.
    pub fn exceptional(self) -> bool {
        self.tail <= calibration::TAIL_THRESHOLD
    }

    /// **This round's** rule applied to the same 999 replicates: the observation
    /// strictly exceeded every null, i.e. no replicate reached it.
    pub fn certified(self) -> bool {
        self.exceedances() == 0
    }
}

/// sprint:19's published 30-cell order-null grid, transcribed from task:29 §9
/// and reproduced cell for cell by sprint:20 §PHASE 0.
///
/// **The reference is the frozen archaeology.** task:32 runs no fresh
/// 999-replicate campaign: the historical benchmark reconstructs honestly from
/// what was published, and rerunning it would replace a frozen reference with a
/// new one. A test asserts this table still yields 23 exceptional cells and 13
/// certified ones, so a transcription slip is caught by the suite.
pub const SPRINT_19_ORDER_NULL_GRID: [HistoricalCell; 30] = [
    HistoricalCell {
        specimen: "8b68dece x 57f18ff9",
        k: 3,
        tail: 0.157,
    },
    HistoricalCell {
        specimen: "8b68dece x 57f18ff9",
        k: 4,
        tail: 0.004,
    },
    HistoricalCell {
        specimen: "8b68dece x 57f18ff9",
        k: 6,
        tail: 0.005,
    },
    HistoricalCell {
        specimen: "8b68dece x 57f18ff9",
        k: 8,
        tail: 0.004,
    },
    HistoricalCell {
        specimen: "8b68dece x 57f18ff9",
        k: 12,
        tail: 0.007,
    },
    HistoricalCell {
        specimen: "8b68dece x f5c18299",
        k: 3,
        tail: 0.004,
    },
    HistoricalCell {
        specimen: "8b68dece x f5c18299",
        k: 4,
        tail: 0.003,
    },
    HistoricalCell {
        specimen: "8b68dece x f5c18299",
        k: 6,
        tail: 0.015,
    },
    HistoricalCell {
        specimen: "8b68dece x f5c18299",
        k: 8,
        tail: 0.002,
    },
    HistoricalCell {
        specimen: "8b68dece x f5c18299",
        k: 12,
        tail: 0.075,
    },
    HistoricalCell {
        specimen: "8b68dece x 7d95c414",
        k: 3,
        tail: 0.133,
    },
    HistoricalCell {
        specimen: "8b68dece x 7d95c414",
        k: 4,
        tail: 0.031,
    },
    HistoricalCell {
        specimen: "8b68dece x 7d95c414",
        k: 6,
        tail: 0.027,
    },
    HistoricalCell {
        specimen: "8b68dece x 7d95c414",
        k: 8,
        tail: 0.003,
    },
    HistoricalCell {
        specimen: "8b68dece x 7d95c414",
        k: 12,
        tail: 0.002,
    },
    HistoricalCell {
        specimen: "57f18ff9 x f5c18299",
        k: 3,
        tail: 0.035,
    },
    HistoricalCell {
        specimen: "57f18ff9 x f5c18299",
        k: 4,
        tail: 0.001,
    },
    HistoricalCell {
        specimen: "57f18ff9 x f5c18299",
        k: 6,
        tail: 0.001,
    },
    HistoricalCell {
        specimen: "57f18ff9 x f5c18299",
        k: 8,
        tail: 0.001,
    },
    HistoricalCell {
        specimen: "57f18ff9 x f5c18299",
        k: 12,
        tail: 0.001,
    },
    HistoricalCell {
        specimen: "57f18ff9 x 7d95c414",
        k: 3,
        tail: 0.002,
    },
    HistoricalCell {
        specimen: "57f18ff9 x 7d95c414",
        k: 4,
        tail: 0.001,
    },
    HistoricalCell {
        specimen: "57f18ff9 x 7d95c414",
        k: 6,
        tail: 0.001,
    },
    HistoricalCell {
        specimen: "57f18ff9 x 7d95c414",
        k: 8,
        tail: 0.001,
    },
    HistoricalCell {
        specimen: "57f18ff9 x 7d95c414",
        k: 12,
        tail: 0.001,
    },
    HistoricalCell {
        specimen: "f5c18299 x 7d95c414",
        k: 3,
        tail: 0.001,
    },
    HistoricalCell {
        specimen: "f5c18299 x 7d95c414",
        k: 4,
        tail: 0.001,
    },
    HistoricalCell {
        specimen: "f5c18299 x 7d95c414",
        k: 6,
        tail: 0.001,
    },
    HistoricalCell {
        specimen: "f5c18299 x 7d95c414",
        k: 8,
        tail: 0.001,
    },
    HistoricalCell {
        specimen: "f5c18299 x 7d95c414",
        k: 12,
        tail: 0.001,
    },
];

/// The frozen cell for a `(specimen, k)`, or `None` for a cell sprint:19 never
/// published — every control, and any pair outside decision:8's inventory.
pub fn historical_cell(specimen: &str, k: usize) -> Option<HistoricalCell> {
    SPRINT_19_ORDER_NULL_GRID
        .iter()
        .find(|row| row.specimen == specimen && row.k == k)
        .copied()
}

// ---------------------------------------------------------------------------
// PHASE 9 — classification, and PHASE 7 — cost
// ---------------------------------------------------------------------------

/// task:32 §PHASE 9's threshold: cells that must certify for a STRONG result.
pub const STRONG_THRESHOLD: usize = 15;

/// The preregistered classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Classification {
    /// A control failed, or the assay yields nothing beyond a cheaper
    /// non-rejection.
    Falsification,
    /// Both controls passed and at least [`STRONG_THRESHOLD`] cells certified.
    Strong,
    /// Both controls passed and fewer than [`STRONG_THRESHOLD`] cells certified.
    WeakMixed,
}

impl Classification {
    /// The label a report prints.
    pub fn label(self) -> &'static str {
        match self {
            Self::Falsification => "FALSIFICATION",
            Self::Strong => "STRONG",
            Self::WeakMixed => "WEAK / MIXED",
        }
    }
}

/// Classify by §PHASE 9's precedence. Falsification is checked first.
///
/// The two remaining branches are complementary on `certified`, so no outcome
/// falls between them.
pub fn classify(controls_passed: bool, certified: usize) -> Classification {
    if !controls_passed {
        Classification::Falsification
    } else if certified >= STRONG_THRESHOLD {
        Classification::Strong
    } else {
        Classification::WeakMixed
    }
}

/// task:32 §PHASE 7's cost accounting, in the quantities the run reports.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Cost {
    /// Complete null searches this run performed.
    pub null_searches: usize,
    /// Null sequence realizations generated — two per replicate.
    pub null_datasets: usize,
    /// Window pairs enumerated inside null searches, from existing
    /// instrumentation only.
    pub null_candidate_evaluations: u128,
    /// What the same coverage costs at sprint:19's `B = 999`.
    pub reference_null_searches: usize,
    /// `reference_null_searches - null_searches`.
    pub searches_avoided: usize,
    /// `reference_null_searches / null_searches`.
    pub ratio: f64,
}

/// Sum a run's cost over its cells.
pub fn cost(cells: &[Cell]) -> Cost {
    let null_searches: usize = cells.iter().map(|cell| cell.null_searches).sum();
    let reference = cells.len() * HISTORICAL_REPLICATES;
    Cost {
        null_searches,
        null_datasets: cells.iter().map(|cell| cell.null_datasets).sum(),
        null_candidate_evaluations: cells
            .iter()
            .map(|cell| (cell.null_considered_mean * cell.null_searches as f64).round() as u128)
            .sum(),
        reference_null_searches: reference,
        searches_avoided: reference.saturating_sub(null_searches),
        ratio: if null_searches == 0 {
            f64::NAN
        } else {
            reference as f64 / null_searches as f64
        },
    }
}
