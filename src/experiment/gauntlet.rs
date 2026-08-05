//! An adversarial gauntlet for sprint:11's null-relative statistic.
//!
//! **Disposable.** sprint:12, task:22. A separate module so deleting the attack
//! is deleting one file, and so that nothing in it can be mistaken for part of
//! the machinery it attacks.
//!
//! # What this is for
//!
//! sprint:11 found that on one real specimen, adding a fourth event made raw
//! agreement worse and the agreement *more exceptional* relative to the order
//! null. That is one specimen. This module builds families of synthetic
//! specimen pairs in which the answer is known by construction, runs the frozen
//! machinery over hundreds of them, and scores the outcome against directional
//! expectations task:22 recorded before any trial ran.
//!
//! # What it may not do
//!
//! It calls [`crate::experiment::event_sequence`] and changes nothing in it. No
//! cost, no weight, no timing policy, no normalization, no search radius, no
//! length floor, and no part of the null construction is touched or
//! parameterized from here. If a family fails, the failure is the result.
//!
//! # How a specimen is built
//!
//! Every trial writes two synthetic NDJSON recordings and runs them through the
//! ordinary replay → inspect → project path, so the gauntlet exercises the real
//! pipeline rather than a shortcut around it. Each sequence has the shape
//!
//! ```text
//! context · core · boundary · context
//! ```
//!
//! where the two contexts are drawn **independently** on the two sides from a
//! background vocabulary, with non-repeating gaps; the core carries the same
//! marks in the same order with identical gaps on both sides; and the boundary
//! is the single event under test.
//!
//! **The backgrounds are deliberately non-periodic.** task:22 §2 records why: in
//! a fixture whose figure repeats, no objective that measures agreement can
//! identify where the figure begins, because many spans agree perfectly. A
//! family with repeating context would measure that ambiguity a second time
//! instead of measuring what it was built for.

use serde::Serialize;

use super::conditional_null;
use super::event_sequence::{
    ChannelScope, EventSequence, LENGTH_FLOOR, NullEnsemble, REFINE_RADIUS, align, null_ensemble,
    null_evidence, project, refine,
};
use super::identifiability::{Observation, SCORERS};
use crate::inspection::inspect;
use crate::replay_bytes;

/// Order-null realizations per trial. task:22 §6.
pub const REALIZATIONS: usize = 1_000;

/// Background vocabulary size. Six marks, so a permutation has somewhere to put
/// things and the null is not degenerate by construction.
const BACKGROUND_MARKS: usize = 6;

/// Prevalence of the boundary mark inside the background, in the `common`
/// family: roughly this fraction of context events are forced to carry it.
const COMMON_PREVALENCE: f64 = 0.35;

/// The eight families task:22 §5 fixed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Family {
    /// Same mark both sides, absent from the background, gaps disagreeing.
    Informative,
    /// Different marks on the two sides, both from the background vocabulary.
    Noise,
    /// Same mark both sides, also injected into the background at high
    /// prevalence. Paired with [`Family::Rare`].
    Common,
    /// Same mark both sides, absent from the background. Identical to
    /// [`Family::Common`] in seed, core, and gaps, so raw agreement matches by
    /// construction and only prevalence differs.
    Rare,
    /// Same mark both sides, but one already present in the core.
    Redundant,
    /// Two independent streams with nothing planted.
    Accidental,
    /// A planted core inside a varying amount of unrelated context.
    Diluted,
    /// One short tight common core and one longer imperfect rare core, together.
    Competing,
}

impl Family {
    /// A stable label for output.
    pub fn label(self) -> &'static str {
        match self {
            Family::Informative => "informative",
            Family::Noise => "noise",
            Family::Common => "common",
            Family::Rare => "rare",
            Family::Redundant => "redundant",
            Family::Accidental => "accidental",
            Family::Diluted => "diluted",
            Family::Competing => "competing",
        }
    }
}

/// One trial's parameters. Everything needed to regenerate it exactly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Trial {
    /// Which family this trial belongs to.
    pub family: Family,
    /// Events in the shared core.
    pub core_len: usize,
    /// Events of unrelated background on each side of the core, per sequence.
    pub context_len: usize,
    /// Factor by which the boundary event's gap differs between the two sides.
    pub gap_ratio: u64,
    /// Trial seed. Recorded so any failure regenerates.
    pub seed: u64,
}

impl Trial {
    /// The seed for one side of one trial, derived and never ambient.
    fn side_seed(&self, side: u64) -> u64 {
        0x6741_554E_544C_4554
            ^ self.seed.wrapping_mul(0x9E37_79B9_7F4A_7C15)
            ^ side.wrapping_mul(0xD1B5_4A32_D192_ED03)
            ^ (self.core_len as u64).wrapping_mul(0x0000_1000_0000_0001)
            ^ (self.context_len as u64).wrapping_mul(0x0000_0010_0000_0001)
            ^ self.family as u64
    }
}

/// One trial's measured outcome.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Outcome {
    /// The trial that produced it.
    pub trial: Trial,
    /// Raw combined distance of the core spans.
    pub core_total: f64,
    /// Raw combined distance of the extended spans.
    pub expanded_total: f64,
    /// Standardized separation of the core spans, absent on a degenerate null.
    pub core_z: Option<f64>,
    /// Standardized separation of the extended spans.
    pub expanded_z: Option<f64>,
    /// Empirical tail probability of each, reported and — per task:22 §6 — not
    /// scored, because the ensemble floor saturates it on specimens like these.
    pub core_p: f64,
    /// Tail probability of the extended spans.
    pub expanded_p: f64,
    /// `expanded_total − core_total`. Positive means raw agreement got worse.
    pub delta_total: f64,
    /// `expanded_z − core_z`, absent when either side's null was degenerate.
    pub delta_z: Option<f64>,
    /// The marks of the extended A span, verbatim, so a counterexample can be
    /// read rather than merely located.
    pub a_marks: Vec<String>,
    /// The marks of the extended B span.
    pub b_marks: Vec<String>,
    /// `diluted` only: whether the best-`z` candidate found by the frozen search
    /// overlaps the planted core.
    pub overlap: Option<bool>,
    /// The same question asked of the challenger: whether the best-surprisal
    /// candidate overlaps the planted core.
    pub overlap_s: Option<bool>,
    /// `competing` only: the standardized separation of the *other* motif — the
    /// short, tight, common one — against which the scored core competes.
    pub alt_z: Option<f64>,
    /// sprint:13's challenger, on the core spans: conditional match surprisal in
    /// nats. `None` where the spans differ in length, which the statistic cannot
    /// score.
    pub core_s: Option<f64>,
    /// The challenger on the extended spans.
    pub expanded_s: Option<f64>,
    /// `expanded_s − core_s`.
    pub delta_s: Option<f64>,
    /// `competing` only: the challenger on the other motif.
    pub alt_s: Option<f64>,
    /// sprint:14's enumeration, on the core spans: one value per preregistered
    /// function of the representation, in `SCORERS` order.
    pub core_scores: Vec<Option<f64>>,
    /// The same on the extended spans.
    pub expanded_scores: Vec<Option<f64>>,
    /// The same on the competing motif, where a family has one.
    pub alt_scores: Vec<Option<f64>>,
    /// Per function, whether its best-scoring candidate overlaps the planted
    /// core. `diluted` only, recomputed per function rather than inherited.
    pub overlap_scores: Vec<Option<bool>>,
}

/// A generated pair of recordings, as NDJSON, with the spans that matter.
struct Specimen {
    a: String,
    b: String,
    core_a: (usize, usize),
    core_b: (usize, usize),
    expanded_a: (usize, usize),
    expanded_b: (usize, usize),
    /// The second, competing motif, on the `competing` family only.
    alt_a: Option<(usize, usize)>,
    alt_b: Option<(usize, usize)>,
}

/// A fixed linear congruential generator, the same shape the fixtures use.
struct Lcg(u64);

impl Lcg {
    fn new(seed: u64) -> Self {
        Self(seed | 1)
    }

    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0 >> 33
    }

    fn below(&mut self, bound: usize) -> usize {
        if bound == 0 {
            0
        } else {
            self.next() as usize % bound
        }
    }
}

/// One generated event: a delivered tool name and the gap that precedes it.
#[derive(Clone)]
struct Step {
    tool: String,
    gap_ms: u64,
}

/// Background filler. Non-repeating gaps, so nothing in the context is periodic.
fn background(
    generator: &mut Lcg,
    count: usize,
    forced: Option<&str>,
    prevalence: f64,
) -> Vec<Step> {
    (0..count)
        .map(|_| {
            let force = forced.is_some() && (generator.below(1000) as f64) < prevalence * 1000.0;
            let tool = match (force, forced) {
                (true, Some(name)) => name.to_owned(),
                _ => format!("SyntheticBg{}", generator.below(BACKGROUND_MARKS)),
            };
            Step {
                tool,
                // 700..4500 ms, never a repeating cadence.
                gap_ms: 700 + generator.next() % 3_800,
            }
        })
        .collect()
}

/// The shared core: the same marks in the same order with identical gaps.
fn core_steps(generator: &mut Lcg, len: usize) -> Vec<Step> {
    (0..len)
        .map(|index| Step {
            tool: format!("SyntheticCore{index}"),
            gap_ms: 300 + generator.next() % 500,
        })
        .collect()
}

/// Render a side as NDJSON through the ordinary record vocabulary.
fn ndjson(session: &str, steps: &[Step]) -> String {
    use crate::record::{Channel, Provenance, v2};

    let origin: jiff::Timestamp = "2026-06-01T00:00:00Z"
        .parse()
        .unwrap_or(jiff::Timestamp::UNIX_EPOCH);
    let mut at = 0u64;
    let mut out = String::new();
    for (index, step) in steps.iter().enumerate() {
        at += step.gap_ms;
        let record = v2::Record {
            schema_version: v2::SCHEMA_VERSION,
            session_id: session.to_owned(),
            sequence: index as u64 + 1,
            recorded_at: origin + jiff::SignedDuration::from_millis(at as i64),
            context: v2::Context::default(),
            provenance: Provenance {
                channel: Channel::Observed,
                adapter: "synthetic-gauntlet-adapter".to_owned(),
                mechanism: "synthetic:Gauntlet".to_owned(),
            },
            event: v2::Event::ToolRequested(v2::ToolRequested {
                tool_use_id: format!("toolu_synthetic_gauntlet_{index:04}"),
                tool_name: step.tool.clone(),
                requested_input: serde_json::json!({ "target": "/synthetic/gauntlet" }),
            }),
        };
        if let Ok(line) = serde_json::to_string(&record) {
            out.push_str(&line);
            out.push('\n');
        }
    }
    out
}

/// Build one trial's pair of recordings.
fn specimen(trial: &Trial) -> Specimen {
    let mut left = Lcg::new(trial.side_seed(0));
    let mut right = Lcg::new(trial.side_seed(1));
    let mut shared = Lcg::new(trial.side_seed(2));

    let core = core_steps(&mut shared, trial.core_len);
    let boundary_gap = 400 + shared.next() % 600;

    // What the boundary event is, per family.
    let (a_boundary, b_boundary, forced, prevalence) = match trial.family {
        Family::Informative | Family::Rare | Family::Diluted | Family::Accidental => (
            "SyntheticBoundaryRare".to_owned(),
            "SyntheticBoundaryRare".to_owned(),
            None,
            0.0,
        ),
        Family::Noise => {
            // The two marks must *differ*, and the first draft of this generator
            // drew them independently — which produced the same mark on both
            // sides in 20 of 60 trials, quietly turning a third of the noise
            // family into informative trials. The offset is applied to the first
            // draw rather than to a second one, so a collision is impossible by
            // construction. task:22's Result records the defect, the corrected
            // numbers, and the numbers it produced before the correction.
            let first = shared.below(BACKGROUND_MARKS);
            let second = (first + 1 + shared.below(BACKGROUND_MARKS - 1)) % BACKGROUND_MARKS;
            (
                format!("SyntheticBg{first}"),
                format!("SyntheticBg{second}"),
                None,
                0.0,
            )
        }
        // Identical to Rare in every respect but background prevalence.
        Family::Common => (
            "SyntheticBoundaryRare".to_owned(),
            "SyntheticBoundaryRare".to_owned(),
            Some("SyntheticBoundaryRare"),
            COMMON_PREVALENCE,
        ),
        // A mark the core already carries.
        Family::Redundant => (
            "SyntheticCore0".to_owned(),
            "SyntheticCore0".to_owned(),
            None,
            0.0,
        ),
        Family::Competing => (
            "SyntheticBoundaryRare".to_owned(),
            "SyntheticBoundaryRare".to_owned(),
            None,
            0.0,
        ),
    };

    let a_lead = background(&mut left, trial.context_len, forced, prevalence);
    let b_lead = background(&mut right, trial.context_len, forced, prevalence);
    let a_tail = background(&mut left, trial.context_len, forced, prevalence);
    let b_tail = background(&mut right, trial.context_len, forced, prevalence);

    let assemble = |lead: &[Step], boundary: &str, gap: u64, tail: &[Step]| {
        let mut steps = lead.to_vec();
        // The accidental family plants nothing: both streams are pure
        // background, so any agreement found in them arose by chance.
        if trial.family != Family::Accidental {
            steps.extend(core.iter().cloned());
            steps.push(Step {
                tool: boundary.to_owned(),
                gap_ms: gap,
            });
        }
        steps.extend(tail.iter().cloned());
        steps
    };

    let a_steps = assemble(&a_lead, &a_boundary, boundary_gap, &a_tail);
    let b_steps = assemble(
        &b_lead,
        &b_boundary,
        boundary_gap * trial.gap_ratio,
        &b_tail,
    );

    let core_start = trial.context_len;
    Specimen {
        a: ndjson("sess-synthetic-gauntlet-a", &a_steps),
        b: ndjson("sess-synthetic-gauntlet-b", &b_steps),
        core_a: (core_start, core_start + trial.core_len),
        core_b: (core_start, core_start + trial.core_len),
        expanded_a: (core_start, core_start + trial.core_len + 1),
        expanded_b: (core_start, core_start + trial.core_len + 1),
        alt_a: None,
        alt_b: None,
    }
}

/// The competing-motif specimen: a short exact core built from **common** marks
/// and, later in the same pair, a longer core built from **rare** marks with one
/// substitution. The conflict between raw fit and evidential weight, in one pair.
fn competing_specimen(trial: &Trial) -> Specimen {
    let mut left = Lcg::new(trial.side_seed(0));
    let mut right = Lcg::new(trial.side_seed(1));
    let mut shared = Lcg::new(trial.side_seed(2));

    // Short and tight: three events, exact, drawn from the background vocabulary
    // so the marks are ubiquitous.
    let tight: Vec<Step> = (0..3)
        .map(|_| Step {
            tool: format!("SyntheticBg{}", shared.below(BACKGROUND_MARKS)),
            gap_ms: 300 + shared.next() % 300,
        })
        .collect();
    // Longer and imperfect: six events from marks that appear nowhere else, with
    // the fourth substituted on the B side.
    let long_a: Vec<Step> = (0..6)
        .map(|index| Step {
            tool: format!("SyntheticScarce{index}"),
            gap_ms: 400 + shared.next() % 400,
        })
        .collect();
    let mut long_b = long_a.clone();
    long_b[3].tool = "SyntheticScarceOther".to_owned();

    let a_lead = background(&mut left, trial.context_len, None, 0.0);
    let b_lead = background(&mut right, trial.context_len, None, 0.0);
    let a_mid = background(&mut left, trial.context_len, None, 0.0);
    let b_mid = background(&mut right, trial.context_len, None, 0.0);
    let a_tail = background(&mut left, trial.context_len, None, 0.0);
    let b_tail = background(&mut right, trial.context_len, None, 0.0);

    let build = |lead: &[Step], long: &[Step], mid: &[Step], tail: &[Step]| {
        let mut steps = lead.to_vec();
        steps.extend(tight.iter().cloned());
        steps.extend(mid.iter().cloned());
        steps.extend(long.iter().cloned());
        steps.extend(tail.iter().cloned());
        steps
    };

    let tight_start = trial.context_len;
    let long_start = trial.context_len + 3 + trial.context_len;
    Specimen {
        a: ndjson(
            "sess-synthetic-gauntlet-a",
            &build(&a_lead, &long_a, &a_mid, &a_tail),
        ),
        b: ndjson(
            "sess-synthetic-gauntlet-b",
            &build(&b_lead, &long_b, &b_mid, &b_tail),
        ),
        // The scored core is the longer, rarer, imperfect one.
        core_a: (long_start, long_start + 6),
        core_b: (long_start, long_start + 6),
        expanded_a: (long_start, long_start + 6),
        expanded_b: (long_start, long_start + 6),
        // The competitor is the short tight common one.
        alt_a: Some((tight_start, tight_start + 3)),
        alt_b: Some((tight_start, tight_start + 3)),
    }
}

/// Score one pair of spans against an ensemble, returning `(total, z, p)`.
fn score(
    a: &EventSequence<'_>,
    b: &EventSequence<'_>,
    ensemble: &NullEnsemble<'_>,
    span_a: (usize, usize),
    span_b: (usize, usize),
) -> Option<(f64, Option<f64>, f64)> {
    let observed = align(
        a.window(span_a.0, span_a.1 - span_a.0)?,
        b.window(span_b.0, span_b.1 - span_b.0)?,
    );
    let evidence = null_evidence(ensemble, span_a, span_b, &observed)?;
    Some((
        observed.total,
        evidence.total.standardized_separation,
        evidence.total.empirical_p,
    ))
}

/// The verbatim marks of one span, so a counterexample can be read.
fn marks(sequence: &EventSequence<'_>, span: (usize, usize)) -> Vec<String> {
    sequence
        .window(span.0, span.1 - span.0)
        .map(|events| events.iter().map(|event| event.mark.label()).collect())
        .unwrap_or_default()
}

/// Run one trial. `None` when the generated pair could not be projected, which
/// would be a defect in the generator rather than a result.
pub fn run(trial: &Trial) -> Option<Outcome> {
    let built = if trial.family == Family::Competing {
        competing_specimen(trial)
    } else {
        specimen(trial)
    };
    let a_replay = replay_bytes(built.a.as_bytes()).ok()?;
    let b_replay = replay_bytes(built.b.as_bytes()).ok()?;
    let (a_inspection, b_inspection) = (inspect(&a_replay), inspect(&b_replay));
    let a = project(&a_inspection, ChannelScope::Observed)?;
    let b = project(&b_inspection, ChannelScope::Observed)?;
    let ensemble = null_ensemble(&a, &b, REALIZATIONS);

    // The accidental family has no planted core: the "core" is whatever the
    // frozen boundary search finds by chance near an arbitrary seed, and the
    // question is what evidence a coincidence attracts.
    let (core_a, core_b, expanded_a, expanded_b) = if trial.family == Family::Accidental {
        let seed_a = (built.core_a.0, built.core_a.0 + trial.core_len + 2);
        let seed_b = (built.core_b.0, built.core_b.0 + trial.core_len + 2);
        let refined = refine(&a, seed_a, &b, seed_b, REFINE_RADIUS, LENGTH_FLOOR)?;
        let best = refined
            .frontier
            .iter()
            .min_by(|left, right| {
                left.pair
                    .comparison
                    .alignment
                    .total
                    .partial_cmp(&right.pair.comparison.alignment.total)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .copied()?;
        let (wa, wb) = (&best.pair.comparison.a, &best.pair.comparison.b);
        let found_a = (wa.start, wa.start + wa.k);
        let found_b = (wb.start, wb.start + wb.k);
        (found_a, found_b, found_a, found_b)
    } else {
        (
            built.core_a,
            built.core_b,
            built.expanded_a,
            built.expanded_b,
        )
    };

    let (core_total, core_z, core_p) = score(&a, &b, &ensemble, core_a, core_b)?;
    let (expanded_total, expanded_z, expanded_p) =
        score(&a, &b, &ensemble, expanded_a, expanded_b)?;

    // `diluted` asks a different question from the rest: not whether one added
    // event helps, but whether the frozen search's best-`z` candidate still
    // lands on the planted core once the context grows around it.
    let overlap = (trial.family == Family::Diluted)
        .then(|| {
            let pad = 2usize;
            let seed_a = (built.core_a.0.saturating_sub(pad), built.core_a.1 + pad);
            let seed_b = (built.core_b.0.saturating_sub(pad), built.core_b.1 + pad);
            let refined = refine(&a, seed_a, &b, seed_b, REFINE_RADIUS, LENGTH_FLOOR)?;
            let best = refined
                .frontier
                .iter()
                .filter_map(|candidate| {
                    let (wa, wb) = (&candidate.pair.comparison.a, &candidate.pair.comparison.b);
                    let evidence = null_evidence(
                        &ensemble,
                        (wa.start, wa.start + wa.k),
                        (wb.start, wb.start + wb.k),
                        &candidate.pair.comparison.alignment,
                    )?;
                    Some((evidence.total.standardized_separation?, *candidate))
                })
                .max_by(|left, right| {
                    left.0
                        .partial_cmp(&right.0)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })?;
            let window = best.1.pair.comparison.a;
            Some(window.start < built.core_a.1 && window.start + window.k > built.core_a.0)
        })
        .flatten();

    let alt_z = match (built.alt_a, built.alt_b) {
        (Some(span_a), Some(span_b)) => {
            score(&a, &b, &ensemble, span_a, span_b).and_then(|(_, z, _)| z)
        }
        _ => None,
    };

    // The same dilution question, asked of the challenger. Computed separately
    // rather than reusing the incumbent's answer, which would have reported the
    // incumbent's verdict in the challenger's column.
    let overlap_s = (trial.family == Family::Diluted)
        .then(|| {
            let pad = 2usize;
            let seed_a = (built.core_a.0.saturating_sub(pad), built.core_a.1 + pad);
            let seed_b = (built.core_b.0.saturating_sub(pad), built.core_b.1 + pad);
            let refined = refine(&a, seed_a, &b, seed_b, REFINE_RADIUS, LENGTH_FLOOR)?;
            let best = refined
                .frontier
                .iter()
                .filter_map(|candidate| {
                    let (wa, wb) = (&candidate.pair.comparison.a, &candidate.pair.comparison.b);
                    let value = conditional_null::surprisal(
                        &a,
                        (wa.start, wa.start + wa.k),
                        &b,
                        (wb.start, wb.start + wb.k),
                    )?;
                    Some((value, *candidate))
                })
                .max_by(|left, right| {
                    left.0
                        .partial_cmp(&right.0)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })?;
            let window = best.1.pair.comparison.a;
            Some(window.start < built.core_a.1 && window.start + window.k > built.core_a.0)
        })
        .flatten();

    // sprint:14's enumeration over the same spans. Each function is a function
    // of the representation and of nothing else; `Observation::of` is what
    // closes it off from the recording.
    let observe = |span_a: (usize, usize), span_b: (usize, usize)| -> Vec<Option<f64>> {
        match Observation::of(&a, span_a, &b, span_b) {
            Some(observation) => SCORERS
                .iter()
                .map(|scorer| (scorer.score)(&observation))
                .collect(),
            None => vec![None; SCORERS.len()],
        }
    };
    let core_scores = observe(core_a, core_b);
    let expanded_scores = observe(expanded_a, expanded_b);
    let alt_scores = match (built.alt_a, built.alt_b) {
        (Some(span_a), Some(span_b)) => observe(span_a, span_b),
        _ => vec![None; SCORERS.len()],
    };

    // The dilution question, asked once per function.
    let overlap_scores = if trial.family == Family::Diluted {
        let pad = 2usize;
        let seed_a = (built.core_a.0.saturating_sub(pad), built.core_a.1 + pad);
        let seed_b = (built.core_b.0.saturating_sub(pad), built.core_b.1 + pad);
        match refine(&a, seed_a, &b, seed_b, REFINE_RADIUS, LENGTH_FLOOR) {
            Some(refined) => SCORERS
                .iter()
                .enumerate()
                .map(|(index, _)| {
                    let best = refined
                        .frontier
                        .iter()
                        .filter_map(|candidate| {
                            let (wa, wb) =
                                (&candidate.pair.comparison.a, &candidate.pair.comparison.b);
                            let observation = Observation::of(
                                &a,
                                (wa.start, wa.start + wa.k),
                                &b,
                                (wb.start, wb.start + wb.k),
                            )?;
                            Some(((SCORERS[index].score)(&observation)?, *candidate))
                        })
                        .max_by(|left, right| {
                            left.0
                                .partial_cmp(&right.0)
                                .unwrap_or(std::cmp::Ordering::Equal)
                        })?;
                    let window = best.1.pair.comparison.a;
                    Some(window.start < built.core_a.1 && window.start + window.k > built.core_a.0)
                })
                .collect(),
            None => vec![None; SCORERS.len()],
        }
    } else {
        vec![None; SCORERS.len()]
    };

    let core_s = conditional_null::surprisal(&a, core_a, &b, core_b);
    let expanded_s = conditional_null::surprisal(&a, expanded_a, &b, expanded_b);
    let alt_s = match (built.alt_a, built.alt_b) {
        (Some(span_a), Some(span_b)) => conditional_null::surprisal(&a, span_a, &b, span_b),
        _ => None,
    };

    Some(Outcome {
        trial: *trial,
        core_total,
        expanded_total,
        core_z,
        expanded_z,
        core_p,
        expanded_p,
        delta_total: expanded_total - core_total,
        delta_z: match (expanded_z, core_z) {
            (Some(expanded), Some(core)) => Some(expanded - core),
            _ => None,
        },
        a_marks: marks(&a, expanded_a),
        b_marks: marks(&b, expanded_b),
        overlap,
        overlap_s,
        alt_z,
        core_s,
        expanded_s,
        delta_s: match (expanded_s, core_s) {
            (Some(expanded), Some(core)) => Some(expanded - core),
            _ => None,
        },
        alt_s,
        core_scores,
        expanded_scores,
        alt_scores,
        overlap_scores,
    })
}

/// One side's generated recording, as NDJSON. Exposed so a test can measure what
/// the generator actually built rather than trusting the family's description.
pub fn recording_for(trial: &Trial, side: usize) -> Option<String> {
    let built = if trial.family == Family::Competing {
        competing_specimen(trial)
    } else {
        specimen(trial)
    };
    match side {
        0 => Some(built.a),
        1 => Some(built.b),
        _ => None,
    }
}

/// Apply task:22 §7's rule to a bare list of values. Exposed so the rule itself
/// can be checked against hand-written cases rather than only through a family.
pub fn score_values(values: &[f64], expect_positive: bool) -> Verdict {
    let scored: Vec<Scored> = values
        .iter()
        .map(|value| Scored {
            value: *value,
            outcome: placeholder_outcome(),
        })
        .collect();
    assemble(Family::Informative, "", "", "", scored, 0, expect_positive).verdict
}

/// A zeroed outcome, so [`score_values`] can exercise the rule without building
/// a specimen. Never returned from [`run`].
fn placeholder_outcome() -> Outcome {
    Outcome {
        trial: Trial {
            family: Family::Informative,
            core_len: 0,
            context_len: 0,
            gap_ratio: 0,
            seed: 0,
        },
        core_total: 0.0,
        expanded_total: 0.0,
        core_z: None,
        expanded_z: None,
        core_s: None,
        expanded_s: None,
        delta_s: None,
        alt_s: None,
        core_scores: Vec::new(),
        expanded_scores: Vec::new(),
        alt_scores: Vec::new(),
        overlap_scores: Vec::new(),
        core_p: 0.0,
        expanded_p: 0.0,
        delta_total: 0.0,
        delta_z: None,
        a_marks: Vec::new(),
        b_marks: Vec::new(),
        overlap: None,
        overlap_s: None,
        alt_z: None,
    }
}

/// The grid task:22 §6 fixed. Deterministic and in a fixed order.
pub fn grid() -> Vec<Trial> {
    let mut trials = Vec::new();
    let mut push =
        |family: Family, core_len: usize, context_len: usize, gap_ratio: u64, seed: u64| {
            trials.push(Trial {
                family,
                core_len,
                context_len,
                gap_ratio,
                seed,
            });
        };

    for &core_len in &[3usize, 4, 5] {
        for &context_len in &[20usize, 40] {
            for seed in 0u64..5 {
                for &gap_ratio in &[2u64, 4] {
                    push(Family::Informative, core_len, context_len, gap_ratio, seed);
                    push(Family::Noise, core_len, context_len, gap_ratio, seed);
                }
                push(Family::Common, core_len, context_len, 2, seed);
                push(Family::Rare, core_len, context_len, 2, seed);
                push(Family::Redundant, core_len, context_len, 2, seed);
                push(Family::Accidental, core_len, context_len, 2, seed);
            }
        }
    }
    for &core_len in &[4usize, 6] {
        for &context_len in &[10usize, 20, 40, 80] {
            for seed in 0u64..5 {
                push(Family::Diluted, core_len, context_len, 2, seed);
            }
        }
    }
    for &context_len in &[20usize, 40] {
        for seed in 0u64..10 {
            push(Family::Competing, 3, context_len, 2, seed);
        }
    }
    trials
}

// ---------------------------------------------------------------------------
// Scoring — task:22 §7, one rule applied to every family alike
// ---------------------------------------------------------------------------

/// The three outcomes the preregistered rule can produce.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    /// Median has the expected sign and at least two thirds of trials agree.
    Pass,
    /// Median has the expected sign but fewer than two thirds agree.
    Mixed,
    /// Median has the wrong sign, or is exactly zero.
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

/// One family's scored quantity for one trial, with the trial that produced it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Scored {
    /// The quantity whose sign the family's expectation is about.
    pub value: f64,
    /// The trial it came from.
    pub outcome: Outcome,
}

/// Everything one family produced.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FamilyReport {
    /// Which family.
    pub family: Family,
    /// Which statistic scored it: the incumbent `z` or sprint:13's challenger.
    pub statistic: &'static str,
    /// What the scored quantity is, in words.
    pub quantity: &'static str,
    /// What was expected of it, recorded before any trial ran.
    pub expectation: &'static str,
    /// Trials that produced a usable value.
    pub trials: usize,
    /// Trials discarded because a null had zero variance, so `z` was undefined.
    pub undefined: usize,
    /// Fraction of usable trials whose value has the expected sign.
    pub expected_fraction: f64,
    /// Quartiles of the scored quantity.
    pub q1: f64,
    /// Median of the scored quantity.
    pub median: f64,
    /// Upper quartile of the scored quantity.
    pub q3: f64,
    /// Median `expanded_total − core_total`, reported beside the surprise delta
    /// so the two can be read together.
    pub median_delta_total: f64,
    /// The preregistered verdict.
    pub verdict: Verdict,
    /// The three worst counterexamples, most contrary to the expectation first.
    pub counterexamples: Vec<Scored>,
    /// Every scored trial, so the report can plot rather than summarize.
    pub scored: Vec<Scored>,
}

fn quantile(sorted: &[f64], fraction: f64) -> f64 {
    if sorted.is_empty() {
        return f64::NAN;
    }
    let position = fraction * (sorted.len() - 1) as f64;
    let lower = position.floor() as usize;
    let upper = position.ceil() as usize;
    if lower == upper {
        sorted[lower]
    } else {
        let weight = position - lower as f64;
        sorted[lower] * (1.0 - weight) + sorted[upper] * weight
    }
}

/// Apply task:22 §7's rule. `expect_positive` is false only for the `noise`
/// family, whose expectation is an absence of effect and whose rule inverts.
#[allow(clippy::too_many_arguments)]
fn assemble(
    family: Family,
    statistic: &'static str,
    quantity: &'static str,
    expectation: &'static str,
    scored: Vec<Scored>,
    undefined: usize,
    expect_positive: bool,
) -> FamilyReport {
    let mut values: Vec<f64> = scored.iter().map(|entry| entry.value).collect();
    values.sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));
    let median = quantile(&values, 0.5);
    let trials = values.len();
    let agreeing = scored
        .iter()
        .filter(|entry| {
            if expect_positive {
                entry.value > 0.0
            } else {
                entry.value <= 0.0
            }
        })
        .count();
    let fraction = if trials == 0 {
        f64::NAN
    } else {
        agreeing as f64 / trials as f64
    };

    let verdict = if trials == 0 {
        Verdict::Fail
    } else if expect_positive {
        if median <= 0.0 {
            Verdict::Fail
        } else if fraction >= 2.0 / 3.0 {
            Verdict::Pass
        } else {
            Verdict::Mixed
        }
    } else {
        // The inverted rule: an absence of effect. PASS when the median is not
        // positive and at most half of trials are.
        let positives = scored.iter().filter(|entry| entry.value > 0.0).count();
        let positive_fraction = positives as f64 / trials as f64;
        if median > 0.0 {
            Verdict::Fail
        } else if positive_fraction <= 0.5 {
            Verdict::Pass
        } else {
            Verdict::Mixed
        }
    };

    let mut deltas: Vec<f64> = scored
        .iter()
        .map(|entry| entry.outcome.delta_total)
        .collect();
    deltas.sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));

    // Worst first: most contrary to the expectation.
    let mut worst = scored.clone();
    worst.sort_by(|left, right| {
        let ordering = left
            .value
            .partial_cmp(&right.value)
            .unwrap_or(std::cmp::Ordering::Equal);
        if expect_positive {
            ordering
        } else {
            ordering.reverse()
        }
    });
    worst.truncate(3);

    FamilyReport {
        family,
        statistic,
        quantity,
        expectation,
        trials,
        undefined,
        expected_fraction: fraction,
        q1: quantile(&values, 0.25),
        median,
        q3: quantile(&values, 0.75),
        median_delta_total: quantile(&deltas, 0.5),
        verdict,
        counterexamples: worst,
        scored,
    }
}

/// Pair two families on everything but the family itself.
fn paired(
    outcomes: &[Outcome],
    left: Family,
    right: Family,
    value: impl Fn(&Outcome, &Outcome) -> Option<f64>,
) -> (Vec<Scored>, usize) {
    let mut scored = Vec::new();
    let mut undefined = 0usize;
    for a in outcomes.iter().filter(|o| o.trial.family == left) {
        let Some(b) = outcomes.iter().find(|o| {
            o.trial.family == right
                && o.trial.core_len == a.trial.core_len
                && o.trial.context_len == a.trial.context_len
                && o.trial.seed == a.trial.seed
                && o.trial.gap_ratio == a.trial.gap_ratio
        }) else {
            continue;
        };
        match value(a, b) {
            Some(difference) => scored.push(Scored {
                value: difference,
                outcome: a.clone(),
            }),
            None => undefined += 1,
        }
    }
    (scored, undefined)
}

/// Run the whole grid and score every family under both statistics.
///
/// sprint:13 adds the second column. The trials, their generation, the
/// expectations, and the pass rule are exactly sprint:12's; only the quantity
/// being scored changes.
pub fn report() -> (Vec<Outcome>, Vec<FamilyReport>) {
    let outcomes: Vec<Outcome> = grid().iter().filter_map(run).collect();
    let mut reports = score_all(
        &outcomes,
        "z",
        &|o| o.delta_z,
        &|o| o.core_z,
        &|o| o.alt_z,
        &|o| o.overlap,
    );
    reports.extend(score_all(
        &outcomes,
        "surprisal",
        &|o| o.delta_s,
        &|o| o.core_s,
        &|o| o.alt_s,
        &|o| o.overlap_s,
    ));
    (outcomes, reports)
}

/// The sprint:14 enumeration: every preregistered function of the representation,
/// scored by the frozen machinery over the same trials and the same rule.
///
/// The trials are generated once and shared, so every function sees identical
/// specimens — which is what makes the matrix a comparison rather than ten
/// separate experiments.
pub fn enumeration() -> (Vec<Outcome>, Vec<FamilyReport>) {
    let outcomes: Vec<Outcome> = grid().iter().filter_map(run).collect();
    let mut reports = Vec::new();
    for (index, scorer) in SCORERS.iter().enumerate() {
        let pick = move |values: &[Option<f64>]| values.get(index).copied().flatten();
        reports.extend(score_all(
            &outcomes,
            scorer.name,
            &move |o| match (pick(&o.expanded_scores), pick(&o.core_scores)) {
                (Some(expanded), Some(core)) => Some(expanded - core),
                _ => None,
            },
            &move |o| pick(&o.core_scores),
            &move |o| pick(&o.alt_scores),
            &move |o| o.overlap_scores.get(index).copied().flatten(),
        ));
    }
    (outcomes, reports)
}

/// Score every family under one statistic.
fn score_all(
    outcomes: &[Outcome],
    statistic: &'static str,
    delta: &dyn Fn(&Outcome) -> Option<f64>,
    core: &dyn Fn(&Outcome) -> Option<f64>,
    alt: &dyn Fn(&Outcome) -> Option<f64>,
    overlap: &dyn Fn(&Outcome) -> Option<bool>,
) -> Vec<FamilyReport> {
    let unpaired = |family: Family| -> (Vec<Scored>, usize) {
        let mut scored = Vec::new();
        let mut undefined = 0usize;
        for outcome in outcomes.iter().filter(|o| o.trial.family == family) {
            match delta(outcome) {
                Some(value) => scored.push(Scored {
                    value,
                    outcome: outcome.clone(),
                }),
                None => undefined += 1,
            }
        }
        (scored, undefined)
    };

    let (informative, informative_undefined) = unpaired(Family::Informative);
    let (noise, noise_undefined) = unpaired(Family::Noise);

    // Rare against common: identical raw agreement by construction, differing
    // only in the boundary mark's background prevalence.
    let (rarity, rarity_undefined) =
        paired(outcomes, Family::Rare, Family::Common, |rare, common| {
            Some(delta(rare)? - delta(common)?)
        });
    // A novel rare boundary against one the core already carries.
    let (redundancy, redundancy_undefined) = paired(
        outcomes,
        Family::Redundant,
        Family::Informative,
        |redundant, novel| Some(delta(novel)? - delta(redundant)?),
    );
    // A genuine planted core against the best a coincidence can manage.
    let (accident, accident_undefined) = paired(
        outcomes,
        Family::Accidental,
        Family::Rare,
        |chance, planted| Some(core(planted)? - core(chance)?),
    );

    let mut dilution = Vec::new();
    let mut dilution_undefined = 0usize;
    for outcome in outcomes
        .iter()
        .filter(|o| o.trial.family == Family::Diluted)
    {
        match overlap(outcome) {
            Some(hit) => dilution.push(Scored {
                value: if hit { 1.0 } else { -1.0 },
                outcome: outcome.clone(),
            }),
            None => dilution_undefined += 1,
        }
    }

    let mut competing = Vec::new();
    let mut competing_undefined = 0usize;
    for outcome in outcomes
        .iter()
        .filter(|o| o.trial.family == Family::Competing)
    {
        match (core(outcome), alt(outcome)) {
            (Some(long_rare), Some(short_common)) => competing.push(Scored {
                value: long_rare - short_common,
                outcome: outcome.clone(),
            }),
            _ => competing_undefined += 1,
        }
    }

    vec![
        assemble(
            Family::Informative,
            statistic,
            "Δz on adding one shared rare boundary event",
            "Δz > 0 even where raw distance worsens",
            informative,
            informative_undefined,
            true,
        ),
        assemble(
            Family::Noise,
            statistic,
            "Δz on adding one unrelated boundary event",
            "no systematic Δz > 0",
            noise,
            noise_undefined,
            false,
        ),
        assemble(
            Family::Rare,
            statistic,
            "Δz(rare) − Δz(common), matched pairs",
            "rare boundary carries more evidence than a ubiquitous one",
            rarity,
            rarity_undefined,
            true,
        ),
        assemble(
            Family::Redundant,
            statistic,
            "Δz(novel) − Δz(redundant), matched pairs",
            "a novel mark carries more evidence than one the core repeats",
            redundancy,
            redundancy_undefined,
            true,
        ),
        assemble(
            Family::Accidental,
            statistic,
            "z(planted core) − z(best chance match), matched pairs",
            "a coincidence attracts less evidence than a planted figure",
            accident,
            accident_undefined,
            true,
        ),
        assemble(
            Family::Diluted,
            statistic,
            "best-z candidate overlaps the planted core (+1) or not (−1)",
            "the surprising region stays on the motif as context grows",
            dilution,
            dilution_undefined,
            true,
        ),
        assemble(
            Family::Competing,
            statistic,
            "z(longer, rarer, imperfect) − z(shorter, tighter, common)",
            "evidential weight beats raw fit",
            competing,
            competing_undefined,
            true,
        ),
    ]
}
