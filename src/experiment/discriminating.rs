//! **Disposable research experiment.** sprint:18, task:28.
//!
//! A gauntlet built so that S0 `rarity_of_agreements` and R1 `pooled sum` make
//! *different* numerical predictions. sprint:17 left R1 the one eligible
//! candidate on weak evidence: the two were numerically identical at 65 of 68
//! points of the existing gauntlet, because nine of its ten families give both
//! recordings the same marginals. Those families are regression evidence now.
//!
//! Every construction here except [`f0_shared_marginal_control`] separates the
//! two recordings' marginals, and every predicted value was computed
//! analytically in task:28 §PHASE 3 before this file existed. The code asserts
//! the closed forms rather than reporting whatever it happens to produce.
//!
//! # Three verdict kinds, and why LIMITATION is not a euphemism
//!
//! task:28 §PHASE 3 assigns each family a [`Rule`]. A **PASS** family is one the
//! contract adjudicates. A **LIMITATION** family is one where R1's behaviour is
//! a real consequence of pooling that no preregistered clause settles — F4's
//! blindness to the direction of a marginal imbalance, F5's unequal influence,
//! F6a's dependence on specimen length, F8's blindness to dependence. Calling
//! those failures would mean inventing a clause after seeing the result;
//! calling them passes would mean hiding what pooling costs. They are recorded,
//! and §PHASE 6 draws the verdict from them.

use std::collections::BTreeMap;

use serde::Serialize;

use super::identifiability::Observation;
use super::repair::{Candidate, candidate};

/// Tolerance for the closed forms. Every prediction in task:28 §PHASE 3 is an
/// exact expression, so agreement is expected to floating-point noise only.
pub const EXACT: f64 = 1e-12;

/// What a family's outcome is allowed to mean.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Rule {
    /// The contract adjudicates this; failing it is a rejection.
    Pass,
    /// A real consequence of pooling that no preregistered clause settles.
    /// Confirming it bounds what the score may be claimed to mean.
    Limitation,
}

/// Whether a family's preregistered rule was met.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Outcome {
    /// A `Pass` family whose rule held, or a `Limitation` family confirmed.
    Held,
    /// A `Pass` family whose rule failed. Only this rejects.
    Broken,
}

/// One swept point: what was computed under each statistic, and whether it
/// matched the value task:28 predicted before the code existed.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Point {
    /// The swept parameters.
    pub params: String,
    /// `S0`'s value.
    pub frozen: f64,
    /// `R1`'s value.
    pub pooled: f64,
    /// What §PHASE 3 predicted for the quantity this family measures.
    pub predicted: f64,
    /// What was observed for that quantity.
    pub observed: f64,
    /// Whether observation matched prediction to [`EXACT`].
    pub matched: bool,
}

/// Everything one family produced.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FamilyResult {
    /// Short label.
    pub name: &'static str,
    /// How the fixture is built.
    pub construction: &'static str,
    /// The exact quantity the code computes, in decision:7's sense.
    pub quantity: &'static str,
    /// What the intended semantics say should happen.
    pub semantic_expectation: &'static str,
    /// Whether this family adjudicates or merely bounds.
    pub rule: Rule,
    /// Whether it discriminates S0 from R1 at all.
    pub discriminating: bool,
    /// The swept points.
    pub points: Vec<Point>,
    /// Whether the rule was met.
    pub outcome: Outcome,
    /// The first point that failed, when one did.
    pub boundary: Option<String>,
    /// A mechanical precondition the fixture had to satisfy for the prediction
    /// to be testable at all — task:28 §PHASE 4 M4 and M7.
    pub precondition: Option<&'static str>,
    /// Whether that precondition held.
    pub precondition_held: bool,
}

fn s0() -> &'static Candidate {
    candidate("S0 rarity_of_agreements").expect("the frozen incumbent must remain")
}

fn r1() -> &'static Candidate {
    candidate("R1 pooled sum").expect("the candidate under commission must remain")
}

/// Build an observation with independent control of both recordings' marginals.
///
/// `agreeing` gives each agreeing position `(mark, count in A, count in B)`.
/// Nothing here is shared with [`super::adversarial`] or [`super::repair`]:
/// task:28 §PHASE 7 forbids modifying a prior fixture, and a shared constructor
/// is a modification waiting to happen.
fn observation(
    agreeing: &[(&str, usize, usize)],
    disagreements: usize,
    a_total: usize,
    b_total: usize,
) -> Observation {
    let mut a = Vec::new();
    let mut b = Vec::new();
    let mut a_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut b_counts: BTreeMap<String, usize> = BTreeMap::new();
    for (mark, in_a, in_b) in agreeing {
        a.push((*mark).to_owned());
        b.push((*mark).to_owned());
        a_counts.insert((*mark).to_owned(), *in_a);
        b_counts.insert((*mark).to_owned(), *in_b);
    }
    for index in 0..disagreements {
        a.push(format!("dl{index}"));
        b.push(format!("dr{index}"));
        a_counts.insert(format!("dl{index}"), 1);
        b_counts.insert(format!("dr{index}"), 1);
    }
    Observation {
        a,
        b,
        a_counts,
        b_counts,
        a_total,
        b_total,
    }
}

fn swapped(observation: &Observation) -> Observation {
    Observation {
        a: observation.b.clone(),
        b: observation.a.clone(),
        a_counts: observation.b_counts.clone(),
        b_counts: observation.a_counts.clone(),
        a_total: observation.b_total,
        b_total: observation.a_total,
    }
}

fn frozen_at(observation: &Observation) -> f64 {
    (s0().score)(observation).unwrap_or(f64::NAN)
}

fn pooled_at(observation: &Observation) -> f64 {
    (r1().score)(observation).unwrap_or(f64::NAN)
}

/// Assemble a family, deciding its outcome from its points and precondition.
#[allow(clippy::too_many_arguments)]
fn family(
    name: &'static str,
    construction: &'static str,
    quantity: &'static str,
    semantic_expectation: &'static str,
    rule: Rule,
    discriminating: bool,
    precondition: Option<&'static str>,
    precondition_held: bool,
    points: Vec<Point>,
) -> FamilyResult {
    let all_matched = points.iter().all(|point| point.matched);
    let boundary = points
        .iter()
        .find(|point| !point.matched)
        .map(|point| point.params.clone());
    let outcome = if all_matched && precondition_held {
        Outcome::Held
    } else {
        Outcome::Broken
    };
    FamilyResult {
        name,
        construction,
        quantity,
        semantic_expectation,
        rule,
        discriminating,
        points,
        outcome,
        boundary,
        precondition,
        precondition_held,
    }
}

fn point(params: String, frozen: f64, pooled: f64, predicted: f64, observed: f64) -> Point {
    Point {
        params,
        frozen,
        pooled,
        predicted,
        observed,
        matched: (predicted - observed).abs() <= EXACT,
    }
}

// ---------------------------------------------------------------------------
// F0 — shared-marginal control
// ---------------------------------------------------------------------------

/// The only family here where the two recordings share marginals. Establishes
/// that a later difference comes from the marginals rather than the harness.
pub fn f0_shared_marginal_control() -> FamilyResult {
    let mut points = Vec::new();
    for (count, total, disagreements) in [(1usize, 1_000usize, 3usize), (50, 1_000, 2), (7, 44, 0)]
    {
        let case = observation(&[("m", count, count)], disagreements, total, total);
        let frozen = frozen_at(&case);
        let pooled = pooled_at(&case);
        points.push(point(
            format!("c={count} N={total}"),
            frozen,
            pooled,
            0.0,
            frozen - pooled,
        ));
    }
    family(
        "F0 shared-marginal control",
        "ĉ_B = ĉ_A and N_B = N_A, the configuration nine of the ten old families use",
        "S0 − R1",
        "exactly zero: with shared marginals p̂ = ĉ_A/N_A identically",
        Rule::Pass,
        false,
        None,
        true,
        points,
    )
}

// ---------------------------------------------------------------------------
// F1 — argument reversal
// ---------------------------------------------------------------------------

/// R1's exchange invariance, and the closed form for S0's failure of it.
pub fn f1_argument_reversal() -> FamilyResult {
    let mut points = Vec::new();
    let cases = [
        (
            vec![("m", 1usize, 500usize)],
            3usize,
            1_000usize,
            1_000usize,
        ),
        (vec![("m", 2, 400), ("n", 50, 3)], 2, 1_000, 4_000),
        (vec![("m", 10, 1)], 0, 500, 20_000),
    ];
    for (index, (agreeing, disagreements, a_total, b_total)) in cases.iter().enumerate() {
        let forward = observation(agreeing, *disagreements, *a_total, *b_total);
        let backward = swapped(&forward);

        // The closed form task:28 §PHASE 3 gives for S0's asymmetry.
        let expected_frozen_delta: f64 = agreeing
            .iter()
            .map(|(_, in_a, in_b)| {
                ((*in_b as f64 / *b_total as f64) / (*in_a as f64 / *a_total as f64)).ln()
            })
            .sum::<f64>()
            .abs();

        let frozen_delta = (frozen_at(&forward) - frozen_at(&backward)).abs();
        let pooled_delta = (pooled_at(&forward) - pooled_at(&backward)).abs();

        // Both halves of the prediction must hold, so the point matches only if
        // R1's delta is zero *and* S0's matches its closed form.
        let observed = pooled_delta + (frozen_delta - expected_frozen_delta).abs();
        points.push(point(
            format!("case {index}"),
            frozen_delta,
            pooled_delta,
            0.0,
            observed,
        ));
    }
    family(
        "F1 argument reversal",
        "asymmetric marginals, spans fixed, both recordings' roles exchanged",
        "|S(A,B) − S(B,A)| for each, against Σ ln((ĉ_B/N_B)/(ĉ_A/N_A)) for S0",
        "R1 exactly zero; S0 equal to the closed form. Invariance check, weak alone",
        Rule::Pass,
        true,
        None,
        true,
        points,
    )
}

// ---------------------------------------------------------------------------
// F2 / F3 — one marginal moved at a time
// ---------------------------------------------------------------------------

/// A's marginal moves; both statistics should see it, R1 damped.
pub fn f2_different_a_marginal() -> FamilyResult {
    let mut points = Vec::new();
    let baseline = observation(&[("m", 10, 10)], 3, 1_000, 1_000);
    let (base_frozen, base_pooled) = (frozen_at(&baseline), pooled_at(&baseline));
    let mut previous_pooled = f64::INFINITY;
    let mut monotone = true;

    for count in [10usize, 100, 500] {
        let case = observation(&[("m", count, 10)], 3, 1_000, 1_000);
        let frozen = frozen_at(&case);
        let pooled = pooled_at(&case);
        let predicted_frozen = -((count as f64 / 10.0).ln());
        let predicted_pooled = -(((count as f64 + 10.0) / 20.0).ln());
        let observed = ((frozen - base_frozen) - predicted_frozen).abs()
            + ((pooled - base_pooled) - predicted_pooled).abs();
        if pooled > previous_pooled {
            monotone = false;
        }
        previous_pooled = pooled;
        points.push(point(format!("ĉ_A={count}"), frozen, pooled, 0.0, observed));
    }
    family(
        "F2 different A marginal",
        "ĉ_A(m) ∈ {10, 100, 500}, ĉ_B(m) = 10, N_A = N_B = 1000, spans fixed",
        "ΔS0 against −ln(ĉ_A′/10) and ΔR1 against −ln((ĉ_A′+10)/20)",
        "both fall as the mark gets commoner in A; R1 damped, supplying half the pool",
        Rule::Pass,
        true,
        Some("R1 strictly decreasing across the sweep"),
        monotone,
        points,
    )
}

/// B's marginal moves. **The family that carries the round**: S0 cannot see it.
pub fn f3_different_b_marginal() -> FamilyResult {
    let mut points = Vec::new();
    let mut previous_pooled = f64::INFINITY;
    let mut monotone = true;

    for count in [10usize, 100, 500] {
        let case = observation(&[("m", 10, count)], 3, 1_000, 1_000);
        let frozen = frozen_at(&case);
        let pooled = pooled_at(&case);

        // S0 must be flat at ln 100 and R1 must hit the predicted value.
        let predicted_pooled = -(((10.0 + count as f64) / 2_000.0).ln());
        let observed = (frozen - (100.0f64).ln()).abs() + (pooled - predicted_pooled).abs();
        if pooled > previous_pooled {
            monotone = false;
        }
        previous_pooled = pooled;
        points.push(point(format!("ĉ_B={count}"), frozen, pooled, 0.0, observed));
    }
    family(
        "F3 different B marginal",
        "ĉ_B(m) ∈ {10, 100, 500}, ĉ_A(m) = 10, N_A = N_B = 1000 held equal per M3",
        "ΔS0, which must be 0, and R1 against −ln((10+ĉ_B)/2000)",
        "a mark ubiquitous in B makes agreement easy and evidence must fall; S0 cannot see it",
        Rule::Pass,
        true,
        Some("R1 strictly decreasing across the sweep"),
        monotone,
        points,
    )
}

// ---------------------------------------------------------------------------
// F4 — balanced countervailing marginals
// ---------------------------------------------------------------------------

/// Pooling discards the *direction* of a marginal imbalance. Contested, and
/// deliberately not resolved by fiat.
pub fn f4_balanced_countervailing() -> FamilyResult {
    let mut points = Vec::new();
    let mut pooled_values = Vec::new();
    let mut frozen_values = Vec::new();
    let configurations = [(10usize, 500usize), (255, 255), (500, 10)];

    // task:28 §PHASE 4 M4: both sums must be fixed or the prediction is untestable.
    let precondition_held = configurations.iter().all(|(in_a, in_b)| in_a + in_b == 510);

    for (in_a, in_b) in configurations {
        let case = observation(&[("m", in_a, in_b)], 3, 1_000, 1_000);
        let frozen = frozen_at(&case);
        let pooled = pooled_at(&case);
        frozen_values.push(frozen);
        pooled_values.push(pooled);
        points.push(point(
            format!("(ĉ_A,ĉ_B)=({in_a},{in_b})"),
            frozen,
            pooled,
            -(510.0f64 / 2_000.0).ln(),
            pooled,
        ));
    }

    let pooled_range = spread(&pooled_values);
    let frozen_range = spread(&frozen_values);
    points.push(point(
        "range across the three".to_owned(),
        frozen_range,
        pooled_range,
        0.0,
        pooled_range + if frozen_range > 1.0 { 0.0 } else { 1.0 },
    ));

    family(
        "F4 balanced countervailing marginals",
        "ĉ_A + ĉ_B = 510 and N_A + N_B = 2000 both held fixed; the split varies",
        "max R1 − min R1 across the three splits, and the same for S0",
        "contested: identical under the shared-source model, but agreement is cheap \
         when B supplies the mark constantly",
        Rule::Limitation,
        true,
        Some("ĉ_A + ĉ_B = 510 at every configuration"),
        precondition_held,
        points,
    )
}

fn spread(values: &[f64]) -> f64 {
    let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let min = values.iter().copied().fold(f64::INFINITY, f64::min);
    max - min
}

// ---------------------------------------------------------------------------
// F5 — corpus-size imbalance
// ---------------------------------------------------------------------------

/// R1 is exchange-invariant in *value* and not in *influence*: the pooled
/// estimate is the length-weighted mean of the two relative frequencies.
pub fn f5_corpus_size_imbalance() -> FamilyResult {
    let mut points = Vec::new();
    let mut pooled_values = Vec::new();
    const F_A: f64 = 0.01;
    const F_B: f64 = 0.50;

    for (a_total, b_total) in [
        (1_000usize, 1_000usize),
        (100, 10_000),
        (10_000, 100),
        (100, 100_000),
    ] {
        let in_a = (a_total as f64 * F_A).round() as usize;
        let in_b = (b_total as f64 * F_B).round() as usize;
        let case = observation(&[("m", in_a, in_b)], 3, a_total, b_total);
        let pooled = pooled_at(&case);
        pooled_values.push(pooled);

        // The weighted-mean form, computed independently of the statistic.
        let weighted = (a_total as f64 * F_A + b_total as f64 * F_B) / (a_total + b_total) as f64;
        let predicted = -weighted.ln();
        points.push(point(
            format!("N_A={a_total} N_B={b_total}"),
            frozen_at(&case),
            pooled,
            predicted,
            pooled,
        ));
    }

    // The limitation is confirmed only if the imbalance actually moves R1.
    let moved = (pooled_values[0] - pooled_values[3]).abs() > 1.0;
    points.push(point(
        "movement, balanced vs 100:100000".to_owned(),
        f64::NAN,
        (pooled_values[0] - pooled_values[3]).abs(),
        1.0,
        if moved { 1.0 } else { f64::NAN },
    ));

    family(
        "F5 corpus-size imbalance",
        "f_A = 0.01 and f_B = 0.50 held fixed while N_A and N_B vary over two orders",
        "p̂ against (N_A·f_A + N_B·f_B)/(N_A+N_B), and R1's movement across the sweep",
        "the longer recording dominates the pooled estimate by exactly N_A : N_B; \
         exchange invariance is a statement about value, not influence",
        Rule::Limitation,
        true,
        None,
        true,
        points,
    )
}

// ---------------------------------------------------------------------------
// F6a / F6b — one-sided background
// ---------------------------------------------------------------------------

/// Background appended to A only. Split from F6b by task:28 §PHASE 4 M5,
/// because both statistics move here and the family discriminates nothing.
pub fn f6a_duplicate_a_background() -> FamilyResult {
    one_sided_background(
        "F6a duplicate A background only",
        "D irrelevant events appended to A; the agreeing marks' counts untouched; k = 2",
        "ΔS0 against k·ln((N_A+D)/N_A) and ΔR1 against k·ln((N+D)/N)",
        "a mark appearing as often in a longer recording is genuinely rarer, so a rise is \
         defensible — but scores are not comparable across recordings of different length",
        Rule::Limitation,
        false,
        true,
    )
}

/// The same background appended to B only. S0 is blind; R1 is not.
pub fn f6b_duplicate_b_background() -> FamilyResult {
    one_sided_background(
        "F6b duplicate B background only",
        "the same D appended to B instead; the agreeing marks' counts untouched; k = 2",
        "ΔS0, which must be 0, and ΔR1 against k·ln((N+D)/N)",
        "S0's blindness to B is a blindness to specimen size on one side only",
        Rule::Pass,
        true,
        false,
    )
}

fn one_sided_background(
    name: &'static str,
    construction: &'static str,
    quantity: &'static str,
    semantic_expectation: &'static str,
    rule: Rule,
    discriminating: bool,
    on_a: bool,
) -> FamilyResult {
    const BASE: usize = 1_000;
    let agreeing = [("m", 20usize, 20usize), ("n", 20, 20)];
    let baseline = observation(&agreeing, 2, BASE, BASE);
    let (base_frozen, base_pooled) = (frozen_at(&baseline), pooled_at(&baseline));
    let mut points = Vec::new();

    for added in [1_000usize, 5_000, 20_000] {
        let case = if on_a {
            observation(&agreeing, 2, BASE + added, BASE)
        } else {
            observation(&agreeing, 2, BASE, BASE + added)
        };
        let frozen_delta = frozen_at(&case) - base_frozen;
        let pooled_delta = pooled_at(&case) - base_pooled;

        let predicted_frozen = if on_a {
            2.0 * (((BASE + added) as f64) / BASE as f64).ln()
        } else {
            0.0
        };
        let predicted_pooled = 2.0 * (((2 * BASE + added) as f64) / (2 * BASE) as f64).ln();
        let observed =
            (frozen_delta - predicted_frozen).abs() + (pooled_delta - predicted_pooled).abs();

        points.push(point(
            format!("D={added}"),
            frozen_delta,
            pooled_delta,
            0.0,
            observed,
        ));
    }

    family(
        name,
        construction,
        quantity,
        semantic_expectation,
        rule,
        discriminating,
        None,
        true,
        points,
    )
}

// ---------------------------------------------------------------------------
// F7 — rare few versus common many
// ---------------------------------------------------------------------------

/// The crossover surface, scored against the analytic ordering.
///
/// task:28 §PHASE 4 M1 struck the "more agreements must win" phrasing this
/// family would otherwise have carried: PHASE 1 proves that criterion is
/// unsatisfiable by any statistic that weights rarity at all.
pub fn f7_rare_few_versus_common_many() -> FamilyResult {
    const TOTAL: usize = 100_000;
    const P_COMMON: f64 = 0.25;
    let mut points = Vec::new();
    let mut crossovers = Vec::new();

    for p_rare in [0.001f64, 0.01, 0.05] {
        // Split each count across the two recordings so the pooled probability
        // is exactly the swept value.
        let rare_count = (p_rare * TOTAL as f64).round() as usize;
        let common_count = (P_COMMON * TOTAL as f64).round() as usize;
        let mut mismatches = 0usize;

        for k_rare in 1..=8usize {
            for k_common in 1..=8usize {
                let rare_marks: Vec<(String, usize, usize)> = (0..k_rare)
                    .map(|i| (format!("r{i}"), rare_count, rare_count))
                    .collect();
                let common_marks: Vec<(String, usize, usize)> = (0..k_common)
                    .map(|i| (format!("c{i}"), common_count, common_count))
                    .collect();
                let rare_refs: Vec<(&str, usize, usize)> = rare_marks
                    .iter()
                    .map(|(mark, in_a, in_b)| (mark.as_str(), *in_a, *in_b))
                    .collect();
                let common_refs: Vec<(&str, usize, usize)> = common_marks
                    .iter()
                    .map(|(mark, in_a, in_b)| (mark.as_str(), *in_a, *in_b))
                    .collect();
                let rare_case = observation(&rare_refs, 0, TOTAL, TOTAL);
                let common_case = observation(&common_refs, 0, TOTAL, TOTAL);

                let observed_sign =
                    (pooled_at(&rare_case) - pooled_at(&common_case)).partial_cmp(&0.0);
                let analytic =
                    k_rare as f64 * (1.0 / p_rare).ln() - k_common as f64 * (1.0 / P_COMMON).ln();
                let analytic_sign = analytic.partial_cmp(&0.0);
                if observed_sign != analytic_sign {
                    mismatches += 1;
                }
            }
        }

        let crossover = (1.0f64 / P_COMMON).ln() / (1.0f64 / p_rare).ln();
        crossovers.push(crossover);
        points.push(point(
            format!("p_rare={p_rare}"),
            f64::NAN,
            crossover,
            0.0,
            mismatches as f64,
        ));
    }

    family(
        "F7 rare few versus common many",
        "k_rare ∈ 1..8 at pooled p ∈ {0.001, 0.01, 0.05} against k_common ∈ 1..8 at p = 0.25",
        "sign(R1(rare) − R1(common)) against sign(k_r·ln(1/p_r) − k_c·ln(1/p_c)) at all 64 points",
        "crossings are required by PHASE 1, not tolerated; the criterion is agreement with \
         the analytic ordering",
        Rule::Pass,
        false,
        None,
        true,
        points,
    )
}

// ---------------------------------------------------------------------------
// F8 — dependent repetition
// ---------------------------------------------------------------------------

/// The independence assumption, exhibited. One rare mark repeated by a planted
/// deterministic run against five independently planted rare marks.
pub fn f8_dependent_repetition() -> FamilyResult {
    const TOTAL: usize = 1_000;
    const COUNT: usize = 10;

    // X: the same mark at all five agreeing positions — a deterministic run.
    let dependent = {
        let mut case = observation(&[("m", COUNT, COUNT)], 0, TOTAL, TOTAL);
        for _ in 0..4 {
            case.a.push("m".to_owned());
            case.b.push("m".to_owned());
        }
        case
    };
    // Y: five distinct marks, each planted independently at the same count.
    let independent = observation(
        &[
            ("m0", COUNT, COUNT),
            ("m1", COUNT, COUNT),
            ("m2", COUNT, COUNT),
            ("m3", COUNT, COUNT),
            ("m4", COUNT, COUNT),
        ],
        0,
        TOTAL,
        TOTAL,
    );

    // task:28 §PHASE 4 M7: equal pooled probabilities, or an equality of scores
    // would be arithmetic coincidence rather than the intended blindness.
    let precondition_held = dependent.a.len() == independent.a.len()
        && dependent.agreeing().len() == independent.agreeing().len();

    let dependent_score = pooled_at(&dependent);
    let independent_score = pooled_at(&independent);
    let predicted = 5.0 * -((2.0 * COUNT as f64) / (2.0 * TOTAL as f64)).ln();

    let points = vec![
        point(
            "one mark repeated five times".to_owned(),
            frozen_at(&dependent),
            dependent_score,
            predicted,
            dependent_score,
        ),
        point(
            "five distinct marks".to_owned(),
            frozen_at(&independent),
            independent_score,
            predicted,
            independent_score,
        ),
        point(
            "difference".to_owned(),
            f64::NAN,
            dependent_score - independent_score,
            0.0,
            dependent_score - independent_score,
        ),
    ];

    family(
        "F8 dependent repetition",
        "one rare mark repeated five times by a planted run, against five independently \
         planted rare marks, every mark at the same count",
        "R1(dependent) − R1(independent), with equal span length and agreement count asserted",
        "the two processes differ fivefold in independent events and R1 cannot tell them \
         apart, because PHASE 2's i.i.d. assumption is false here",
        Rule::Limitation,
        false,
        Some("equal span length and equal agreement count on both sides"),
        precondition_held,
        points,
    )
}

/// Every fresh family, in task:28 §PHASE 3's order.
pub fn families() -> Vec<FamilyResult> {
    vec![
        f0_shared_marginal_control(),
        f1_argument_reversal(),
        f2_different_a_marginal(),
        f3_different_b_marginal(),
        f4_balanced_countervailing(),
        f5_corpus_size_imbalance(),
        f6a_duplicate_a_background(),
        f6b_duplicate_b_background(),
        f7_rare_few_versus_common_many(),
        f8_dependent_repetition(),
    ]
}

/// task:28 §PHASE 6's partition, decided by precedence so that it tiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Verdict {
    /// Every PASS family held and no confirmed limitation bounds the claim.
    CoherentSurvivor,
    /// Every PASS family held; at least one confirmed limitation bounds it.
    UsefulHeuristic,
    /// A PASS family failed.
    Reject,
}

/// Apply the partition. Precedence: reject, then survivor, then heuristic.
pub fn verdict(families: &[FamilyResult]) -> Verdict {
    let broken_pass = families
        .iter()
        .any(|family| family.rule == Rule::Pass && family.outcome == Outcome::Broken);
    if broken_pass {
        return Verdict::Reject;
    }
    let confirmed_limitation = families
        .iter()
        .any(|family| family.rule == Rule::Limitation && family.outcome == Outcome::Held);
    if confirmed_limitation {
        Verdict::UsefulHeuristic
    } else {
        Verdict::CoherentSurvivor
    }
}
