//! **Disposable research experiment.** sprint:17, task:27.
//!
//! Candidate repairs to `rarity_of_agreements`, and the semantic contract they
//! are derived from. Nothing here is adopted, and nothing outside
//! [`crate::experiment`] depends on any of it.
//!
//! task:27 §B states what the statistic is supposed to *mean* — six clauses,
//! each naming the quantity that would witness its violation — and §C derives
//! three candidates from that meaning rather than from the tests they will
//! face. This module implements both, plus the contract checks.
//!
//! # The one derivation that shapes everything else
//!
//! task:27 §D2 proves that any statistic of the form `Σ over agreeing positions
//! of w(mark)`, with `w` non-constant, admits candidates where fewer agreements
//! outscore more. Since a non-constant `w` is exactly what clause C3 demands,
//! an accumulation crossing is not a defect of `rarity_of_agreements`: it is a
//! consequence of weighting positions by rarity at all. No candidate here is
//! built to remove crossings, and none is rejected for exhibiting them.

use std::collections::BTreeMap;

use serde::Serialize;

use super::identifiability::Observation;

/// A statistic over the mark-only representation, as sprint:14 defines it.
pub type Score = fn(&Observation) -> Option<f64>;

/// One candidate repair, with the interpretation it was derived from.
#[derive(Debug, Clone, Copy)]
pub struct Candidate {
    /// Short label, used in every table.
    pub name: &'static str,
    /// The formula, in task:27 §C's notation.
    pub formula: &'static str,
    /// What the number means, stated without reference to any test it faces.
    ///
    /// task:27 §G rejects any candidate justifiable only by what it passes, so
    /// this field is the one that has to survive on its own.
    pub interpretation: &'static str,
    /// Whether this is the frozen incumbent rather than a proposed repair.
    pub frozen: bool,
    /// The statistic itself.
    pub score: Score,
}

/// The pooled frequency of a mark: the maximum-likelihood estimate under the
/// hypothesis that both recordings are draws from one shared distribution.
///
/// This is not a taste among {mean, min, max, geometric}. It is the estimator
/// that hypothesis licenses, and the only symmetric construction in task:27 §C
/// that requires no free choice.
fn pooled(observation: &Observation, mark: &str) -> f64 {
    let a = observation.a_counts.get(mark).copied().unwrap_or(1).max(1);
    let b = observation.b_counts.get(mark).copied().unwrap_or(1).max(1);
    let total = observation.a_total.max(1) + observation.b_total.max(1);
    a.saturating_add(b) as f64 / total as f64
}

/// Total pooled surprisal of the agreeing marks, in nats.
fn pooled_total(observation: &Observation) -> Option<f64> {
    observation.len()?;
    Some(
        observation
            .agreeing()
            .into_iter()
            .map(|index| -pooled(observation, &observation.a[index]).ln())
            .sum(),
    )
}

/// **S0 — the frozen incumbent.** Total surprisal of the agreeing marks against
/// recording A's marginals alone. Reproduced here so every table scores the
/// incumbent through the same path as the candidates.
fn rarity_of_agreements(observation: &Observation) -> Option<f64> {
    observation.len()?;
    Some(
        observation
            .agreeing()
            .into_iter()
            .map(|index| {
                let count = observation
                    .a_counts
                    .get(&observation.a[index])
                    .copied()
                    .unwrap_or(1)
                    .max(1);
                -((count as f64 / observation.a_total.max(1) as f64).ln())
            })
            .sum(),
    )
}

/// **R1 — pooled sum.** Changes exactly one thing about S0: whose marginals the
/// surprisal is measured against.
fn pooled_rarity(observation: &Observation) -> Option<f64> {
    pooled_total(observation)
}

/// **R2 — pooled mean.** Surprisal of a *typical* agreeing position.
///
/// Zero at zero agreements, which is what "no evidence" should score and what
/// clause C6 checks.
fn pooled_mean_rarity(observation: &Observation) -> Option<f64> {
    let total = pooled_total(observation)?;
    let agreements = observation.agreeing().len();
    if agreements == 0 {
        return Some(0.0);
    }
    Some(total / agreements as f64)
}

/// **R3 — pooled sum per position examined.** Evidence per unit of opportunity,
/// so a long span is not rewarded for having had more chances.
fn pooled_density_rarity(observation: &Observation) -> Option<f64> {
    let span = observation.len()?;
    if span == 0 {
        return Some(0.0);
    }
    Some(pooled_total(observation)? / span as f64)
}

/// The frozen incumbent and the three candidates, in task:27 §C's order.
pub const CANDIDATES: [Candidate; 4] = [
    Candidate {
        name: "S0 rarity_of_agreements",
        formula: "Σ_agreeing −ln( ĉ_A(m)/N_A )",
        interpretation: "total surprisal of the agreeing marks, against recording A's marginals alone",
        frozen: true,
        score: rarity_of_agreements,
    },
    Candidate {
        name: "R1 pooled sum",
        formula: "Σ_agreeing −ln p̂(m)",
        interpretation: "total surprisal of the agreeing marks under the shared-source model",
        frozen: false,
        score: pooled_rarity,
    },
    Candidate {
        name: "R2 pooled mean",
        formula: "(1/k) Σ_agreeing −ln p̂(m)",
        interpretation: "how surprising a typical agreeing position is, rather than how much surprise in total",
        frozen: false,
        score: pooled_mean_rarity,
    },
    Candidate {
        name: "R3 pooled density",
        formula: "(1/L) Σ_agreeing −ln p̂(m)",
        interpretation: "surprisal per position examined — evidence per unit of opportunity offered",
        frozen: false,
        score: pooled_density_rarity,
    },
];

/// Look a candidate up by name.
pub fn candidate(name: &str) -> Option<&'static Candidate> {
    CANDIDATES.iter().find(|entry| entry.name == name)
}

// ---------------------------------------------------------------------------
// Probe fixtures for the contract checks
// ---------------------------------------------------------------------------

/// A contract probe. Deliberately a *separate* fixture from
/// [`super::adversarial`]'s `case`, because task:27 §J forbids modifying any
/// prior family and a shared constructor would be a modification waiting to
/// happen.
#[derive(Debug, Clone, Copy)]
struct Probe {
    /// Each agreeing position's mark count in A.
    agreeing_a: &'static [usize],
    /// The same positions' counts in B, when they differ.
    agreeing_b: Option<&'static [usize]>,
    /// How many positions disagree.
    disagreements: usize,
    /// The count carried by each disagreeing mark.
    disagreement_count: usize,
    /// Recording A's length.
    a_total: usize,
    /// Recording B's length.
    b_total: usize,
}

impl Probe {
    fn build(&self) -> Observation {
        let mut a = Vec::new();
        let mut b = Vec::new();
        let mut a_counts: BTreeMap<String, usize> = BTreeMap::new();
        let mut b_counts: BTreeMap<String, usize> = BTreeMap::new();

        for (index, count) in self.agreeing_a.iter().enumerate() {
            let mark = format!("m{index}");
            a.push(mark.clone());
            b.push(mark.clone());
            a_counts.insert(mark.clone(), *count);
            let in_b = self
                .agreeing_b
                .and_then(|counts| counts.get(index).copied())
                .unwrap_or(*count);
            b_counts.insert(mark, in_b);
        }
        for index in 0..self.disagreements {
            let left = format!("dl{index}");
            let right = format!("dr{index}");
            a.push(left.clone());
            b.push(right.clone());
            a_counts.insert(left, self.disagreement_count);
            b_counts.insert(right, self.disagreement_count);
        }

        Observation {
            a,
            b,
            a_counts,
            b_counts,
            a_total: self.a_total,
            b_total: self.b_total,
        }
    }
}

/// Exchange the two recordings' roles: sequences, counts, and totals together.
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

const fn probe(
    agreeing_a: &'static [usize],
    agreeing_b: Option<&'static [usize]>,
    disagreements: usize,
    a_total: usize,
    b_total: usize,
) -> Probe {
    Probe {
        agreeing_a,
        agreeing_b,
        disagreements,
        disagreement_count: 1,
        a_total,
        b_total,
    }
}

// ---------------------------------------------------------------------------
// The contract
// ---------------------------------------------------------------------------

/// One clause's result for one candidate.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ClauseResult {
    /// `C1` … `C6`.
    pub clause: &'static str,
    /// What the clause requires.
    pub requirement: &'static str,
    /// The exact quantity task:27 §B named as the witness.
    pub quantity: &'static str,
    /// Its measured value.
    pub value: f64,
    /// Whether the clause holds.
    pub satisfied: bool,
    /// Whether task:27 §D1 makes this clause free by construction, in which
    /// case satisfying it is evidence of correct code and nothing more.
    ///
    /// §I excludes these from eligibility, so they are marked rather than
    /// quietly counted.
    pub free_by_construction: bool,
    /// The configuration that produced the value.
    pub witness: String,
}

/// Every clause, for one candidate.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ContractReport {
    /// Which candidate.
    pub candidate: String,
    /// Its formula.
    pub formula: String,
    /// Clause by clause, in C1..C6 order.
    pub clauses: Vec<ClauseResult>,
}

impl ContractReport {
    /// The clauses that failed, ignoring none.
    pub fn violations(&self) -> Vec<&ClauseResult> {
        self.clauses
            .iter()
            .filter(|clause| !clause.satisfied)
            .collect()
    }

    /// Whether every clause that confers eligibility under task:27 §I holds.
    ///
    /// C1, C4 and C5 are free by construction and confer no credit; a violation
    /// of one still disqualifies, since it can only mean a coding error.
    pub fn eligible(&self) -> bool {
        self.clauses.iter().all(|clause| clause.satisfied)
    }
}

/// A tolerance for exact-equality clauses. Two `f64` sums of the same terms in
/// the same order should be bit-identical; this leaves room for the ones that
/// are not summed in the same order.
const EXACT: f64 = 1e-12;

/// Run the six contract clauses of task:27 §B against one statistic.
pub fn contract(candidate: &Candidate) -> ContractReport {
    let score = candidate.score;
    let at = |probe: &Probe| (score)(&probe.build()).unwrap_or(f64::NAN);

    let mut clauses = Vec::new();

    // C1 — exchange invariance. Every asymmetric probe available: the marks
    // agree, but the two recordings' marginals and lengths do not.
    let c1_probes = [
        probe(&[1], Some(&[500]), 3, 1_000, 1_000),
        probe(&[2, 50], Some(&[400, 3]), 2, 1_000, 4_000),
        probe(&[10], Some(&[1]), 0, 500, 20_000),
        probe(&[7, 7, 7], Some(&[70, 1, 300]), 1, 2_000, 900),
    ];
    let mut worst = 0.0f64;
    let mut worst_witness = String::from("none");
    for (index, spec) in c1_probes.iter().enumerate() {
        let forward = spec.build();
        let delta = ((score)(&forward).unwrap_or(f64::NAN)
            - (score)(&swapped(&forward)).unwrap_or(f64::NAN))
        .abs();
        if delta > worst {
            worst = delta;
            worst_witness = format!("probe {index}");
        }
    }
    clauses.push(ClauseResult {
        clause: "C1",
        requirement: "S(A,B) = S(B,A)",
        quantity: "max |S(A,B) − S(B,A)| over 4 asymmetric probes",
        value: worst,
        satisfied: worst <= EXACT,
        free_by_construction: true,
        witness: worst_witness,
    });

    // C2 — agreement monotonicity. One disagreeing position becomes an agreeing
    // one, span length fixed. The worst case over a rare, a middling and a
    // common added mark is what decides the clause.
    let c2_cases: [(&str, Probe, Probe); 3] = [
        (
            "add count-1 agreement to {50,50}",
            probe(&[50, 50], None, 2, 1_000, 1_000),
            probe(&[50, 50, 1], None, 1, 1_000, 1_000),
        ),
        (
            "add count-50 agreement to {50,50}",
            probe(&[50, 50], None, 2, 1_000, 1_000),
            probe(&[50, 50, 50], None, 1, 1_000, 1_000),
        ),
        (
            "add count-500 agreement to {1}",
            probe(&[1], None, 3, 1_000, 1_000),
            probe(&[1, 500], None, 2, 1_000, 1_000),
        ),
    ];
    let mut c2_worst = f64::INFINITY;
    let mut c2_witness = String::new();
    for (label, before, after) in &c2_cases {
        let delta = at(after) - at(before);
        if delta < c2_worst {
            c2_worst = delta;
            c2_witness = (*label).to_owned();
        }
    }
    clauses.push(ClauseResult {
        clause: "C2",
        requirement: "converting a disagreement into an agreement must not lower the score",
        quantity: "min over 3 cases of S(after) − S(before)",
        value: c2_worst,
        satisfied: c2_worst >= -EXACT,
        free_by_construction: false,
        witness: c2_witness,
    });

    // C3 — rare agreement is more informative than common agreement.
    let c3_rare = probe(&[1], None, 3, 1_000, 1_000);
    let c3_common = probe(&[500], None, 3, 1_000, 1_000);
    let c3 = at(&c3_rare) - at(&c3_common);
    clauses.push(ClauseResult {
        clause: "C3",
        requirement: "agreement on a rarer mark scores strictly higher",
        quantity: "S(count 1) − S(count 500), one agreement, span 4",
        value: c3,
        satisfied: c3 > EXACT,
        free_by_construction: false,
        witness: "N=1000, c ∈ {1, 500}".to_owned(),
    });

    // C4 — rarity that does not agree contributes nothing. The disagreeing
    // positions' marks change count by a factor of 500; nothing else moves.
    let mut c4_plain = probe(&[2, 50], None, 2, 1_000, 1_000);
    c4_plain.disagreement_count = 1;
    let mut c4_loud = c4_plain;
    c4_loud.disagreement_count = 500;
    let c4 = (at(&c4_loud) - at(&c4_plain)).abs();
    clauses.push(ClauseResult {
        clause: "C4",
        requirement: "changing a disagreeing position's marks must not change the score",
        quantity: "|S(disagreement count 500) − S(disagreement count 1)|",
        value: c4,
        satisfied: c4 <= EXACT,
        free_by_construction: true,
        witness: "2 agreements, 2 disagreements, N=1000".to_owned(),
    });

    // C5 — proportional duplication invariance. Every count and both totals
    // scale by t; the score must not move.
    let c5_base = probe(&[2, 50], Some(&[3, 40]), 2, 1_000, 800);
    let mut c5_scaled = probe(&[6, 150], Some(&[9, 120]), 2, 3_000, 2_400);
    c5_scaled.disagreement_count = 3;
    let c5 = (at(&c5_scaled) - at(&c5_base)).abs();
    clauses.push(ClauseResult {
        clause: "C5",
        requirement: "scaling every count and both totals by t leaves the score unchanged",
        quantity: "|S(×3) − S(×1)|",
        value: c5,
        satisfied: c5 <= EXACT,
        free_by_construction: true,
        witness: "t = 3".to_owned(),
    });

    // C6 — rarity is not motif-ness: nothing agreeing must never outscore
    // something agreeing, at equal length.
    let c6_none = probe(&[], None, 4, 1_000, 1_000);
    let c6_some = probe(&[500], None, 3, 1_000, 1_000);
    let c6 = at(&c6_some) - at(&c6_none);
    clauses.push(ClauseResult {
        clause: "C6",
        requirement: "a candidate with no agreements never outscores one with agreements",
        quantity: "S(1 agreement, span 4) − S(0 agreements, span 4)",
        value: c6,
        satisfied: c6 > EXACT,
        free_by_construction: false,
        witness: "agreeing mark of count 500, N=1000".to_owned(),
    });

    ContractReport {
        candidate: candidate.name.to_owned(),
        formula: candidate.formula.to_owned(),
        clauses,
    }
}

/// The contract for every candidate, incumbent first.
pub fn contracts() -> Vec<ContractReport> {
    CANDIDATES.iter().map(contract).collect()
}

// ---------------------------------------------------------------------------
// The crossing theorem, exhibited
// ---------------------------------------------------------------------------

/// A concrete witness to task:27 §D2: at one span length, a candidate with
/// strictly fewer agreements outscoring one with strictly more.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CrossingWitness {
    /// Which statistic.
    pub candidate: String,
    /// The span length both sides share.
    pub span: usize,
    /// Agreements held by the winner.
    pub fewer: usize,
    /// Agreements held by the loser.
    pub more: usize,
    /// The winner's score.
    pub fewer_score: f64,
    /// The loser's score.
    pub more_score: f64,
    /// Whether the crossing occurred.
    pub crossed: bool,
}

/// Construct §D2's proof directly: `k` agreements on the rarest mark against
/// `k + 1` on the commonest, both padded to the same span length.
///
/// The theorem says this crosses for every `k > w(c)/(w(r) − w(c))`. This
/// searches for the smallest such `k` rather than asserting one, so a candidate
/// that genuinely escapes shows up as `crossed: false`.
pub fn crossing_witness(candidate: &Candidate, rare: usize, common: usize) -> CrossingWitness {
    let score = candidate.score;
    const TOTAL: usize = 100_000;

    for k in 1..=24usize {
        let span = k + 1;
        let mut fewer = Observation {
            a: (0..k).map(|i| format!("r{i}")).collect(),
            b: (0..k).map(|i| format!("r{i}")).collect(),
            a_counts: (0..k).map(|i| (format!("r{i}"), rare)).collect(),
            b_counts: (0..k).map(|i| (format!("r{i}"), rare)).collect(),
            a_total: TOTAL,
            b_total: TOTAL,
        };
        // Pad the shorter side to the common span with one disagreement.
        fewer.a.push("pl".to_owned());
        fewer.b.push("pr".to_owned());
        fewer.a_counts.insert("pl".to_owned(), 1);
        fewer.b_counts.insert("pr".to_owned(), 1);
        fewer.a_counts.insert("pr".to_owned(), 1);
        fewer.b_counts.insert("pl".to_owned(), 1);

        let more = Observation {
            a: (0..span).map(|i| format!("c{i}")).collect(),
            b: (0..span).map(|i| format!("c{i}")).collect(),
            a_counts: (0..span).map(|i| (format!("c{i}"), common)).collect(),
            b_counts: (0..span).map(|i| (format!("c{i}"), common)).collect(),
            a_total: TOTAL,
            b_total: TOTAL,
        };

        let fewer_score = (score)(&fewer).unwrap_or(f64::NAN);
        let more_score = (score)(&more).unwrap_or(f64::NAN);
        if fewer_score > more_score {
            return CrossingWitness {
                candidate: candidate.name.to_owned(),
                span,
                fewer: k,
                more: span,
                fewer_score,
                more_score,
                crossed: true,
            };
        }
    }

    CrossingWitness {
        candidate: candidate.name.to_owned(),
        span: 0,
        fewer: 0,
        more: 0,
        fewer_score: f64::NAN,
        more_score: f64::NAN,
        crossed: false,
    }
}

/// §D2's witness for every candidate, at one rare/common pair.
pub fn crossing_witnesses(rare: usize, common: usize) -> Vec<CrossingWitness> {
    CANDIDATES
        .iter()
        .map(|entry| crossing_witness(entry, rare, common))
        .collect()
}
