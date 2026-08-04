//! The sprint:5 Haar decomposition, and the sparse companion oracle.
//!
//! **Disposable.** These tests are deleted with the experiment.
//!
//! **What they check.** That the transform is the transform: hand-checkable
//! coefficients, an exact energy identity, the documented odd-length policy, and
//! the two invariances the experiment's reading depends on. Then that the sparse
//! fixture is what it claims to be, and that the structural predictions recorded
//! in task:15 *before the transform was run* hold or do not.
//!
//! **What they do not check.** Matrix Profile, changepoint detection, or any
//! interpretation of a peak. A scale carrying energy is a fact about a number
//! series; what an agent was doing is not in evidence here and no test pretends
//! otherwise.

use witnessglass::experiment::haar;
use witnessglass::experiment::oracle;
use witnessglass::experiment::signal::{BucketWidth, DEFAULT_BUCKET_MS, project};
use witnessglass::inspection::inspect;
use witnessglass::replay_bytes;

const LEGIBLE: &str = "fixtures/synthetic-behavioral-oracle.ndjson";
const SPARSE: &str = "fixtures/synthetic-behavioral-oracle-sparse.ndjson";

/// Records the sparse oracle is built to contain.
const SPARSE_RECORDS: usize = 365;
/// Buckets the sparse oracle produces at the default width.
const SPARSE_BUCKETS: usize = 2401;

const SQRT_2: f64 = std::f64::consts::SQRT_2;
/// Tolerance for values that should agree exactly but pass through `f64`.
const EPSILON: f64 = 1e-12;

fn read(path: &str) -> Vec<u8> {
    std::fs::read(path).unwrap_or_else(|err| panic!("fixture {path} should be readable: {err}"))
}

fn width(ms: u64) -> BucketWidth {
    BucketWidth::from_ms(ms).expect("a non-zero width")
}

/// Replay bytes and pair them with a width, since the projection borrows the
/// replay and a helper cannot return the finished signal.
macro_rules! signal_over {
    ($bytes:expr, $width_ms:expr) => {{
        (
            replay_bytes($bytes).expect("the recording should replay"),
            $width_ms,
        )
    }};
}

// ---------------------------------------------------------------------------
// The transform, against vectors small enough to check by hand
// ---------------------------------------------------------------------------

#[test]
fn one_pair_decomposes_to_one_sum_and_one_difference() {
    let decomposition = haar::decompose(&[1.0, 0.0], 500);
    assert_eq!(decomposition.levels(), 1);

    let level = &decomposition.levels[0];
    assert_eq!(level.level, 1);
    assert_eq!(level.scale_ms, 1_000, "level 1 is a two-sample window");
    assert_eq!(level.contrast_ms, 500, "contrasting two 500 ms means");
    assert!((level.detail[0] - 1.0 / SQRT_2).abs() < EPSILON);
    assert!((level.energy - 0.5).abs() < EPSILON);

    assert!((decomposition.approximation[0] - 1.0 / SQRT_2).abs() < EPSILON);
    assert!((decomposition.approximation_energy - 0.5).abs() < EPSILON);
    assert!(decomposition.energy_identity_residual().abs() < EPSILON);
}

#[test]
fn a_constant_series_has_no_detail_at_any_scale() {
    let decomposition = haar::decompose(&[3.0; 8], 500);
    assert_eq!(decomposition.levels(), 3);
    for level in &decomposition.levels {
        assert!(
            level.detail.iter().all(|d| d.abs() < EPSILON),
            "a difference of equal values is zero"
        );
    }
    assert!(decomposition.detail_energy < EPSILON);
    // All the energy survives in the approximation. Nothing is lost by having
    // nothing to say.
    assert!((decomposition.approximation_energy - 8.0 * 9.0).abs() < 1e-9);
    assert!(decomposition.energy_identity_residual().abs() < 1e-9);

    // And the spectrum reads as "no variation" rather than as a flat one.
    for band in decomposition.spectrum() {
        assert_eq!(band.share, 0.0);
        assert_eq!(band.ratio_to_impulse_null, 0.0);
    }
}

#[test]
fn a_four_sample_vector_decomposes_exactly_as_written_out_by_hand() {
    // x = [1, 2, 3, 4]
    // L1: a = [3/√2, 7/√2]        d = [-1/√2, -1/√2]      E1 = 1
    // L2: a = [10/2] = [5]        d = [-4/2] = [-2]       E2 = 4
    // identity: 1 + 4 + 25 = 30 = 1 + 4 + 9 + 16
    let decomposition = haar::decompose(&[1.0, 2.0, 3.0, 4.0], 500);
    assert_eq!(decomposition.levels(), 2);

    let first = &decomposition.levels[0];
    assert!((first.detail[0] + 1.0 / SQRT_2).abs() < EPSILON);
    assert!((first.detail[1] + 1.0 / SQRT_2).abs() < EPSILON);
    assert!((first.energy - 1.0).abs() < EPSILON);

    let second = &decomposition.levels[1];
    assert_eq!(second.scale_ms, 2_000);
    assert!((second.detail[0] + 2.0).abs() < EPSILON);
    assert!((second.energy - 4.0).abs() < EPSILON);

    assert!((decomposition.approximation[0] - 5.0).abs() < EPSILON);
    assert!((decomposition.input_energy - 30.0).abs() < EPSILON);
    assert!(decomposition.energy_identity_residual().abs() < 1e-9);
}

#[test]
fn an_isolated_impulse_is_exactly_the_null_the_spectrum_is_read_against() {
    // The claim the whole experiment's reading rests on: a single impulse has
    // detail energy 2^-L at level L, and therefore a ratio-to-null of exactly 1
    // at every level. If this were not so, the null column would be decoration.
    for position in 0..8 {
        let mut samples = [0.0; 8];
        samples[position] = 1.0;
        let decomposition = haar::decompose(&samples, 500);

        for (index, level) in decomposition.levels.iter().enumerate() {
            let expected = 2f64.powi(-((index as i32) + 1));
            assert!(
                (level.energy - expected).abs() < EPSILON,
                "impulse at {position}, level {}: {} vs {expected}",
                level.level,
                level.energy
            );
        }
        for band in decomposition.spectrum() {
            assert!(
                (band.ratio_to_impulse_null - 1.0).abs() < 1e-9,
                "impulse at {position}, level {}: ratio {}",
                band.level,
                band.ratio_to_impulse_null
            );
        }
        assert!(decomposition.energy_identity_residual().abs() < 1e-9);
    }
}

#[test]
fn an_odd_tail_is_set_aside_with_its_energy_and_never_dropped() {
    // x = [1, 2, 3]
    // L1 pairs (1,2) only: d = [-1/√2] E = 0.5, a = [3/√2] E = 4.5
    // the 3 is unpaired: remainder, energy 9
    // identity: 0.5 + 4.5 + 9 = 14 = 1 + 4 + 9
    let decomposition = haar::decompose(&[1.0, 2.0, 3.0], 500);
    assert_eq!(decomposition.levels(), 1);
    assert!((decomposition.detail_energy - 0.5).abs() < EPSILON);

    let remainders = decomposition.remainders();
    assert_eq!(remainders.len(), 1);
    assert_eq!(remainders[0].level, 1);
    assert_eq!(remainders[0].index, 2);
    assert_eq!(remainders[0].value, 3.0);
    assert_eq!(remainders[0].energy, 9.0);

    assert!((decomposition.remainder_energy - 9.0).abs() < EPSILON);
    assert!((decomposition.approximation_energy - 4.5).abs() < EPSILON);
    assert!((decomposition.input_energy - 14.0).abs() < EPSILON);
    assert!(
        decomposition.energy_identity_residual().abs() < 1e-9,
        "nothing is padded and nothing is discarded, so the balance closes"
    );
}

#[test]
fn the_energy_identity_holds_for_every_awkward_length() {
    // The identity is the only exact check this module has, so it is checked
    // across every length that could break the pairing, not just a tidy one.
    for length in 0..=64usize {
        let samples: Vec<f64> = (0..length).map(|i| ((i * 7) % 13) as f64 - 6.0).collect();
        let decomposition = haar::decompose(&samples, 500);
        assert!(
            decomposition.energy_identity_residual().abs() < 1e-9,
            "length {length}: residual {}",
            decomposition.energy_identity_residual()
        );
        assert_eq!(decomposition.input_len, length);
    }
}

#[test]
fn an_input_too_short_to_pair_yields_no_levels_rather_than_an_invented_one() {
    let empty = haar::decompose(&[], 500);
    assert_eq!(empty.levels(), 0);
    assert_eq!(empty.input_energy, 0.0);
    assert!(empty.spectrum().is_empty());

    let single = haar::decompose(&[5.0], 500);
    assert_eq!(single.levels(), 0);
    assert_eq!(single.approximation, vec![5.0]);
    assert_eq!(single.approximation_energy, 25.0);
    assert!(single.energy_identity_residual().abs() < EPSILON);
}

#[test]
fn detail_coefficients_are_invariant_to_a_constant_offset() {
    // Half of why sprint:4's normalization policy cannot move a spectrum. The
    // difference of two values does not move when both move.
    let base: Vec<f64> = (0..16).map(|i| (i % 5) as f64).collect();
    let shifted: Vec<f64> = base.iter().map(|value| value + 1000.0).collect();

    let first = haar::decompose(&base, 500);
    let second = haar::decompose(&shifted, 500);
    for (a, b) in first.levels.iter().zip(second.levels.iter()) {
        for (x, y) in a.detail.iter().zip(b.detail.iter()) {
            assert!((x - y).abs() < 1e-9, "level {}: {x} vs {y}", a.level);
        }
    }
}

#[test]
fn energy_shares_are_invariant_to_a_constant_factor() {
    // The other half. Together these mean a z-score changes no share, and that a
    // dimension's magnitude cannot reach any other dimension's spectrum.
    let base: Vec<f64> = (0..32).map(|i| ((i * 3) % 7) as f64).collect();
    let scaled: Vec<f64> = base.iter().map(|value| value * 1_000.0).collect();

    let first = haar::decompose(&base, 500).spectrum();
    let second = haar::decompose(&scaled, 500).spectrum();
    for (a, b) in first.iter().zip(second.iter()) {
        assert!((a.share - b.share).abs() < 1e-9, "level {}", a.level);
        assert!((a.ratio_to_impulse_null - b.ratio_to_impulse_null).abs() < 1e-9);
    }
}

#[test]
fn the_reported_scale_is_the_window_and_the_half_window_travels_with_it() {
    let decomposition = haar::decompose(&vec![0.0; 1024], 500);
    let expected = [
        (1u32, 1_000u64, 500u64),
        (2, 2_000, 1_000),
        (3, 4_000, 2_000),
        (4, 8_000, 4_000),
        (5, 16_000, 8_000),
        (6, 32_000, 16_000),
    ];
    for (level, scale_ms, contrast_ms) in expected {
        let found = &decomposition.levels[(level - 1) as usize];
        assert_eq!(found.level, level);
        assert_eq!(found.scale_ms, scale_ms, "level {level} window");
        assert_eq!(found.contrast_ms, contrast_ms, "level {level} contrast");
    }
}

// ---------------------------------------------------------------------------
// The sparse companion fixture
// ---------------------------------------------------------------------------

#[test]
fn the_committed_sparse_fixture_is_exactly_what_the_generator_produces() {
    let committed = String::from_utf8(read(SPARSE)).expect("the fixture is UTF-8");
    assert_eq!(
        committed,
        oracle::sparse::ndjson(),
        "the committed sparse fixture has drifted from its generator; regenerate with: \
         cargo run --example behavioral-signal -- --emit-sparse-oracle > {SPARSE}"
    );
}

#[test]
fn the_sparse_fixture_is_synthetic_and_obviously_so() {
    let text = String::from_utf8(read(SPARSE)).expect("the fixture is UTF-8");
    assert!(text.contains(oracle::sparse::SESSION_ID));
    for line in text.lines() {
        assert!(
            line.contains("synthetic"),
            "every record should be self-evidently synthetic: {line}"
        );
    }
    for leak in [
        "/Users/",
        "/home/",
        "github.com",
        "witnessglass/.witnessglass",
    ] {
        assert!(!text.contains(leak), "fixture mentions {leak:?}");
    }
}

#[test]
fn the_sparse_fixture_is_as_empty_as_a_real_recording_and_the_legible_one_is_not() {
    // The whole reason the second fixture exists. sprint:4 measured a real
    // 234-record session at 94% empty at 500 ms and the legible oracle at 78%;
    // a detector validated only on the legible one has been validated on a best
    // case reality does not supply.
    let sparse = emptiness(SPARSE, DEFAULT_BUCKET_MS);
    assert!(
        (0.90..=0.95).contains(&sparse),
        "the sparse fixture should sit in the band the real recording motivates, got {sparse}"
    );

    let legible = emptiness(LEGIBLE, DEFAULT_BUCKET_MS);
    assert!(
        legible < 0.85,
        "the legible fixture is the best case and should stay visibly denser, got {legible}"
    );
    assert!(
        sparse - legible > 0.10,
        "the two fixtures must differ enough in density for the contrast to mean anything"
    );
}

/// Fraction of buckets holding no record.
fn emptiness(path: &str, bucket_ms: u64) -> f64 {
    let (replay, ms) = signal_over!(&read(path), bucket_ms);
    let inspection = inspect(&replay);
    let signal = project(&inspection, width(ms)).expect("the fixture has records");
    let empty = signal
        .samples
        .iter()
        .filter(|sample| sample.records.is_empty())
        .count();
    empty as f64 / signal.len() as f64
}

#[test]
fn the_sparse_fixture_replays_and_projects_like_any_other_recording() {
    let (replay, ms) = signal_over!(&read(SPARSE), DEFAULT_BUCKET_MS);
    let inspection = inspect(&replay);
    assert_eq!(inspection.schema_version, Some(2));
    assert_eq!(inspection.record_count(), SPARSE_RECORDS);
    assert!(!inspection.scope.is_truncated());
    assert!(
        inspection.anomalies.is_empty(),
        "a generated fixture can easily produce an unpaired lifecycle by accident: {:?}",
        inspection.anomalies
    );

    let signal = project(&inspection, width(ms)).expect("the fixture has records");
    assert_eq!(signal.len(), SPARSE_BUCKETS);
    assert_eq!(signal.axis.span_ms, oracle::sparse::SESSION_END_MS);
    let placed: usize = signal.samples.iter().map(|s| s.records.len()).sum();
    assert_eq!(placed, SPARSE_RECORDS, "conservation still holds");
}

#[test]
fn the_two_fixtures_cannot_be_confused_for_one_another() {
    let legible = String::from_utf8(read(LEGIBLE)).expect("UTF-8");
    let sparse = String::from_utf8(read(SPARSE)).expect("UTF-8");
    assert_ne!(oracle::SESSION_ID, oracle::sparse::SESSION_ID);
    assert!(!legible.contains(oracle::sparse::SESSION_ID));
    assert!(!sparse.contains(oracle::SESSION_ID));
    // Distinct tool vocabularies, so a spectrum printed without its header is
    // still attributable.
    for name in oracle::TOOL_NAMES {
        assert!(!sparse.contains(&format!("\"{name}\"")));
    }
}

// ---------------------------------------------------------------------------
// Decomposing a real projection
// ---------------------------------------------------------------------------

/// Every dimension of a fixture, decomposed independently at the default width.
macro_rules! spectra_of {
    ($signal:expr, $normalized:expr) => {{
        let mut out: Vec<(String, Vec<haar::Band>)> = Vec::new();
        for index in 0..$signal.dimensions.len() {
            let column = $normalized.column(index).expect("the column exists");
            out.push((
                $signal.dimensions[index].label(),
                haar::decompose(&column, $signal.bucket_ms).spectrum(),
            ));
        }
        out
    }};
}

#[test]
fn the_energy_identity_holds_over_every_dimension_of_both_fixtures() {
    for path in [LEGIBLE, SPARSE] {
        let (replay, ms) = signal_over!(&read(path), DEFAULT_BUCKET_MS);
        let inspection = inspect(&replay);
        let signal = project(&inspection, width(ms)).expect("records exist");
        let normalized = signal.normalize();

        for index in 0..signal.dimensions.len() {
            let column = normalized.column(index).expect("the column exists");
            let decomposition = haar::decompose(&column, signal.bucket_ms);
            assert!(
                decomposition.energy_identity_residual().abs() < 1e-6,
                "{path} {}: residual {}",
                signal.dimensions[index].label(),
                decomposition.energy_identity_residual()
            );
        }
    }
}

#[test]
fn the_round_one_normalization_policy_cannot_move_a_single_share() {
    // task:15 prediction 5, tested on real projections rather than on reasoning.
    // If this holds, the heavy-tailed dimension cannot contaminate any other
    // dimension's spectrum and this round's findings are independent of the
    // Round 1 normalization question entirely.
    for path in [LEGIBLE, SPARSE] {
        let (replay, ms) = signal_over!(&read(path), DEFAULT_BUCKET_MS);
        let inspection = inspect(&replay);
        let signal = project(&inspection, width(ms)).expect("records exist");
        let normalized = signal.normalize();

        for index in 0..signal.dimensions.len() {
            let raw = haar::decompose(
                &signal.column(index).expect("the column exists"),
                signal.bucket_ms,
            );
            let scored = haar::decompose(
                &normalized.column(index).expect("the column exists"),
                signal.bucket_ms,
            );
            for (a, b) in raw.spectrum().iter().zip(scored.spectrum().iter()) {
                assert!(
                    (a.share - b.share).abs() < 1e-9,
                    "{path} {} level {}: raw {} vs z-scored {}",
                    signal.dimensions[index].label(),
                    a.level,
                    a.share,
                    b.share
                );
            }
        }
    }
}

#[test]
fn excluding_the_heavy_tailed_dimension_changes_no_other_dimension_at_all() {
    // task:15's with/without comparison, computed rather than assumed. The
    // transform is applied per column, so this ought to be trivially true — and
    // "ought to be trivially true" is exactly the kind of claim this project
    // does not accept without a measurement.
    const HEAVY: &str = "recorded_response_json_bytes";

    for path in [LEGIBLE, SPARSE] {
        let (replay, ms) = signal_over!(&read(path), DEFAULT_BUCKET_MS);
        let inspection = inspect(&replay);
        let signal = project(&inspection, width(ms)).expect("records exist");
        let normalized = signal.normalize();

        let all = spectra_of!(signal, normalized);
        assert!(all.iter().any(|(label, _)| label == HEAVY), "{path} has it");

        let without: Vec<_> = all
            .iter()
            .filter(|(label, _)| label != HEAVY)
            .cloned()
            .collect();
        for (label, bands) in &without {
            let (_, reference) = all
                .iter()
                .find(|(other, _)| other == label)
                .expect("present in both");
            assert_eq!(
                bands, reference,
                "{path} {label}: dropping {HEAVY} must not move it by one bit"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// task:15's predictions, tested as written
//
// The predictions were recorded in task:15 before this transform was run, and
// they are not revised here. Where one held, the test says so; where one did
// not, the test asserts what is actually true and names the prediction it
// contradicts. Both outcomes are results.
// ---------------------------------------------------------------------------

/// Ratio-to-null per level for one dimension of one fixture, at the default
/// width. Panics rather than returning an option: every label used below is one
/// the fixture is asserted elsewhere to have.
macro_rules! ratios {
    ($path:expr, $label:expr) => {{
        let (replay, ms) = signal_over!(&read($path), DEFAULT_BUCKET_MS);
        let inspection = inspect(&replay);
        let signal = project(&inspection, width(ms)).expect("records exist");
        let normalized = signal.normalize();
        let index = signal
            .dimension_index($label)
            .unwrap_or_else(|| panic!("{} has no dimension {}", $path, $label));
        let column = normalized.column(index).expect("the column exists");
        let decomposition = haar::decompose(&column, signal.bucket_ms);
        (
            decomposition.silence(),
            decomposition
                .spectrum()
                .iter()
                .map(|band| band.ratio_to_impulse_null)
                .collect::<Vec<f64>>(),
        )
    }};
}

#[test]
fn prediction_4_holds_a_lone_impulse_tracks_the_null_exactly() {
    // The control that makes the null column readable rather than assumed. If
    // this drifted, every other reading in this round would be uninterpretable.
    for path in [LEGIBLE, SPARSE] {
        let (silence, ratios) = ratios!(path, "kind:v2:session_started");
        assert!(silence.is_none(), "{path}: the control has a spectrum");
        for (level, ratio) in ratios.iter().enumerate() {
            assert!(
                (ratio - 1.0).abs() < 0.02,
                "{path} level {}: a single record should sit on the null, got {ratio}",
                level + 1
            );
        }
    }
}

#[test]
fn prediction_3_holds_a_constant_dimension_has_no_spectrum_at_all() {
    for path in [LEGIBLE, SPARSE] {
        let (silence, ratios) = ratios!(path, "kind:v2:tool_denied");
        assert_eq!(
            silence,
            Some(haar::Silence::Empty),
            "{path}: a kind the recording contains none of is zero everywhere"
        );
        assert!(ratios.iter().all(|ratio| *ratio == 0.0));
    }
}

#[test]
fn prediction_1_holds_the_eight_second_motif_shows_as_a_cutoff_above_its_period() {
    // The prediction: energy through level 4 (8 s), then a ratio materially
    // below 1 from level 5 (16 s). Not a peak at the period — a cliff above it,
    // because at half-windows reaching the period both halves hold equal numbers
    // of instances and the difference cancels.
    for (path, motif_only_tool) in [
        (LEGIBLE, "tool_name:SyntheticSearcher"),
        (SPARSE, "tool_name:SparseSyntheticSearcher"),
    ] {
        for label in [
            "channel:reported",
            "kind:v2:reported_intent",
            motif_only_tool,
        ] {
            let (_, ratios) = ratios!(path, label);
            for (level, ratio) in ratios.iter().take(4).enumerate() {
                assert!(
                    *ratio > 0.6,
                    "{path} {label} level {}: energy should survive at and below the 8 s \
                     period, got {ratio}",
                    level + 1
                );
            }
            assert!(
                ratios[4] < 0.5,
                "{path} {label} level 5 (16 s): the period should cancel above 8 s, got {}",
                ratios[4]
            );
        }
    }
}

#[test]
fn prediction_2_holds_in_direction_and_the_data_sharpened_where() {
    // Predicted: a block produces a coarse-scale excess. Confirmed. What the run
    // added is *which* coarse scale: the excess sits where one half of the window
    // is inside the block and the other is outside — around twice the block
    // width — and a window that fits entirely inside the block cancels like any
    // other constant stretch.
    //
    // The sparse block is 300 s. Its excess lands at 512 s, and the 256 s level
    // that fits inside it is a deficit.
    let (_, block) = ratios!(SPARSE, "tool_name:SparseSyntheticShell");
    assert!(
        block[9] > 10.0,
        "512 s spans the 300 s block's edge and should carry a large excess, got {}",
        block[9]
    );
    assert!(
        block[8] < 0.5,
        "256 s fits inside the block and should cancel, got {}",
        block[8]
    );

    // The legible oracle's block is 60 s, and its excess lands coarser than the
    // motif's cutoff, in the 32-64 s range.
    let (_, bytes) = ratios!(LEGIBLE, "recorded_response_json_bytes");
    let coarse = bytes[5].max(bytes[6]);
    assert!(
        coarse > 3.0,
        "the 60 s regime should show as excess at 32-64 s, got {coarse}"
    );
}

#[test]
fn prediction_6_is_falsified_the_sparse_fixture_shows_a_sharper_signature_not_a_duller_one() {
    // Predicted: at ~93% empty the sparse oracle would show *smaller* departures
    // from the null than the legible one. It shows larger ones. Density was not
    // the limiting factor; how isolated a structure is from everything else in
    // the same dimension, and how long the recording is, mattered more.
    //
    // Recorded as a failed prediction rather than quietly dropped.
    let (_, legible) = ratios!(LEGIBLE, "kind:v2:reported_intent");
    let (_, sparse) = ratios!(SPARSE, "kind:v2:reported_intent");
    assert!(
        sparse[4] < legible[4],
        "the sparse motif cutoff should have been shallower if prediction 6 held: \
         sparse {} vs legible {}",
        sparse[4],
        legible[4]
    );
}

#[test]
fn the_falsification_condition_fixed_in_advance_is_not_met() {
    // task:15: "if every dimension of both fixtures produces a ratio-to-null
    // within roughly ±25% of 1 at every level, Haar has found the sparsity and
    // not the structure, and the recommendation is to stop." It did not.
    for path in [LEGIBLE, SPARSE] {
        let (replay, ms) = signal_over!(&read(path), DEFAULT_BUCKET_MS);
        let inspection = inspect(&replay);
        let signal = project(&inspection, width(ms)).expect("records exist");
        let normalized = signal.normalize();

        let departed = (0..signal.dimensions.len()).any(|index| {
            let column = normalized.column(index).expect("the column exists");
            let decomposition = haar::decompose(&column, signal.bucket_ms);
            decomposition.silence().is_none()
                && decomposition
                    .spectrum()
                    .iter()
                    .any(|band| !(0.75..=1.25).contains(&band.ratio_to_impulse_null))
        });
        assert!(
            departed,
            "{path}: nothing departed from the isolated-impulse decay, which is the \
             stop condition"
        );
    }
}

#[test]
fn the_odd_length_policy_costs_the_end_of_the_recording_and_says_so() {
    // Found by running rather than predicted. Both fixtures have odd sample
    // counts, so the final base sample is set aside at level 1 and reaches no
    // level at all — and in both, that sample is the only one a `session_ended`
    // dimension has. The transform reports this as distinct from a flat
    // dimension, which is the difference between its own limitation and a
    // property of the recording.
    for path in [LEGIBLE, SPARSE] {
        let (silence, _) = ratios!(path, "kind:v2:session_ended");
        assert_eq!(
            silence,
            Some(haar::Silence::OnlyInRemainders),
            "{path}: the last bucket's only record reaches no scale"
        );
    }

    // And coverage falls at coarse levels, so a coarse reading is about less of
    // the recording than a fine one. The legible oracle's 481 samples lose
    // nearly half by the coarsest level.
    let (replay, ms) = signal_over!(&read(LEGIBLE), DEFAULT_BUCKET_MS);
    let inspection = inspect(&replay);
    let signal = project(&inspection, width(ms)).expect("records exist");
    let decomposition = haar::decompose(&signal.column(0).expect("records column"), ms);
    let coarsest = decomposition.levels.last().expect("levels exist");
    let coverage = coarsest.covered_samples as f64 / decomposition.input_len as f64;
    assert!(
        coverage < 0.6,
        "the coarsest level of a 481-sample series covers {} of {} ({coverage:.3}), and a \
         coarse-scale reading is only about that much of the recording",
        coarsest.covered_samples,
        decomposition.input_len
    );
    assert_eq!(decomposition.levels[0].covered_samples, 480, "481 is odd");
}
