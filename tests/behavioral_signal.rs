//! The sprint:4 behavioral-signal substrate, against the oracle whose structure
//! is known before the projection runs.
//!
//! **Disposable.** These tests exist to catch the substrate being wrong, and they
//! are deleted with it.
//!
//! **What they check.** That the projection recovers the properties the oracle
//! fixture deliberately encodes — bucket geometry, dimension identity and order,
//! receipt conservation, motif periodicity, the regime boundary, the recurrence
//! and its injected noise — and that bucketing and normalization behave as
//! documented at every boundary the module claims to have defined.
//!
//! **What they do not check.** Whether any algorithm finds any of it. No Matrix
//! Profile, no changepoint search, no wavelet transform, and no assertion that
//! the structure is *detectable* — only that it is *present and preserved*. That
//! distinction is the whole point of doing this round before the next one.

use witnessglass::experiment::oracle;
use witnessglass::experiment::signal::{BucketWidth, DEFAULT_BUCKET_MS, Dimension, project};
use witnessglass::inspection::{ExaminedScope, inspect};
use witnessglass::record::{Channel, Provenance, v1, v2};
use witnessglass::replay_bytes;

const FIXTURE: &str = "fixtures/synthetic-behavioral-oracle.ndjson";

/// Records the oracle is built to contain.
const ORACLE_RECORDS: usize = 196;
/// Buckets the oracle produces at the default width: 240 s of span, inclusive of
/// the bucket the last record lands in.
const ORACLE_BUCKETS: usize = 481;

fn fixture_bytes() -> Vec<u8> {
    std::fs::read(FIXTURE)
        .unwrap_or_else(|err| panic!("fixture {FIXTURE} should be readable: {err}"))
}

/// Replay some recording bytes and pair them with the width the test wants.
///
/// A macro rather than a function because the projection borrows the inspection,
/// which borrows the replay: the replay has to be a local of the calling test, so
/// a helper cannot return the finished signal.
macro_rules! signal_over {
    ($bytes:expr, $width_ms:expr) => {{
        (
            replay_bytes($bytes).expect("the recording should replay"),
            $width_ms,
        )
    }};
}

/// Bucket width helper, since every call site knows its width is non-zero.
fn width(ms: u64) -> BucketWidth {
    BucketWidth::from_ms(ms).expect("a non-zero width")
}

// ---------------------------------------------------------------------------
// The fixture: synthetic, obviously so, and regenerable from committed code
// ---------------------------------------------------------------------------

#[test]
fn the_committed_fixture_is_exactly_what_the_generator_produces() {
    let committed = String::from_utf8(fixture_bytes()).expect("the fixture is UTF-8");
    assert_eq!(
        committed,
        oracle::ndjson(),
        "the committed oracle fixture has drifted from the generator that declares its \
         structure; regenerate it with: cargo run --example behavioral-signal -- \
         --emit-oracle > {FIXTURE}"
    );
}

#[test]
fn the_oracle_fixture_is_synthetic_and_obviously_so() {
    let text = String::from_utf8(fixture_bytes()).expect("the fixture is UTF-8");
    assert!(text.contains(oracle::SESSION_ID));
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
fn the_oracle_fixture_replays_as_a_complete_v2_recording() {
    let replay = replay_bytes(&fixture_bytes()).expect("the fixture should replay");
    let inspection = inspect(&replay);
    assert_eq!(inspection.schema_version, Some(2));
    assert_eq!(inspection.record_count(), ORACLE_RECORDS);
    assert!(!inspection.scope.is_truncated());
    // A generated fixture could easily have produced an unpaired or duplicated
    // lifecycle by accident. It did not.
    assert!(
        inspection.anomalies.is_empty(),
        "the oracle should be an ordinary well-formed recording: {:?}",
        inspection.anomalies
    );
}

// ---------------------------------------------------------------------------
// Bucket geometry
// ---------------------------------------------------------------------------

#[test]
fn buckets_are_contiguous_evenly_spaced_and_inclusive_of_the_last_record() {
    let (replay, ms) = signal_over!(&fixture_bytes(), DEFAULT_BUCKET_MS);
    let inspection = inspect(&replay);
    let signal = project(&inspection, width(ms)).expect("the oracle has records");

    assert_eq!(signal.bucket_ms, 500);
    assert_eq!(signal.len(), ORACLE_BUCKETS);
    assert_eq!(signal.axis.span_ms, oracle::SESSION_END_MS);
    // The last record lands exactly on a boundary, so the final bucket holds an
    // instant of evidence and 500 ms of width.
    assert_eq!(signal.axis.final_bucket_observed_ms, 0);
    assert_eq!(signal.axis.origin_sequence, 1);
    assert_eq!(signal.axis.latest_sequence, ORACLE_RECORDS as u64);
    assert_eq!(signal.axis.non_monotonic.count(), 0);

    for (index, sample) in signal.samples.iter().enumerate() {
        assert_eq!(sample.bucket, index, "buckets are dense and in order");
        assert_eq!(sample.offset_ms, index as u64 * 500, "even spacing");
        assert_eq!(
            sample.values.len(),
            signal.dimensions.len(),
            "every row is the full width of the matrix"
        );
    }
}

#[test]
fn every_record_lands_in_exactly_one_bucket_and_the_receipts_say_which() {
    let (replay, ms) = signal_over!(&fixture_bytes(), DEFAULT_BUCKET_MS);
    let inspection = inspect(&replay);
    let signal = project(&inspection, width(ms)).expect("the oracle has records");

    let mut placed: Vec<u64> = signal
        .samples
        .iter()
        .flat_map(|sample| sample.records.iter().copied())
        .collect();
    assert_eq!(
        placed.len(),
        ORACLE_RECORDS,
        "no record placed twice or lost"
    );
    placed.sort_unstable();
    let expected: Vec<u64> = (1..=ORACLE_RECORDS as u64).collect();
    assert_eq!(placed, expected, "the receipts are exactly the record set");

    for sample in &signal.samples {
        assert!(
            sample.records.windows(2).all(|pair| pair[0] < pair[1]),
            "receipts are collected in canonical order and stay ascending"
        );
        // The receipts and the total-records column are two derivations of the
        // same fact, so they have to agree.
        assert_eq!(sample.values[0], sample.records.len() as f64);
    }
}

#[test]
fn an_empty_bucket_is_present_with_zeros_rather_than_absent() {
    let (replay, ms) = signal_over!(&fixture_bytes(), DEFAULT_BUCKET_MS);
    let inspection = inspect(&replay);
    let signal = project(&inspection, width(ms)).expect("the oracle has records");

    // 1 s into the first baseline is between calls: a real interval with no
    // record in it.
    let quiet = &signal.samples[2];
    assert_eq!(quiet.offset_ms, 1_000);
    assert!(quiet.records.is_empty());
    assert!(quiet.values.iter().all(|value| *value == 0.0));

    let empty = signal
        .samples
        .iter()
        .filter(|sample| sample.records.is_empty())
        .count();
    assert!(
        empty > ORACLE_BUCKETS / 2,
        "a sparse recording should be mostly empty buckets, and this one is: {empty}"
    );
}

#[test]
fn bucket_width_is_a_parameter_and_zero_is_not_a_width() {
    assert!(BucketWidth::from_ms(0).is_none());
    assert_eq!(BucketWidth::default().ms(), DEFAULT_BUCKET_MS);

    let replay = replay_bytes(&fixture_bytes()).expect("the fixture should replay");
    let inspection = inspect(&replay);

    for (ms, buckets) in [(500u64, 481usize), (1_000, 241), (8_000, 31), (240_000, 2)] {
        let signal = project(&inspection, width(ms)).expect("the oracle has records");
        assert_eq!(signal.len(), buckets, "at {ms} ms");
        let placed: usize = signal.samples.iter().map(|s| s.records.len()).sum();
        assert_eq!(placed, ORACLE_RECORDS, "conservation holds at {ms} ms");
    }
}

// ---------------------------------------------------------------------------
// Dimensions: what they are, and what they refuse to be
// ---------------------------------------------------------------------------

#[test]
fn the_dimension_set_is_derived_from_the_recording_in_a_fixed_order() {
    let (replay, ms) = signal_over!(&fixture_bytes(), DEFAULT_BUCKET_MS);
    let inspection = inspect(&replay);
    let signal = project(&inspection, width(ms)).expect("the oracle has records");

    let labels: Vec<String> = signal.dimensions.iter().map(Dimension::label).collect();
    assert_eq!(
        labels,
        vec![
            "records",
            "channel:reported",
            "channel:observed",
            "channel:recorder",
            "kind:v2:session_started",
            "kind:v2:session_ended",
            "kind:v2:reported_intent",
            "kind:v2:tool_requested",
            "kind:v2:tool_succeeded",
            "kind:v2:tool_failed",
            "kind:v2:tool_denied",
            "kind:v2:subagent_started",
            "kind:v2:subagent_stopped",
            "tool_name:SyntheticReader",
            "tool_name:SyntheticSearcher",
            "tool_name:SyntheticEditor",
            "tool_name:SyntheticShell",
            "distinct_correlation_ids",
            "recorded_response_json_bytes",
        ]
    );

    // The whole v2 vocabulary is present including the kinds this recording has
    // none of, so a zero column is a stated absence rather than a missing one.
    assert!(labels.iter().any(|label| label == "kind:v2:tool_denied"));
}

#[test]
fn tool_name_dimensions_are_the_delivered_strings_and_are_never_classified() {
    // Names chosen to be maximally tempting: a shell, a reader, and a searcher
    // by convention. The substrate must produce three verbatim columns and no
    // fourth column claiming to know what any of them does.
    let recording = recording_of(&[
        (
            0,
            requested("Bash", "id-1", serde_json::json!({"command": "cargo test"})),
        ),
        (
            100,
            requested("Read", "id-2", serde_json::json!({"file_path": "/x/y.rs"})),
        ),
        (
            200,
            requested("Grep", "id-3", serde_json::json!({"pattern": "fn main"})),
        ),
    ]);
    let (replay, ms) = signal_over!(recording.as_bytes(), 500);
    let inspection = inspect(&replay);
    let signal = project(&inspection, width(ms)).expect("records exist");

    let tool_labels: Vec<String> = signal
        .dimensions
        .iter()
        .filter(|d| matches!(d, Dimension::DeliveredToolName(_)))
        .map(Dimension::label)
        .collect();
    assert_eq!(
        tool_labels,
        vec!["tool_name:Bash", "tool_name:Read", "tool_name:Grep"],
        "delivered strings, verbatim, in first-appearance order"
    );

    // The refusals, asserted rather than merely documented. `cargo test` is in
    // this recording and no dimension knows it is a test run; `/x/y.rs` is in it
    // and no dimension counts files.
    let all: Vec<String> = signal.dimensions.iter().map(Dimension::label).collect();
    for forbidden in [
        "shell",
        "read_activity",
        "write",
        "edit",
        "search",
        "compiler",
        "test",
        "files",
        "file",
        "path",
        "output_volume",
        "duration",
        "prompt",
    ] {
        assert!(
            !all.iter().any(|label| {
                let lowered = label.to_lowercase();
                // A delivered tool name may legitimately contain any word at
                // all; only the substrate's own vocabulary is under test.
                !label.starts_with("tool_name:") && lowered.contains(forbidden)
            }),
            "no dimension may name the category {forbidden:?}: {all:?}"
        );
    }
}

#[test]
fn v1_and_v2_kinds_are_separate_columns_and_never_merge() {
    let recording = v1_recording();
    let (replay, ms) = signal_over!(recording.as_bytes(), 500);
    let inspection = inspect(&replay);
    let signal = project(&inspection, width(ms)).expect("records exist");

    let labels: Vec<String> = signal.dimensions.iter().map(Dimension::label).collect();
    assert!(labels.iter().any(|l| l == "kind:v1:observed_tool_started"));
    assert!(labels.iter().any(|l| l == "kind:v1:observed_tool_finished"));
    assert!(
        !labels.iter().any(|l| l.starts_with("kind:v2:")),
        "a v1 recording gets v1 columns only: {labels:?}"
    );
    // v1 results are responses too, and are measured under the same rule.
    let bytes = signal
        .dimension_index("recorded_response_json_bytes")
        .and_then(|index| signal.column(index))
        .expect("the column exists");
    assert!(bytes.iter().sum::<f64>() > 0.0);
}

#[test]
fn distinct_correlation_ids_counts_ids_present_not_calls() {
    // One call whose two records share a bucket contributes one; a second call
    // in the same bucket contributes another.
    let recording = recording_of(&[
        (0, requested("T", "id-1", serde_json::json!({}))),
        (10, succeeded("T", "id-1", serde_json::json!({}))),
        (20, requested("T", "id-2", serde_json::json!({}))),
    ]);
    let (replay, ms) = signal_over!(recording.as_bytes(), 500);
    let inspection = inspect(&replay);
    let signal = project(&inspection, width(ms)).expect("records exist");

    let index = signal
        .dimension_index("distinct_correlation_ids")
        .expect("the column exists");
    assert_eq!(signal.samples[0].values[index], 2.0);
    assert_eq!(signal.samples[0].values[0], 3.0, "three records, two ids");
}

// ---------------------------------------------------------------------------
// The structure the oracle encodes, recovered from the signal
// ---------------------------------------------------------------------------

/// The value vector of the bucket starting at this offset.
fn bucket_at(signal: &witnessglass::experiment::signal::BehavioralSignal<'_>, ms: u64) -> Vec<f64> {
    let index = (ms / signal.bucket_ms) as usize;
    signal.samples[index].values.clone()
}

/// Records observed in `[from, until)`, summed from the receipts.
fn records_in(
    signal: &witnessglass::experiment::signal::BehavioralSignal<'_>,
    from: u64,
    until: u64,
) -> usize {
    signal
        .samples
        .iter()
        .filter(|sample| sample.offset_ms >= from && sample.offset_ms < until)
        .map(|sample| sample.records.len())
        .sum()
}

#[test]
fn the_first_motif_repeats_exactly_at_its_declared_period() {
    let (replay, ms) = signal_over!(&fixture_bytes(), DEFAULT_BUCKET_MS);
    let inspection = inspect(&replay);
    let signal = project(&inspection, width(ms)).expect("the oracle has records");

    // The clean motif has no jitter, so every instance produces a byte-identical
    // run of buckets. This is the property a Matrix Profile would key on, and it
    // is asserted here without running one.
    let first = oracle::FIRST_MOTIF_START_MS;
    let instance: Vec<Vec<f64>> = (0..10)
        .map(|b| bucket_at(&signal, first + b * 500))
        .collect();
    for repeat in 1..oracle::MOTIF_INSTANCES {
        let at = first + repeat * oracle::MOTIF_PERIOD_MS;
        let observed: Vec<Vec<f64>> = (0..10).map(|b| bucket_at(&signal, at + b * 500)).collect();
        assert_eq!(
            observed, instance,
            "motif instance {repeat} should repeat exactly"
        );
    }

    let expected = oracle::MOTIF_INSTANCES as usize * oracle::MOTIF_RECORDS_PER_INSTANCE;
    assert_eq!(
        records_in(&signal, first, oracle::FIRST_MOTIF_END_MS),
        expected
    );
}

#[test]
fn the_regime_change_is_visible_on_four_dimensions_at_once() {
    let (replay, ms) = signal_over!(&fixture_bytes(), DEFAULT_BUCKET_MS);
    let inspection = inspect(&replay);
    let signal = project(&inspection, width(ms)).expect("the oracle has records");

    let column = |label: &str| {
        signal
            .dimension_index(label)
            .and_then(|index| signal.column(index))
            .unwrap_or_else(|| panic!("column {label} exists"))
    };
    let window = |values: &[f64], from: u64, until: u64| -> f64 {
        values[(from / 500) as usize..(until / 500) as usize]
            .iter()
            .sum()
    };

    let baseline = (oracle::FIRST_MOTIF_END_MS, oracle::REGIME_CHANGE_MS);
    let elevated = (oracle::REGIME_CHANGE_MS, oracle::ELEVATED_END_MS);

    // 1. Rate rises.
    let records = column("records");
    assert!(
        window(&records, elevated.0, elevated.1) > 3.0 * window(&records, baseline.0, baseline.1),
        "the elevated regime carries far more records than the baseline it follows"
    );

    // 2. The delivered tool mix changes: the reader stops, the shell and editor
    //    take over.
    let reader = column("tool_name:SyntheticReader");
    assert!(window(&reader, baseline.0, baseline.1) > 0.0);
    assert_eq!(window(&reader, elevated.0, elevated.1), 0.0);
    let shell = column("tool_name:SyntheticShell");
    assert_eq!(window(&shell, baseline.0, baseline.1), 0.0);
    assert!(window(&shell, elevated.0, elevated.1) > 0.0);

    // 3. Recorded response sizes grow by an order of magnitude.
    let bytes = column("recorded_response_json_bytes");
    assert!(
        window(&bytes, elevated.0, elevated.1) > 10.0 * window(&bytes, baseline.0, baseline.1),
        "the elevated regime's recorded responses are much larger"
    );

    // 4. The reported channel goes silent, and stays a separate column from the
    //    observed one throughout. This is the epistemic invariant surviving into
    //    the numbers: nothing sums these two together.
    let reported = column("channel:reported");
    let observed = column("channel:observed");
    assert_eq!(window(&reported, elevated.0, elevated.1), 0.0);
    assert!(
        window(
            &reported,
            oracle::FIRST_MOTIF_START_MS,
            oracle::FIRST_MOTIF_END_MS
        ) > 0.0
    );
    assert!(window(&observed, elevated.0, elevated.1) > 0.0);
}

#[test]
fn the_recurrence_carries_the_motif_and_the_noise_that_was_injected_into_it() {
    let (replay, ms) = signal_over!(&fixture_bytes(), DEFAULT_BUCKET_MS);
    let inspection = inspect(&replay);
    let signal = project(&inspection, width(ms)).expect("the oracle has records");

    let expected = oracle::MOTIF_INSTANCES as usize * oracle::MOTIF_RECORDS_PER_INSTANCE;
    assert_eq!(
        records_in(
            &signal,
            oracle::SECOND_MOTIF_START_MS,
            oracle::SESSION_END_MS
        ),
        expected,
        "the recurrence carries the same record count as the original"
    );

    // The injected failure lands inside the recurrence and nowhere else.
    let failed = signal
        .dimension_index("kind:v2:tool_failed")
        .and_then(|index| signal.column(index))
        .expect("the column exists");
    let inside: f64 = failed[(oracle::SECOND_MOTIF_START_MS / 500) as usize..]
        .iter()
        .sum();
    assert_eq!(inside, 1.0);
    assert_eq!(failed.iter().sum::<f64>(), 1.0);

    // The jitter moved the recurrence off the clean motif's exact alignment,
    // which is what makes it a recurrence with noise rather than a copy.
    let clean: Vec<Vec<f64>> = (0..10)
        .map(|b| bucket_at(&signal, oracle::FIRST_MOTIF_START_MS + b * 500))
        .collect();
    let noisy: Vec<Vec<f64>> = (0..10)
        .map(|b| bucket_at(&signal, oracle::SECOND_MOTIF_START_MS + b * 500))
        .collect();
    assert_ne!(clean, noisy, "the recurrence should not be bit-identical");
}

// ---------------------------------------------------------------------------
// Normalization
// ---------------------------------------------------------------------------

#[test]
fn normalization_is_derived_and_leaves_the_counts_untouched() {
    let (replay, ms) = signal_over!(&fixture_bytes(), DEFAULT_BUCKET_MS);
    let inspection = inspect(&replay);
    let signal = project(&inspection, width(ms)).expect("the oracle has records");

    let before = signal.clone();
    let normalized = signal.normalize();
    assert_eq!(signal, before, "normalizing must not disturb the signal");

    assert_eq!(normalized.values.len(), signal.len());
    for row in &normalized.values {
        assert_eq!(row.len(), signal.dimensions.len());
        assert!(
            row.iter().all(|value| value.is_finite()),
            "no NaN, no infinity"
        );
    }
    // A second normalization of the same signal is identical: the operation is
    // pure.
    assert_eq!(signal.normalize(), normalized);
}

#[test]
fn every_varying_dimension_normalizes_to_zero_mean_and_unit_variance() {
    let (replay, ms) = signal_over!(&fixture_bytes(), DEFAULT_BUCKET_MS);
    let inspection = inspect(&replay);
    let signal = project(&inspection, width(ms)).expect("the oracle has records");
    let normalized = signal.normalize();

    let buckets = signal.len() as f64;
    for (index, dimension) in signal.dimensions.iter().enumerate() {
        let column = normalized.column(index).expect("the column exists");
        let mean = column.iter().sum::<f64>() / buckets;
        let variance = column.iter().map(|z| (z - mean) * (z - mean)).sum::<f64>() / buckets;

        if normalized.stats[index].constant {
            assert!(
                column.iter().all(|z| *z == 0.0),
                "a constant dimension is defined to be exactly zero: {}",
                dimension.label()
            );
            continue;
        }
        assert!(mean.abs() < 1e-9, "{} mean {mean}", dimension.label());
        assert!(
            (variance - 1.0).abs() < 1e-9,
            "{} variance {variance}",
            dimension.label()
        );
    }
}

#[test]
fn a_zero_variance_dimension_is_flagged_and_defined_to_be_zero() {
    let (replay, ms) = signal_over!(&fixture_bytes(), DEFAULT_BUCKET_MS);
    let inspection = inspect(&replay);
    let signal = project(&inspection, width(ms)).expect("the oracle has records");
    let normalized = signal.normalize();

    // The oracle contains no denial, so its column is constant zero throughout —
    // an absence stated as a column rather than an omitted one.
    let index = signal
        .dimension_index("kind:v2:tool_denied")
        .expect("the column exists");
    let stats = &normalized.stats[index];
    assert!(stats.constant);
    assert_eq!(stats.stddev, 0.0);
    assert_eq!(stats.nonzero_buckets, 0);
    assert!(normalized.column(index).unwrap().iter().all(|z| *z == 0.0));

    // Constant and non-zero is the same rule: a dimension present in every
    // bucket at the same value carries no variation either.
    let single = recording_of(&[(0, requested("T", "id-1", serde_json::json!({})))]);
    let (replay, ms) = signal_over!(single.as_bytes(), 500);
    let inspection = inspect(&replay);
    let signal = project(&inspection, width(ms)).expect("records exist");
    let normalized = signal.normalize();
    assert_eq!(signal.len(), 1, "one record spans one bucket");
    assert!(
        normalized.stats.iter().all(|stat| stat.constant),
        "with one bucket every dimension is constant by construction"
    );
    assert!(normalized.values[0].iter().all(|z| *z == 0.0));
}

#[test]
fn the_data_is_sparse_enough_that_median_and_mad_would_degenerate() {
    // The measurement behind the documented choice of mean/stddev over
    // median/MAD. If a future fixture stops being sparse, this fails and the
    // policy has to be re-argued rather than inherited.
    let (replay, ms) = signal_over!(&fixture_bytes(), DEFAULT_BUCKET_MS);
    let inspection = inspect(&replay);
    let signal = project(&inspection, width(ms)).expect("the oracle has records");
    let normalized = signal.normalize();

    let buckets = signal.len();
    for (index, dimension) in signal.dimensions.iter().enumerate() {
        let nonzero = normalized.stats[index].nonzero_buckets;
        assert!(
            nonzero * 2 < buckets,
            "{} is non-zero in {nonzero} of {buckets} buckets; with fewer than half \
             non-zero the median is zero and so is the MAD, which is why median/MAD is \
             not the policy",
            dimension.label()
        );
    }
}

// ---------------------------------------------------------------------------
// Boundaries: absences stay absences
// ---------------------------------------------------------------------------

#[test]
fn a_recording_with_no_records_has_no_axis_and_therefore_no_signal() {
    let replay = replay_bytes(b"").expect("an empty recording replays");
    let inspection = inspect(&replay);
    assert!(
        project(&inspection, width(500)).is_none(),
        "no earliest timestamp means no origin means no axis; a zero-row matrix over an \
         invented axis would be a fabrication"
    );
}

#[test]
fn a_truncated_recording_projects_its_valid_prefix_and_says_so() {
    let bytes = fixture_bytes();
    // Drop the final record and leave a fragment with no newline in its place,
    // which is what a recording cut short mid-write looks like.
    let last_newline = bytes.iter().rposition(|b| *b == b'\n').expect("newlines");
    let previous = bytes[..last_newline]
        .iter()
        .rposition(|b| *b == b'\n')
        .expect("more newlines");
    let mut truncated = bytes[..previous + 1].to_vec();
    truncated.extend_from_slice(b"{\"schema_version\":2,\"session");

    let replay = replay_bytes(&truncated).expect("the valid prefix replays");
    let inspection = inspect(&replay);
    let signal = project(&inspection, width(500)).expect("the prefix has records");

    match signal.scope {
        ExaminedScope::ValidPrefix { records, .. } => {
            assert_eq!(records, ORACLE_RECORDS - 1, "one record was cut");
        }
        other => panic!("expected a valid-prefix scope, got {other:?}"),
    }
    // The axis stops where the evidence does. Nothing extends it to the session
    // boundary the complete recording had.
    assert!(signal.axis.span_ms < oracle::SESSION_END_MS);
    assert!(signal.len() < ORACLE_BUCKETS);
}

#[test]
fn a_span_shorter_than_one_bucket_is_one_bucket() {
    let recording = recording_of(&[
        (0, requested("T", "id-1", serde_json::json!({}))),
        (100, succeeded("T", "id-1", serde_json::json!({}))),
        (499, requested("T", "id-2", serde_json::json!({}))),
    ]);
    let (replay, ms) = signal_over!(recording.as_bytes(), 500);
    let inspection = inspect(&replay);
    let signal = project(&inspection, width(ms)).expect("records exist");

    assert_eq!(signal.len(), 1);
    assert_eq!(signal.axis.span_ms, 499);
    assert_eq!(signal.axis.final_bucket_observed_ms, 499);
    assert_eq!(signal.samples[0].records, vec![1, 2, 3]);
}

#[test]
fn a_partial_final_bucket_is_reported_and_never_scaled() {
    let recording = recording_of(&[
        (0, requested("T", "id-1", serde_json::json!({}))),
        (1_600, succeeded("T", "id-1", serde_json::json!({}))),
    ]);
    let (replay, ms) = signal_over!(recording.as_bytes(), 500);
    let inspection = inspect(&replay);
    let signal = project(&inspection, width(ms)).expect("records exist");

    assert_eq!(
        signal.len(),
        4,
        "buckets 0..=3, the last holding the last record"
    );
    assert_eq!(signal.axis.span_ms, 1_600);
    assert_eq!(signal.axis.final_bucket_observed_ms, 100);
    // The final bucket's one record is counted as one record. It is not weighted
    // up to a full bucket's worth, and the bucket's offset is a full width from
    // its predecessor's.
    assert_eq!(signal.samples[3].values[0], 1.0);
    assert_eq!(signal.samples[3].offset_ms, 1_500);
}

#[test]
fn timestamps_that_move_backwards_are_reported_rather_than_repaired() {
    // Record 3 was written before record 2, which leaves the append chain
    // perfectly intact and the time axis disordered.
    let recording = recording_of(&[
        (0, requested("T", "id-1", serde_json::json!({}))),
        (2_000, requested("T", "id-2", serde_json::json!({}))),
        (100, requested("T", "id-3", serde_json::json!({}))),
        (3_000, requested("T", "id-4", serde_json::json!({}))),
    ]);
    let (replay, ms) = signal_over!(recording.as_bytes(), 500);
    let inspection = inspect(&replay);
    let signal = project(&inspection, width(ms)).expect("records exist");

    assert_eq!(
        signal.axis.non_monotonic.count(),
        1,
        "the disagreement is counted"
    );
    assert_eq!(signal.axis.non_monotonic.records.sequences(), &[3]);

    // Record 3 lands in the bucket its own timestamp names, beside record 1 —
    // not in append position. Nothing was reordered to hide it.
    assert_eq!(signal.samples[0].records, vec![1, 3]);
    assert_eq!(signal.samples[4].records, vec![2]);
    assert_eq!(signal.samples[6].records, vec![4]);
    // Conservation still holds.
    let placed: usize = signal.samples.iter().map(|s| s.records.len()).sum();
    assert_eq!(placed, 4);
}

#[test]
fn a_projection_reads_no_file_and_borrows_rather_than_owns() {
    // Two projections of one replay agree exactly, and the replay is unchanged
    // by either. Determinism and non-mutation, stated as a test rather than as a
    // comment.
    let replay = replay_bytes(&fixture_bytes()).expect("the fixture should replay");
    let before = replay.clone();
    let inspection = inspect(&replay);
    let first = project(&inspection, width(500)).expect("records exist");
    let second = project(&inspection, width(500)).expect("records exist");
    assert_eq!(first, second);
    assert_eq!(replay, before);
}

// ---------------------------------------------------------------------------
// Small hand-built recordings
// ---------------------------------------------------------------------------

const TEST_SESSION: &str = "sess-synthetic-signal-test";

fn requested(tool: &str, id: &str, input: serde_json::Value) -> v2::Event {
    v2::Event::ToolRequested(v2::ToolRequested {
        tool_use_id: id.to_owned(),
        tool_name: tool.to_owned(),
        requested_input: input,
    })
}

fn succeeded(tool: &str, id: &str, input: serde_json::Value) -> v2::Event {
    v2::Event::ToolSucceeded(v2::ToolSucceeded {
        tool_use_id: id.to_owned(),
        tool_name: tool.to_owned(),
        effective_input: input,
        response: serde_json::json!({ "ok": true }),
        duration_ms: None,
    })
}

/// A complete v2 recording from `(offset_ms, event)` pairs, in the order given.
fn recording_of(events: &[(i64, v2::Event)]) -> String {
    let origin: jiff::Timestamp = "2026-03-01T00:00:00Z".parse().expect("a valid timestamp");
    let mut out = String::new();
    for (index, (offset_ms, event)) in events.iter().enumerate() {
        let record = v2::Record {
            schema_version: 2,
            session_id: TEST_SESSION.to_owned(),
            sequence: index as u64 + 1,
            recorded_at: origin + jiff::SignedDuration::from_millis(*offset_ms),
            context: v2::Context::default(),
            provenance: Provenance {
                channel: Channel::Observed,
                adapter: "synthetic-test-adapter".to_owned(),
                mechanism: "synthetic:test".to_owned(),
            },
            event: event.clone(),
        };
        out.push_str(&serde_json::to_string(&record).expect("a record serializes"));
        out.push('\n');
    }
    out
}

/// A minimal frozen-v1 recording, to prove the two vocabularies stay apart.
fn v1_recording() -> String {
    let origin: jiff::Timestamp = "2026-03-01T00:00:00Z".parse().expect("a valid timestamp");
    let events = [
        v1::Event::ObservedToolStarted(v1::ObservedToolStarted {
            tool_call_id: "call-1".to_owned(),
            tool_name: "SyntheticLegacy".to_owned(),
            arguments: serde_json::json!({ "path": "/synthetic/legacy" }),
        }),
        v1::Event::ObservedToolFinished(v1::ObservedToolFinished {
            tool_call_id: "call-1".to_owned(),
            outcome: v1::ToolOutcome::Succeeded,
            result: serde_json::json!({ "text": "synthetic legacy result" }),
        }),
    ];
    let mut out = String::new();
    for (index, event) in events.into_iter().enumerate() {
        let record = v1::Record {
            schema_version: 1,
            session_id: TEST_SESSION.to_owned(),
            sequence: index as u64 + 1,
            recorded_at: origin + jiff::SignedDuration::from_millis(index as i64 * 100),
            provenance: Provenance {
                channel: Channel::Observed,
                adapter: "synthetic-test-adapter".to_owned(),
                mechanism: "synthetic:test".to_owned(),
            },
            event,
        };
        out.push_str(&serde_json::to_string(&record).expect("a record serializes"));
        out.push('\n');
    }
    out
}
