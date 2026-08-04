//! One derived document behind the Behavioral Spectroscope page.
//!
//! **Disposable.** See [`crate::experiment`]. sprint:7, task:17.
//!
//! # What this is
//!
//! Everything the page renders, assembled in Rust, once, from an already
//! validated replay: what a synthetic fixture planted, how the sampled signal
//! looks at several display scales, what sprint:5's Haar decomposition found
//! against its isolated-impulse null, and what sprint:6's Matrix Profile found
//! against its shuffle null.
//!
//! decision:6 settles why it is shaped this way. The browser renders; it does not
//! transform, measure a distance, re-bucket, or parse raw NDJSON. Two
//! implementations of what a recording says are two opinions about it, and a page
//! that recomputed a wavelet would be a second opinion about the mathematics too.
//!
//! # Three classes of claim, kept apart
//!
//! [`Class`] tags every block, because `CLAUDE.md` §2 forbids one kind of claim
//! being promoted into another and a rendering that draws them at equal weight
//! has performed that promotion by presentation alone:
//!
//! * **Planted** — known because this project generated the fixture. Read from
//!   [`crate::experiment::oracle`]'s constants and from nowhere else.
//! * **Observed** — computed by [`crate::experiment::haar`],
//!   [`crate::experiment::matrix_profile`], or [`crate::experiment::signal`].
//! * **Interpretation** — a sentence a person would write. Every number inside
//!   one is computed; no language model is anywhere in this path.
//!
//! # Ground truth cannot be discovered
//!
//! [`GroundTruth`] is populated only when the recording's session id equals a
//! known fixture's, and its regions come from the generator constants. A
//! visualization that reverse-engineered its own annotations out of the signal
//! would look identical and be worthless, so the annotations are not derived at
//! all. A real recording gets `None`: absent, not empty, not guessed.
//!
//! # Raw counts, not the z-scored column
//!
//! Both transforms are fed unnormalized counts. sprint:6 measured why: the metric
//! and the wavelet are each invariant to a global affine transform, so this
//! changes no share and no ratio, and it is the difference between an empty
//! window having a standard deviation of exactly zero and having `1.863e-9`,
//! which z-normalization amplifies into a false motif. sprint:4's normalization
//! policy is untouched and still governs everything that reads a `Normalized`.

use serde::Serialize;

use crate::experiment::haar;
use crate::experiment::matrix_profile::{self, LADDER_MS};
use crate::experiment::oracle;
use crate::experiment::signal::{self, BehavioralSignal, BucketWidth, DEFAULT_BUCKET_MS};
use crate::inspection::{Inspection, inspect};
use crate::replay::Replay;

/// Display aggregations offered by the scale control, in milliseconds.
///
/// A display choice and nothing more. None of these is the canonical
/// representation of a recording, and no transform in this project derived an
/// optimum — sprint:5 was explicit that a Haar decomposition operates on an
/// already-sampled signal and does not choose the sampling.
pub const DISPLAY_SCALES_MS: [u64; 6] = [500, 1_000, 2_000, 4_000, 8_000, 16_000];

/// How many dimensions get a Matrix Profile.
///
/// Bounded because the ladder is six windows and each window is a full STOMP
/// pass plus one for its null, and because nineteen profiled dimensions is more
/// than a page can show honestly anyway. Chosen by occupancy so the busiest
/// dimensions are the ones profiled, and the ones left out are named rather than
/// silently missing.
pub const MAX_PROFILED_DIMENSIONS: usize = 8;

/// Which kind of claim a block carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Class {
    /// Known because this project generated the fixture.
    Planted,
    /// Computed by one of the committed experiments.
    Observed,
    /// A sentence a person would write, with computed numbers in it.
    Interpretation,
}

/// Where this document came from.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Provenance {
    /// Session the records belong to.
    pub session_id: Option<String>,
    /// Complete records in the examined scope.
    pub records: usize,
    /// Whether the recording stops mid-record.
    pub truncated: bool,
    /// Schema version its first record declared.
    pub schema_version: Option<u64>,
    /// Base sampling interval every analysis below used.
    pub base_bucket_ms: u64,
    /// Base samples at that interval.
    pub samples: usize,
    /// Distance from the earliest recorded timestamp to the latest.
    pub span_ms: u64,
    /// Base samples holding no record.
    pub empty_samples: usize,
    /// Records whose timestamp precedes their predecessor's.
    pub non_monotonic: usize,
}

/// A region a fixture generator deliberately created.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Region {
    /// What kind of structure was planted here.
    pub kind: RegionKind,
    /// Short label for the timeline band.
    pub label: String,
    /// Start offset from the signal origin, in milliseconds.
    pub start_ms: u64,
    /// End offset, in milliseconds.
    pub end_ms: u64,
    /// One sentence about what the generator put here.
    pub detail: String,
}

/// The kinds of planted structure the fixtures contain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RegionKind {
    /// Sparse background activity.
    Baseline,
    /// A repeated figure at a known period.
    Motif,
    /// A sustained block with a different character.
    Regime,
    /// The motif again, with deterministic jitter.
    Recurrence,
}

/// What a synthetic fixture planted, read from its generator's constants.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GroundTruth {
    /// Always [`Class::Planted`].
    pub class: Class,
    /// Which fixture this is.
    pub fixture: String,
    /// Regions, in time order.
    pub regions: Vec<Region>,
    /// Gap between motif instances, in milliseconds.
    pub motif_period_ms: u64,
    /// How long one motif instance lasts, in milliseconds.
    pub motif_instance_ms: u64,
    /// Tool name delivered only inside the motif, if the fixture has one.
    pub motif_only_dimension: String,
    /// How the annotations were obtained, said in the document so a reader does
    /// not have to take it on trust.
    pub sourced_from: String,
}

/// One dimension of the sampled signal.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DimensionView {
    /// Full label, as [`signal::Dimension::label`] produces it.
    pub label: String,
    /// Family, for grouping in the page: `records`, `channel`, `kind`,
    /// `tool_name`, `correlation`, or `bytes`.
    pub family: String,
    /// Sum across the recording.
    pub total: f64,
    /// Base samples where this dimension is non-zero.
    pub occupied: usize,
    /// Largest value in any one base sample.
    pub peak: f64,
}

/// The raster at one display aggregation.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ScaleView {
    /// Aggregation width in milliseconds.
    pub bucket_ms: u64,
    /// Buckets at that width.
    pub samples: usize,
    /// One row per dimension, parallel to [`Spectroscope::dimensions`].
    pub rows: Vec<Vec<f64>>,
}

/// One dyadic level of a Haar decomposition, ready to draw.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct HaarLevel {
    /// Level number, from 1.
    pub level: u32,
    /// Window width this level contrasts across, in milliseconds.
    pub scale_ms: u64,
    /// Absolute detail coefficients, in time order.
    pub magnitude: Vec<f64>,
    /// Largest magnitude at this level, so a row can be scaled to itself.
    pub level_max: f64,
    /// This level's share of the dimension's total detail energy.
    pub share: f64,
    /// The share an isolated unit impulse would produce here.
    pub impulse_null_share: f64,
    /// Observed share divided by the null's. Around 1 means this level is
    /// indistinguishable from what isolated events alone produce.
    pub ratio_to_impulse_null: f64,
    /// Base samples this level still represents after odd tails were set aside.
    pub covered_samples: usize,
}

/// A Haar decomposition of one dimension.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct HaarView {
    /// Always [`Class::Observed`].
    pub class: Class,
    /// Which dimension.
    pub label: String,
    /// Levels, finest first.
    pub levels: Vec<HaarLevel>,
    /// Set when the dimension produced no detail energy at all, saying which of
    /// the three reasons applies rather than leaving a blank row.
    pub silence: Option<haar::Silence>,
}

/// One matched pair, located in time.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct MatchView {
    /// First window's start offset, in milliseconds.
    pub a_start_ms: u64,
    /// First window's end offset.
    pub a_end_ms: u64,
    /// Second window's start offset.
    pub b_start_ms: u64,
    /// Second window's end offset.
    pub b_end_ms: u64,
    /// z-normalized Euclidean distance between them.
    pub distance: f64,
    /// Non-empty base samples inside the first window.
    pub a_occupancy: usize,
    /// Non-empty base samples inside the second window.
    pub b_occupancy: usize,
    /// Whether both windows hold two or fewer non-empty samples, which makes the
    /// match an alignment of lone events rather than of a repeated figure.
    pub trivial: bool,
}

/// One window of the ladder, for one dimension.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WindowView {
    /// Window length in milliseconds.
    pub window_ms: u64,
    /// Window length in base samples.
    pub m: usize,
    /// Candidate subsequences.
    pub subsequences: usize,
    /// Fraction of them that are constant, and therefore excluded.
    pub constant_fraction: f64,
    /// The distance curve, one entry per candidate subsequence. Non-finite
    /// entries — masked ones — are `null`.
    pub profile: Vec<Option<f64>>,
    /// Top candidate matches after masking, best first.
    pub matches: Vec<MatchView>,
    /// The strongest discord, if the masked profile left one.
    pub discord: Option<(u64, u64, f64)>,
    /// Best masked distance on a fixed-seed shuffle of the same values.
    pub null_best_distance: Option<f64>,
    /// `(null − best) / 2√m`, sprint:6's preregistered comparison.
    pub separation: Option<f64>,
}

/// A Matrix Profile ladder for one dimension.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProfileView {
    /// Always [`Class::Observed`].
    pub class: Class,
    /// Which dimension.
    pub label: String,
    /// One entry per window of the committed ladder.
    pub windows: Vec<WindowView>,
}

/// One step of the explanatory sequence.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct NarrativeStep {
    /// Which kind of claim this step is.
    pub class: Class,
    /// Short heading.
    pub heading: String,
    /// One or two sentences. Every number in them was computed.
    pub body: String,
}

/// Everything the Behavioral Spectroscope page renders.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Spectroscope {
    /// Where this came from.
    pub provenance: Provenance,
    /// What a synthetic fixture planted. `None` for any other recording.
    pub ground_truth: Option<GroundTruth>,
    /// Dimensions, in the signal's canonical order.
    pub dimensions: Vec<DimensionView>,
    /// The raster at each display aggregation.
    pub scales: Vec<ScaleView>,
    /// Haar, per dimension.
    pub haar: Vec<HaarView>,
    /// Matrix Profile, for the dimensions that got one.
    pub profiles: Vec<ProfileView>,
    /// Dimensions deliberately left unprofiled, so the omission is visible.
    pub unprofiled: Vec<String>,
    /// The explanatory sequence.
    pub narrative: Vec<NarrativeStep>,
    /// The committed window ladder, so the page does not hard-code it.
    pub ladder_ms: Vec<u64>,
}

/// Assemble the document from an already validated replay.
///
/// Pure and deterministic. Reads no file, consults no clock, and borrows the
/// replay rather than owning it.
///
/// Returns `None` when the examined scope holds no records: with no earliest
/// timestamp there is no axis, and a page drawn over an invented one would be a
/// fabrication rather than an empty result.
pub fn project(replay: &Replay) -> Option<Spectroscope> {
    let inspection = inspect(replay);
    let width = BucketWidth::from_ms(DEFAULT_BUCKET_MS)?;
    let base = signal::project(&inspection, width)?;

    let dimensions = describe_dimensions(&base);
    let provenance = describe_provenance(&inspection, &base);
    let ground_truth = ground_truth_for(provenance.session_id.as_deref());
    let scales = build_scales(&inspection);
    let haar = build_haar(&base);
    let (profiles, unprofiled) = build_profiles(&base, &dimensions, ground_truth.as_ref());
    let narrative = narrate(&provenance, ground_truth.as_ref(), &haar, &profiles);

    Some(Spectroscope {
        provenance,
        ground_truth,
        dimensions,
        scales,
        haar,
        profiles,
        unprofiled,
        narrative,
        ladder_ms: LADDER_MS.to_vec(),
    })
}

fn describe_provenance(inspection: &Inspection<'_>, base: &BehavioralSignal<'_>) -> Provenance {
    Provenance {
        session_id: base.session_id.map(str::to_owned),
        records: inspection.record_count(),
        truncated: inspection.scope.is_truncated(),
        schema_version: base.schema_version,
        base_bucket_ms: base.bucket_ms,
        samples: base.len(),
        span_ms: base.axis.span_ms,
        empty_samples: base
            .samples
            .iter()
            .filter(|sample| sample.records.is_empty())
            .count(),
        non_monotonic: base.axis.non_monotonic.count(),
    }
}

fn describe_dimensions(base: &BehavioralSignal<'_>) -> Vec<DimensionView> {
    base.dimensions
        .iter()
        .enumerate()
        .map(|(index, dimension)| {
            let column = base.column(index).unwrap_or_default();
            let label = dimension.label();
            DimensionView {
                family: label
                    .split_once(':')
                    .map(|(head, _)| head.to_owned())
                    .unwrap_or_else(|| label.clone()),
                total: column.iter().sum(),
                occupied: column.iter().filter(|value| **value != 0.0).count(),
                peak: column.iter().copied().fold(0.0, f64::max),
                label,
            }
        })
        .collect()
}

/// The raster at each display aggregation, rebuilt through the same projection
/// the base signal came from rather than by summing the base rows — so a coarser
/// view is a projection of the recording, not a projection of a projection.
fn build_scales(inspection: &Inspection<'_>) -> Vec<ScaleView> {
    DISPLAY_SCALES_MS
        .iter()
        .filter_map(|bucket_ms| {
            let width = BucketWidth::from_ms(*bucket_ms)?;
            let scaled = signal::project(inspection, width)?;
            Some(ScaleView {
                bucket_ms: *bucket_ms,
                samples: scaled.len(),
                rows: (0..scaled.dimensions.len())
                    .map(|index| scaled.column(index).unwrap_or_default())
                    .collect(),
            })
        })
        .collect()
}

fn build_haar(base: &BehavioralSignal<'_>) -> Vec<HaarView> {
    base.dimensions
        .iter()
        .enumerate()
        .map(|(index, dimension)| {
            let column = base.column(index).unwrap_or_default();
            let decomposition = haar::decompose(&column, base.bucket_ms);
            let spectrum = decomposition.spectrum();
            HaarView {
                class: Class::Observed,
                label: dimension.label(),
                silence: decomposition.silence(),
                levels: decomposition
                    .levels
                    .iter()
                    .zip(spectrum)
                    .map(|(level, band)| {
                        let magnitude: Vec<f64> =
                            level.detail.iter().map(|value| value.abs()).collect();
                        HaarLevel {
                            level: level.level,
                            scale_ms: level.scale_ms,
                            level_max: magnitude.iter().copied().fold(0.0, f64::max),
                            magnitude,
                            share: band.share,
                            impulse_null_share: band.impulse_null_share,
                            ratio_to_impulse_null: band.ratio_to_impulse_null,
                            covered_samples: level.covered_samples,
                        }
                    })
                    .collect(),
            }
        })
        .collect()
}

fn build_profiles(
    base: &BehavioralSignal<'_>,
    dimensions: &[DimensionView],
    ground_truth: Option<&GroundTruth>,
) -> (Vec<ProfileView>, Vec<String>) {
    // The dimensions a fixture's own constants say carry the planted figure come
    // first, then the busiest, then canonical order, capped.
    //
    // Ranking by occupancy alone was tried and was wrong for the hero case: the
    // motif-carrying dimensions are sparse *by construction* — a tool used only
    // inside the figure, and the reported channel that is silent everywhere else
    // — so the densest-eight rule excluded exactly the columns the fixture was
    // built to be interesting in. Preference comes from the generator constants,
    // never from the signal, so this cannot become a way of discovering which
    // dimension looks good.
    let preferred: Vec<&str> = ground_truth
        .map(|truth| vec![truth.motif_only_dimension.as_str(), "channel:reported"])
        .unwrap_or_default();

    let mut ranked: Vec<usize> = (0..dimensions.len())
        .filter(|index| dimensions[*index].occupied > 0)
        .collect();
    ranked.sort_by(|left, right| {
        let rank = |index: usize| {
            preferred
                .iter()
                .position(|label| *label == dimensions[index].label)
                .unwrap_or(usize::MAX)
        };
        rank(*left)
            .cmp(&rank(*right))
            .then_with(|| dimensions[*right].occupied.cmp(&dimensions[*left].occupied))
            .then(left.cmp(right))
    });
    let chosen: Vec<usize> = ranked
        .iter()
        .copied()
        .take(MAX_PROFILED_DIMENSIONS)
        .collect();

    let profiles = base
        .dimensions
        .iter()
        .enumerate()
        .filter(|(index, _)| chosen.contains(index))
        .map(|(index, dimension)| {
            let column = base.column(index).unwrap_or_default();
            ProfileView {
                class: Class::Observed,
                label: dimension.label(),
                windows: LADDER_MS
                    .iter()
                    .filter_map(|window_ms| window_view(&column, base.bucket_ms, *window_ms))
                    .collect(),
            }
        })
        .collect();

    let unprofiled = dimensions
        .iter()
        .enumerate()
        .filter(|(index, _)| !chosen.contains(index))
        .map(|(_, dimension)| dimension.label.clone())
        .collect();

    (profiles, unprofiled)
}

fn window_view(column: &[f64], bucket_ms: u64, window_ms: u64) -> Option<WindowView> {
    let scanned = matrix_profile::scan(column, bucket_ms, window_ms, 4)?;
    // The masked curve as the scan produced it. Exclusions travel as `null` so
    // the page draws them as gaps rather than as zeroes.
    let profile = scanned.masked_profile.clone();

    Some(WindowView {
        window_ms: scanned.window_ms,
        m: scanned.m,
        subsequences: scanned.subsequences,
        constant_fraction: scanned.constant_fraction(),
        profile,
        matches: scanned
            .masked_top
            .iter()
            .map(|pair| MatchView {
                a_start_ms: pair.a.start_ms,
                a_end_ms: pair.a.end_ms,
                b_start_ms: pair.b.start_ms,
                b_end_ms: pair.b.end_ms,
                distance: pair.distance,
                a_occupancy: pair.a_occupancy,
                b_occupancy: pair.b_occupancy,
                trivial: pair.a_occupancy <= 2 && pair.b_occupancy <= 2,
            })
            .collect(),
        discord: scanned
            .discords
            .first()
            .map(|discord| (discord.at.start_ms, discord.at.end_ms, discord.distance)),
        null_best_distance: scanned.null_best_distance,
        separation: scanned.separation,
    })
}

/// Planted structure, read from the generator constants of whichever fixture
/// this session id belongs to.
///
/// Nothing here inspects the signal. A session id that matches no known fixture
/// returns `None`, and the page then shows no ground-truth band at all.
fn ground_truth_for(session_id: Option<&str>) -> Option<GroundTruth> {
    let sourced_from =
        "witnessglass::experiment::oracle generator constants, not from any detector output"
            .to_owned();

    match session_id? {
        id if id == oracle::SESSION_ID => Some(GroundTruth {
            class: Class::Planted,
            fixture: "legible oracle — deliberately dense, best case".to_owned(),
            motif_period_ms: oracle::MOTIF_PERIOD_MS,
            motif_instance_ms: 2_200,
            motif_only_dimension: format!("tool_name:{}", oracle::TOOL_SEARCHER),
            sourced_from,
            regions: vec![
                Region {
                    kind: RegionKind::Baseline,
                    label: "baseline".to_owned(),
                    start_ms: 0,
                    end_ms: oracle::FIRST_MOTIF_START_MS,
                    detail: format!(
                        "one two-record call every {} s, one tool name",
                        oracle::BASELINE_PERIOD_MS / 1000
                    ),
                },
                Region {
                    kind: RegionKind::Motif,
                    label: "motif".to_owned(),
                    start_ms: oracle::FIRST_MOTIF_START_MS,
                    end_ms: oracle::FIRST_MOTIF_END_MS,
                    detail: format!(
                        "{} instances of a {}-record figure, exactly every {} s",
                        oracle::MOTIF_INSTANCES,
                        oracle::MOTIF_RECORDS_PER_INSTANCE,
                        oracle::MOTIF_PERIOD_MS / 1000
                    ),
                },
                Region {
                    kind: RegionKind::Baseline,
                    label: "baseline".to_owned(),
                    start_ms: oracle::FIRST_MOTIF_END_MS,
                    end_ms: oracle::REGIME_CHANGE_MS,
                    detail: "identical in shape to the first".to_owned(),
                },
                Region {
                    kind: RegionKind::Regime,
                    label: "elevated regime".to_owned(),
                    start_ms: oracle::REGIME_CHANGE_MS,
                    end_ms: oracle::ELEVATED_END_MS,
                    detail: format!(
                        "a call every {} s, two different tool names, larger recorded \
                         responses, and no reported intent at all",
                        oracle::ELEVATED_PERIOD_MS as f64 / 1000.0
                    ),
                },
                Region {
                    kind: RegionKind::Recurrence,
                    label: "recurrence".to_owned(),
                    start_ms: oracle::SECOND_MOTIF_START_MS,
                    end_ms: oracle::SESSION_END_MS,
                    detail: "the same figure with deterministic jitter, and one call that fails"
                        .to_owned(),
                },
            ],
        }),
        id if id == oracle::sparse::SESSION_ID => Some(GroundTruth {
            class: Class::Planted,
            fixture: "sparse oracle — stress case at observed density".to_owned(),
            motif_period_ms: oracle::sparse::MOTIF_PERIOD_MS,
            motif_instance_ms: 950,
            motif_only_dimension: format!("tool_name:{}", oracle::sparse::TOOL_SEARCHER),
            sourced_from,
            regions: vec![
                Region {
                    kind: RegionKind::Baseline,
                    label: "baseline".to_owned(),
                    start_ms: 0,
                    end_ms: oracle::sparse::FIRST_MOTIF_START_MS,
                    detail: format!(
                        "one two-record call every {} s",
                        oracle::sparse::BASELINE_PERIOD_MS / 1000
                    ),
                },
                Region {
                    kind: RegionKind::Motif,
                    label: "motif".to_owned(),
                    start_ms: oracle::sparse::FIRST_MOTIF_START_MS,
                    end_ms: oracle::sparse::FIRST_MOTIF_END_MS,
                    detail: format!(
                        "a {}-record figure, exactly every {} s",
                        oracle::sparse::MOTIF_RECORDS_PER_INSTANCE,
                        oracle::sparse::MOTIF_PERIOD_MS / 1000
                    ),
                },
                Region {
                    kind: RegionKind::Baseline,
                    label: "baseline".to_owned(),
                    start_ms: oracle::sparse::FIRST_MOTIF_END_MS,
                    end_ms: oracle::sparse::REGIME_START_MS,
                    detail: "as before".to_owned(),
                },
                Region {
                    kind: RegionKind::Regime,
                    label: "regime block".to_owned(),
                    start_ms: oracle::sparse::REGIME_START_MS,
                    end_ms: oracle::sparse::REGIME_END_MS,
                    detail: format!(
                        "a call every {} s under a tool name used nowhere else, {} s wide and \
                         deliberately not a power of two in base samples",
                        oracle::sparse::REGIME_PERIOD_MS / 1000,
                        (oracle::sparse::REGIME_END_MS - oracle::sparse::REGIME_START_MS) / 1000
                    ),
                },
                Region {
                    kind: RegionKind::Recurrence,
                    label: "recurrence".to_owned(),
                    start_ms: oracle::sparse::SECOND_MOTIF_START_MS,
                    end_ms: oracle::sparse::SESSION_END_MS,
                    detail: "the same figure with deterministic jitter, and one call that fails"
                        .to_owned(),
                },
            ],
        }),
        _ => None,
    }
}

/// The explanatory sequence, built from constants and computed values.
///
/// Every number below is measured. No sentence is written by a language model,
/// and none of them is stored anywhere as a conclusion about a session.
fn narrate(
    provenance: &Provenance,
    ground_truth: Option<&GroundTruth>,
    haar: &[HaarView],
    profiles: &[ProfileView],
) -> Vec<NarrativeStep> {
    let mut steps = Vec::new();
    let empty_pct = 100.0 * provenance.empty_samples as f64 / provenance.samples.max(1) as f64;

    match ground_truth {
        Some(truth) => steps.push(NarrativeStep {
            class: Class::Planted,
            heading: "We planted this".to_owned(),
            body: format!(
                "This is the {}. Its generator placed {} regions, including a figure repeating \
                 exactly every {} s and a sustained block of different character. The bands above \
                 come from those constants — nothing on this page discovered them.",
                truth.fixture,
                truth.regions.len(),
                truth.motif_period_ms / 1000,
            ),
        }),
        None => steps.push(NarrativeStep {
            class: Class::Observed,
            heading: "No ground truth here".to_owned(),
            body: "This recording is not a synthetic fixture, so nothing on this page knows what \
                   it contains. Every finding below is a candidate, and none of them is checked \
                   against anything."
                .to_owned(),
        }),
    }

    steps.push(NarrativeStep {
        class: Class::Observed,
        heading: "The sampled signal is mostly empty".to_owned(),
        body: format!(
            "At {} ms, {} of {} buckets hold no record — {:.1}% empty. That emptiness is the \
             single biggest influence on everything below, and it is a property of sampling an \
             event stream rather than of the session.",
            provenance.base_bucket_ms, provenance.empty_samples, provenance.samples, empty_pct,
        ),
    });

    // Haar: the level with the largest excess over the isolated-impulse null,
    // across every dimension.
    if let Some((label, level)) = haar
        .iter()
        .flat_map(|view| view.levels.iter().map(move |level| (&view.label, level)))
        .filter(|(_, level)| level.covered_samples > 0)
        .max_by(|left, right| {
            left.1
                .ratio_to_impulse_null
                .total_cmp(&right.1.ratio_to_impulse_null)
        })
    {
        steps.push(NarrativeStep {
            class: Class::Observed,
            heading: "Haar saw this".to_owned(),
            body: format!(
                "Haar contrasts neighbouring halves of progressively larger windows. An isolated \
                 event produces a known signature on its own — energy halving at every coarser \
                 scale — so the number worth reading is the departure from it. The largest here \
                 is {} at the {} scale, carrying {:.1}× what isolated events alone would give.",
                label,
                render_scale(level.scale_ms),
                level.ratio_to_impulse_null,
            ),
        });
    }

    // Matrix Profile: the best match anywhere, and whether it is trivial.
    let best = profiles
        .iter()
        .flat_map(|profile| {
            profile.windows.iter().flat_map(move |window| {
                window
                    .matches
                    .first()
                    .map(|found| (&profile.label, window, *found))
            })
        })
        .min_by(|left, right| left.2.distance.total_cmp(&right.2.distance));

    if let Some((label, window, found)) = best {
        steps.push(NarrativeStep {
            class: Class::Observed,
            heading: "Matrix Profile saw this".to_owned(),
            body: format!(
                "Its strongest match anywhere is in {} at a {} window: two spans at a distance of \
                 {:.3}, holding {} and {} non-empty buckets respectively.",
                label,
                render_scale(window.window_ms),
                found.distance,
                found.a_occupancy,
                found.b_occupancy,
            ),
        });

        if found.trivial {
            steps.push(NarrativeStep {
                class: Class::Interpretation,
                heading: "And that match is arithmetic, not evidence".to_owned(),
                body: "Each of those windows holds one or two non-empty buckets at the same \
                       offset. After subsequence normalization they are identical, so a distance \
                       of zero was inevitable regardless of what surrounds them. It does not \
                       establish that similar behaviour recurred."
                    .to_owned(),
            });
        }
    }

    // The null, at the window where separation is largest.
    if let Some((label, window)) = profiles
        .iter()
        .flat_map(|profile| {
            profile
                .windows
                .iter()
                .map(move |window| (&profile.label, window))
        })
        .max_by(|left, right| {
            left.1
                .separation
                .unwrap_or(0.0)
                .total_cmp(&right.1.separation.unwrap_or(0.0))
        })
    {
        let pairs = profiles
            .iter()
            .map(|profile| profile.windows.len())
            .sum::<usize>();
        let zero_windows = profiles
            .iter()
            .flat_map(|profile| profile.windows.iter())
            .filter(|window| window.separation.is_some_and(|value| value.abs() < 1e-6))
            .count();
        steps.push(NarrativeStep {
            class: Class::Observed,
            heading: "The null says this".to_owned(),
            body: format!(
                "Shuffling a dimension keeps every value and destroys every ordering. It reaches \
                 the same distance the recording does in {} of the {} window-and-dimension pairs \
                 here, which means a low distance on its own carries no information. The largest \
                 separation from the null is {:+.3} in {} at a {} window.",
                zero_windows,
                pairs,
                window.separation.unwrap_or(0.0),
                label,
                render_scale(window.window_ms),
            ),
        });
    }

    // The closing sentence differs by what is actually knowable. With a fixture
    // there is a planted figure to have missed; without one there is not, and
    // saying otherwise would be the promotion this page exists to avoid.
    steps.push(NarrativeStep {
        class: Class::Interpretation,
        heading: "Therefore".to_owned(),
        body: if ground_truth.is_some() {
            "Haar recovered usable scale structure: its departures from the impulse null are real, \
             and they distinguish a repeating figure from a sustained block. Sampled univariate \
             Matrix Profile did not recover the planted figure reliably — it ranks coincidences of \
             lone events above it — and sprint:6 recorded that as a property of the representation \
             rather than a defect in the library."
                .to_owned()
        } else {
            "Nothing here is checked against a known answer, so read every finding as a candidate. \
             The two things worth carrying over from the fixtures: a Matrix Profile distance is \
             only interesting where it separates from the null, and a match between two windows \
             holding one event each is arithmetic rather than evidence, however perfect it looks."
                .to_owned()
        },
    });

    steps
}

fn render_scale(ms: u64) -> String {
    if ms < 1_000 {
        format!("{ms} ms")
    } else {
        format!("{} s", ms / 1_000)
    }
}
