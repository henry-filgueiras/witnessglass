//! The whole invocation surface of the sprint:6 Matrix Profile experiment.
//!
//! An example rather than a subcommand, like `behavioral-signal` and
//! `haar-scales` before it. Requires `--features experiment-matrix-profile`,
//! which is also what keeps the numerics dependency out of a default build.
//!
//! ```text
//! cargo run --features experiment-matrix-profile --example matrix-profile -- \
//!     --recording <PATH> [--bucket-ms N] [--only LABEL]... [--top-k N]
//!     [--region-a MS:MS --region-b MS:MS] [--detail] [--json]
//! ```
//!
//! Each dimension is scanned **independently**. There is no multivariate fusion,
//! and the window ladder is the preregistered one from task:16 — this tool does
//! not search around it.
//!
//! Output derived from a real recording is exactly as sensitive as that
//! recording. Nothing here redacts anything.

use std::path::PathBuf;
use std::process::ExitCode;

use witnessglass::experiment::matrix_profile::{LADDER_MS, WindowScan, scan};
use witnessglass::experiment::signal::{BucketWidth, DEFAULT_BUCKET_MS, Dimension, project};
use witnessglass::inspection::inspect;
use witnessglass::replay_file;

const USAGE: &str = "\
matrix-profile — a disposable sprint:6 experiment, not a product surface

USAGE:
    matrix-profile --recording <PATH> [--bucket-ms <N>] [--only <LABEL>]...
                   [--top-k <N>] [--region-a <MS:MS> --region-b <MS:MS>]
                   [--detail] [--json]

    --recording <PATH>   Replay, inspect, project, and scan one recording.
    --bucket-ms <N>      Base sampling interval in milliseconds. Default 500.
    --only <LABEL>       Restrict to this dimension. May be repeated.
    --top-k <N>          Motifs and discords to extract per window. Default 5.
    --region-a <MS:MS>   Two half-open millisecond regions, from the signal
    --region-b <MS:MS>   origin. When both are given, each window reports the
                         rank of the first motif linking them. This is the
                         cross-region criterion task:16 preregistered; it is
                         supplied by the caller and never guessed from a fixture.
    --detail             Print motif spans and discords, not just the table.
    --json               Print every scan as JSON.

THE WINDOW LADDER IS PREREGISTERED:
    2 s, 8 s, 16 s, 32 s, 64 s, 128 s. task:16 derives each and records what it
    controls for. This tool does not search other lengths.

WHAT A WINDOW IS NOT:
    It is the length of the pattern being matched. It is not a Haar scale, not a
    repetition period, and not a motif instance duration. Those are three
    different numbers and task:16 keeps them apart.

READING THE OUTPUT:
    These signals are mostly empty, and two constant subsequences are at distance
    0 by the library's convention. So the unmasked top motif is normally two
    stretches of nothing, shown as `raw` precisely because it is not a finding.
    `masked` excludes constant subsequences using the library's own threshold,
    and `const%` says how much of the candidate set that removed.

    `null` is the same quantity on a deterministically shuffled copy: same
    values, same density, no temporal order. `sep` is (null - masked) / 2*sqrt(m).
    A low masked distance means nothing on its own; separation from the null is
    the number to read.

    Distances are z-normalized, so they are blind to amplitude: two bursts of the
    same shape at different intensity are a perfect match. That is not evidence
    that the same behaviour recurred.";

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(message) => {
            eprintln!("matrix-profile: {message}");
            ExitCode::from(1)
        }
    }
}

struct Options {
    recording: PathBuf,
    bucket_ms: u64,
    only: Vec<String>,
    top_k: usize,
    region_a: Option<(u64, u64)>,
    region_b: Option<(u64, u64)>,
    detail: bool,
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
    let selected: Vec<usize> = (0..signal.dimensions.len())
        .filter(|index| {
            options.only.is_empty() || options.only.contains(&signal.dimensions[*index].label())
        })
        .collect();
    if selected.is_empty() {
        return Err("no dimension matched --only".to_owned());
    }

    // One dimension at a time, one window at a time. Nothing is combined.
    let mut scans: Vec<(usize, Vec<WindowScan>)> = Vec::new();
    for &index in &selected {
        // Raw counts, deliberately, not sprint:4's z-scored column. The metric
        // z-normalizes each subsequence itself, so this changes no distance in
        // exact arithmetic — and it is the difference between an empty window
        // having a standard deviation of exactly zero and having 1.863e-9, which
        // z-normalization amplifies into a false motif. See the module header.
        let column = signal.column(index).unwrap_or_default();
        let per_window = LADDER_MS
            .iter()
            .filter_map(|window_ms| scan(&column, options.bucket_ms, *window_ms, options.top_k))
            .collect();
        scans.push((index, per_window));
    }

    if options.json {
        let payload: Vec<_> = scans
            .iter()
            .map(|(index, windows)| {
                serde_json::json!({
                    "dimension": signal.dimensions[*index].label(),
                    "windows": windows,
                })
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&payload)
                .map_err(|err| format!("could not render JSON: {err}"))?
        );
        return Ok(ExitCode::SUCCESS);
    }

    print_header(&options, &signal);
    for (index, windows) in &scans {
        print_dimension(&options, &signal.dimensions[*index], windows);
    }
    Ok(ExitCode::SUCCESS)
}

fn parse(args: &[String]) -> Result<Options, String> {
    let mut recording = None;
    let mut bucket_ms = DEFAULT_BUCKET_MS;
    let mut only = Vec::new();
    let mut top_k = 5usize;
    let mut region_a = None;
    let mut region_b = None;
    let mut detail = false;
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
            "--top-k" => {
                let raw = value("--top-k")?;
                top_k = raw
                    .parse()
                    .map_err(|_| format!("--top-k {raw:?} is not a number"))?;
            }
            "--region-a" => region_a = Some(region(&value("--region-a")?)?),
            "--region-b" => region_b = Some(region(&value("--region-b")?)?),
            "--detail" => detail = true,
            "--json" => json = true,
            other => return Err(format!("unexpected argument {other:?}\n\n{USAGE}")),
        }
    }

    Ok(Options {
        recording: recording.ok_or_else(|| format!("--recording <PATH> is required\n\n{USAGE}"))?,
        bucket_ms,
        only,
        top_k,
        region_a,
        region_b,
        detail,
        json,
    })
}

fn region(text: &str) -> Result<(u64, u64), String> {
    let (start, end) = text
        .split_once(':')
        .ok_or_else(|| format!("region {text:?} should be <START_MS>:<END_MS>"))?;
    let parsed = |part: &str| {
        part.parse::<u64>()
            .map_err(|_| format!("region bound {part:?} is not a number"))
    };
    Ok((parsed(start)?, parsed(end)?))
}

fn print_header(
    options: &Options,
    signal: &witnessglass::experiment::signal::BehavioralSignal<'_>,
) {
    println!("matrix-profile — disposable sprint:6 experiment; the window ladder is preregistered");
    println!("recording: {}", options.recording.display());
    println!("samples: {} at {} ms", signal.len(), options.bucket_ms);
    println!(
        "metric: z-normalized Euclidean via motif-rs 0.1.0; exclusion zone ceil(m/4); two \
         constant subsequences are at distance 0 by convention"
    );
    println!(
        "null: the same values in a fixed-seed shuffle — same density, no temporal order. \
         sep = (null - masked) / 2*sqrt(m)"
    );
    if let (Some(a), Some(b)) = (options.region_a, options.region_b) {
        println!(
            "cross-region criterion: a motif with one span starting in [{}, {}) ms and the other \
             in [{}, {}) ms",
            a.0, a.1, b.0, b.1
        );
    }
    println!();
}

fn print_dimension(options: &Options, dimension: &Dimension<'_>, windows: &[WindowScan]) {
    println!("dimension: {}", dimension.label());
    if windows.is_empty() {
        println!("  the series is too short to hold any window of the ladder.");
        println!();
        return;
    }
    println!(
        "  {:>8} {:>5} {:>7} {:>7} {:>9} {:>9} {:>9} {:>8} {:>6} {:>7} {:>9}",
        "window", "m", "subseq", "const%", "raw", "masked", "null", "sep", "lag", "occ", "link/ovl"
    );
    for window in windows {
        let raw = window
            .raw_top
            .map(|pair| format!("{:.3}", pair.distance))
            .unwrap_or_else(|| "-".to_owned());
        let (masked, lag, occupancy) = match window.best() {
            Some(pair) => (
                format!("{:.3}", pair.distance),
                format!("{}", pair.lag_samples),
                format!("{}/{}", pair.a_occupancy, pair.b_occupancy),
            ),
            None => ("-".to_owned(), "-".to_owned(), "-".to_owned()),
        };
        let null = window
            .null_best_distance
            .map(|d| format!("{d:.3}"))
            .unwrap_or_else(|| "-".to_owned());
        let separation = window
            .separation
            .map(|s| format!("{s:+.3}"))
            .unwrap_or_else(|| "-".to_owned());
        // Two criteria, both shown: the one task:16 preregistered (a span must
        // *start* inside a region) and the overlap one it should have used.
        let rank_of = |found: Option<(usize, _)>| match found {
            Some((rank, _)) => format!("{}", rank + 1),
            None => "-".to_owned(),
        };
        let link = match (options.region_a, options.region_b) {
            (Some(a), Some(b)) => format!(
                "{}/{}",
                rank_of(window.linking(a, b)),
                rank_of(window.overlapping(a, b))
            ),
            _ => "-".to_owned(),
        };
        println!(
            "  {:>7}s {:>5} {:>7} {:>6.1}% {:>9} {:>9} {:>9} {:>8} {:>6} {:>7} {:>9}",
            window.window_ms as f64 / 1000.0,
            window.m,
            window.subsequences,
            100.0 * window.constant_fraction(),
            raw,
            masked,
            null,
            separation,
            lag,
            occupancy,
            link,
        );
    }

    if options.detail {
        for window in windows {
            println!("  -- window {}s --", window.window_ms as f64 / 1000.0);
            if let Some(pair) = window.raw_top {
                println!(
                    "     raw top: [{:.1}s,{:.1}s) <-> [{:.1}s,{:.1}s) d={:.4} constant={}/{}",
                    pair.a.start_ms as f64 / 1000.0,
                    pair.a.end_ms as f64 / 1000.0,
                    pair.b.start_ms as f64 / 1000.0,
                    pair.b.end_ms as f64 / 1000.0,
                    pair.distance,
                    pair.a_constant,
                    pair.b_constant
                );
            }
            for (rank, pair) in window.masked_top.iter().enumerate() {
                println!(
                    "     motif {}: [{:.1}s,{:.1}s) <-> [{:.1}s,{:.1}s) d={:.4} lag={} occupancy={}/{}",
                    rank + 1,
                    pair.a.start_ms as f64 / 1000.0,
                    pair.a.end_ms as f64 / 1000.0,
                    pair.b.start_ms as f64 / 1000.0,
                    pair.b.end_ms as f64 / 1000.0,
                    pair.distance,
                    pair.lag_samples,
                    pair.a_occupancy,
                    pair.b_occupancy
                );
            }
            for (rank, discord) in window.discords.iter().enumerate() {
                println!(
                    "     discord {}: [{:.1}s,{:.1}s) d={:.4}",
                    rank + 1,
                    discord.at.start_ms as f64 / 1000.0,
                    discord.at.end_ms as f64 / 1000.0,
                    discord.distance
                );
            }
        }
    }
    println!();
}
