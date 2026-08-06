//! An adversarial gauntlet built against inverse-frequency weighting.
//!
//! **Disposable.** sprint:15, task:25. A commissioning test of one candidate,
//! not a search: `rarity_of_agreements` is frozen for the whole round, is not
//! adopted by it, and is not repaired if it fails.
//!
//! # Why a second gauntlet
//!
//! sprint:12's gauntlet was constructed against the **permutation null's**
//! failure modes, and sprint:14 found `rarity_of_agreements` by scoring ten
//! preregistered functions against it. A statistic selected on a suite has not
//! been validated by it. So sprint:12's gauntlet runs this round as a
//! **regression suite** — it shows nothing broke — and these families are the
//! fresh evidence, built against the mechanism of the statistic under test.
//!
//! # How specimens are built, and why differently
//!
//! `rarity_of_agreements` is a function of the representation sprint:14
//! formalized and of nothing else, so a case is constructed **directly as an
//! [`Observation`]** rather than generated as a recording and projected. That
//! removes the generator as a confound — sprint:12 lost a family to one — and
//! makes every case readable in a line. The pipeline is not exercised here
//! because the pipeline is not what is on trial, and sprint:12's gauntlet still
//! runs through it.
//!
//! # The rule
//!
//! Each family is a set of comparisons with an expected ordering, evaluated at a
//! **nominal point** and over a **sweep**. PASS when the ordering holds at the
//! nominal point and everywhere in the sweep; MIXED when it holds at nominal and
//! fails somewhere; FAIL when it fails at nominal. The constructions are
//! deterministic, so one violation anywhere is a real violation.

use std::collections::BTreeMap;

use serde::Serialize;

use super::identifiability::{Observation, SCORERS};

/// The statistic under test, by name in [`SCORERS`].
pub const UNDER_TEST: &str = "rarity_of_agreements";

/// One point of a family's sweep: two candidates, their scores, and whether the
/// preregistered ordering held.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Point {
    /// The swept parameters, as `name=value` for reporting.
    pub params: String,
    /// Whether this is the family's nominal point.
    pub nominal: bool,
    /// Score of the candidate expected to be **lower**.
    pub weaker: f64,
    /// Score of the candidate expected to be **higher**.
    pub stronger: f64,
    /// Whether `stronger > weaker` held, or — for an invariance family —
    /// whether the invariant held.
    pub holds: bool,
}

/// Whether a family passed, and how.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    /// Held at nominal and everywhere in the sweep.
    Pass,
    /// Held at nominal, failed somewhere in the sweep.
    Mixed,
    /// Failed at the nominal point.
    Fail,
}

impl Verdict {
    /// A stable label for output.
    pub fn label(self) -> &'static str {
        match self {
            Verdict::Pass => "PASS",
            Verdict::Mixed => "MIXED",
            Verdict::Fail => "FAIL",
        }
    }
}

/// Everything one adversarial family produced.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FamilyResult {
    /// Short name, as preregistered.
    pub name: &'static str,
    /// How the two candidates are built.
    pub construction: &'static str,
    /// The ordering or invariant expected of them.
    pub invariant: &'static str,
    /// Why the statistic might fail it.
    pub mechanism: &'static str,
    /// What task:25 predicted, before running.
    pub predicted: Verdict,
    /// What happened.
    pub verdict: Verdict,
    /// Every swept point.
    pub points: Vec<Point>,
    /// The first point at which the ordering broke, if any — the phase
    /// transition, reduced to one line.
    pub boundary: Option<String>,
}

/// Build an observation with the given agreeing marks and disagreements.
///
/// `agreeing` gives each agreeing position's count **in A**; `b_counts_override`
/// lets a family make a mark's count in B differ, which the statistic under test
/// famously does not read.
fn case(
    agreeing: &[usize],
    disagreements: usize,
    a_total: usize,
    b_counts_override: Option<&[usize]>,
    b_total: usize,
) -> Observation {
    let mut a = Vec::new();
    let mut b = Vec::new();
    let mut a_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut b_counts: BTreeMap<String, usize> = BTreeMap::new();

    for (index, count) in agreeing.iter().enumerate() {
        let mark = format!("m{index}");
        a.push(mark.clone());
        b.push(mark.clone());
        a_counts.insert(mark.clone(), *count);
        let b_count = b_counts_override
            .and_then(|counts| counts.get(index).copied())
            .unwrap_or(*count);
        b_counts.insert(mark, b_count);
    }
    for index in 0..disagreements {
        let left = format!("dl{index}");
        let right = format!("dr{index}");
        a.push(left.clone());
        b.push(right.clone());
        a_counts.insert(left, 1);
        b_counts.insert(right, 1);
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

/// The statistic every family is scored with.
///
/// sprint:17 parameterized this. **No construction, sweep, invariant or
/// expectation below changed** — task:27 §J forbids modifying a prior family,
/// and a family that moved when a candidate needed it to would be worthless as
/// commissioning evidence. Only the choice of what does the arithmetic is new.
pub type Stat = fn(&Observation) -> Option<f64>;

/// The frozen statistic these families were built against, for callers that
/// want sprint:15's original run. Panics only if the preregistered name is ever
/// removed from [`SCORERS`], which a test forbids.
pub fn under_test() -> Stat {
    SCORERS
        .iter()
        .find(|scorer| scorer.name == UNDER_TEST)
        .expect("the statistic under test must remain in the preregistered family")
        .score
}

fn score(stat: Stat, observation: &Observation) -> f64 {
    (stat)(observation).unwrap_or(f64::NAN)
}

/// Assemble a family from its swept comparisons.
fn family(
    name: &'static str,
    construction: &'static str,
    invariant: &'static str,
    mechanism: &'static str,
    predicted: Verdict,
    points: Vec<Point>,
) -> FamilyResult {
    let nominal_holds = points
        .iter()
        .find(|point| point.nominal)
        .map(|point| point.holds)
        .unwrap_or(false);
    let all_hold = points.iter().all(|point| point.holds);
    let verdict = if !nominal_holds {
        Verdict::Fail
    } else if all_hold {
        Verdict::Pass
    } else {
        Verdict::Mixed
    };
    let boundary = points
        .iter()
        .find(|point| !point.holds)
        .map(|point| point.params.clone());
    FamilyResult {
        name,
        construction,
        invariant,
        mechanism,
        predicted,
        verdict,
        points,
        boundary,
    }
}

fn point(params: String, nominal: bool, weaker: f64, stronger: f64) -> Point {
    Point {
        params,
        nominal,
        weaker,
        stronger,
        holds: stronger > weaker,
    }
}

/// AG1 — a lone accidental agreement on a singleton against a repeated motif.
fn ag1(stat: Stat) -> FamilyResult {
    let mut points = Vec::new();
    for total in [100usize, 1_000, 10_000, 100_000, 1_000_000] {
        for common in [10usize, 50, 200] {
            let lone = score(stat, &case(&[1], 3, total, None, total));
            let motif = score(stat, &case(&[common; 4], 0, total, None, total));
            points.push(point(
                format!("N={total} c={common}"),
                total == 1_000 && common == 50,
                lone,
                motif,
            ));
        }
    }
    family(
        "AG1 singleton vs motif",
        "X = one agreement on a count-1 mark; Y = four agreements on marks of count c",
        "Y > X: a substantially stronger repeated motif beats a lone accidental agreement",
        "X scores ln N, unbounded in corpus size; Y scores 4·ln(N/c) and is not",
        Verdict::Mixed,
        points,
    )
}

/// AG2 — does a single agreement acquire unbounded dominance as a mark's
/// frequency approaches zero?
fn ag2(stat: Stat) -> FamilyResult {
    let mut points = Vec::new();
    for total in [100usize, 1_000, 10_000, 100_000, 1_000_000] {
        let lone = score(stat, &case(&[1], 3, total, None, total));
        // Four agreements at a fixed relative frequency of 0.05.
        let held = (total / 20).max(1);
        let motif = score(stat, &case(&[held; 4], 0, total, None, total));
        points.push(point(format!("N={total}"), total == 1_000, lone, motif));
    }
    family(
        "AG2 rarity explosion",
        "X = one agreement on a count-1 mark; Y = four agreements at fixed frequency 0.05",
        "X < Y for every N up to a million: one weak agreement must not dominate without bound",
        "−ln(1/N) = ln N grows without limit while Y is 4·ln(1/p) and constant in N",
        Verdict::Fail,
        points,
    )
}

/// AG3 — rarity that does not agree must contribute nothing.
fn ag3(stat: Stat) -> FamilyResult {
    let mut points = Vec::new();
    for total in [1_000usize, 100_000] {
        for common in [20usize, 100] {
            // X carries a count-1 mark at a *disagreeing* position; Y does not
            // carry it at all. Both agree on the same two common marks.
            let with_rare = score(stat, &case(&[common, common], 1, total, None, total));
            let without = score(stat, &case(&[common, common], 0, total, None, total));
            points.push(Point {
                params: format!("N={total} c={common}"),
                nominal: total == 1_000 && common == 20,
                weaker: with_rare,
                stronger: without,
                // The invariant is equality, not an ordering.
                holds: (with_rare - without).abs() < 1e-12,
            });
        }
    }
    family(
        "AG3 rare disagreement",
        "X carries a count-1 mark at a disagreeing position; Y does not carry it at all",
        "X = Y exactly: rarity that does not agree contributes nothing",
        "a statistic summing span rarity rather than agreement rarity would reward X",
        Verdict::Pass,
        points,
    )
}

/// AG3b — the statistic never reads B's marginals.
fn ag3b(stat: Stat) -> FamilyResult {
    let mut points = Vec::new();
    for b_count in [1usize, 10, 100, 500] {
        let total = 1_000usize;
        let ubiquitous_in_b = score(stat, &case(&[1], 3, total, Some(&[b_count]), total));
        let rare_in_both = score(stat, &case(&[1], 3, total, Some(&[1]), total));
        points.push(Point {
            params: format!("count_B={b_count}"),
            nominal: b_count == 500,
            weaker: ubiquitous_in_b,
            stronger: rare_in_both,
            // Strict, because the invariant is that they should differ.
            holds: rare_in_both > ubiquitous_in_b,
        });
    }
    family(
        "AG3b one-sided rarity",
        "X = agreement on a mark of count 1 in A and many in B; Y = count 1 in both",
        "Y > X: an agreement on a mark ubiquitous in B is easy and must count for less",
        "the statistic reads a_counts and a_total only, never B's",
        Verdict::Fail,
        points,
    )
}

/// AG4 — a repeated figure of common marks must stay recoverable.
fn ag4(stat: Stat) -> FamilyResult {
    let mut points = Vec::new();
    for total in [100usize, 1_000, 10_000, 100_000, 1_000_000] {
        for percent in [10usize, 20, 35] {
            let held = (total * percent / 100).max(1);
            let structural = score(stat, &case(&[held; 6], 0, total, None, total));
            let lone_rare = score(stat, &case(&[1], 5, total, None, total));
            points.push(point(
                format!("N={total} p={percent}%"),
                total == 1_000 && percent == 20,
                lone_rare,
                structural,
            ));
        }
    }
    family(
        "AG4 common but structural",
        "X = six agreements on marks of frequency p; Y = one agreement on a count-1 mark",
        "X > Y: a repeated figure of common marks stays recoverable; rarity is not motif-ness",
        "X is fixed at 6·ln(1/p) while Y is ln N and grows",
        Verdict::Mixed,
        points,
    )
}

/// AG5 — appending unrelated marks must not reorder unchanged candidates.
fn ag5(stat: Stat) -> FamilyResult {
    let mut points = Vec::new();
    let base = 1_000usize;
    // X: one agreement on a count-2 mark. Y: three agreements on count-300 marks.
    // Neither candidate's own evidence changes as unrelated events are appended.
    for added in [0usize, 1_000, 10_000, 100_000] {
        let total = base + added;
        let sparse = score(stat, &case(&[2], 2, total, None, total));
        let broad = score(stat, &case(&[300; 3], 0, total, None, total));
        // At M = 0 the sparse candidate leads; the invariant is that the
        // ordering established there survives.
        points.push(Point {
            params: format!("M={added}"),
            nominal: added == 0,
            weaker: broad,
            stronger: sparse,
            holds: sparse > broad,
        });
    }
    family(
        "AG5 vocabulary growth",
        "two unchanged candidates, k=1 and k=3, in a corpus gaining M events of new marks",
        "their ordering must not change: neither candidate's own evidence changed",
        "each agreement gains ln(N′/N), so the gain is proportional to k and not order-preserving",
        Verdict::Fail,
        points,
    )
}

/// AG6a — duplicating the whole corpus must change nothing.
fn ag6a(stat: Stat) -> FamilyResult {
    let mut points = Vec::new();
    let base = 1_000usize;
    let reference = score(stat, &case(&[50, 50, 7], 1, base, None, base));
    for factor in [1usize, 2, 4, 10] {
        let scaled = score(
            stat,
            &case(
                &[50 * factor, 50 * factor, 7 * factor],
                1,
                base * factor,
                None,
                base * factor,
            ),
        );
        points.push(Point {
            params: format!("×{factor}"),
            nominal: factor == 2,
            weaker: reference,
            stronger: scaled,
            holds: (scaled - reference).abs() < 1e-9,
        });
    }
    family(
        "AG6a whole-corpus duplication",
        "every count and N multiplied by the same factor",
        "every score exactly unchanged: c/N is unchanged, so the empirical distribution is too",
        "any dependence on absolute counts rather than frequencies would show up here",
        Verdict::Pass,
        points,
    )
}

/// AG6b — duplicating background only, which is AG5's mechanism by another name.
fn ag6b(stat: Stat) -> FamilyResult {
    let mut points = Vec::new();
    let base = 1_000usize;
    for added in [0usize, 1_000, 10_000] {
        let total = base + added;
        let sparse = score(stat, &case(&[2], 2, total, None, total));
        let broad = score(stat, &case(&[300; 3], 0, total, None, total));
        points.push(Point {
            params: format!("+{added} background"),
            nominal: added == 0,
            weaker: broad,
            stronger: sparse,
            holds: sparse > broad,
        });
    }
    family(
        "AG6b background duplication",
        "background events appended; the two candidates' own marks and counts untouched",
        "no reordering of two candidates whose own evidence is unchanged",
        "the same denominator effect as AG5",
        Verdict::Fail,
        points,
    )
}

/// AG7 — the same distribution observed at a different sample size.
fn ag7(stat: Stat) -> FamilyResult {
    let mut points = Vec::new();
    let base = 1_000usize;
    // Proportional candidate: relative frequency held at 0.05 as N grows.
    let proportional_at = |total: usize| {
        score(
            stat,
            &case(&[(total / 20).max(1); 3], 0, total, None, total),
        )
    };
    // Singleton candidate: count 1 at every N, so its frequency moves with N.
    let singleton_at = |total: usize| score(stat, &case(&[1], 2, total, None, total));

    let proportional_reference = proportional_at(base);
    let singleton_reference = singleton_at(base);
    for ratio in [15usize, 20, 50, 100] {
        let total = base * ratio / 10;
        let proportional = proportional_at(total);
        let singleton = singleton_at(total);
        // The invariant is on the proportional candidate: stable to 0.1 nats per
        // agreement. The singleton's drift is reported in the same row.
        points.push(Point {
            params: format!(
                "N′/N={:.1} · singleton drift {:+.3}",
                ratio as f64 / 10.0,
                singleton - singleton_reference
            ),
            nominal: ratio == 20,
            weaker: proportional_reference,
            stronger: proportional,
            holds: (proportional - proportional_reference).abs() < 0.3,
        });
    }
    family(
        "AG7 sample-size stability",
        "the same distribution observed at N and N′, with a proportional and a singleton candidate",
        "the proportional candidate stable to 0.1 nats per agreement across N",
        "a singleton's term is ln N by definition and cannot be stable in N",
        Verdict::Pass,
        points,
    )
}

/// AG8 — one spectacular coincidence against several moderate ones.
fn ag8(stat: Stat) -> FamilyResult {
    let mut points = Vec::new();
    for total in [100usize, 1_000, 10_000, 1_000_000, 1_000_000_000] {
        for common in [5usize, 50] {
            let coincidence = score(stat, &case(&[1], 3, total, None, total));
            let repeated = score(stat, &case(&[common; 4], 0, total, None, total));
            points.push(point(
                format!("N={total} c={common}"),
                total == 1_000 && common == 5,
                coincidence,
                repeated,
            ));
        }
    }
    family(
        "AG8 coincidence vs repetition",
        "X = one agreement on a count-1 mark; Y = four agreements on marks of count c",
        "Y > X: four independent moderate agreements outweigh one spectacular coincidence, \
         which at rate 1/N is expected about once per corpus by construction",
        "the sum is indifferent to how many terms it has, so one large term and several \
         moderate ones are interchangeable",
        Verdict::Mixed,
        points,
    )
}

/// Every preregistered adversarial family, in task:25 §2.4's order.
pub fn families() -> Vec<FamilyResult> {
    families_with(under_test())
}

/// The same ten families, scored with any statistic. sprint:17, task:27 §D.
pub fn families_with(stat: Stat) -> Vec<FamilyResult> {
    vec![
        ag1(stat),
        ag2(stat),
        ag3(stat),
        ag3b(stat),
        ag4(stat),
        ag5(stat),
        ag6a(stat),
        ag6b(stat),
        ag7(stat),
        ag8(stat),
    ]
}
