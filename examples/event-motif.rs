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
    ChannelScope, Comparison, CrossPair, EventSequence, cross_pairs, dedupe_overlapping, ladder,
    neighbours, order_null, perturbation, project, timing_null, top_pairs, top_pairs_where,
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
