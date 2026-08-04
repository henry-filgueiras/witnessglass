//! The whole invocation surface of the sprint:4 behavioral-signal experiment.
//!
//! An example rather than a subcommand on purpose. `witnessglass` has four verbs
//! and this experiment does not add a fifth: a disposable research surface on the
//! product CLI is a claim that the project ships an analysis, which it does not.
//!
//! ```text
//! cargo run --example behavioral-signal -- --recording <PATH> [--bucket-ms N]
//!                                          [--samples] [--json]
//! cargo run --example behavioral-signal -- --emit-oracle > fixtures/synthetic-behavioral-oracle.ndjson
//! ```
//!
//! It reads one recording through the ordinary path — `replay_file`, then
//! `inspect`, then the experimental projection — and prints numbers. It writes
//! nothing, serves nothing, and opens nothing. A recording is not redacted and
//! this dump is a rendering of one, so it is exactly as sensitive as its input.

use std::path::PathBuf;
use std::process::ExitCode;

use witnessglass::experiment::signal::{
    BucketWidth, DEFAULT_BUCKET_MS, Dimension, Normalized, project,
};
use witnessglass::experiment::{oracle, signal};
use witnessglass::inspection::{ExaminedScope, inspect};
use witnessglass::replay_file;

const USAGE: &str = "\
behavioral-signal — a disposable sprint:4 experiment, not a product surface

USAGE:
    behavioral-signal --recording <PATH> [--bucket-ms <N>] [--samples] [--json]
    behavioral-signal --emit-oracle

    --recording <PATH>  Replay, inspect, and project one recording.
    --bucket-ms <N>     Bucket width in milliseconds. Default 500.
    --samples           Print every bucket. Without it, the first and last few.
    --json              Print the whole signal and its normalization as JSON
                        instead of the human-readable summary.
    --emit-oracle       Write the deterministic synthetic oracle recording to
                        stdout as NDJSON and exit. This is how the committed
                        fixture is regenerated; it reads nothing.

The time axis is `recorded_at`, which is descriptive metadata and NOT the
canonical order of a recording. See the module documentation for what that costs.

Output derived from a real recording is exactly as sensitive as that recording.
Nothing here redacts anything.";

/// Buckets shown at each end when `--samples` is not given.
const PREVIEW: usize = 6;

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(message) => {
            eprintln!("behavioral-signal: {message}");
            ExitCode::from(1)
        }
    }
}

struct Options {
    recording: PathBuf,
    bucket_ms: u64,
    all_samples: bool,
    json: bool,
}

fn run() -> Result<ExitCode, String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() || args.iter().any(|a| a == "-h" || a == "--help") {
        println!("{USAGE}");
        return Ok(ExitCode::SUCCESS);
    }
    if args.iter().any(|a| a == "--emit-oracle") {
        print!("{}", oracle::ndjson());
        return Ok(ExitCode::SUCCESS);
    }

    let options = parse(&args)?;
    let width = BucketWidth::from_ms(options.bucket_ms)
        .ok_or_else(|| "--bucket-ms must be greater than zero".to_owned())?;

    let replay = replay_file(&options.recording)
        .map_err(|err| format!("could not replay {}: {err}", options.recording.display()))?;
    let inspection = inspect(&replay);

    let Some(signal) = project(&inspection, width) else {
        eprintln!(
            "no records in the examined scope, so there is no earliest timestamp, so there is no \
             time axis and no signal. This is an absence, not an empty result."
        );
        return Ok(ExitCode::from(2));
    };
    let normalized = signal.normalize();

    if options.json {
        let payload = serde_json::json!({
            "signal": &signal,
            "normalized": &normalized,
        });
        let text = serde_json::to_string_pretty(&payload)
            .map_err(|err| format!("could not render JSON: {err}"))?;
        println!("{text}");
        return Ok(ExitCode::SUCCESS);
    }

    print_summary(&options, &signal, &normalized);
    Ok(ExitCode::SUCCESS)
}

fn parse(args: &[String]) -> Result<Options, String> {
    let mut recording = None;
    let mut bucket_ms = DEFAULT_BUCKET_MS;
    let mut all_samples = false;
    let mut json = false;

    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--recording" => {
                let value = rest
                    .next()
                    .ok_or_else(|| "--recording requires a path".to_owned())?;
                recording = Some(PathBuf::from(value));
            }
            "--bucket-ms" => {
                let value = rest
                    .next()
                    .ok_or_else(|| "--bucket-ms requires a number".to_owned())?;
                bucket_ms = value
                    .parse()
                    .map_err(|_| format!("--bucket-ms {value:?} is not a number"))?;
            }
            "--samples" => all_samples = true,
            "--json" => json = true,
            other => return Err(format!("unexpected argument {other:?}\n\n{USAGE}")),
        }
    }

    Ok(Options {
        recording: recording.ok_or_else(|| format!("--recording <PATH> is required\n\n{USAGE}"))?,
        bucket_ms,
        all_samples,
        json,
    })
}

fn print_summary(
    options: &Options,
    signal: &signal::BehavioralSignal<'_>,
    normalized: &Normalized,
) {
    println!("behavioral-signal — disposable sprint:4 experiment; no detector is run here");
    println!("recording: {}", options.recording.display());
    println!(
        "session: {}",
        signal.session_id.unwrap_or("(no complete records)")
    );
    match signal.schema_version {
        Some(version) => println!("schema: v{version}"),
        None => println!("schema: none established"),
    }
    match signal.scope {
        ExaminedScope::CompleteRecording { records } => {
            println!("scope: complete recording, {records} record(s)");
        }
        ExaminedScope::ValidPrefix {
            records,
            fragment_byte_offset,
            fragment_bytes,
        } => {
            println!(
                "scope: VALID PREFIX ONLY, {records} record(s); the recording ends with a \
                 {fragment_bytes}-byte unterminated fragment at byte {fragment_byte_offset}. \
                 Every zero below is scoped to this prefix."
            );
        }
    }

    println!();
    println!("axis: recorded_at — descriptive metadata, NOT the canonical order");
    println!(
        "  origin: {} (seq {})",
        signal.axis.origin, signal.axis.origin_sequence
    );
    println!(
        "  latest: {} (seq {})",
        signal.axis.latest, signal.axis.latest_sequence
    );
    println!("  span_ms: {}", signal.axis.span_ms);
    println!("  bucket_ms: {}", signal.bucket_ms);
    println!("  samples: {}", signal.len());
    println!(
        "  final bucket: full width, observed to {} ms into it; nothing is scaled or extrapolated",
        signal.axis.final_bucket_observed_ms
    );
    let non_monotonic = signal.axis.non_monotonic.count();
    if non_monotonic == 0 {
        println!("  non-monotonic records: 0");
    } else {
        println!(
            "  non-monotonic records: {non_monotonic} — this recording's clock and its append \
             chain disagree, so bucket order is not append order here. Not repaired. Receipts: {:?}",
            signal.axis.non_monotonic.records.sequences()
        );
    }

    let receipts: usize = signal.samples.iter().map(|s| s.records.len()).sum();
    println!(
        "  receipts across all buckets: {receipts} (every record placed exactly once, by its own \
         timestamp)"
    );

    println!();
    println!("dimensions ({}):", signal.dimensions.len());
    println!(
        "  {:>3}  {:<44} {:>9} {:>9} {:>9} {:>7} {:>9} {:>8}",
        "idx", "dimension", "sum", "mean", "stddev", "max", "nonzero", "constant"
    );
    for (index, dimension) in signal.dimensions.iter().enumerate() {
        let stats = &normalized.stats[index];
        println!(
            "  {index:>3}  {:<44} {:>9.0} {:>9.4} {:>9.4} {:>7.0} {:>9} {:>8}",
            dimension.label(),
            stats.sum,
            stats.mean,
            stats.stddev,
            stats.max,
            stats.nonzero_buckets,
            if stats.constant { "yes" } else { "no" },
        );
    }
    let unlicensed = signal
        .dimensions
        .iter()
        .filter(|d| matches!(d, Dimension::DeliveredToolName(_)))
        .count();
    println!(
        "  {unlicensed} of these are verbatim delivered tool names. They are not classified into \
         roles; see the module documentation for what was refused and why."
    );

    println!();
    print_rows("raw", options.all_samples, signal.len(), |bucket| {
        let sample = &signal.samples[bucket];
        (
            sample.offset_ms,
            sample.values.clone(),
            sample.records.len(),
        )
    });

    println!();
    print_rows(
        "normalized (z)",
        options.all_samples,
        signal.len(),
        |bucket| {
            (
                signal.samples[bucket].offset_ms,
                normalized.values[bucket].clone(),
                signal.samples[bucket].records.len(),
            )
        },
    );
}

/// Print a matrix, eliding the middle unless every row was asked for.
fn print_rows(
    title: &str,
    all: bool,
    buckets: usize,
    row: impl Fn(usize) -> (u64, Vec<f64>, usize),
) {
    if all {
        println!("{title}: all {buckets} bucket(s)");
    } else {
        println!("{title}: {PREVIEW} from each end of {buckets}; --samples for all");
    }
    for bucket in 0..buckets {
        let elided = !all && bucket >= PREVIEW && bucket + PREVIEW < buckets;
        if elided {
            if bucket == PREVIEW {
                println!("  … {} bucket(s) elided …", buckets - 2 * PREVIEW);
            }
            continue;
        }
        let (offset_ms, values, receipts) = row(bucket);
        let rendered: Vec<String> = values
            .iter()
            .map(|value| {
                if value.fract() == 0.0 && value.abs() < 1e6 {
                    format!("{value:.0}")
                } else {
                    format!("{value:.2}")
                }
            })
            .collect();
        println!(
            "  t={:>9.3}s  n={receipts:<3} [{}]",
            offset_ms as f64 / 1000.0,
            rendered.join(" ")
        );
    }
}
