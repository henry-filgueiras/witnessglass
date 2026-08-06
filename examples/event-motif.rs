//! The whole invocation surface of the sprint:8 event-native motif experiment.
//!
//! An example rather than a subcommand, like `behavioral-signal`, `haar-scales`,
//! `matrix-profile`, and `spectroscope` before it. No feature is required: this
//! experiment adds no dependency.
//!
//! ```text
//! cargo run --example event-motif -- \
//!     --recording <PATH> [--scope observed|all] [--figure N | --k N]...
//!     [--top-k N] [--region-a MS:MS --region-b MS:MS] [--detail] [--json]
//! cargo run --example event-motif -- --perturbation
//! ```
//!
//! The event-count ladder is preregistered in task:18 and derived from the
//! figure length with `--figure`; this tool searches nothing outside it unless a
//! caller names lengths explicitly with `--k`.
//!
//! Output derived from a real recording is exactly as sensitive as that
//! recording. Nothing here redacts anything.

use std::path::PathBuf;
use std::process::ExitCode;

use witnessglass::experiment::event_sequence::{
    ChannelScope, Comparison, CrossPair, EventSequence, LENGTH_FLOOR, REFINE_RADIUS,
    RefinedCandidate, Refinement, cross_pairs, dedupe_overlapping, enumerate_candidates, ladder,
    neighbours, null_ensemble, null_evidence, order_null, perturbation, project, refine,
    timing_null, top_pairs, top_pairs_where,
};
use witnessglass::experiment::{
    adversarial, boundary_page, envelope, gauntlet, identifiability, repair,
};
use witnessglass::inspection::inspect;
use witnessglass::replay_file;

const USAGE: &str = "\
event-motif — a disposable sprint:8 experiment, not a product surface

USAGE:
    event-motif --recording <PATH> [--scope observed|all] [--figure <N>]
                [--k <N>]... [--top-k <N>] [--region-a <MS:MS>]
                [--region-b <MS:MS>] [--detail] [--json]
    event-motif --perturbation

    --recording <PATH>   Replay, inspect, project, and scan one recording.
    --scope <SCOPE>      Which channels the sequence retains: `observed`
                         (the primary scope) or `all`. Default observed.
    --figure <N>         Length of the known planted figure, in events. The
                         preregistered ladder {3} u {N-2..N+2} is derived from
                         it. Supplied by the caller; never guessed from a
                         fixture.
    --k <N>              Scan exactly this event count. Repeatable. Overrides
                         --figure.
    --top-k <N>          Pairs and neighbours to report. Default 5.
    --region-a <MS:MS>   Two half-open millisecond regions from the sequence
    --region-b <MS:MS>   origin. When both are given, windows are labelled A or
                         B, the query window is the first window lying entirely
                         inside A, and matches are labelled with the regions
                         they link.
    --detail             Print the global top pairs and the query's neighbours.
    --json               Print the scans as JSON.
    --perturbation       Print the controlled perturbation sweep and exit. Needs
                         no recording: the base figure is hand-built from the
                         legible oracle's own generator constants.
    --against <PATH>     Cross-recording mode, sprint:9. Ranks only
                         (A-window, B-window) pairs between two different
                         recordings; same-recording pairs are a different
                         question and are not ranked here. Needs explicit --k.
    --blind              Cross-recording mode only: print the candidate packet
                         with distances withheld, so a classification can be
                         recorded before they are seen.
    --refine             Boundary refinement, sprint:10. Exhaustively scores
                         every combination of the four boundaries within
                         --radius events of a seed, and reports the Pareto
                         frontier over (distance, retained events). Local only:
                         it never looks outside the seed's neighbourhood.
    --seed-a <S..E>      The seed spans, as half-open event indices.
    --seed-b <S..E>
    --truth-a <S..E>     Known planted boundaries, where a fixture has them.
    --truth-b <S..E>     Supplied by the caller; never guessed from a fixture.
    --radius <N>         Boundary movement allowed, in events. Default 3.
    --floor <N>          Fewest events a refined span may hold. Default 3. Set
                         to 1 to run the anti-collapse negative control.
    --label / --role     Names for the specimen in output.
    --render <OUT.html>  Write one static page from specimen JSON documents.
    --from <IN.json>     A specimen document, from `--refine --json`. Repeatable.
                         The page holds no measurement of its own.
    --nulls <N>          sprint:11. Evaluate every enumerated boundary candidate
                         against N deterministic order-null realizations of both
                         recordings. The geometry scope.
    --frontier-nulls <N> The same over the Pareto frontier only, at a larger N,
                         where the tail needs finer resolution.
    --note-a <S..E>      A span to mark on the page beside the planted one, for
    --note-b <S..E>      a specimen with no ground truth. Supplied by the caller.
    --note-label <TEXT>  What to call it. Not a workflow name.
    --gauntlet           sprint:12. Run the adversarial gauntlet: eight families
                         of controlled synthetic specimen pairs, scored against
                         expectations task:22 recorded before any trial ran.
                         Needs no recording.
    --enumerate          sprint:14. Score the same gauntlet under every
                         preregistered function of the mark-only representation,
                         as a representation audit. Not a search for a scorer.
    --adversarial        sprint:15. Commission `rarity_of_agreements` against a
                         gauntlet built for its own failure modes. The statistic
                         is frozen and is not adopted by this mode.
    --repair             sprint:17. Compare candidate repairs to
                         rarity_of_agreements against the task:27 semantic
                         contract, the ten sprint:15 families, and — with
                         --corpus — the real operating envelope. Adopts nothing.

    --envelope           sprint:16. Measure where the two known failure surfaces
                         lie relative to the supplied corpus. Measures exposure,
                         not accuracy, and repairs nothing.
    --corpus <PATH>      A recording to include in the envelope study.
                         Repeatable. Output is as sensitive as the recordings.

WHAT A WINDOW IS HERE:
    A fixed number of *events*, not a fixed wall-clock width. A window of k
    events carries k-1 within-window gaps; the first event's gap points outside
    the window and is not used, so a window's timing does not depend on when it
    starts.

THE EXCLUSION POLICY:
    Two windows are compared only when they share no event at all. Stricter than
    the ceil(m/4) exclusion zone sprint:6 inherited from motif-rs, and stricter
    on purpose.

READING THE OUTPUT:
    `ev` is the event-edit component, `tm` the timing component, `tot` the
    combined ranking distance. They are reported separately because the round's
    question includes whether timing helps, hurts, or does nothing.

    `marks` counts distinct marks in a window. It replaces sprint:6's occupancy
    column: a window whose events are all one mark, or which alternates two, is
    a degenerate figure however perfectly it repeats.

    `sep-ord` and `sep-tim` are the query window's best distance under the order
    null and the timing null, minus its best distance on the real sequence. The
    order null permutes marks and leaves timing alone; the timing null permutes
    gaps and leaves marks alone.

    A low distance means nothing on its own. Separation from a null does.

WHAT A MATCH IS NOT:
    Two windows carrying the same delivered tool names in the same order at
    similar spacings. It is not evidence that the same behaviour recurred, and
    no tool name here has been read as a category.";

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(message) => {
            eprintln!("event-motif: {message}");
            ExitCode::from(1)
        }
    }
}

struct Options {
    recording: Option<PathBuf>,
    scope: ChannelScope,
    figure: Option<usize>,
    lengths: Vec<usize>,
    top_k: usize,
    region_a: Option<(u64, u64)>,
    region_b: Option<(u64, u64)>,
    detail: bool,
    json: bool,
    perturbation: bool,
    against: Option<PathBuf>,
    blind: bool,
    refine: bool,
    seed_a: Option<(usize, usize)>,
    seed_b: Option<(usize, usize)>,
    truth_a: Option<(usize, usize)>,
    truth_b: Option<(usize, usize)>,
    radius: usize,
    floor: usize,
    label: Option<String>,
    role: Option<String>,
    render: Option<PathBuf>,
    from: Vec<PathBuf>,
    gauntlet: bool,
    enumerate_scorers: bool,
    adversarial: bool,
    envelope: bool,
    repair: bool,
    corpus: Vec<PathBuf>,
    nulls: usize,
    frontier_nulls: usize,
    note_a: Option<(usize, usize)>,
    note_b: Option<(usize, usize)>,
    note_label: Option<String>,
}

fn run() -> Result<ExitCode, String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() || args.iter().any(|a| a == "-h" || a == "--help") {
        println!("{USAGE}");
        return Ok(ExitCode::SUCCESS);
    }
    let options = parse(&args)?;

    if options.perturbation {
        print_perturbation();
        return Ok(ExitCode::SUCCESS);
    }

    if options.render.is_some() {
        return render_mode(&options);
    }

    if options.repair {
        return repair_mode(&options);
    }

    if options.envelope {
        return envelope_mode(&options);
    }

    if options.adversarial {
        return adversarial_mode(&options);
    }

    if options.enumerate_scorers {
        return enumeration_mode(&options);
    }

    if options.gauntlet {
        return gauntlet_mode(&options);
    }

    let path = options
        .recording
        .clone()
        .ok_or_else(|| format!("--recording <PATH> is required\n\n{USAGE}"))?;
    let replay =
        replay_file(&path).map_err(|err| format!("could not replay {}: {err}", path.display()))?;
    let inspection = inspect(&replay);
    let Some(sequence) = project(&inspection, options.scope) else {
        eprintln!("no records in the examined scope, so there is no origin and no sequence.");
        return Ok(ExitCode::from(2));
    };

    if options.refine {
        return refinement_mode(&path, &sequence, &options);
    }

    if let Some(other) = options.against.clone() {
        return cross_recording(&path, &sequence, &other, &options);
    }

    let lengths = if !options.lengths.is_empty() {
        options.lengths.clone()
    } else if let Some(figure) = options.figure {
        ladder(figure)
    } else {
        return Err("one of --figure <N> or --k <N> is required".to_owned());
    };

    let order = order_null(&sequence);
    let timing = timing_null(&sequence);

    let scans: Vec<Scan> = lengths
        .iter()
        .map(|k| Scan::of(&sequence, &order, &timing, *k, &options))
        .collect();

    if options.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&scans.iter().map(Scan::as_json).collect::<Vec<_>>())
                .map_err(|err| format!("could not render JSON: {err}"))?
        );
        return Ok(ExitCode::SUCCESS);
    }

    print_header(&path, &options, &sequence);
    print_table(&scans);
    if options.detail {
        for scan in &scans {
            print_detail(scan, &options);
        }
    }
    Ok(ExitCode::SUCCESS)
}

/// Everything one event-count produced.
struct Scan {
    k: usize,
    windows: usize,
    comparable_pairs: usize,
    global: Vec<Comparison>,
    global_non_degenerate: Vec<Comparison>,
    query: Option<usize>,
    query_neighbours: Vec<Comparison>,
    null_order_best: Option<f64>,
    null_timing_best: Option<f64>,
    null_order_global: Option<f64>,
    null_timing_global: Option<f64>,
}

impl Scan {
    fn of(
        sequence: &EventSequence<'_>,
        order: &EventSequence<'_>,
        timing: &EventSequence<'_>,
        k: usize,
        options: &Options,
    ) -> Self {
        let windows = sequence.window_count(k);
        // Pairs sharing no event: for w windows and window length k, the count
        // of (i, j) with j >= i + k.
        let comparable_pairs = (0..windows)
            .map(|start| windows.saturating_sub(start + k))
            .sum();

        let query = options
            .region_a
            .and_then(|region| sequence.first_window_within(region, k));

        // The nulls are anchored at the *same event index*, so the comparison is
        // between the same position in three sequences rather than between three
        // different questions.
        let global_best = |nulled: &EventSequence<'_>| {
            top_pairs(nulled, k, 1)
                .first()
                .map(|found| found.alignment.total)
        };
        let null_best = |nulled: &EventSequence<'_>| {
            query.and_then(|start| {
                neighbours(nulled, start, k, 1)
                    .first()
                    .map(|found| found.alignment.total)
            })
        };

        Self {
            k,
            windows,
            comparable_pairs,
            global: top_pairs(sequence, k, options.top_k),
            // The second view, reported beside the first and never in place of
            // it: pairs where neither window is one or two marks repeating.
            global_non_degenerate: top_pairs_where(sequence, k, options.top_k, |a, b| {
                !a.degenerate() && !b.degenerate()
            }),
            query,
            query_neighbours: query
                .map(|start| neighbours(sequence, start, k, options.top_k))
                .unwrap_or_default(),
            null_order_best: null_best(order),
            null_timing_best: null_best(timing),
            // A second, unanchored comparison: the best pair anywhere in the
            // nulled sequence. Without a caller-supplied region there is no
            // query window, and this is the only null question that stays well
            // posed — it is also the shape sprint:6's null took.
            null_order_global: global_best(order),
            null_timing_global: global_best(timing),
        }
    }

    fn query_best(&self) -> Option<f64> {
        self.query_neighbours.first().map(|c| c.alignment.total)
    }

    fn separation(&self, null: Option<f64>) -> Option<f64> {
        match (null, self.query_best()) {
            (Some(null), Some(real)) => Some(null - real),
            _ => None,
        }
    }

    fn as_json(&self) -> serde_json::Value {
        serde_json::json!({
            "k": self.k,
            "windows": self.windows,
            "comparable_pairs": self.comparable_pairs,
            "query_window": self.query,
            "query_neighbours": self.query_neighbours,
            "global_top": self.global,
            "global_top_non_degenerate": self.global_non_degenerate,
            "null_order_global_best": self.null_order_global,
            "null_timing_global_best": self.null_timing_global,
            "null_order_query_best": self.null_order_best,
            "null_timing_query_best": self.null_timing_best,
            "separation_order": self.separation(self.null_order_best),
            "separation_timing": self.separation(self.null_timing_best),
        })
    }
}

fn parse(args: &[String]) -> Result<Options, String> {
    let mut recording = None;
    let mut scope = ChannelScope::Observed;
    let mut figure = None;
    let mut lengths = Vec::new();
    let mut top_k = 5usize;
    let mut region_a = None;
    let mut region_b = None;
    let mut detail = false;
    let mut json = false;
    let mut perturbation = false;
    let mut against = None;
    let mut blind = false;
    let mut refine_mode = false;
    let mut seed_a = None;
    let mut seed_b = None;
    let mut truth_a = None;
    let mut truth_b = None;
    let mut radius = REFINE_RADIUS;
    let mut floor = LENGTH_FLOOR;
    let mut label = None;
    let mut role = None;
    let mut render = None;
    let mut from = Vec::new();
    let mut gauntlet = false;
    let mut enumerate_scorers = false;
    let mut adversarial_mode_on = false;
    let mut envelope_on = false;
    let mut repair_on = false;
    let mut corpus = Vec::new();
    let mut nulls = 0usize;
    let mut frontier_nulls = 0usize;
    let mut note_a = None;
    let mut note_b = None;
    let mut note_label = None;

    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        let mut value = |name: &str| {
            rest.next()
                .cloned()
                .ok_or_else(|| format!("{name} requires a value"))
        };
        match arg.as_str() {
            "--recording" => recording = Some(PathBuf::from(value("--recording")?)),
            "--scope" => {
                let raw = value("--scope")?;
                scope = match raw.as_str() {
                    "observed" => ChannelScope::Observed,
                    "all" => ChannelScope::All,
                    other => return Err(format!("--scope {other:?} is not observed or all")),
                };
            }
            "--figure" => figure = Some(number(&value("--figure")?, "--figure")?),
            "--k" => lengths.push(number(&value("--k")?, "--k")?),
            "--top-k" => top_k = number(&value("--top-k")?, "--top-k")?,
            "--region-a" => region_a = Some(region(&value("--region-a")?)?),
            "--region-b" => region_b = Some(region(&value("--region-b")?)?),
            "--detail" => detail = true,
            "--json" => json = true,
            "--perturbation" => perturbation = true,
            "--against" => against = Some(PathBuf::from(value("--against")?)),
            "--blind" => blind = true,
            "--refine" => refine_mode = true,
            "--seed-a" => seed_a = Some(span(&value("--seed-a")?)?),
            "--seed-b" => seed_b = Some(span(&value("--seed-b")?)?),
            "--truth-a" => truth_a = Some(span(&value("--truth-a")?)?),
            "--truth-b" => truth_b = Some(span(&value("--truth-b")?)?),
            "--radius" => radius = number(&value("--radius")?, "--radius")?,
            "--floor" => floor = number(&value("--floor")?, "--floor")?,
            "--label" => label = Some(value("--label")?),
            "--role" => role = Some(value("--role")?),
            "--render" => render = Some(PathBuf::from(value("--render")?)),
            "--from" => from.push(PathBuf::from(value("--from")?)),
            "--note-a" => note_a = Some(span(&value("--note-a")?)?),
            "--note-b" => note_b = Some(span(&value("--note-b")?)?),
            "--note-label" => note_label = Some(value("--note-label")?),
            "--gauntlet" => gauntlet = true,
            "--enumerate" => enumerate_scorers = true,
            "--adversarial" => adversarial_mode_on = true,
            "--envelope" => envelope_on = true,
            "--repair" => repair_on = true,
            "--corpus" => corpus.push(PathBuf::from(value("--corpus")?)),
            "--nulls" => nulls = number(&value("--nulls")?, "--nulls")?,
            "--frontier-nulls" => {
                frontier_nulls = number(&value("--frontier-nulls")?, "--frontier-nulls")?
            }
            other => return Err(format!("unexpected argument {other:?}\n\n{USAGE}")),
        }
    }

    lengths.sort_unstable();
    lengths.dedup();
    Ok(Options {
        recording,
        scope,
        figure,
        lengths,
        top_k,
        region_a,
        region_b,
        detail,
        json,
        perturbation,
        against,
        blind,
        refine: refine_mode,
        seed_a,
        seed_b,
        truth_a,
        truth_b,
        radius,
        floor,
        label,
        role,
        render,
        from,
        gauntlet,
        enumerate_scorers,
        adversarial: adversarial_mode_on,
        envelope: envelope_on,
        repair: repair_on,
        corpus,
        nulls,
        frontier_nulls,
        note_a,
        note_b,
        note_label,
    })
}

fn number(text: &str, name: &str) -> Result<usize, String> {
    text.parse()
        .map_err(|_| format!("{name} {text:?} is not a number"))
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

/// Which planted region a window lies entirely inside, if the caller named any.
fn label(options: &Options, start_ms: u64, last_ms: u64) -> &'static str {
    let within = |region: Option<(u64, u64)>| {
        region.is_some_and(|(from, until)| start_ms >= from && last_ms < until)
    };
    if within(options.region_a) {
        "A"
    } else if within(options.region_b) {
        "B"
    } else {
        "-"
    }
}

fn pair_label(options: &Options, comparison: &Comparison) -> String {
    format!(
        "{}{}",
        label(options, comparison.a.start_ms, comparison.a.last_ms),
        label(options, comparison.b.start_ms, comparison.b.last_ms)
    )
}

fn print_header(path: &std::path::Path, options: &Options, sequence: &EventSequence<'_>) {
    println!(
        "event-motif — disposable sprint:8 experiment; the event-count ladder is preregistered"
    );
    println!("recording: {}", path.display());
    println!(
        "scope: {} — {} events retained, {} records filtered out by channel",
        sequence.channels.label(),
        sequence.len(),
        sequence.filtered_out
    );
    println!(
        "axis: recorded_at, which says when the recorder wrote a record and establishes no order, \
         duration, overlap, or causality; canonical order is `sequence` and nothing was reordered"
    );
    println!(
        "clamped gaps: {} (a gap that came out negative because the clock moved backwards)",
        sequence.clamped_gaps
    );
    println!(
        "metric: weighted alignment — substitution 1.0, indel 1.0, timing 0.5 x bounded log-ratio \
         (floor 100 ms, full scale 4x)"
    );
    println!("exclusion: two windows are compared only when they share no event");
    if let (Some(a), Some(b)) = (options.region_a, options.region_b) {
        println!(
            "regions: A = [{}, {}) ms, B = [{}, {}) ms, supplied by the caller",
            a.0, a.1, b.0, b.1
        );
    }
    println!();
}

fn print_table(scans: &[Scan]) {
    println!("  global, over every pair of windows sharing no event:");
    println!(
        "  {:>3} {:>8} {:>10} {:>8} {:>8} {:>9} {:>9}",
        "k", "windows", "pairs", "best", "best-nd", "null-ord", "null-tim"
    );
    for scan in scans {
        println!(
            "  {:>3} {:>8} {:>10} {:>8} {:>8} {:>9} {:>9}",
            scan.k,
            scan.windows,
            scan.comparable_pairs,
            show(scan.global.first().map(|c| c.alignment.total)),
            show(
                scan.global_non_degenerate
                    .first()
                    .map(|c| c.alignment.total)
            ),
            show(scan.null_order_global),
            show(scan.null_timing_global),
        );
    }
    println!();

    if scans.iter().all(|scan| scan.query.is_none()) {
        println!("  no region was supplied, so there is no query window and no anchored null.");
        println!();
        return;
    }
    println!("  anchored at the query window — the first window lying entirely inside region A:");
    println!(
        "  {:>3} {:>6} {:>8} {:>8} {:>7} {:>6} {:>9} {:>9} {:>8} {:>8}",
        "k",
        "query",
        "q-best",
        "q-ev",
        "q-tm",
        "q-nbr",
        "null-ord",
        "null-tim",
        "sep-ord",
        "sep-tim"
    );
    for scan in scans {
        let (best, event, timing, neighbour) = match scan.query_neighbours.first() {
            Some(found) => (
                format!("{:.3}", found.alignment.total),
                format!("{:.3}", found.alignment.event_norm),
                format!("{:.3}", found.alignment.timing_norm),
                format!("{}", found.b.start),
            ),
            None => (
                "-".to_owned(),
                "-".to_owned(),
                "-".to_owned(),
                "-".to_owned(),
            ),
        };
        let separation = |null: Option<f64>| {
            scan.separation(null)
                .map(|value| format!("{value:+.3}"))
                .unwrap_or_else(|| "-".to_owned())
        };
        println!(
            "  {:>3} {:>6} {:>8} {:>8} {:>7} {:>6} {:>9} {:>9} {:>8} {:>8}",
            scan.k,
            scan.query
                .map(|q| q.to_string())
                .unwrap_or_else(|| "-".to_owned()),
            best,
            event,
            timing,
            neighbour,
            show(scan.null_order_best),
            show(scan.null_timing_best),
            separation(scan.null_order_best),
            separation(scan.null_timing_best),
        );
    }
    println!();
}

/// A distance, or a dash where there was none to report.
fn show(value: Option<f64>) -> String {
    value
        .map(|v| format!("{v:.3}"))
        .unwrap_or_else(|| "-".to_owned())
}

fn print_comparison(options: &Options, rank: usize, comparison: &Comparison) {
    let alignment = &comparison.alignment;
    println!(
        "     {:>2}. idx {:>4} [{:>8.1}s +{:>5.1}s] marks {} <-> idx {:>4} [{:>8.1}s +{:>5.1}s] \
         marks {}  {}  ev {:.3} tm {:.3} tot {:.3}  sub {} ins {} del {} timed {}",
        rank + 1,
        comparison.a.start,
        comparison.a.start_ms as f64 / 1000.0,
        comparison.a.extent_ms() as f64 / 1000.0,
        comparison.a.distinct_marks,
        comparison.b.start,
        comparison.b.start_ms as f64 / 1000.0,
        comparison.b.extent_ms() as f64 / 1000.0,
        comparison.b.distinct_marks,
        pair_label(options, comparison),
        alignment.event_norm,
        alignment.timing_norm,
        alignment.total,
        alignment.substitutions,
        alignment.insertions,
        alignment.deletions,
        alignment.timed_pairs,
    );
}

fn print_detail(scan: &Scan, options: &Options) {
    println!("  -- k = {} --", scan.k);
    println!("     global top pairs, any position:");
    for (rank, comparison) in scan.global.iter().enumerate() {
        print_comparison(options, rank, comparison);
    }
    println!("     global top pairs, neither window degenerate:");
    for (rank, comparison) in scan.global_non_degenerate.iter().enumerate() {
        print_comparison(options, rank, comparison);
    }
    match scan.query {
        Some(start) => {
            println!("     query window: idx {start}, the first window lying entirely inside A");
            for (rank, comparison) in scan.query_neighbours.iter().enumerate() {
                print_comparison(options, rank, comparison);
            }
        }
        None => println!("     no window lies entirely inside region A at this k"),
    }
    println!(
        "     nulls at the same index: order {:?}, timing {:?}",
        scan.null_order_best, scan.null_timing_best
    );
    println!();
}

fn print_perturbation() {
    println!("event-motif — the sprint:8 controlled perturbation sweep");
    println!(
        "base: the legible oracle's planted figure, observed records only, hand-built from the \
         fixture's own generator constants"
    );
    println!(
        "meaningful only if basic recovery was earned; it asks whether the distance degrades \
         gracefully, not where a threshold should sit"
    );
    println!();
    println!(
        "  {:<26} {:>7} {:>7} {:>7}   sub/ins/del/timed",
        "variant", "event", "timing", "total"
    );
    for (name, alignment) in perturbation::sweep() {
        println!(
            "  {:<26} {:>7.3} {:>7.3} {:>7.3}   {}/{}/{}/{}",
            name,
            alignment.event_norm,
            alignment.timing_norm,
            alignment.total,
            alignment.substitutions,
            alignment.insertions,
            alignment.deletions,
            alignment.timed_pairs,
        );
    }
}

// ---------------------------------------------------------------------------
// Cross-recording mode — sprint:9, task:19
// ---------------------------------------------------------------------------

/// Distinct-mark strata, reported beside the unrestricted ranking.
///
/// **Diagnostic slices, not definitions of motifhood.** task:18 was explicit
/// that its degenerate-window diagnostic fits those fixtures rather than stating
/// a principle, and task:19 §6 forbids reading a verdict off whichever slice
/// looks best. They are here so a reader can see whether the top of the
/// unrestricted ranking is one mark repeating.
const STRATA: [usize; 3] = [2, 3, 4];

/// Candidates inspected per rung, fixed by task:19 §8.
const INSPECTED: usize = 3;

/// Everything one rung of the cross-recording ladder produced.
struct Rung<'a> {
    k: usize,
    /// Every cross pair, ranked. Unrestricted: task:19 §6's view A.
    ranked: Vec<CrossPair<'a>>,
    /// The same ranking after §4's de-duplication.
    kept: Vec<CrossPair<'a>>,
    /// Best cross pair with both sides order-nulled.
    null_order: Option<f64>,
    /// Best cross pair with both sides timing-nulled.
    null_timing: Option<f64>,
}

fn cross_recording(
    a_path: &std::path::Path,
    a: &EventSequence<'_>,
    b_path: &std::path::Path,
    options: &Options,
) -> Result<ExitCode, String> {
    let replay = replay_file(b_path)
        .map_err(|err| format!("could not replay {}: {err}", b_path.display()))?;
    let inspection = inspect(&replay);
    let Some(b) = project(&inspection, options.scope) else {
        eprintln!("no records in the examined scope of the second recording.");
        return Ok(ExitCode::from(2));
    };

    if options.lengths.is_empty() {
        return Err(
            "cross-recording mode needs the preregistered ladder: --k 3 --k 4 ...".to_owned(),
        );
    }

    // Nulled copies of both sides, computed once. task:19 §5 nulls both,
    // because the coherent control for "do two recordings share a figure" is
    // "neither recording's ordering carries information".
    let (a_order, b_order) = (order_null(a), order_null(&b));
    let (a_timing, b_timing) = (timing_null(a), timing_null(&b));

    println!("event-motif — cross-recording reality check; sprint:9, task:19");
    println!("the metric is task:18's, unchanged. Nothing here is tuned against these recordings.");
    println!("output derived from a real recording is exactly as sensitive as that recording;");
    println!("nothing here redacts anything.");
    println!();
    describe("A", a_path, a);
    describe("B", b_path, &b);
    println!();
    frequencies("A", a);
    frequencies("B", &b);
    println!();

    let mut rungs: Vec<Rung<'_>> = Vec::new();
    for &k in &options.lengths {
        let ranked = cross_pairs(a, &b, k, usize::MAX).ok_or_else(|| {
            "both recordings carry the same session id; that is not a cross-recording question"
                .to_owned()
        })?;
        let kept = dedupe_overlapping(&ranked, INSPECTED.max(options.top_k));
        let null_of = |left: &EventSequence<'_>, right: &EventSequence<'_>| {
            cross_pairs(left, right, k, 1)
                .and_then(|pairs| pairs.first().map(|pair| pair.comparison.alignment.total))
        };
        rungs.push(Rung {
            k,
            ranked,
            kept,
            null_order: null_of(&a_order, &b_order),
            null_timing: null_of(&a_timing, &b_timing),
        });
    }

    if options.blind {
        print_blind(a, &b, &rungs);
        return Ok(ExitCode::SUCCESS);
    }

    print_cross_table(a, &b, &rungs);
    for rung in &rungs {
        let k = rung.k;
        println!("  -- k = {k}, top {INSPECTED} de-duplicated candidates --");
        for (rank, pair) in rung.kept.iter().take(INSPECTED).enumerate() {
            print_candidate(a, &b, k, rank, pair, true);
        }
        println!("     (unrestricted rank 1 before de-duplication:)");
        if let Some(first) = rung.ranked.first() {
            print_candidate(a, &b, k, 0, first, true);
        }
        println!();
    }
    Ok(ExitCode::SUCCESS)
}

/// Aggregate characterization. No path content, no payload, no prompt.
fn describe(name: &str, path: &std::path::Path, sequence: &EventSequence<'_>) {
    let span = sequence
        .events
        .last()
        .map(|last| last.offset_ms)
        .unwrap_or(0);
    println!(
        "{name}: {} — session {}, {} retained {} events, {} filtered by channel, span {:.1} s",
        path.display(),
        sequence.session_id.unwrap_or("<none>"),
        sequence.len(),
        sequence.channels.label(),
        sequence.filtered_out,
        span as f64 / 1000.0,
    );
}

/// Marginal mark frequencies, so task:19 §7's hypothesis can be judged: are the
/// strongest matches produced by common vocabulary alone?
fn frequencies(name: &str, sequence: &EventSequence<'_>) {
    let mut counts: Vec<(String, usize)> = Vec::new();
    for event in &sequence.events {
        let label = event.mark.label();
        match counts.iter_mut().find(|(seen, _)| *seen == label) {
            Some((_, count)) => *count += 1,
            None => counts.push((label, 1)),
        }
    }
    counts.sort_by(|left, right| right.1.cmp(&left.1).then(left.0.cmp(&right.0)));
    let total = sequence.len().max(1);
    println!("{name} marginal marks ({} distinct):", counts.len());
    for (label, count) in &counts {
        println!(
            "    {:>4}  {:>5.1}%  {}",
            count,
            100.0 * *count as f64 / total as f64,
            label
        );
    }
}

fn smaller_distinct(pair: &CrossPair<'_>) -> usize {
    pair.comparison
        .a
        .distinct_marks
        .min(pair.comparison.b.distinct_marks)
}

fn print_cross_table(a: &EventSequence<'_>, b: &EventSequence<'_>, rungs: &[Rung<'_>]) {
    println!(
        "  view A unrestricted, and view B stratified by distinct marks in the smaller window:"
    );
    println!(
        "  {:>3} {:>6} {:>6} {:>7} {:>8} {:>8} {:>8} {:>8} {:>9} {:>9} {:>8} {:>8}",
        "k",
        "A win",
        "B win",
        "pairs",
        "best",
        ">=2",
        ">=3",
        ">=4",
        "null-ord",
        "null-tim",
        "sep-ord",
        "sep-tim"
    );
    for rung in rungs {
        let (k, ranked) = (&rung.k, &rung.ranked);
        let (null_order, null_timing) = (&rung.null_order, &rung.null_timing);
        let best = ranked.first().map(|p| p.comparison.alignment.total);
        let stratum = |floor: usize| {
            ranked
                .iter()
                .find(|pair| smaller_distinct(pair) >= floor)
                .map(|pair| pair.comparison.alignment.total)
        };
        let separation = |null: Option<f64>| match (null, best) {
            (Some(null), Some(best)) => format!("{:+.3}", null - best),
            _ => "-".to_owned(),
        };
        println!(
            "  {:>3} {:>6} {:>6} {:>7} {:>8} {:>8} {:>8} {:>8} {:>9} {:>9} {:>8} {:>8}",
            k,
            a.window_count(*k),
            b.window_count(*k),
            ranked.len(),
            show(best),
            show(stratum(STRATA[0])),
            show(stratum(STRATA[1])),
            show(stratum(STRATA[2])),
            show(*null_order),
            show(*null_timing),
            separation(*null_order),
            separation(*null_timing),
        );
    }
    println!();
}

/// One candidate, with both verbatim mark sequences and their gaps.
///
/// `reveal` is false in blind mode, which withholds the distances so a
/// classification can be recorded before they are seen. task:19 §9.
fn print_candidate(
    a: &EventSequence<'_>,
    b: &EventSequence<'_>,
    k: usize,
    rank: usize,
    pair: &CrossPair<'_>,
    reveal: bool,
) {
    let alignment = &pair.comparison.alignment;
    let scores = if reveal {
        format!(
            "ev {:.3} tm {:.3} tot {:.3}  sub {} ins {} del {}",
            alignment.event_norm,
            alignment.timing_norm,
            alignment.total,
            alignment.substitutions,
            alignment.insertions,
            alignment.deletions
        )
    } else {
        "[withheld]".to_owned()
    };
    println!(
        "     k{k}-c{}  A idx {} [{:.1}s +{:.1}s] marks {} · B idx {} [{:.1}s +{:.1}s] marks {}  {}",
        rank + 1,
        pair.comparison.a.start,
        pair.comparison.a.start_ms as f64 / 1000.0,
        pair.comparison.a.extent_ms() as f64 / 1000.0,
        pair.comparison.a.distinct_marks,
        pair.comparison.b.start,
        pair.comparison.b.start_ms as f64 / 1000.0,
        pair.comparison.b.extent_ms() as f64 / 1000.0,
        pair.comparison.b.distinct_marks,
        scores,
    );
    render_side(a, pair.a_session, pair.comparison.a.start, k, "A");
    render_side(b, pair.b_session, pair.comparison.b.start, k, "B");
}

fn render_side(
    sequence: &EventSequence<'_>,
    session: Option<&str>,
    start: usize,
    k: usize,
    side: &str,
) {
    let Some(events) = sequence.window(start, k) else {
        return;
    };
    let session = session.unwrap_or("<none>");
    let short = session.get(0..8).unwrap_or(session);
    let mut line = String::new();
    for (index, event) in events.iter().enumerate() {
        if index > 0 {
            let gap = event.gap_from_previous_ms.unwrap_or(0);
            line.push_str(&format!(" --{:.1}s-> ", gap as f64 / 1000.0));
        }
        line.push_str(&event.mark.label());
    }
    println!("        {side}[{short}] {line}");
}

/// The blind packet: sequences and timing, no distance and no rank ordering
/// beyond the label. task:19 §9 — cheap, and honest about being self-blinding.
fn print_blind(a: &EventSequence<'_>, b: &EventSequence<'_>, rungs: &[Rung<'_>]) {
    println!("event-motif — blind candidate packet; distances withheld");
    println!("classify each as TRIVIAL / STRUCTURALLY SIMILAR / AMBIGUOUS / NOT SIMILAR before");
    println!("re-running without --blind. One agent blinding itself is weak evidence and the");
    println!("Result says so.");
    println!();
    for rung in rungs {
        for (rank, pair) in rung.kept.iter().take(INSPECTED).enumerate() {
            print_candidate(a, b, rung.k, rank, pair, false);
            println!();
        }
    }
}

// ---------------------------------------------------------------------------
// Boundary refinement — sprint:10, task:20
// ---------------------------------------------------------------------------

/// A half-open event span, as `START:END`.
fn span(text: &str) -> Result<(usize, usize), String> {
    let (start, end) = text
        .split_once("..")
        .or_else(|| text.split_once(':'))
        .ok_or_else(|| format!("span {text:?} should be <START>..<END>"))?;
    let parsed = |part: &str| {
        part.parse::<usize>()
            .map_err(|_| format!("span bound {part:?} is not a number"))
    };
    let (start, end) = (parsed(start)?, parsed(end)?);
    if start >= end {
        return Err(format!("span {text:?} is empty or inverted"));
    }
    Ok((start, end))
}

fn refinement_mode(
    a_path: &std::path::Path,
    a: &EventSequence<'_>,
    options: &Options,
) -> Result<ExitCode, String> {
    let b_path = options
        .against
        .clone()
        .ok_or_else(|| "--refine needs --against <PATH>".to_owned())?;
    let replay = replay_file(&b_path)
        .map_err(|err| format!("could not replay {}: {err}", b_path.display()))?;
    let inspection = inspect(&replay);
    let Some(b) = project(&inspection, options.scope) else {
        eprintln!("no records in the examined scope of the second recording.");
        return Ok(ExitCode::from(2));
    };

    let seed_a = options
        .seed_a
        .ok_or_else(|| "--seed-a <S..E> is required".to_owned())?;
    let seed_b = options
        .seed_b
        .ok_or_else(|| "--seed-b <S..E> is required".to_owned())?;

    let refinement = refine(a, seed_a, &b, seed_b, options.radius, options.floor)
        .ok_or_else(|| "the seed is not a valid pair of spans in these recordings".to_owned())?;

    if options.json {
        let mut document = specimen_json(options, a_path, &b_path, &refinement);
        if let Some(nulls) = null_scopes(a, &b, options, &refinement) {
            document["null"] = nulls;
        }
        println!(
            "{}",
            serde_json::to_string_pretty(&document)
                .map_err(|err| format!("could not render JSON: {err}"))?
        );
        return Ok(ExitCode::SUCCESS);
    }

    print_refinement(a, &b, options, &refinement);
    print_null_frontier(a, &b, options, &refinement);
    Ok(ExitCode::SUCCESS)
}

/// The specimen document: computed values only, and the shape the static page
/// consumes. Nothing here is transcribed by hand.
fn specimen_json(
    options: &Options,
    a_path: &std::path::Path,
    b_path: &std::path::Path,
    refinement: &Refinement<'_>,
) -> serde_json::Value {
    serde_json::json!({
        "label": options.label.clone().unwrap_or_else(|| "specimen".to_owned()),
        "role": options.role.clone().unwrap_or_else(|| "unlabelled".to_owned()),
        "a_recording": a_path.display().to_string(),
        "b_recording": b_path.display().to_string(),
        "truth_a": options.truth_a,
        "truth_b": options.truth_b,
        "note_a": options.note_a,
        "note_b": options.note_b,
        "note_label": options.note_label,
        "refinement": refinement,
    })
}

/// One side's events around the interesting region, with verbatim marks.
///
/// Verbatim rather than abbreviated: task:20 requires the marks stay
/// inspectable, and a one-letter code would be an invented vocabulary sitting
/// exactly where this project refuses to have one.
fn print_side(
    label: &str,
    sequence: &EventSequence<'_>,
    seed: (usize, usize),
    pick: Option<(usize, usize)>,
    truth: Option<(usize, usize)>,
) {
    let lower = [Some(seed.0), pick.map(|p| p.0), truth.map(|t| t.0)]
        .into_iter()
        .flatten()
        .min()
        .unwrap_or(seed.0)
        .saturating_sub(1);
    let upper = [Some(seed.1), pick.map(|p| p.1), truth.map(|t| t.1)]
        .into_iter()
        .flatten()
        .max()
        .unwrap_or(seed.1)
        + 1;

    println!("     {label} events — S seed, P pick, T planted truth");
    for index in lower..upper {
        let Some(event) = sequence.events.get(index) else {
            continue;
        };
        let flag = |range: Option<(usize, usize)>, mark: char| match range {
            Some((from, until)) if index >= from && index < until => mark,
            _ => ' ',
        };
        let gap = match event.gap_from_previous_ms {
            Some(ms) if index > lower => format!("{:>7.1}s", ms as f64 / 1000.0),
            _ => "       -".to_owned(),
        };
        println!(
            "       {:>4} {}{}{} {} {}",
            index,
            flag(Some(seed), 'S'),
            flag(pick, 'P'),
            flag(truth, 'T'),
            gap,
            event.mark.label()
        );
    }
}

fn candidate_line(tag: &str, candidate: &RefinedCandidate<'_>) -> String {
    let (a, b) = (&candidate.pair.comparison.a, &candidate.pair.comparison.b);
    let alignment = &candidate.pair.comparison.alignment;
    format!(
        "  {tag:<9} A[{}..{}) len {:>2} marks {:>2}   B[{}..{}) len {:>2} marks {:>2}   \
         ev {:.3} tm {:.3} tot {:.3}",
        a.start,
        a.start + a.k,
        a.k,
        a.distinct_marks,
        b.start,
        b.start + b.k,
        b.k,
        b.distinct_marks,
        alignment.event_norm,
        alignment.timing_norm,
        alignment.total,
    )
}

fn print_refinement(
    a: &EventSequence<'_>,
    b: &EventSequence<'_>,
    options: &Options,
    refinement: &Refinement<'_>,
) {
    let label = options
        .label
        .clone()
        .unwrap_or_else(|| "specimen".to_owned());
    let role = options
        .role
        .clone()
        .unwrap_or_else(|| "unlabelled".to_owned());
    println!("SPECIMEN {label} — {role}");
    println!(
        "  radius {}, floor {}, {} boundary combinations scored, {} rejected",
        refinement.radius, refinement.floor, refinement.evaluated, refinement.rejected
    );
    println!("{}", candidate_line("seed", &refinement.seed));
    match &refinement.pick {
        Some(pick) => {
            println!("{}", candidate_line("pick", pick));
            println!(
                "  delta     A start {:+} end {:+}   B start {:+} end {:+}",
                pick.delta.a_start, pick.delta.a_end, pick.delta.b_start, pick.delta.b_end
            );
        }
        None => println!("  pick      none — no combination survived the floor"),
    }
    if let (Some(truth_a), Some(truth_b)) = (options.truth_a, options.truth_b) {
        let on_frontier = refinement.frontier.iter().position(|candidate| {
            let (ca, cb) = (&candidate.pair.comparison.a, &candidate.pair.comparison.b);
            (ca.start, ca.start + ca.k) == truth_a && (cb.start, cb.start + cb.k) == truth_b
        });
        println!(
            "  truth     A[{}..{}) B[{}..{}) — on frontier: {}",
            truth_a.0,
            truth_a.1,
            truth_b.0,
            truth_b.1,
            match on_frontier {
                Some(rank) => format!("yes, position {}", rank + 1),
                None => "no".to_owned(),
            }
        );
    }

    println!("  Pareto frontier — retained events against total distance:");
    println!(
        "  {:>9} {:>12} {:>12} {:>7} {:>7} {:>7}   deltas",
        "retained", "A span", "B span", "ev", "tm", "tot"
    );
    for candidate in &refinement.frontier {
        let (ca, cb) = (&candidate.pair.comparison.a, &candidate.pair.comparison.b);
        let alignment = &candidate.pair.comparison.alignment;
        println!(
            "  {:>9} {:>12} {:>12} {:>7.3} {:>7.3} {:>7.3}   {:+} {:+} {:+} {:+}{}",
            candidate.retained,
            format!("[{}..{})", ca.start, ca.start + ca.k),
            format!("[{}..{})", cb.start, cb.start + cb.k),
            alignment.event_norm,
            alignment.timing_norm,
            alignment.total,
            candidate.delta.a_start,
            candidate.delta.a_end,
            candidate.delta.b_start,
            candidate.delta.b_end,
            if candidate.delta.is_seed() {
                "  (the seed)"
            } else {
                ""
            },
        );
    }

    let pick_spans = refinement.pick.as_ref().map(|pick| {
        let (ca, cb) = (&pick.pair.comparison.a, &pick.pair.comparison.b);
        ((ca.start, ca.start + ca.k), (cb.start, cb.start + cb.k))
    });
    print_side(
        "A",
        a,
        seed_span(&refinement.seed, true),
        pick_spans.map(|p| p.0),
        options.truth_a,
    );
    print_side(
        "B",
        b,
        seed_span(&refinement.seed, false),
        pick_spans.map(|p| p.1),
        options.truth_b,
    );
    println!();
}

fn seed_span(seed: &RefinedCandidate<'_>, first: bool) -> (usize, usize) {
    let window = if first {
        &seed.pair.comparison.a
    } else {
        &seed.pair.comparison.b
    };
    (window.start, window.start + window.k)
}

// ---------------------------------------------------------------------------
// The static specimen page — sprint:10, task:20 §9
// ---------------------------------------------------------------------------

fn render_mode(options: &Options) -> Result<ExitCode, String> {
    let out = options
        .render
        .clone()
        .ok_or_else(|| "--render <OUT.html> is required".to_owned())?;
    if options.from.is_empty() {
        return Err("--render needs at least one --from <SPECIMEN.json>".to_owned());
    }
    let mut documents = Vec::new();
    for path in &options.from {
        let text = std::fs::read_to_string(path)
            .map_err(|err| format!("could not read {}: {err}", path.display()))?;
        documents.push(
            serde_json::from_str::<serde_json::Value>(&text)
                .map_err(|err| format!("{} is not a specimen document: {err}", path.display()))?,
        );
    }
    std::fs::write(&out, boundary_page::render(&documents))
        .map_err(|err| format!("could not write {}: {err}", out.display()))?;
    println!(
        "wrote {} from {} specimen(s)",
        out.display(),
        documents.len()
    );
    println!("output derived from a real recording is as sensitive as that recording.");
    Ok(ExitCode::SUCCESS)
}

// ---------------------------------------------------------------------------
// Null-referenced evidence — sprint:11, task:21
// ---------------------------------------------------------------------------

/// A candidate's spans, as the half-open pairs the evidence call wants.
fn candidate_spans(candidate: &RefinedCandidate<'_>) -> ((usize, usize), (usize, usize)) {
    let (a, b) = (&candidate.pair.comparison.a, &candidate.pair.comparison.b);
    ((a.start, a.start + a.k), (b.start, b.start + b.k))
}

/// One candidate as the page consumes it: the raw numbers task:20 already had,
/// plus whatever null evidence was computed for it.
fn candidate_json(
    candidate: &RefinedCandidate<'_>,
    evidence: Option<&witnessglass::experiment::event_sequence::CandidateNull>,
    with_histogram: bool,
) -> serde_json::Value {
    let (a, b) = (&candidate.pair.comparison.a, &candidate.pair.comparison.b);
    let alignment = &candidate.pair.comparison.alignment;
    let mut node = serde_json::json!({
        "retained": candidate.retained,
        "a": [a.start, a.start + a.k],
        "b": [b.start, b.start + b.k],
        "a_marks": a.distinct_marks,
        "b_marks": b.distinct_marks,
        "event_norm": alignment.event_norm,
        "timing_norm": alignment.timing_norm,
        "total": alignment.total,
        "delta": candidate.delta,
    });
    if let Some(evidence) = evidence {
        let mut total = serde_json::to_value(&evidence.total).unwrap_or(serde_json::Value::Null);
        let mut event = serde_json::to_value(&evidence.event).unwrap_or(serde_json::Value::Null);
        if !with_histogram {
            // 2400 candidates x 20 bins is noise in a document nobody reads at
            // that granularity; the frontier carries the distributions.
            for value in [&mut total, &mut event] {
                if let Some(object) = value.as_object_mut() {
                    object.remove("histogram");
                }
            }
        }
        node["null"] = serde_json::json!({ "total": total, "event": event });
    }
    node
}

/// Compute both preregistered scopes and attach them to the specimen document.
fn null_scopes(
    a: &EventSequence<'_>,
    b: &EventSequence<'_>,
    options: &Options,
    refinement: &Refinement<'_>,
) -> Option<serde_json::Value> {
    if options.nulls == 0 && options.frontier_nulls == 0 {
        return None;
    }
    let seed_a = candidate_spans(&refinement.seed).0;
    let seed_b = candidate_spans(&refinement.seed).1;

    let geometry = (options.nulls > 0).then(|| {
        let ensemble = null_ensemble(a, b, options.nulls);
        let scored =
            enumerate_candidates(a, seed_a, b, seed_b, refinement.radius, refinement.floor)
                .map(|(_, scored, _)| scored)
                .unwrap_or_default();
        let points: Vec<serde_json::Value> = scored
            .iter()
            .map(|candidate| {
                let (span_a, span_b) = candidate_spans(candidate);
                let evidence = null_evidence(
                    &ensemble,
                    span_a,
                    span_b,
                    &candidate.pair.comparison.alignment,
                );
                candidate_json(candidate, evidence.as_ref(), false)
            })
            .collect();
        serde_json::json!({ "realizations": options.nulls, "points": points })
    });

    let frontier = (options.frontier_nulls > 0).then(|| {
        let ensemble = null_ensemble(a, b, options.frontier_nulls);
        let points: Vec<serde_json::Value> = refinement
            .frontier
            .iter()
            .map(|candidate| {
                let (span_a, span_b) = candidate_spans(candidate);
                let evidence = null_evidence(
                    &ensemble,
                    span_a,
                    span_b,
                    &candidate.pair.comparison.alignment,
                );
                candidate_json(candidate, evidence.as_ref(), true)
            })
            .collect();
        serde_json::json!({ "realizations": options.frontier_nulls, "points": points })
    });

    Some(serde_json::json!({ "geometry": geometry, "frontier": frontier }))
}

/// The frontier's null evidence, in the terminal.
fn print_null_frontier(
    a: &EventSequence<'_>,
    b: &EventSequence<'_>,
    options: &Options,
    refinement: &Refinement<'_>,
) {
    if options.frontier_nulls == 0 {
        return;
    }
    let ensemble = null_ensemble(a, b, options.frontier_nulls);
    println!(
        "  null-referenced evidence over the frontier, {} order-null realizations of both sides:",
        options.frontier_nulls
    );
    println!(
        "  {:>9} {:>12} {:>12} {:>7} {:>8} {:>8} {:>8} {:>8} {:>8}",
        "retained", "A span", "B span", "tot", "null-mu", "null-sd", "emp-p", "sep", "z"
    );
    for candidate in &refinement.frontier {
        let (span_a, span_b) = candidate_spans(candidate);
        let Some(evidence) = null_evidence(
            &ensemble,
            span_a,
            span_b,
            &candidate.pair.comparison.alignment,
        ) else {
            continue;
        };
        let (ca, cb) = (&candidate.pair.comparison.a, &candidate.pair.comparison.b);
        println!(
            "  {:>9} {:>12} {:>12} {:>7.3} {:>8.3} {:>8.3} {:>8} {:>8.3} {:>8}",
            candidate.retained,
            format!("[{}..{})", ca.start, ca.start + ca.k),
            format!("[{}..{})", cb.start, cb.start + cb.k),
            candidate.pair.comparison.alignment.total,
            evidence.total.null_mean,
            evidence.total.null_stddev,
            format!("{:.2e}", evidence.total.empirical_p),
            evidence.total.separation,
            match evidence.total.standardized_separation {
                Some(z) => format!("{z:.2}"),
                None => "-".to_owned(),
            },
        );
    }
    println!(
        "  emp-p is (1 + realizations at or below observed) / (1 + realizations); its floor here is \
         {:.1e}",
        1.0 / (1 + options.frontier_nulls) as f64
    );
    println!();
}

// ---------------------------------------------------------------------------
// The adversarial gauntlet — sprint:12, task:22
// ---------------------------------------------------------------------------

fn gauntlet_mode(options: &Options) -> Result<ExitCode, String> {
    let (outcomes, reports) = gauntlet::report();

    if options.json {
        let document = serde_json::json!({
            "label": "gauntlet",
            "role": "controlled synthetic validation — sprint:12, task:22",
            "realizations": gauntlet::REALIZATIONS,
            "trials": outcomes.len(),
            "families": reports,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&document)
                .map_err(|err| format!("could not render JSON: {err}"))?
        );
        return Ok(ExitCode::SUCCESS);
    }

    println!("event-motif — the sprint:12 adversarial gauntlet");
    println!(
        "{} trials, {} order-null realizations each. The metric, the null, and the boundary search \
         are frozen; this only calls them.",
        outcomes.len(),
        gauntlet::REALIZATIONS
    );
    println!(
        "Scoring is task:22's single rule: PASS when the median has the expected sign and at least \
         two thirds of trials agree."
    );
    println!();
    println!(
        "  {:<13} {:>10} {:>7} {:>6} {:>9} {:>9} {:>9} {:>9} {:>7}",
        "family", "statistic", "trials", "undef", "frac", "q1", "median", "q3", "verdict"
    );
    for report in &reports {
        println!(
            "  {:<13} {:>10} {:>7} {:>6} {:>9.3} {:>9.3} {:>9.3} {:>9.3} {:>7}",
            report.family.label(),
            report.statistic,
            report.trials,
            report.undefined,
            report.expected_fraction,
            report.q1,
            report.median,
            report.q3,
            report.verdict.label(),
        );
    }
    println!();
    for report in &reports {
        println!("  -- {} [{}] --", report.family.label(), report.statistic);
        println!("     quantity:    {}", report.quantity);
        println!("     expectation: {}", report.expectation);
        println!(
            "     median Δtotal beside it: {:+.4}",
            report.median_delta_total
        );
        println!("     worst counterexamples:");
        for entry in &report.counterexamples {
            let trial = &entry.outcome.trial;
            println!(
                "       value {:+.4}  seed {} core {} context {} ratio {}  Δtotal {:+.4}",
                entry.value,
                trial.seed,
                trial.core_len,
                trial.context_len,
                trial.gap_ratio,
                entry.outcome.delta_total,
            );
            println!("         A {}", entry.outcome.a_marks.join(" · "));
            println!("         B {}", entry.outcome.b_marks.join(" · "));
        }
        println!();
    }
    Ok(ExitCode::SUCCESS)
}

// ---------------------------------------------------------------------------
// The representation audit — sprint:14, task:24
// ---------------------------------------------------------------------------

fn enumeration_mode(options: &Options) -> Result<ExitCode, String> {
    let (_, reports) = gauntlet::enumeration();
    let families = [
        "informative",
        "noise",
        "rare",
        "redundant",
        "accidental",
        "diluted",
        "competing",
    ];

    if options.json {
        let document = serde_json::json!({
            "label": "identifiability",
            "role": "representation audit — sprint:14, task:24",
            "realizations": gauntlet::REALIZATIONS,
            "trials": 300,
            "families": reports,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&document)
                .map_err(|err| format!("could not render JSON: {err}"))?
        );
        return Ok(ExitCode::SUCCESS);
    }

    println!("event-motif — the sprint:14 representation audit");
    println!(
        "Every function below is a function of the mark-only representation and of nothing else. \
         The trials, the families, the expectations, and the pass rule are frozen."
    );
    println!(
        "task:24 §A.2 already settled that Family E's arms are identifiable from that \
         representation; this asks whether any simple function of it reconciles E with the rest."
    );
    println!();
    print!("  {:<28}", "function");
    for family in families {
        print!(" {:>12}", family);
    }
    println!("  {:>6}", "clean");
    for scorer in identifiability::SCORERS.iter() {
        let name = if scorer.probe {
            format!("{} *", scorer.name)
        } else {
            scorer.name.to_owned()
        };
        print!("  {name:<28}");
        let mut clean = true;
        for family in families {
            let cell = reports
                .iter()
                .find(|r| r.statistic == scorer.name && r.family.label() == family);
            match cell {
                Some(report) => {
                    if report.verdict != gauntlet::Verdict::Pass {
                        clean = false;
                    }
                    print!(" {:>12}", report.verdict.label());
                }
                None => {
                    clean = false;
                    print!(" {:>12}", "-");
                }
            }
        }
        println!("  {:>6}", if clean { "YES" } else { "no" });
    }
    println!();
    println!("  * a probe: an inverse-frequency weighting, admissible in an identifiability");
    println!("    enumeration and inadmissible as a proposed statistic.");
    println!();
    println!("  Family E's column, in detail:");
    println!(
        "  {:<28} {:>8} {:>10} {:>10} {:>8}",
        "function", "trials", "frac", "median", "verdict"
    );
    for scorer in identifiability::SCORERS.iter() {
        if let Some(report) = reports
            .iter()
            .find(|r| r.statistic == scorer.name && r.family == gauntlet::Family::Redundant)
        {
            println!(
                "  {:<28} {:>8} {:>10.3} {:>10.4} {:>8}",
                scorer.name,
                report.trials,
                report.expected_fraction,
                report.median,
                report.verdict.label(),
            );
        }
    }
    println!();
    Ok(ExitCode::SUCCESS)
}

// ---------------------------------------------------------------------------
// Adversarial commissioning — sprint:15, task:25
// ---------------------------------------------------------------------------

fn adversarial_mode(options: &Options) -> Result<ExitCode, String> {
    let families = adversarial::families();

    if options.json {
        let document = serde_json::json!({
            "label": "adversarial",
            "role": "adversarial commissioning — sprint:15, task:25",
            "under_test": adversarial::UNDER_TEST,
            "adversarial_families": families,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&document)
                .map_err(|err| format!("could not render JSON: {err}"))?
        );
        return Ok(ExitCode::SUCCESS);
    }

    println!(
        "event-motif — the sprint:15 adversarial commissioning of {}",
        adversarial::UNDER_TEST
    );
    println!(
        "The statistic is frozen and is not adopted here. sprint:12's gauntlet is regression; these \
         families are the fresh evidence."
    );
    println!("Expectations, sweeps, and predictions were fixed before any family ran.");
    println!();
    println!(
        "  {:<30} {:>10} {:>9} {:>7}   first failing point",
        "family", "predicted", "result", "points"
    );
    for family in &families {
        println!(
            "  {:<30} {:>10} {:>9} {:>7}   {}",
            family.name,
            family.predicted.label(),
            family.verdict.label(),
            family.points.len(),
            family.boundary.clone().unwrap_or_else(|| "—".to_owned()),
        );
    }
    println!();
    for family in &families {
        println!("  -- {} --", family.name);
        println!("     construction: {}", family.construction);
        println!("     invariant:    {}", family.invariant);
        println!("     risk:         {}", family.mechanism);
        println!(
            "     {:<34} {:>10} {:>10}  holds",
            "point", "weaker", "stronger"
        );
        for value in &family.points {
            println!(
                "     {:<34} {:>10.3} {:>10.3}  {}{}",
                value.params,
                value.weaker,
                value.stronger,
                if value.holds { "yes" } else { "NO" },
                if value.nominal { "   (nominal)" } else { "" },
            );
        }
        println!();
    }
    Ok(ExitCode::SUCCESS)
}

// ---------------------------------------------------------------------------
// The operating-envelope exposure study — sprint:16, task:26
// ---------------------------------------------------------------------------

/// The span lengths sprint:9 and sprint:10 actually produce, frozen.
const OBSERVED_SPANS: [usize; 7] = [3, 4, 5, 6, 8, 10, 12];

fn envelope_mode(options: &Options) -> Result<ExitCode, String> {
    if options.corpus.len() < 2 {
        return Err("--envelope needs at least two --corpus <PATH> recordings".to_owned());
    }

    // Replays and inspections must outlive the sequences that borrow them.
    let mut replays = Vec::new();
    for path in &options.corpus {
        replays.push((
            path.clone(),
            replay_file(path)
                .map_err(|err| format!("could not replay {}: {err}", path.display()))?,
        ));
    }
    let inspections: Vec<_> = replays
        .iter()
        .map(|(path, replay)| (path.clone(), inspect(replay)))
        .collect();
    let mut sequences = Vec::new();
    for (path, inspection) in &inspections {
        match project(inspection, options.scope) {
            Some(sequence) => sequences.push((path.clone(), sequence)),
            None => eprintln!("skipping {}: no records in scope", path.display()),
        }
    }

    let profiles: Vec<_> = sequences
        .iter()
        .map(|(path, sequence)| (path.clone(), envelope::profile(sequence)))
        .collect();

    // Asymmetry over every unordered pair, using the frozen sprint:9 ladder and
    // the frozen cross-recording search. No new search procedure.
    let mut samples = Vec::new();
    let mut orderings = Vec::new();
    let mut crossings: Vec<envelope::Crossing> = Vec::new();
    for (index, (_, left)) in sequences.iter().enumerate() {
        for (_, right) in sequences.iter().skip(index + 1) {
            for k in [3usize, 4, 6, 8, 12] {
                let Some(ranked) = cross_pairs(left, right, k, usize::MAX) else {
                    continue;
                };
                let kept = dedupe_overlapping(&ranked, 5);
                let mut per_pair = Vec::new();
                for candidate in &kept {
                    let (wa, wb) = (&candidate.comparison.a, &candidate.comparison.b);
                    if let Some(sample) = envelope::asymmetry_of(
                        left,
                        (wa.start, wa.start + wa.k),
                        right,
                        (wb.start, wb.start + wb.k),
                        &format!("cross_pairs k={k}"),
                    ) {
                        per_pair.push(sample);
                    }
                }
                let origin_label = format!(
                    "{} x {} k={k}",
                    per_pair
                        .first()
                        .map(|s| s.a_session.clone())
                        .unwrap_or_default(),
                    per_pair
                        .first()
                        .map(|s| s.b_session.clone())
                        .unwrap_or_default(),
                );
                crossings.extend(envelope::crossings(&origin_label, &per_pair));
                if let Some(check) = envelope::ordering_check(
                    &format!(
                        "{} x {} k={k}",
                        per_pair
                            .first()
                            .map(|s| s.a_session.clone())
                            .unwrap_or_default(),
                        per_pair
                            .first()
                            .map(|s| s.b_session.clone())
                            .unwrap_or_default(),
                    ),
                    &per_pair,
                ) {
                    orderings.push(check);
                }
                samples.extend(per_pair);
            }
        }
    }

    if options.json {
        let document = serde_json::json!({
            "label": "envelope",
            "role": "operating-envelope exposure study — sprint:16, task:26",
            "under_study": envelope::UNDER_STUDY,
            "profiles": profiles.iter().map(|(_, profile)| profile).collect::<Vec<_>>(),
            "approaches": profiles
                .iter()
                .map(|(_, profile)| serde_json::json!({
                    "session": profile.session,
                    "approaches": envelope::approaches(profile, &OBSERVED_SPANS),
                }))
                .collect::<Vec<_>>(),
            "asymmetry": samples,
            "orderings": orderings,
            "crossings": crossings,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&document)
                .map_err(|err| format!("could not render JSON: {err}"))?
        );
        return Ok(ExitCode::SUCCESS);
    }

    println!("event-motif — the sprint:16 operating-envelope exposure study");
    println!(
        "Statistic under study: {}. Frozen; this measures exposure to known failure surfaces, \
         not accuracy.",
        envelope::UNDER_STUDY
    );
    println!("Output derived from real recordings is as sensitive as those recordings.");
    println!();

    println!("  corpus profiles, {} scope:", options.scope.label());
    println!(
        "  {:<10} {:>7} {:>6} {:>9} {:>9} {:>10} {:>11}",
        "session", "events", "vocab", "max count", "max freq", "singletons", "median freq"
    );
    for (_, profile) in &profiles {
        println!(
            "  {:<10} {:>7} {:>6} {:>9} {:>9.4} {:>10} {:>11.4}",
            profile.session,
            profile.events,
            profile.vocabulary,
            profile.max_count,
            profile
                .frequencies
                .first()
                .map(|f| f.frequency)
                .unwrap_or(0.0),
            profile.singletons,
            profile
                .frequency_deciles
                .get(5)
                .copied()
                .unwrap_or(f64::NAN),
        );
    }
    println!();

    println!(
        "  accumulation surface — a singleton agreement beats a k-agreement motif when c > N^((k-1)/k):"
    );
    println!(
        "  {:<10} {:>3} {:>10} {:>10} {:>10} {:>8} {:>8} {:>14}",
        "session", "k", "boundary", "max count", "abs margin", "rel", "above", "constructible"
    );
    for (_, profile) in &profiles {
        for approach in envelope::approaches(profile, &OBSERVED_SPANS) {
            println!(
                "  {:<10} {:>3} {:>10.1} {:>10} {:>+10.1} {:>8.2} {:>8} {:>14}",
                profile.session,
                approach.k,
                approach.boundary,
                approach.max_count,
                approach.absolute_margin,
                approach.relative_margin,
                approach.marks_above,
                if approach.constructible { "YES" } else { "no" },
            );
        }
    }
    println!();

    println!(
        "  observed accumulation crossings — a candidate outscoring one with strictly more \
         agreements:"
    );
    if crossings.is_empty() {
        println!("    none in {} candidate sets", orderings.len());
    } else {
        println!(
            "    {} found across {} candidate sets",
            crossings.len(),
            orderings.len()
        );
        let worst = crossings.iter().max_by(|left, right| {
            left.margin
                .partial_cmp(&right.margin)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for crossing in crossings.iter().take(4) {
            println!(
                "      {} — {} agreements scored {:.3}, beating {} agreements at {:.3} (margin {:+.3})",
                crossing.origin,
                crossing.fewer_agreements,
                crossing.fewer_score,
                crossing.more_agreements,
                crossing.more_score,
                crossing.margin,
            );
        }
        if let Some(worst) = worst {
            println!(
                "      largest margin {:+.3} nats: {} agreements over {}",
                worst.margin, worst.fewer_agreements, worst.more_agreements
            );
        }
    }
    println!();

    let deltas: Vec<f64> = samples.iter().map(|sample| sample.delta).collect();
    let zero = deltas.iter().filter(|delta| **delta < 1e-12).count();
    println!(
        "  asymmetry over {} real candidate pairs from the frozen machinery:",
        samples.len()
    );
    println!(
        "    delta = 0 in {} of {} ({:.1}%)",
        zero,
        deltas.len(),
        100.0 * zero as f64 / deltas.len().max(1) as f64
    );
    let q = envelope::quantiles(&deltas);
    if q.len() == 6 {
        println!(
            "    quantiles  min {:.3}  q1 {:.3}  median {:.3}  q3 {:.3}  p90 {:.3}  max {:.3}  nats",
            q[0], q[1], q[2], q[3], q[4], q[5]
        );
    }
    if let Some(worst) = samples.iter().max_by(|left, right| {
        left.delta
            .partial_cmp(&right.delta)
            .unwrap_or(std::cmp::Ordering::Equal)
    }) {
        println!(
            "    largest: {} x {} {} span {} agreements {} — forward {:.3} backward {:.3} delta {:.3}",
            worst.a_session,
            worst.b_session,
            worst.origin,
            worst.span,
            worst.agreements,
            worst.forward,
            worst.backward,
            worst.delta
        );
    }
    let changed = orderings.iter().filter(|check| check.pick_changed).count();
    let inversions: usize = orderings.iter().map(|check| check.inversions).sum();
    let comparisons: usize = orderings.iter().map(|check| check.comparisons).sum();
    println!(
        "    designated pick changed in {} of {} candidate sets; {} of {} pairwise orders reversed",
        changed,
        orderings.len(),
        inversions,
        comparisons
    );
    for check in orderings.iter().filter(|check| check.pick_changed) {
        println!(
            "      pick moved: {} — forward picks #{}, backward picks #{}",
            check.origin, check.forward_pick, check.backward_pick
        );
    }
    println!();
    Ok(ExitCode::SUCCESS)
}

// ---------------------------------------------------------------------------
// The comparative repair experiment — sprint:17, task:27
// ---------------------------------------------------------------------------

/// The rare/common pair task:27 §D2's witness is exhibited at.
const WITNESS_RARE: usize = 1;
const WITNESS_COMMON: usize = 500;

fn repair_mode(options: &Options) -> Result<ExitCode, String> {
    println!("event-motif — the sprint:17 comparative repair experiment");
    println!(
        "Candidates are compared, not adopted. task:27 §D2 proves rarity weighting implies\n\
         accumulation crossings, so no candidate is built to remove them or rejected for having them.\n"
    );

    // A. The contract, clause by clause.
    println!(
        "  semantic contract — task:27 §B. (free) marks a clause §D1 makes free by construction:"
    );
    println!(
        "  {:<26} {:>6} {:>6} {:>6} {:>6} {:>6} {:>6}",
        "candidate", "C1", "C2", "C3", "C4", "C5", "C6"
    );
    let reports = repair::contracts();
    for report in &reports {
        let cells: Vec<String> = report
            .clauses
            .iter()
            .map(|clause| {
                if clause.satisfied {
                    "ok".to_owned()
                } else {
                    "NO".to_owned()
                }
            })
            .collect();
        println!(
            "  {:<26} {:>6} {:>6} {:>6} {:>6} {:>6} {:>6}",
            report.candidate, cells[0], cells[1], cells[2], cells[3], cells[4], cells[5]
        );
    }
    println!(
        "  {:<26} {:>6} {:>6} {:>6} {:>6} {:>6} {:>6}",
        "", "(free)", "", "", "(free)", "(free)", ""
    );
    for report in &reports {
        for clause in report.violations() {
            println!(
                "    {} violates {}: {} = {:+.4}   [{}]",
                report.candidate, clause.clause, clause.quantity, clause.value, clause.witness
            );
        }
    }

    // B. §D2's crossing theorem, exhibited constructively.
    println!(
        "\n  the crossing theorem — task:27 §D2, exhibited at rare c={WITNESS_RARE}, common c={WITNESS_COMMON}:"
    );
    println!(
        "  {:<26} {:>8} {:>8} {:>10} {:>10}  crosses",
        "candidate", "fewer k", "more k", "fewer", "more"
    );
    for witness in repair::crossing_witnesses(WITNESS_RARE, WITNESS_COMMON) {
        if witness.crossed {
            println!(
                "  {:<26} {:>8} {:>8} {:>10.3} {:>10.3}  YES",
                witness.candidate,
                witness.fewer,
                witness.more,
                witness.fewer_score,
                witness.more_score
            );
        } else {
            println!(
                "  {:<26} {:>8} {:>8} {:>10} {:>10}  no (k ≤ 24)",
                witness.candidate, "—", "—", "—", "—"
            );
        }
    }

    // C. The ten sprint:15 families, unchanged, under every candidate.
    println!("\n  the ten sprint:15 adversarial families, constructions unchanged, per candidate:");
    let per_candidate: Vec<_> = repair::CANDIDATES
        .iter()
        .map(|entry| (entry, adversarial::families_with(entry.score)))
        .collect();
    let names: Vec<&str> = per_candidate[0].1.iter().map(|f| f.name).collect();
    print!("  {:<32}", "family");
    for (entry, _) in &per_candidate {
        print!(
            " {:>12}",
            entry.name.split_whitespace().next().unwrap_or(entry.name)
        );
    }
    println!();
    for (index, name) in names.iter().enumerate() {
        print!("  {name:<32}");
        for (_, families) in &per_candidate {
            print!(
                " {:>12}",
                format!("{:?}", families[index].verdict).to_uppercase()
            );
        }
        println!();
    }

    println!("\n  first failing point per family, where one exists:");
    for (index, name) in names.iter().enumerate() {
        let boundaries: Vec<String> = per_candidate
            .iter()
            .map(|(entry, families)| {
                families[index]
                    .boundary
                    .clone()
                    .map(|point| {
                        format!(
                            "{}: {point}",
                            entry.name.split_whitespace().next().unwrap_or(entry.name)
                        )
                    })
                    .unwrap_or_default()
            })
            .filter(|text| !text.is_empty())
            .collect();
        if !boundaries.is_empty() {
            println!("    {name:<32} {}", boundaries.join("   "));
        }
    }

    // Point-level identity between S0 and R1, which §D4 predicts wherever the
    // two recordings share marginals.
    let frozen = &per_candidate[0].1;
    let pooled = &per_candidate[1].1;
    let mut identical = 0usize;
    let mut differing = Vec::new();
    for (left, right) in frozen.iter().zip(pooled.iter()) {
        for (a, b) in left.points.iter().zip(right.points.iter()) {
            if (a.weaker - b.weaker).abs() <= 1e-12 && (a.stronger - b.stronger).abs() <= 1e-12 {
                identical += 1;
            } else {
                differing.push(format!("{} {}", left.name, a.params));
            }
        }
    }
    println!(
        "\n  §D4 check — S0 and R1 numerically identical at {identical} of {} family points",
        identical + differing.len()
    );
    if !differing.is_empty() {
        println!("    they differ only at: {}", differing.join(", "));
    }

    if options.corpus.len() < 2 {
        println!(
            "\n  real operating envelope not replayed: --repair needs at least two --corpus <PATH>."
        );
        return Ok(ExitCode::SUCCESS);
    }
    repair_envelope(options)
}

/// §E — replay every candidate against sprint:16's exact candidate sets.
fn repair_envelope(options: &Options) -> Result<ExitCode, String> {
    let mut replays = Vec::new();
    for path in &options.corpus {
        replays.push((
            path.clone(),
            replay_file(path)
                .map_err(|err| format!("could not replay {}: {err}", path.display()))?,
        ));
    }
    let inspections: Vec<_> = replays
        .iter()
        .map(|(path, replay)| (path.clone(), inspect(replay)))
        .collect();
    let mut sequences = Vec::new();
    for (path, inspection) in &inspections {
        if let Some(sequence) = project(inspection, options.scope) {
            sequences.push(sequence);
        } else {
            eprintln!("skipping {}: no records in scope", path.display());
        }
    }

    println!("\n  real operating envelope — sprint:16's exact candidate sets, per candidate:");
    println!(
        "  {:<26} {:>7} {:>9} {:>9} {:>8} {:>10} {:>10}",
        "candidate", "pairs", "delta=0", "med delta", "max", "crossings", "picks moved"
    );

    let mut crossing_signatures: Vec<(String, Vec<String>)> = Vec::new();

    for entry in repair::CANDIDATES.iter() {
        let mut samples = Vec::new();
        let mut sets = 0usize;
        let mut picks_moved = 0usize;
        let mut reversals = 0usize;
        let mut orders = 0usize;
        let mut crossings: Vec<envelope::Crossing> = Vec::new();

        for (index, left) in sequences.iter().enumerate() {
            for right in sequences.iter().skip(index + 1) {
                for k in [3usize, 4, 6, 8, 12] {
                    let Some(ranked) = cross_pairs(left, right, k, usize::MAX) else {
                        continue;
                    };
                    let kept = dedupe_overlapping(&ranked, 5);
                    let mut per_pair = Vec::new();
                    for candidate in &kept {
                        let (wa, wb) = (&candidate.comparison.a, &candidate.comparison.b);
                        if let Some(sample) = envelope::asymmetry_with(
                            entry.score,
                            left,
                            (wa.start, wa.start + wa.k),
                            right,
                            (wb.start, wb.start + wb.k),
                            &format!("cross_pairs k={k}"),
                        ) {
                            per_pair.push(sample);
                        }
                    }
                    if per_pair.is_empty() {
                        continue;
                    }
                    let label = format!(
                        "{} x {} k={k}",
                        per_pair[0].a_session, per_pair[0].b_session
                    );
                    crossings.extend(envelope::crossings(&label, &per_pair));
                    if let Some(check) = envelope::ordering_check(&label, &per_pair) {
                        // Counted exactly as sprint:16 counted it: a set needs
                        // two candidates before a pick can move. One set in this
                        // corpus is a singleton and is excluded here as it was
                        // there, which is what makes 29 comparable to 29.
                        sets += 1;
                        if check.pick_changed {
                            picks_moved += 1;
                        }
                        reversals += check.inversions;
                        orders += check.comparisons;
                    }
                    samples.extend(per_pair);
                }
            }
        }

        let mut deltas: Vec<f64> = samples.iter().map(|s| s.delta).collect();
        deltas.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let exact = samples.iter().filter(|s| s.delta <= 1e-12).count();
        let quantiles = envelope::quantiles(&deltas);
        let median = quantiles.get(2).copied().unwrap_or(f64::NAN);
        let max = deltas.last().copied().unwrap_or(f64::NAN);

        println!(
            "  {:<26} {:>7} {:>9} {:>9.3} {:>8.3} {:>10} {:>10}",
            entry.name,
            samples.len(),
            format!("{}/{}", exact, samples.len()),
            median,
            max,
            crossings.len(),
            format!("{picks_moved}/{sets}")
        );
        if reversals > 0 || orders > 0 {
            println!("      pairwise orders reversed: {reversals} of {orders}");
        }

        let mut signature: Vec<String> = crossings
            .iter()
            .map(|c| format!("{}|{}|{}", c.origin, c.fewer_agreements, c.more_agreements))
            .collect();
        signature.sort();
        crossing_signatures.push((entry.name.to_owned(), signature));
    }

    // §D3's falsification target: R1 and R3 must produce identical crossings.
    let r1 = crossing_signatures
        .iter()
        .find(|(name, _)| name.starts_with("R1"));
    let r3 = crossing_signatures
        .iter()
        .find(|(name, _)| name.starts_with("R3"));
    if let (Some((_, a)), Some((_, b))) = (r1, r3) {
        println!(
            "\n  §D3 falsification target — R1 and R3 crossings identical: {}  ({} vs {})",
            if a == b {
                "YES, as derived"
            } else {
                "NO — the derivation is unsound"
            },
            a.len(),
            b.len()
        );
    }

    println!(
        "\n  Output derived from real recordings is as sensitive as those recordings.\n\
         Counts and frequencies only; decision:8 forbids publishing contents."
    );
    Ok(ExitCode::SUCCESS)
}
