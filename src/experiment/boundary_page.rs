//! One small static page over the sprint:10 boundary-refinement specimens.
//!
//! **Disposable.** sprint:10, task:20. A separate module so deleting the
//! visualization is deleting one file.
//!
//! # What it is
//!
//! A function from the JSON documents `event-motif --refine --json` produced to
//! a self-contained HTML string. **It holds no measurement of its own** — every
//! number it prints is read out of a document the experiment computed, and
//! `tests/event_sequence.rs` asserts that by rendering a real refinement and
//! looking for its distances on the page.
//!
//! # What it is not
//!
//! Not a visualization framework, not a second application, not a change to the
//! Behavioral Spectroscope, and not served by anything. It writes a file. There
//! is no transport, no capability, no listener, and nothing that outlives the
//! command.
//!
//! # Hygiene
//!
//! A page rendered over a real recording carries that recording's delivered
//! marks and timings and is exactly as sensitive as the recording. task:20 §9
//! commits this generator and not its output.

// ---------------------------------------------------------------------------

/// Escape text for HTML. Every string on the page is a delivered mark, a label,
/// or a number this experiment computed, and none of it is trusted as markup.
pub(crate) fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn as_number(value: &serde_json::Value) -> f64 {
    value.as_f64().unwrap_or(f64::NAN)
}

fn as_integer(value: &serde_json::Value) -> i64 {
    value.as_i64().unwrap_or(0)
}

/// `[start..start+k)` for a serialized window.
fn window_bounds(window: &serde_json::Value) -> (i64, i64) {
    let start = as_integer(&window["start"]);
    (start, start + as_integer(&window["k"]))
}

/// One horizontal band: a labelled extent drawn against a shared index scale.
fn band(class: &str, label: &str, from: i64, to: i64, lower: i64, upper: i64) -> String {
    let width = (upper - lower).max(1) as f64;
    let left = 100.0 * (from - lower) as f64 / width;
    let size = 100.0 * (to - from) as f64 / width;
    format!(
        "<div class=\"band {class}\" style=\"left:{left:.3}%;width:{size:.3}%\">\
         <span>{} {}..{}</span></div>",
        escape(label),
        from,
        to
    )
}

fn strip(
    side: &str,
    seed: (i64, i64),
    pick: Option<(i64, i64)>,
    truth: Option<(i64, i64)>,
) -> String {
    let mut edges = vec![seed.0, seed.1];
    for extra in [pick, truth].into_iter().flatten() {
        edges.push(extra.0);
        edges.push(extra.1);
    }
    let lower = edges.iter().copied().min().unwrap_or(0) - 1;
    let upper = edges.iter().copied().max().unwrap_or(1) + 1;

    let mut rows = String::new();
    rows.push_str(&band("seed", "seed", seed.0, seed.1, lower, upper));
    if let Some(pick) = pick {
        rows.push_str(&band("pick", "pick", pick.0, pick.1, lower, upper));
    }
    if let Some(truth) = truth {
        rows.push_str(&band("truth", "planted", truth.0, truth.1, lower, upper));
    }
    format!(
        "<div class=\"strip\"><div class=\"axis\">{side} — event index {lower} to {upper}</div>\
         <div class=\"bands\">{rows}</div></div>"
    )
}

fn distances(alignment: &serde_json::Value) -> String {
    format!(
        "<span class=\"num\">ev {:.3}</span><span class=\"num\">tm {:.3}</span>\
         <span class=\"num strong\">tot {:.3}</span>",
        as_number(&alignment["event_norm"]),
        as_number(&alignment["timing_norm"]),
        as_number(&alignment["total"]),
    )
}

fn specimen_card(document: &serde_json::Value) -> String {
    let refinement = &document["refinement"];
    let seed = &refinement["seed"];
    let seed_a = window_bounds(&seed["pair"]["comparison"]["a"]);
    let seed_b = window_bounds(&seed["pair"]["comparison"]["b"]);
    let pick = refinement.get("pick").filter(|value| !value.is_null());
    let pick_a = pick.map(|p| window_bounds(&p["pair"]["comparison"]["a"]));
    let pick_b = pick.map(|p| window_bounds(&p["pair"]["comparison"]["b"]));
    let truth_a = document["truth_a"]
        .as_array()
        .map(|pair| (as_integer(&pair[0]), as_integer(&pair[1])));
    let truth_b = document["truth_b"]
        .as_array()
        .map(|pair| (as_integer(&pair[0]), as_integer(&pair[1])));

    let mut rows = String::new();
    for candidate in refinement["frontier"].as_array().into_iter().flatten() {
        let (a, b) = (
            window_bounds(&candidate["pair"]["comparison"]["a"]),
            window_bounds(&candidate["pair"]["comparison"]["b"]),
        );
        let alignment = &candidate["pair"]["comparison"]["alignment"];
        let is_truth = Some(a) == truth_a && Some(b) == truth_b;
        let is_pick = Some(a) == pick_a && Some(b) == pick_b;
        let mut mark = String::new();
        if is_truth {
            mark.push_str(" planted");
        }
        if is_pick {
            mark.push_str(" pick");
        }
        rows.push_str(&format!(
            "<tr class=\"{}\"><td>{}</td><td>A[{}..{})</td><td>B[{}..{})</td>\
             <td>{}</td><td>{}</td><td>{:.3}</td><td>{:.3}</td><td class=\"strong\">{:.3}</td>\
             <td>{}</td></tr>",
            mark.trim(),
            as_integer(&candidate["retained"]),
            a.0,
            a.1,
            b.0,
            b.1,
            as_integer(&candidate["pair"]["comparison"]["a"]["distinct_marks"]),
            as_integer(&candidate["pair"]["comparison"]["b"]["distinct_marks"]),
            as_number(&alignment["event_norm"]),
            as_number(&alignment["timing_norm"]),
            as_number(&alignment["total"]),
            escape(mark.trim()),
        ));
    }

    let truth_note = match (truth_a, truth_b) {
        (Some(a), Some(b)) => {
            let on_frontier =
                refinement["frontier"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .any(|candidate| {
                        window_bounds(&candidate["pair"]["comparison"]["a"]) == a
                            && window_bounds(&candidate["pair"]["comparison"]["b"]) == b
                    });
            format!(
                "<p class=\"truth\">planted boundaries A[{}..{}) B[{}..{}) — on the frontier: \
                 <strong>{}</strong></p>",
                a.0,
                a.1,
                b.0,
                b.1,
                if on_frontier { "yes" } else { "no" }
            )
        }
        _ => "<p class=\"truth\">no planted boundaries: this specimen has no ground truth</p>"
            .to_owned(),
    };

    format!(
        "<section class=\"card\">\
         <h2>{}</h2><p class=\"role\">{}</p>\
         <p class=\"meta\">radius {} · floor {} · {} boundary combinations scored · {} rejected</p>\
         <p class=\"line\">seed {} &rarr; pick {}</p>\
         {truth_note}\
         {}{}\
         <table><thead><tr><th>retained</th><th>A span</th><th>B span</th><th>A marks</th>\
         <th>B marks</th><th>event</th><th>timing</th><th>total</th><th></th></tr></thead>\
         <tbody>{rows}</tbody></table>\
         </section>",
        escape(document["label"].as_str().unwrap_or("specimen")),
        escape(document["role"].as_str().unwrap_or("unlabelled")),
        as_integer(&refinement["radius"]),
        as_integer(&refinement["floor"]),
        as_integer(&refinement["evaluated"]),
        as_integer(&refinement["rejected"]),
        distances(&seed["pair"]["comparison"]["alignment"]),
        match pick {
            Some(pick) => distances(&pick["pair"]["comparison"]["alignment"]),
            None => "<span class=\"num\">none</span>".to_owned(),
        },
        strip("A", seed_a, pick_a, truth_a),
        strip("B", seed_b, pick_b, truth_b),
    )
}

const PAGE_STYLE: &str = "
:root { color-scheme: light dark; --ink: #16181d; --paper: #fbfbfa; --line: #d5d3cd;
        --seed: #8a8580; --pick: #1f6feb; --truth: #b8860b; }
@media (prefers-color-scheme: dark) { :root { --ink: #e6e4e0; --paper: #14161a; --line: #333;
        --seed: #6f6a65; --pick: #58a6ff; --truth: #d9a441; } }
body { background: var(--paper); color: var(--ink); margin: 0 auto; max-width: 60rem;
       padding: 2rem 1.25rem 4rem; font: 15px/1.55 ui-monospace, SFMono-Regular, Menlo, monospace; }
h1 { font-size: 1.2rem; letter-spacing: .02em; }
.intro { border-left: 3px solid var(--line); padding-left: .9rem; }
.card { border: 1px solid var(--line); border-radius: 6px; padding: 1rem 1.1rem; margin: 1.5rem 0; }
h2 { font-size: 1rem; margin: 0; }
.role { margin: .15rem 0 .8rem; opacity: .75; }
.meta, .truth { opacity: .8; margin: .35rem 0; }
.line { margin: .6rem 0; }
.num { display: inline-block; margin-right: .7rem; opacity: .8; }
.num.strong, .strong { opacity: 1; font-weight: 600; }
.strip { margin: .8rem 0; }
.axis { opacity: .6; font-size: .85em; margin-bottom: .2rem; }
.bands { position: relative; height: 4.4rem; border: 1px solid var(--line); border-radius: 3px; }
.band { position: absolute; height: 1.2rem; border-radius: 2px; }
.band span { position: absolute; left: .35rem; top: .05rem; font-size: .72rem;
             white-space: nowrap; color: var(--paper); }
.band.seed  { top: .3rem;  background: var(--seed); }
.band.pick  { top: 1.7rem; background: var(--pick); }
.band.truth { top: 3.1rem; background: var(--truth); }
table { border-collapse: collapse; width: 100%; margin-top: .9rem; font-size: .9em; }
th, td { text-align: right; padding: .18rem .45rem; border-bottom: 1px solid var(--line); }
th:nth-child(2), td:nth-child(2), th:nth-child(3), td:nth-child(3) { text-align: left; }
tr.planted td { color: var(--truth); }
tr.pick td { color: var(--pick); }
tr.planted.pick td { color: var(--pick); }
";

/// Render one page from specimen documents.
pub fn render(documents: &[serde_json::Value]) -> String {
    let cards: String = documents.iter().map(specimen_card).collect();
    format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\">\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
         <title>WitnessGlass — boundary refinement (experimental)</title>\
         <style>{PAGE_STYLE}</style></head><body>\
         <h1>WitnessGlass — local boundary refinement</h1>\
         <div class=\"intro\">\
         <p>sprint:10, task:20. A disposable experiment, not a product surface. Every number on \
         this page was computed by <code>event-motif --refine</code> and read from its JSON; \
         nothing here measures anything.</p>\
         <p>Each specimen shows the seed boundaries, the designated pick, the planted boundaries \
         where a fixture has them, and the Pareto frontier over retained events against total \
         distance. A row on the frontier retains fewer events than the row above it and scores \
         strictly better, so what each discarded event bought is visible.</p>\
         <p>Marks are the delivered event kind and tool-name string, verbatim. No figure here is \
         given a workflow name; the detector does not know one.</p>\
         </div>{cards}</body></html>"
    )
}
