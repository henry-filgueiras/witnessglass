//! The Behavioral Spectroscope: sprint:7's experimental page, served over the
//! existing viewer's transport.
//!
//! Requires `--features experiment-matrix-profile`. Build with `--release`: the
//! ladder is six Matrix Profile windows over up to eight dimensions, each with a
//! shuffled null beside it, and a debug build takes minutes to do what a release
//! build does in seconds.
//!
//! ```text
//! cargo run --release --features experiment-matrix-profile --example spectroscope -- \
//!     --recording fixtures/synthetic-behavioral-oracle.ndjson
//! ```
//!
//! Everything the page shows was computed before the listener bound. The browser
//! renders a derived document; it runs no transform and never sees raw NDJSON.
//!
//! The same process still serves the ordinary workbench at `/`. This page is an
//! extra perspective over one immutable snapshot, not a second application, and
//! it dies with the command that started it.

use std::path::PathBuf;
use std::process::ExitCode;

use witnessglass::experiment::spectroscope;
use witnessglass::replay_file;
use witnessglass::view::{Attachment, Snapshot, Viewer, open_in_browser};

const PAGE_HTML: &str = include_str!("../src/experiment/assets/spectroscope.html");
const PAGE_CSS: &str = include_str!("../src/experiment/assets/spectroscope.css");
const PAGE_JS: &str = include_str!("../src/experiment/assets/spectroscope.js");

const USAGE: &str = "\
spectroscope — a disposable sprint:7 experiment, not a product surface

USAGE:
    spectroscope --recording <PATH> [--no-open] [--json]

    --recording <PATH>  Replay, inspect, analyse, and serve one recording.
    --no-open           Serve without launching a browser.
    --json              Print the derived document to stdout and exit, serving
                        nothing.

The synthetic oracles are the hero case, because their structure was decided in
generator constants before any detector existed and the page can therefore show
what was planted beside what was found:

    fixtures/synthetic-behavioral-oracle.ndjson         legible, best case
    fixtures/synthetic-behavioral-oracle-sparse.ndjson  sparse, stress case

Any other recording works and simply has no ground truth: the planted band is
absent rather than guessed, and every finding is labelled a candidate.

Output derived from a real recording is exactly as sensitive as that recording.
Nothing here redacts anything.";

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(message) => {
            eprintln!("spectroscope: {message}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<ExitCode, String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() || args.iter().any(|a| a == "-h" || a == "--help") {
        println!("{USAGE}");
        return Ok(ExitCode::SUCCESS);
    }

    let mut recording: Option<PathBuf> = None;
    let mut open = true;
    let mut json = false;
    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--recording" => {
                recording = Some(PathBuf::from(
                    rest.next()
                        .ok_or_else(|| "--recording requires a path".to_owned())?,
                ));
            }
            "--no-open" => open = false,
            "--json" => json = true,
            other => return Err(format!("unexpected argument {other:?}\n\n{USAGE}")),
        }
    }
    let recording =
        recording.ok_or_else(|| format!("--recording <PATH> is required\n\n{USAGE}"))?;

    let replay = replay_file(&recording)
        .map_err(|err| format!("could not replay {}: {err}", recording.display()))?;
    let Some(document) = spectroscope::project(&replay) else {
        eprintln!(
            "no records in the examined scope, so there is no time axis and nothing to draw. \
             This is an absence, not an empty page."
        );
        return Ok(ExitCode::from(2));
    };
    let analysis =
        serde_json::to_string(&document).map_err(|err| format!("could not serialize: {err}"))?;

    if json {
        println!("{analysis}");
        return Ok(ExitCode::SUCCESS);
    }

    let snapshot = Snapshot::from_replay(&replay)
        .map_err(|err| format!("could not project {}: {err}", recording.display()))?;

    let viewer = Viewer::bind_with(
        snapshot,
        vec![
            Attachment {
                route: "/spectroscope",
                content_type: "text/html; charset=utf-8",
                body: PAGE_HTML.to_owned(),
            },
            Attachment {
                route: "/spectroscope.css",
                content_type: "text/css; charset=utf-8",
                body: PAGE_CSS.to_owned(),
            },
            Attachment {
                route: "/spectroscope.js",
                content_type: "text/javascript; charset=utf-8",
                body: PAGE_JS.to_owned(),
            },
            Attachment {
                route: "/spectroscope.json",
                content_type: "application/json; charset=utf-8",
                body: analysis,
            },
        ],
    )
    .map_err(|err| format!("could not bind: {err}"))?;

    let base = viewer
        .url()
        .map_err(|err| format!("could not read the bound address: {err}"))?;
    // The workbench URL is `/?c=…`; the spectroscope hangs off the same
    // capability on its own route.
    let page = base.replacen("/?c=", "/spectroscope?c=", 1);

    eprintln!(
        "analysed {} record(s) at {} ms; {} dimension(s), {} profiled",
        document.provenance.records,
        document.provenance.base_bucket_ms,
        document.dimensions.len(),
        document.profiles.len(),
    );
    match &document.ground_truth {
        Some(truth) => eprintln!("ground truth: {} — planted regions shown", truth.fixture),
        None => eprintln!("ground truth: none — not a synthetic fixture, so nothing is annotated"),
    }
    eprintln!("this snapshot is NOT redacted and is not safe to share; rendering is not redacting");
    eprintln!("{page}");
    eprintln!("the ordinary workbench is still at {base}");

    if open && let Err(err) = open_in_browser(&page) {
        eprintln!("could not open a browser ({err}); open the URL above yourself");
    }

    eprintln!("serving in the foreground; press Ctrl-C to stop");
    viewer
        .serve_forever()
        .map_err(|err| format!("stopped serving: {err}"))?;
    Ok(ExitCode::SUCCESS)
}
