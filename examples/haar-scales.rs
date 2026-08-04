//! The whole invocation surface of the sprint:5 Haar experiment.
//!
//! An example rather than a subcommand, for the same reason `behavioral-signal`
//! is: `witnessglass` has four verbs and a disposable research surface on the
//! product CLI would be a claim that the project ships an analysis.
//!
//! ```text
//! cargo run --example haar-scales -- --recording <PATH> [--bucket-ms N]
//!           [--only LABEL]... [--exclude LABEL]... [--summary]
//!           [--exploratory-ln1p] [--json]
//! ```
//!
//! Each dimension is decomposed **independently**. There is no fusion, no
//! aggregation across dimensions, and therefore no path by which one dimension's
//! magnitude can reach another dimension's result.
//!
//! Output derived from a real recording is exactly as sensitive as that
//! recording. Nothing here redacts anything.

use std::path::PathBuf;
use std::process::ExitCode;

use witnessglass::experiment::haar;
use witnessglass::experiment::signal::{BucketWidth, DEFAULT_BUCKET_MS, Dimension, project};
use witnessglass::inspection::{ExaminedScope, inspect};
use witnessglass::replay_file;

const USAGE: &str = "\
haar-scales — a disposable sprint:5 experiment, not a product surface

USAGE:
    haar-scales --recording <PATH> [--bucket-ms <N>] [--only <LABEL>]...
                [--exclude <LABEL>]... [--summary] [--exploratory-ln1p] [--json]

    --recording <PATH>   Replay, inspect, project, and decompose one recording.
    --bucket-ms <N>      Base sampling interval in milliseconds. Default 500.
    --only <LABEL>       Restrict to this dimension. May be repeated.
    --exclude <LABEL>    Drop this dimension. May be repeated. This is how the
                         with/without comparison for a heavy-tailed dimension is
                         run.
    --summary            One ratio-to-null row per dimension instead of a full
                         table each, for reading many dimensions at once.
    --exploratory-ln1p   Decompose ln(1+x) of the raw counts instead. EXPLORATORY
                         ONLY: this is not sprint:4's normalization policy, is not
                         proposed as one, and any policy change needs its own
                         adjudication backed by evidence.
    --json               Print every decomposition as JSON.

WHAT THIS DOES NOT DO:
    It does not choose the sampling interval and cannot see below it. Structure
    faster than <N> ms is absent from the input, which is not the same as absent
    from the session. The interval remains a modeling choice, before and after.

    A peak is evidence about a scale, not about a session. \"This dimension
    carries energy near an 8 s scale\" is a finding; \"the agent has an 8 s loop\"
    is an interpretation this project has no license to make.

READING THE OUTPUT:
    A mostly-empty recording is a train of isolated impulses, and an isolated
    impulse has detail energy 2^-L at level L whatever produced it. The null
    column is that decay. The ratio is the departure from it, and it is the only
    column worth looking at first.";

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(message) => {
            eprintln!("haar-scales: {message}");
            ExitCode::from(1)
        }
    }
}

struct Options {
    recording: PathBuf,
    bucket_ms: u64,
    only: Vec<String>,
    exclude: Vec<String>,
    summary: bool,
    ln1p: bool,
    json: bool,
}

fn run() -> Result<ExitCode, String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() || args.iter().any(|a| a == "-h" || a == "--help") {
        println!("{USAGE}");
        return Ok(ExitCode::SUCCESS);
    }

    let options = parse(&args)?;
    let width = BucketWidth::from_ms(options.bucket_ms)
        .ok_or_else(|| "--bucket-ms must be greater than zero".to_owned())?;

    let replay = replay_file(&options.recording)
        .map_err(|err| format!("could not replay {}: {err}", options.recording.display()))?;
    let inspection = inspect(&replay);
    let Some(signal) = project(&inspection, width) else {
        eprintln!("no records in the examined scope, so there is no time axis and no signal.");
        return Ok(ExitCode::from(2));
    };
    let normalized = signal.normalize();

    // Which columns to decompose, after --only and --exclude. Order is the
    // signal's own, which is canonical.
    let selected: Vec<usize> = (0..signal.dimensions.len())
        .filter(|index| {
            let label = signal.dimensions[*index].label();
            (options.only.is_empty() || options.only.contains(&label))
                && !options.exclude.contains(&label)
        })
        .collect();
    if selected.is_empty() {
        return Err("no dimension matched --only/--exclude".to_owned());
    }

    // Each dimension on its own. The transform never sees two columns at once.
    let mut decompositions = Vec::new();
    for &index in &selected {
        let column = if options.ln1p {
            signal
                .column(index)
                .unwrap_or_default()
                .iter()
                .map(|value| value.max(0.0).ln_1p())
                .collect()
        } else {
            normalized.column(index).unwrap_or_default()
        };
        decompositions.push((index, haar::decompose(&column, options.bucket_ms)));
    }

    if options.json {
        let payload: Vec<_> = decompositions
            .iter()
            .map(|(index, decomposition)| {
                serde_json::json!({
                    "dimension": signal.dimensions[*index].label(),
                    "decomposition": decomposition,
                    "spectrum": decomposition.spectrum(),
                })
            })
            .collect();
        let text = serde_json::to_string_pretty(&payload)
            .map_err(|err| format!("could not render JSON: {err}"))?;
        println!("{text}");
        return Ok(ExitCode::SUCCESS);
    }

    print_header(&options, &signal, &normalized, &decompositions);
    for (index, decomposition) in &decompositions {
        if options.summary {
            print_summary_row(&signal.dimensions[*index], decomposition);
        } else {
            print_dimension(&signal.dimensions[*index], decomposition);
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn parse(args: &[String]) -> Result<Options, String> {
    let mut recording = None;
    let mut bucket_ms = DEFAULT_BUCKET_MS;
    let mut only = Vec::new();
    let mut exclude = Vec::new();
    let mut summary = false;
    let mut ln1p = false;
    let mut json = false;

    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        let mut value = |name: &str| {
            rest.next()
                .cloned()
                .ok_or_else(|| format!("{name} requires a value"))
        };
        match arg.as_str() {
            "--recording" => recording = Some(PathBuf::from(value("--recording")?)),
            "--bucket-ms" => {
                let raw = value("--bucket-ms")?;
                bucket_ms = raw
                    .parse()
                    .map_err(|_| format!("--bucket-ms {raw:?} is not a number"))?;
            }
            "--only" => only.push(value("--only")?),
            "--exclude" => exclude.push(value("--exclude")?),
            "--summary" => summary = true,
            "--exploratory-ln1p" => ln1p = true,
            "--json" => json = true,
            other => return Err(format!("unexpected argument {other:?}\n\n{USAGE}")),
        }
    }

    Ok(Options {
        recording: recording.ok_or_else(|| format!("--recording <PATH> is required\n\n{USAGE}"))?,
        bucket_ms,
        only,
        exclude,
        summary,
        ln1p,
        json,
    })
}

fn print_header(
    options: &Options,
    signal: &witnessglass::experiment::signal::BehavioralSignal<'_>,
    normalized: &witnessglass::experiment::signal::Normalized,
    decompositions: &[(usize, haar::Decomposition)],
) {
    println!("haar-scales — disposable sprint:5 experiment; no other detector is run here");
    println!("recording: {}", options.recording.display());
    match signal.scope {
        ExaminedScope::CompleteRecording { records } => {
            println!("scope: complete recording, {records} record(s)");
        }
        ExaminedScope::ValidPrefix {
            records,
            fragment_bytes,
            ..
        } => println!(
            "scope: VALID PREFIX ONLY, {records} record(s); {fragment_bytes} trailing bytes are \
             not a record and were not decomposed"
        ),
    }
    println!(
        "base sampling: {} ms — a modeling choice, not a measured truth. Haar cannot see below it.",
        options.bucket_ms
    );
    println!("samples: {}", signal.len());
    println!(
        "convention: orthonormal Haar, a=(x0+x1)/√2 and d=(x0-x1)/√2; level L is a window of \
         2^L samples"
    );
    println!(
        "odd tails: the unpaired final sample at a level is set aside as a remainder with its \
         energy, never padded and never dropped"
    );
    if options.ln1p {
        println!(
            "input: EXPLORATORY ln(1+x) of raw counts. NOT sprint:4's policy and not proposed as \
             one."
        );
    } else {
        println!("input: sprint:4's z-scored counts, unchanged");
    }

    // Two live checks, printed rather than assumed.
    let residual = decompositions
        .iter()
        .map(|(_, d)| d.energy_identity_residual().abs())
        .fold(0.0f64, f64::max);
    println!(
        "energy identity: worst |input - (detail + approximation + remainders)| = {residual:.3e}"
    );

    // Detail coefficients are offset-invariant and scale linearly, so shares
    // should be identical whether the input was raw or z-scored. Measured here
    // rather than argued.
    let mut worst_share_delta = 0.0f64;
    if !options.ln1p {
        for (index, decomposition) in decompositions {
            let raw = haar::decompose(
                &signal.column(*index).unwrap_or_default(),
                options.bucket_ms,
            );
            for (a, b) in raw.spectrum().iter().zip(decomposition.spectrum().iter()) {
                worst_share_delta = worst_share_delta.max((a.share - b.share).abs());
            }
        }
        println!(
            "raw vs z-scored: worst per-level share difference = {worst_share_delta:.3e} \
             (shares are offset- and scale-invariant, so the normalization policy cannot move them)"
        );
    }

    let empty = signal
        .samples
        .iter()
        .filter(|sample| sample.records.is_empty())
        .count();
    println!(
        "emptiness: {empty} of {} buckets hold no record ({:.1}%) — the isolated-impulse null \
         below is what that alone produces",
        signal.len(),
        100.0 * empty as f64 / signal.len().max(1) as f64
    );
    let constant = normalized.stats.iter().filter(|s| s.constant).count();
    println!(
        "dimensions decomposed: {} ({constant} constant), each on its own — no fusion",
        decompositions.len()
    );
    println!();

    if options.summary {
        // One legend rather than one table header per dimension. Cells are
        // ratio-to-null: 1.0 means indistinguishable from isolated impulses.
        let widest = decompositions
            .iter()
            .map(|(_, d)| d.spectrum())
            .max_by_key(Vec::len)
            .unwrap_or_default();
        let header: Vec<String> = widest
            .iter()
            .map(|band| format!("{:>4}", render_scale(band.scale_ms)))
            .collect();
        println!("  ratio to the isolated-impulse null, by window scale:");
        println!("  {:<34} {}", "dimension", header.join(" "));
        // How much of the recording each level still represents, after odd tails
        // have been set aside. A coarse ratio computed over half the recording
        // is worth less than the same ratio computed over all of it.
        let coverage: Vec<String> = decompositions
            .first()
            .map(|(_, d)| {
                d.levels
                    .iter()
                    .map(|level| {
                        format!(
                            "{:>3.0}%",
                            100.0 * level.covered_samples as f64 / d.input_len.max(1) as f64
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();
        println!(
            "  {:<34} {}",
            "(base samples still covered)",
            coverage.join(" ")
        );
    }
}

/// A full per-level table for one dimension.
fn print_dimension(dimension: &Dimension<'_>, decomposition: &haar::Decomposition) {
    println!("dimension: {}", dimension.label());
    if let Some(silence) = decomposition.silence() {
        println!("  {}", describe_silence(silence, decomposition));
        println!();
        return;
    }
    println!(
        "  detail {:.3} | approximation {:.3} | remainders {:.3} ({} set aside) | input {:.3}",
        decomposition.detail_energy,
        decomposition.approximation_energy,
        decomposition.remainder_energy,
        decomposition.remainders().len(),
        decomposition.input_energy,
    );
    println!(
        "   {:>2}  {:>9}  {:>12}  {:>7}  {:>7}  {:>6}  {:>8}",
        "L", "window", "energy", "share", "null", "ratio", "covers"
    );
    for (band, level) in decomposition.spectrum().iter().zip(&decomposition.levels) {
        println!(
            "   {:>2}  {:>9}  {:>12.4}  {:>6.2}%  {:>6.2}%  {:>6.2}  {:>7.1}%  {}",
            band.level,
            render_scale(band.scale_ms),
            band.energy,
            100.0 * band.share,
            100.0 * band.impulse_null_share,
            band.ratio_to_impulse_null,
            100.0 * level.covered_samples as f64 / decomposition.input_len.max(1) as f64,
            bar(band.share),
        );
    }
    println!();
}

/// One ratio-to-null row per dimension, for reading many at once.
fn print_summary_row(dimension: &Dimension<'_>, decomposition: &haar::Decomposition) {
    let spectrum = decomposition.spectrum();
    if spectrum.is_empty() {
        return;
    }
    let cells: Vec<String> = match decomposition.silence() {
        // Distinct markers, because a flat dimension and one the odd-length
        // policy removed are different facts and must not print the same.
        Some(haar::Silence::Empty) => spectrum.iter().map(|_| "   .".to_owned()).collect(),
        Some(haar::Silence::Constant) => spectrum.iter().map(|_| "  =".to_owned()).collect(),
        Some(haar::Silence::OnlyInRemainders) => {
            spectrum.iter().map(|_| " REM".to_owned()).collect()
        }
        None => spectrum
            .iter()
            .map(|band| format!("{:>4.1}", band.ratio_to_impulse_null))
            .collect(),
    };
    println!("  {:<34} {}", dimension.label(), cells.join(" "));
}

/// Plain-language reason a dimension produced no spectrum.
fn describe_silence(silence: haar::Silence, decomposition: &haar::Decomposition) -> String {
    match silence {
        haar::Silence::Empty => {
            "zero everywhere: nothing was observed to vary, at any scale.".to_owned()
        }
        haar::Silence::Constant => {
            "constant: non-zero but never changing, so no scale carries a contrast.".to_owned()
        }
        haar::Silence::OnlyInRemainders => format!(
            "NOT a flat dimension. Every non-zero sample fell into an odd-length remainder \
             ({} set aside, energy {:.3}) and reached no level. This is the transform's \
             limitation showing, not the recording's.",
            decomposition.remainders().len(),
            decomposition.remainder_energy,
        ),
    }
}

fn render_scale(ms: u64) -> String {
    if ms < 1_000 {
        format!("{ms}ms")
    } else {
        format!("{:.1}s", ms as f64 / 1000.0)
    }
}

fn bar(share: f64) -> String {
    "█".repeat((share * 40.0).round().max(0.0) as usize)
}
