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
//! task:32 §PHASE 0 D11, **as corrected by maintenance:3.** The original wording
//! here explained `m = 459` as the price of pooling a family of analyses. That
//! is wrong, and the correction matters because it is the only reason this
//! module exists to be read.
//!
//! `m = ceil(ln(1/alpha)/ln(1/(1-alpha)))` is the cost of FewRS's **particular
//! high-probability upper-bound construction** — the budget at which its bound on
//! the null statistic's threshold holds with the probability it wants. The
//! formula takes `alpha` and nothing else. **It applies to a single analysis just
//! as it does to a family**, so the number is not caused by pooling and does not
//! shrink if you stop pooling.
//!
//! What this module computes is not FewRS's procedure: it compares each cell
//! against **its own** null maximum rather than against a maximum pooled over
//! analyses and resamples. That per-cell rule is an ordinary strict-maximum
//! randomization test, and its guarantee comes from exchangeability alone — under
//! the null the observed statistic and its `m` null statistics are exchangeable,
//! so the probability the observation is the strict maximum is at most
//! `1/(m+1)`.
//!
//! **Which is why 459 is the wrong number for this question.** For one
//! exchangeable scalar statistic at `alpha = 0.01`, `m = 99` already gives a null
//! rejection probability of at most `1/(99+1) = 0.01`. task:32 measured both:
//! the 99-draw test certified 22 of 30 cells and agreed with sprint:19's frozen
//! grid on 27 of 30, against 17 and 24 for the 459-draw FewRS assay. For the
//! narrow binary per-cell question, FewRS is **operationally dominated** — 4.6x
//! the computation for fewer certifications.
//!
//! **Do not over-read the 99-draw alternative either.** It is a per-cell test.
//! It does **not** confer family-wise control over this heterogeneous 30-cell
//! grid; a pooled max-statistic test would need a coherent null dataset, a family
//! statistic on a commensurable or defensibly normalized scale, and its own
//! error-control contract, none of which this round built or defended.
//!
//! **And do not rely on the paper's stronger threshold guarantee here.** Its
//! proof carries assumptions — subset pivotality, i.i.d. resamples — that this
//! round did not check against this pipeline. Nothing operational in WitnessGlass
//! should lean on it without independent statistical review.

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
    /// Which null construction produced this cell's replicates.
    ///
    /// Set by [`cell_from`] from the path it actually called, not declared by a
    /// caller: [`assay`] reaches `calibration::calibrate`, which is the order
    /// null and nothing else. The eligibility envelope reads this rather than
    /// taking a runner's word for which null ran.
    pub null_mode: &'static str,
}

/// The null construction every FewRS cell in this round used.
///
/// A `&'static str` rather than an enum because there is exactly one, and this
/// round adds no second construction — sprint:20's are reachable from
/// `calibrate_with` and are deliberately **not** wired into [`assay`].
pub const NULL_ORDER_PERMUTATION: &str = "order-permutation (order_null_seeded)";

/// The seed-range identity a run at `budget` consumes, as one string, so the
/// envelope compares an identity rather than re-deriving one.
pub fn seed_range(budget: usize) -> String {
    format!("null_seed(0..{budget}, {{0,1}})")
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
        seed_range: seed_range(budget),
        null_mode: NULL_ORDER_PERMUTATION,
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
// The classification envelope — maintenance:3
// ---------------------------------------------------------------------------
//
// task:32's classification is a statement about **one** protocol: the frozen
// alpha, the derived budget, the first 459 seeds of the existing schedule, the
// order null, the controls, decision:8's four specimens, and that exact 30-cell
// grid. The first implementation computed it from `controls_passed` and a
// certified count alone, so `--fewrs --replicates 99` — a diagnostic — printed
// `STRONG` beside the frozen 15-of-30 threshold as though it had run the assay.
//
// The repair is one gate, in one place, comparing identities and sets rather
// than counts. Rendering code asks the gate; it does not re-derive it.

/// The four specimens decision:8 admits, as the opaque prefixes the runner
/// builds labels from. A **set**: the order paths are supplied in does not
/// matter, but the membership does.
pub const EXPECTED_SPECIMENS: [&str; 4] = ["8b68dece", "57f18ff9", "f5c18299", "7d95c414"];

/// The two controls task:32 §PHASE 4 requires, by the labels the runner uses.
pub const EXPECTED_CONTROLS: [&str; 2] = ["negative control", "positive control"];

/// One reason a run is not the frozen assay.
///
/// Carries what was found as well as what was wanted, so a machine-readable
/// document and a human line can be rendered from the same value and cannot
/// disagree about why.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "condition", rename_all = "snake_case")]
pub enum Ineligibility {
    /// `alpha` was not the frozen significance level.
    AlphaNotFrozen {
        /// The `alpha` the run derived its budget from.
        found: f64,
        /// [`ALPHA`].
        expected: f64,
    },
    /// The budget was not the one [`fewrs_budget`] derives from that `alpha`.
    BudgetNotDerived {
        /// The budget the run spent.
        found: usize,
        /// [`BUDGET`].
        expected: usize,
    },
    /// Some cell consumed a seed range other than the first `expected` seeds.
    SeedPrefixNotUsed {
        /// [`seed_range`] at [`BUDGET`].
        expected: String,
        /// The distinct seed ranges the run's cells actually consumed.
        found: Vec<String>,
    },
    /// Some cell was produced by a construction other than the order null.
    NullModeNotOrderPermutation {
        /// The distinct null constructions the run's cells report.
        found: Vec<String>,
    },
    /// The controls were not executed over the whole ladder, both of them.
    ControlsNotExecuted {
        /// The `control k=N` cells the run did not produce.
        missing: Vec<String>,
    },
    /// The specimen identities were not decision:8's four.
    SpecimenSetMismatch {
        /// Expected prefixes the run did not project.
        missing: Vec<String>,
        /// Prefixes the run projected that decision:8 does not admit.
        unexpected: Vec<String>,
    },
    /// A `(pair, k)` cell appeared more than once.
    DuplicateCells {
        /// The repeated cell identities.
        duplicates: Vec<String>,
    },
    /// The observational grid was not exactly the frozen 30 cells.
    CellSetMismatch {
        /// Frozen cells the run did not produce.
        missing: Vec<String>,
        /// Cells the run produced that the frozen grid does not contain.
        unexpected: Vec<String>,
    },
    /// The unique observational cell count was not 30.
    CellCountNotThirty {
        /// Unique `(pair, k)` cells the run produced.
        found: usize,
    },
    /// Some cell could not be joined to sprint:19's published grid.
    CellsUnmatchedToHistoricalGrid {
        /// The cell identities with no historical counterpart.
        cells: Vec<String>,
    },
}

impl std::fmt::Display for Ineligibility {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn list(items: &[String]) -> String {
            if items.is_empty() {
                "none".to_owned()
            } else {
                items.join(", ")
            }
        }
        match self {
            Self::AlphaNotFrozen { found, expected } => {
                write!(out, "alpha is {found}, not the frozen {expected}")
            }
            Self::BudgetNotDerived { found, expected } => write!(
                out,
                "budget is {found}, not the {expected} derived from the frozen alpha"
            ),
            Self::SeedPrefixNotUsed { expected, found } => {
                write!(out, "seed range is not {expected}; found {}", list(found))
            }
            Self::NullModeNotOrderPermutation { found } => write!(
                out,
                "a cell used a null other than {NULL_ORDER_PERMUTATION}; found {}",
                list(found)
            ),
            Self::ControlsNotExecuted { missing } => {
                write!(
                    out,
                    "controls not executed over the ladder; missing {}",
                    list(missing)
                )
            }
            Self::SpecimenSetMismatch {
                missing,
                unexpected,
            } => write!(
                out,
                "specimen set is not decision:8's four; missing {}; unexpected {}",
                list(missing),
                list(unexpected)
            ),
            Self::DuplicateCells { duplicates } => {
                write!(out, "duplicated cells: {}", list(duplicates))
            }
            Self::CellSetMismatch {
                missing,
                unexpected,
            } => write!(
                out,
                "grid is not the frozen 30 cells; missing {}; unexpected {}",
                list(missing),
                list(unexpected)
            ),
            Self::CellCountNotThirty { found } => {
                write!(out, "{found} unique observational cells, not 30")
            }
            Self::CellsUnmatchedToHistoricalGrid { cells } => write!(
                out,
                "cells with no match in sprint:19's published grid: {}",
                list(cells)
            ),
        }
    }
}

/// What a run says about itself, for the gate to check.
///
/// Everything derivable from the cells is derived from the cells rather than
/// declared here — the null construction and the seed range in particular, which
/// [`cell_from`] stamps from the path it actually took.
pub struct RunDescriptor<'a> {
    /// The significance level the budget was derived from.
    pub alpha: f64,
    /// The replicate budget the run spent.
    pub budget: usize,
    /// Opaque session prefixes of the projected specimens, in any order.
    pub specimens: &'a [String],
    /// The control cells, in the order they ran.
    pub controls: &'a [Cell],
    /// The observational cells, in the order they ran.
    pub cells: &'a [Cell],
}

/// Whether a run established the frozen protocol, and whether it completed the
/// frozen grid. Two gates, because they answer different questions.
#[derive(Debug, Clone, PartialEq, Serialize, Default)]
pub struct Envelope {
    /// Why the run is not the frozen protocol. Empty means it is.
    pub protocol: Vec<Ineligibility>,
    /// Why the observational grid is not the frozen 30 cells. Empty means it is.
    ///
    /// Checked **only** for a run that reached the observational stage. A run
    /// whose controls failed stops before it, by the frozen protocol, so an
    /// empty grid there is obedience rather than a defect — see [`classify`].
    pub grid: Vec<Ineligibility>,
}

impl Envelope {
    /// The frozen protocol was established: alpha, budget, seeds, null, controls.
    pub fn protocol_established(&self) -> bool {
        self.protocol.is_empty()
    }

    /// The observational grid is exactly the frozen 30 cells.
    pub fn grid_complete(&self) -> bool {
        self.grid.is_empty()
    }

    /// Every reason, protocol first, for rendering.
    pub fn reasons(&self) -> Vec<&Ineligibility> {
        self.protocol.iter().chain(self.grid.iter()).collect()
    }
}

/// A cell's identity, as one comparable string.
fn cell_key(specimen: &str, k: usize) -> String {
    format!("{specimen} k={k}")
}

/// Check a run against the frozen protocol and the frozen grid.
///
/// Compares **identities and sets**, never counts alone: a run with thirty cells
/// carrying the wrong pairs, or the right pairs at the wrong span lengths, fails
/// [`Envelope::grid_complete`] exactly as a run with twenty-nine does.
pub fn envelope(run: &RunDescriptor<'_>) -> Envelope {
    let mut protocol = Vec::new();
    let mut grid = Vec::new();

    if run.alpha != ALPHA {
        protocol.push(Ineligibility::AlphaNotFrozen {
            found: run.alpha,
            expected: ALPHA,
        });
    }

    let derived = fewrs_budget(run.alpha).unwrap_or(BUDGET);
    if run.budget != BUDGET || run.budget != derived {
        protocol.push(Ineligibility::BudgetNotDerived {
            found: run.budget,
            expected: BUDGET,
        });
    }

    let every: Vec<&Cell> = run.controls.iter().chain(run.cells.iter()).collect();

    let expected_seeds = seed_range(BUDGET);
    let mut wrong_seeds: Vec<String> = every
        .iter()
        .filter(|cell| cell.seed_range != expected_seeds)
        .map(|cell| cell.seed_range.clone())
        .collect();
    wrong_seeds.sort();
    wrong_seeds.dedup();
    if !wrong_seeds.is_empty() {
        protocol.push(Ineligibility::SeedPrefixNotUsed {
            expected: expected_seeds,
            found: wrong_seeds,
        });
    }

    let mut wrong_nulls: Vec<String> = every
        .iter()
        .filter(|cell| cell.null_mode != NULL_ORDER_PERMUTATION)
        .map(|cell| cell.null_mode.to_owned())
        .collect();
    wrong_nulls.sort();
    wrong_nulls.dedup();
    if !wrong_nulls.is_empty() {
        protocol.push(Ineligibility::NullModeNotOrderPermutation { found: wrong_nulls });
    }

    let mut missing_controls = Vec::new();
    for label in EXPECTED_CONTROLS {
        for k in calibration::LADDER {
            if !run
                .controls
                .iter()
                .any(|cell| cell.specimen == label && cell.k == k)
            {
                missing_controls.push(cell_key(label, k));
            }
        }
    }
    if !missing_controls.is_empty() {
        protocol.push(Ineligibility::ControlsNotExecuted {
            missing: missing_controls,
        });
    }

    // --- the grid ---------------------------------------------------------

    let mut seen: Vec<String> = Vec::new();
    let mut duplicates: Vec<String> = Vec::new();
    for cell in run.cells {
        let key = cell_key(&cell.specimen, cell.k);
        if seen.contains(&key) {
            if !duplicates.contains(&key) {
                duplicates.push(key);
            }
        } else {
            seen.push(key);
        }
    }
    if !duplicates.is_empty() {
        grid.push(Ineligibility::DuplicateCells { duplicates });
    }

    let expected_keys: Vec<String> = SPRINT_19_ORDER_NULL_GRID
        .iter()
        .map(|row| cell_key(row.specimen, row.k))
        .collect();
    let missing: Vec<String> = expected_keys
        .iter()
        .filter(|key| !seen.contains(key))
        .cloned()
        .collect();
    let unexpected: Vec<String> = seen
        .iter()
        .filter(|key| !expected_keys.contains(key))
        .cloned()
        .collect();
    if !missing.is_empty() || !unexpected.is_empty() {
        grid.push(Ineligibility::CellSetMismatch {
            missing,
            unexpected,
        });
    }
    if seen.len() != SPRINT_19_ORDER_NULL_GRID.len() {
        grid.push(Ineligibility::CellCountNotThirty { found: seen.len() });
    }

    let unmatched: Vec<String> = run
        .cells
        .iter()
        .filter(|cell| cell.historical_tail.is_none())
        .map(|cell| cell_key(&cell.specimen, cell.k))
        .collect();
    if !unmatched.is_empty() {
        grid.push(Ineligibility::CellsUnmatchedToHistoricalGrid { cells: unmatched });
    }

    // Specimen identity, from the projected sequences rather than from the
    // cell labels, so a run that projected the wrong recordings is caught even
    // if it somehow produced right-looking pair labels.
    let found: Vec<&str> = run.specimens.iter().map(String::as_str).collect();
    let missing: Vec<String> = EXPECTED_SPECIMENS
        .iter()
        .filter(|name| !found.contains(name))
        .map(|name| (*name).to_owned())
        .collect();
    let unexpected: Vec<String> = found
        .iter()
        .filter(|name| !EXPECTED_SPECIMENS.contains(name))
        .map(|name| (*name).to_owned())
        .collect();
    if !missing.is_empty() || !unexpected.is_empty() || found.len() != EXPECTED_SPECIMENS.len() {
        grid.push(Ineligibility::SpecimenSetMismatch {
            missing,
            unexpected,
        });
    }

    Envelope { protocol, grid }
}

/// task:32 §PHASE 9's threshold: cells that must certify for a STRONG result.
pub const STRONG_THRESHOLD: usize = 15;

/// The preregistered classification, plus the outcome a run outside the frozen
/// envelope gets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Classification {
    /// The run is not the frozen assay. It is **not** compared against
    /// [`STRONG_THRESHOLD`], and its reasons say which conditions failed.
    DiagnosticUnclassified,
    /// A control failed its §PHASE 4 rule on a run that established the frozen
    /// protocol.
    Falsification,
    /// Both controls passed, the grid is the frozen 30, and at least
    /// [`STRONG_THRESHOLD`] cells certified.
    Strong,
    /// Both controls passed, the grid is the frozen 30, and fewer did.
    WeakMixed,
}

impl Classification {
    /// The label a report prints.
    pub fn label(self) -> &'static str {
        match self {
            Self::DiagnosticUnclassified => "DIAGNOSTIC / UNCLASSIFIED",
            Self::Falsification => "FALSIFICATION",
            Self::Strong => "STRONG",
            Self::WeakMixed => "WEAK / MIXED",
        }
    }

    /// Whether this is one of task:32's preregistered outcomes.
    pub fn is_preregistered(self) -> bool {
        !matches!(self, Self::DiagnosticUnclassified)
    }
}

/// Classify by §PHASE 9's precedence, gated on the envelope.
///
/// The order is load-bearing:
///
/// 1. **Protocol first.** A run that did not establish the frozen protocol gets
///    `DIAGNOSTIC / UNCLASSIFIED` and is never compared against the threshold.
/// 2. **Then the controls.** A run that *did* establish the protocol and whose
///    control failed is `FALSIFICATION` — the preregistered outcome — and its
///    empty grid is not held against it, because the frozen protocol stops the
///    observational stage when a control fails.
/// 3. **Then the grid.** Only a run whose controls passed *and* whose grid is
///    exactly the frozen 30 cells reaches the threshold.
/// 4. **Then the count**, whose two branches are complementary.
pub fn classify(envelope: &Envelope, controls_passed: bool, certified: usize) -> Classification {
    if !envelope.protocol_established() {
        Classification::DiagnosticUnclassified
    } else if !controls_passed {
        Classification::Falsification
    } else if !envelope.grid_complete() {
        Classification::DiagnosticUnclassified
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
