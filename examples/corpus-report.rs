//! The whole invocation surface of the sprint:21 corpus report.
//!
//! An example rather than a subcommand, like every other analysis in this
//! repository, and for one concrete reason beyond convention: `src/main.rs` does
//! not depend on `crate::experiment` and must not start. Deleting
//! `src/experiment/corpus.rs`, this file, and `tests/corpus.rs` removes the
//! workflow completely.
//!
//! ```text
//! cargo run --release --example corpus-report -- \
//!     --recordings <DIR> --label <NAME> --out <DIR> [--replicates N]
//! cargo run --release --example corpus-report -- --render-from <FACTS.json> --out <DIR>
//! cargo run --release --example corpus-report -- --compare <A.json> <B.json> --out <DIR>
//! ```
//!
//! Output derived from a real recording is exactly as sensitive as that
//! recording. Nothing here redacts anything, and no file it writes may be called
//! sanitized or safe to share.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use witnessglass::experiment::corpus::{
    self, Analysis, Facts, Manifest, REPLICATES, Request, render_comparison, render_report,
};

const USAGE: &str = "\
corpus-report — a local, on-demand cross-recording corpus report (sprint:21)

USAGE:
    corpus-report --recordings <DIR> --label <NAME> --out <DIR> [--replicates <N>]
    corpus-report --render-from <FACTS.json> --out <DIR>
    corpus-report --compare <A.json> <B.json> --out <DIR>

    --recordings <DIR>   Directory of *.ndjson recordings to analyse. Read once,
                         on demand. Nothing is written to it, nothing is copied
                         out of it, and nothing outlives this command.
    --label <NAME>       The corpus's name in every document produced.
    --out <DIR>          Where report.md, facts.json and manifest.json are
                         written. Created if missing.
    --replicates <N>     Null replicates. Default 999. 0 skips calibration, in
                         which case every result is descriptive.
    --render-from <F>    Re-render report.md from a stored facts.json, analysing
                         nothing. The report is a function of the facts and this
                         is how that is checked.
    --compare <A> <B>    Read two facts.json documents and write comparison.md.

Every deterministic output is a pure function of the recordings and the
configuration: the same inputs produce the same bytes. run.json carries the
wall-clock metadata and is the only file that changes between identical runs.

A report derived from a recording is as sensitive as that recording. It is not
redacted and it is not safe to share.";

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(message) => {
            eprintln!("corpus-report: {message}");
            ExitCode::from(1)
        }
    }
}

enum Mode {
    Analyse {
        recordings: PathBuf,
        label: String,
        replicates: usize,
    },
    Render {
        facts: PathBuf,
    },
    Compare {
        before: PathBuf,
        after: PathBuf,
    },
}

struct Options {
    mode: Mode,
    out: PathBuf,
}

fn parse(args: &[String]) -> Result<Options, String> {
    let mut recordings: Option<PathBuf> = None;
    let mut label: Option<String> = None;
    let mut out: Option<PathBuf> = None;
    let mut replicates = REPLICATES;
    let mut render_from: Option<PathBuf> = None;
    let mut compare: Option<(PathBuf, PathBuf)> = None;

    let mut index = 0usize;
    while index < args.len() {
        let flag = args[index].as_str();
        let value = |offset: usize| -> Result<String, String> {
            args.get(index + offset)
                .cloned()
                .ok_or_else(|| format!("{flag} needs a value"))
        };
        match flag {
            "--recordings" => {
                recordings = Some(PathBuf::from(value(1)?));
                index += 2;
            }
            "--label" => {
                label = Some(value(1)?);
                index += 2;
            }
            "--out" => {
                out = Some(PathBuf::from(value(1)?));
                index += 2;
            }
            "--replicates" => {
                let text = value(1)?;
                replicates = text
                    .parse()
                    .map_err(|_| format!("--replicates needs a number, got {text:?}"))?;
                index += 2;
            }
            "--render-from" => {
                render_from = Some(PathBuf::from(value(1)?));
                index += 2;
            }
            "--compare" => {
                compare = Some((PathBuf::from(value(1)?), PathBuf::from(value(2)?)));
                index += 3;
            }
            "-h" | "--help" => return Err(USAGE.to_owned()),
            other => return Err(format!("unknown flag {other:?}\n\n{USAGE}")),
        }
    }

    let out = out.ok_or_else(|| format!("--out is required\n\n{USAGE}"))?;
    let mode = match (compare, render_from, recordings) {
        (Some((before, after)), None, None) => Mode::Compare { before, after },
        (None, Some(facts), None) => Mode::Render { facts },
        (None, None, Some(recordings)) => Mode::Analyse {
            recordings,
            label: label.ok_or_else(|| format!("--label is required\n\n{USAGE}"))?,
            replicates,
        },
        _ => {
            return Err(format!(
                "exactly one of --recordings, --render-from and --compare is required\n\n{USAGE}"
            ));
        }
    };
    Ok(Options { mode, out })
}

fn run() -> Result<ExitCode, String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("{USAGE}");
        return Ok(ExitCode::from(1));
    }
    let options = parse(&args)?;
    std::fs::create_dir_all(&options.out)
        .map_err(|error| format!("cannot create {}: {error}", options.out.display()))?;

    match options.mode {
        Mode::Analyse {
            recordings,
            label,
            replicates,
        } => {
            let started = std::time::Instant::now();
            let analysis = corpus::analyze(&Request {
                directory: recordings.clone(),
                label,
                replicates,
            })
            .map_err(|error| format!("cannot analyse {}: {error}", recordings.display()))?;
            write_analysis(&options.out, &analysis)?;
            write_run(&options.out, started.elapsed().as_millis())?;
            eprintln!(
                "corpus-report: {} discovered, {} analysed, {} skipped; wrote {}",
                analysis.facts.discovered,
                analysis.facts.included,
                analysis.facts.skipped,
                options.out.display()
            );
            Ok(ExitCode::SUCCESS)
        }
        Mode::Render { facts } => {
            let facts = read_facts(&facts)?;
            write_text(&options.out.join("report.md"), &render_report(&facts))?;
            Ok(ExitCode::SUCCESS)
        }
        Mode::Compare { before, after } => {
            let (before, after) = (read_facts(&before)?, read_facts(&after)?);
            write_text(
                &options.out.join("comparison.md"),
                &render_comparison(&before, &after),
            )?;
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn read_facts(path: &Path) -> Result<Facts, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    serde_json::from_str(&text)
        .map_err(|error| format!("{} is not a facts document: {error}", path.display()))
}

/// Write the three deterministic documents.
///
/// Serialized with a trailing newline and pretty-printed, so a diff between two
/// runs is readable rather than one enormous line.
pub fn write_analysis(out: &Path, analysis: &Analysis) -> Result<(), String> {
    write_json(&out.join("facts.json"), &analysis.facts)?;
    write_json(&out.join("manifest.json"), &analysis.manifest)?;
    write_text(&out.join("report.md"), &render_report(&analysis.facts))
}

fn write_json<T: serde::Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let text = serde_json::to_string_pretty(value)
        .map_err(|error| format!("cannot serialize {}: {error}", path.display()))?;
    write_text(path, &format!("{text}\n"))
}

fn write_text(path: &Path, text: &str) -> Result<(), String> {
    std::fs::write(path, text).map_err(|error| format!("cannot write {}: {error}", path.display()))
}

/// The one file that is allowed to differ between two identical runs.
fn write_run(out: &Path, elapsed_ms: u128) -> Result<(), String> {
    let document = serde_json::json!({
        "schema": "witnessglass.corpus-run",
        "analyzer": corpus::ANALYZER,
        "analyzer_version": corpus::ANALYZER_VERSION,
        "witnessglass_version": env!("CARGO_PKG_VERSION"),
        "elapsed_ms": elapsed_ms as u64,
        "generated_at": jiff::Timestamp::now().to_string(),
    });
    write_json(&out.join("run.json"), &document)
}

/// Manifests are read back by `tests/corpus.rs`; keeping the type named here
/// stops the import from being dead when the example is compiled alone.
#[allow(dead_code)]
fn _manifest_type_is_used(manifest: &Manifest) -> usize {
    manifest.inputs.len()
}
