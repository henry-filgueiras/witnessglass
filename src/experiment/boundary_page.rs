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

/// A point in one of the two panels: a candidate's retained length against a
/// value. Every coordinate comes from the computed document.
fn plot_point(
    retained: f64,
    value: f64,
    lower: f64,
    upper: f64,
    span: (f64, f64),
    class: &str,
    radius: f64,
) -> String {
    // Longest span on the left, shortest on the right, so the page reads in the
    // direction the argument does: "raw agreement keeps improving as spans
    // shorten". The caption states the same order.
    let x = 6.0 + 88.0 * (span.1 - retained) / (span.1 - span.0).max(1.0);
    let y = 92.0 - 84.0 * (value - lower) / (upper - lower).max(1e-9);
    format!("<circle class=\"{class}\" cx=\"{x:.2}\" cy=\"{y:.2}\" r=\"{radius}\"/>")
}

/// One panel: a scatter of every evaluated candidate, with the frontier and any
/// marked spans drawn on top.
///
/// The visual question task:21 exists to answer is whether raw agreement keeps
/// improving as spans shorten while surprise peaks somewhere richer, so the two
/// panels share an x axis and are stacked.
fn panel(
    title: &str,
    cloud: &[(f64, f64)],
    frontier: &[(f64, f64)],
    marked: &[(f64, f64, &str)],
    span: (f64, f64),
) -> String {
    let values: Vec<f64> = cloud
        .iter()
        .chain(frontier.iter())
        .map(|(_, value)| *value)
        .filter(|value| value.is_finite())
        .collect();
    if values.is_empty() {
        return String::new();
    }
    let lower = values
        .iter()
        .copied()
        .fold(f64::INFINITY, f64::min)
        .min(0.0);
    let upper = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);

    let mut marks = String::new();
    for (retained, value) in cloud {
        marks.push_str(&plot_point(
            *retained, *value, lower, upper, span, "cloud", 0.55,
        ));
    }
    for (retained, value) in frontier {
        marks.push_str(&plot_point(
            *retained, *value, lower, upper, span, "front", 1.3,
        ));
    }
    for (retained, value, class) in marked {
        marks.push_str(&plot_point(
            *retained, *value, lower, upper, span, class, 2.2,
        ));
    }
    format!(
        "<figure class=\"panel\"><figcaption>{}  <span class=\"axis\">y {:.3} to {:.3} · x retained {} (long) to {} (short)</span></figcaption>\
         <svg viewBox=\"0 0 100 100\" preserveAspectRatio=\"none\" role=\"img\">\
         <line class=\"rule\" x1=\"6\" y1=\"92\" x2=\"94\" y2=\"92\"/>\
         <line class=\"rule\" x1=\"6\" y1=\"8\" x2=\"6\" y2=\"92\"/>{marks}</svg></figure>",
        escape(title),
        lower,
        upper,
        span.1 as i64,
        span.0 as i64,
    )
}

/// A candidate's null distribution, drawn rather than summarized.
fn histogram(evidence: &serde_json::Value, label: &str) -> String {
    let Some(bins) = evidence["histogram"].as_array() else {
        return String::new();
    };
    let counts: Vec<f64> = bins.iter().map(as_number).collect();
    let peak = counts.iter().copied().fold(0.0f64, f64::max).max(1.0);
    let width = 100.0 / counts.len().max(1) as f64;
    let mut bars = String::new();
    for (index, count) in counts.iter().enumerate() {
        let height = 74.0 * count / peak;
        bars.push_str(&format!(
            "<rect class=\"bin\" x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\"/>",
            index as f64 * width,
            80.0 - height,
            width * 0.86,
            height
        ));
    }
    let observed = as_number(&evidence["observed"]).clamp(0.0, 1.0) * 100.0;
    format!(
        "<figure class=\"hist\"><figcaption>{} · null distance distribution over {} realizations, \
         observed marked</figcaption>\
         <svg viewBox=\"0 0 100 92\" preserveAspectRatio=\"none\" role=\"img\">{bars}\
         <line class=\"observed\" x1=\"{observed:.2}\" y1=\"2\" x2=\"{observed:.2}\" y2=\"86\"/>\
         <line class=\"rule\" x1=\"0\" y1=\"80\" x2=\"100\" y2=\"80\"/></svg>\
         <p class=\"axis\">0.0 &mdash; distance &mdash; 1.0</p></figure>",
        escape(label),
        as_integer(&evidence["realizations"]),
    )
}

/// `[start, end)` of a document's `a`/`b` array, if it has one.
fn point_bounds(point: &serde_json::Value, side: &str) -> Option<(i64, i64)> {
    let pair = point[side].as_array()?;
    Some((as_integer(pair.first()?), as_integer(pair.get(1)?)))
}

/// The three null-referenced sections, or empty strings when a document carries
/// no null evidence. Everything is read from the document; nothing is measured.
fn null_sections(
    document: &serde_json::Value,
    truth_a: Option<(i64, i64)>,
    truth_b: Option<(i64, i64)>,
) -> (String, String, String) {
    let evidence = &document["null"];
    if evidence.is_null() {
        return (String::new(), String::new(), String::new());
    }
    let note_a = document["note_a"]
        .as_array()
        .map(|pair| (as_integer(&pair[0]), as_integer(&pair[1])));
    let note_b = document["note_b"]
        .as_array()
        .map(|pair| (as_integer(&pair[0]), as_integer(&pair[1])));
    let note_label = document["note_label"]
        .as_str()
        .unwrap_or("previously observed span");

    let empty = Vec::new();
    let cloud_points = evidence["geometry"]["points"].as_array().unwrap_or(&empty);
    let front_points = evidence["frontier"]["points"].as_array().unwrap_or(&empty);

    let is_marked = |point: &serde_json::Value, a: Option<(i64, i64)>, b: Option<(i64, i64)>| {
        matches!((a, b), (Some(a), Some(b))
            if point_bounds(point, "a") == Some(a) && point_bounds(point, "b") == Some(b))
    };
    // The raw-distance winner is identified from the data rather than named by a
    // caller: it is simply the lowest total on the frontier.
    let raw_best = front_points.iter().min_by(|left, right| {
        as_number(&left["total"])
            .partial_cmp(&as_number(&right["total"]))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let series = |points: &[serde_json::Value], key: &str| -> Vec<(f64, f64)> {
        points
            .iter()
            .filter_map(|point| {
                let value = if key == "total" {
                    as_number(&point["total"])
                } else {
                    point["null"]["total"]["standardized_separation"].as_f64()?
                };
                value
                    .is_finite()
                    .then(|| (as_number(&point["retained"]), value))
            })
            .collect()
    };
    let marked = |points: &[serde_json::Value], key: &str| -> Vec<(f64, f64, &'static str)> {
        points
            .iter()
            .filter_map(|point| {
                let value = if key == "total" {
                    as_number(&point["total"])
                } else {
                    point["null"]["total"]["standardized_separation"].as_f64()?
                };
                let class = if is_marked(point, truth_a, truth_b) {
                    "truthmark"
                } else if is_marked(point, note_a, note_b) {
                    "notemark"
                } else if raw_best.map(|best| best["retained"] == point["retained"]) == Some(true) {
                    "rawmark"
                } else {
                    return None;
                };
                value
                    .is_finite()
                    .then_some((as_number(&point["retained"]), value, class))
            })
            .collect()
    };

    let lengths: Vec<f64> = cloud_points
        .iter()
        .chain(front_points.iter())
        .map(|point| as_number(&point["retained"]))
        .collect();
    let span = (
        lengths.iter().copied().fold(f64::INFINITY, f64::min),
        lengths.iter().copied().fold(f64::NEG_INFINITY, f64::max),
    );
    if !span.0.is_finite() || !span.1.is_finite() {
        return (String::new(), String::new(), String::new());
    }

    let panels = format!(
        "<div class=\"panels\">{}{}</div>\
         <p class=\"legend\"><span class=\"key cloud\"></span> every candidate \
         <span class=\"key front\"></span> Pareto frontier \
         <span class=\"key truthmark\"></span> planted figure \
         <span class=\"key notemark\"></span> {} \
         <span class=\"key rawmark\"></span> raw-distance-preferred span</p>",
        panel(
            "raw agreement — total distance, lower is closer",
            &series(cloud_points, "total"),
            &series(front_points, "total"),
            &marked(front_points, "total"),
            span,
        ),
        panel(
            "null-relative surprise — standardized separation, higher is more exceptional",
            &series(cloud_points, "z"),
            &series(front_points, "z"),
            &marked(front_points, "z"),
            span,
        ),
        escape(note_label),
    );

    let mut rows = String::new();
    for point in front_points {
        let total = &point["null"]["total"];
        let label = if is_marked(point, truth_a, truth_b) {
            "planted figure"
        } else if is_marked(point, note_a, note_b) {
            note_label
        } else if raw_best.map(|best| best["retained"] == point["retained"]) == Some(true) {
            "raw-distance-preferred"
        } else {
            ""
        };
        rows.push_str(&format!(
            "<tr class=\"{}\"><td>{}</td><td>A[{}..{})</td><td>B[{}..{})</td><td>{:.3}</td>\
             <td>{:.3}</td><td>{:.3}</td><td>{:.2e}</td><td>{:.3}</td><td>{}</td><td>{}</td></tr>",
            match label {
                "planted figure" => "planted",
                "" => "",
                other if other == note_label => "noted",
                _ => "rawbest",
            },
            as_integer(&point["retained"]),
            point_bounds(point, "a").map(|(from, _)| from).unwrap_or(0),
            point_bounds(point, "a").map(|(_, to)| to).unwrap_or(0),
            point_bounds(point, "b").map(|(from, _)| from).unwrap_or(0),
            point_bounds(point, "b").map(|(_, to)| to).unwrap_or(0),
            as_number(&point["total"]),
            as_number(&total["null_mean"]),
            as_number(&total["null_stddev"]),
            as_number(&total["empirical_p"]),
            as_number(&total["separation"]),
            match total["standardized_separation"].as_f64() {
                Some(z) => format!("{z:.2}"),
                None => "&mdash;".to_owned(),
            },
            escape(label),
        ));
    }
    let evidence_table = if rows.is_empty() {
        String::new()
    } else {
        format!(
            "<p class=\"meta\">null-referenced evidence over the frontier, {} order-null \
             realizations of both recordings. The empirical tail floor is {:.1e}; a candidate at \
             the floor is rarer than this ensemble can resolve.</p>\
             <table><thead><tr><th>retained</th><th>A span</th><th>B span</th><th>total</th>\
             <th>null mean</th><th>null sd</th><th>emp p</th><th>separation</th><th>z</th>\
             <th></th></tr></thead><tbody>{rows}</tbody></table>",
            as_integer(&evidence["frontier"]["realizations"]),
            1.0 / (1 + as_integer(&evidence["frontier"]["realizations"])) as f64,
        )
    };

    let histograms: String = front_points
        .iter()
        .filter(|point| {
            is_marked(point, truth_a, truth_b)
                || is_marked(point, note_a, note_b)
                || raw_best.map(|best| best["retained"] == point["retained"]) == Some(true)
        })
        .map(|point| {
            let label = format!(
                "A[{}..{}) B[{}..{}), {} events",
                point_bounds(point, "a").map(|(from, _)| from).unwrap_or(0),
                point_bounds(point, "a").map(|(_, to)| to).unwrap_or(0),
                point_bounds(point, "b").map(|(from, _)| from).unwrap_or(0),
                point_bounds(point, "b").map(|(_, to)| to).unwrap_or(0),
                as_integer(&point["retained"]),
            );
            histogram(&point["null"]["total"], &label)
        })
        .collect();

    (
        panels,
        evidence_table,
        format!("<div class=\"hists\">{histograms}</div>"),
    )
}

/// The gauntlet scorecard, scatter, and counterexample tables.
///
/// Selected when a document carries `families` rather than a `refinement`.
/// Every number is read from the computed report; nothing here scores anything.
fn gauntlet_card(document: &serde_json::Value) -> String {
    let empty = Vec::new();
    let families = document["families"].as_array().unwrap_or(&empty);

    let mut scorecard = String::new();
    for family in families {
        let verdict = family["verdict"].as_str().unwrap_or("");
        scorecard.push_str(&format!(
            "<tr class=\"{}\"><td>{}</td><td class=\"stat\">{}</td><td>{}</td><td>{}</td>\
             <td>{}</td><td>{:.3}</td><td>{:+.3}</td><td>{:+.3}</td>\
             <td class=\"strong\">{}</td></tr>",
            verdict.to_lowercase(),
            escape(family["family"].as_str().unwrap_or("")),
            escape(family["statistic"].as_str().unwrap_or("")),
            escape(family["expectation"].as_str().unwrap_or("")),
            as_integer(&family["trials"]),
            as_integer(&family["undefined"]),
            as_number(&family["expected_fraction"]),
            as_number(&family["median"]),
            as_number(&family["median_delta_total"]),
            escape(&verdict.to_uppercase()),
        ));
    }

    // With ten functions the per-family scatter becomes seventy panels of noise,
    // so the audit renders its matrix and its counterexamples without them.
    let mut panels = String::new();
    for family in families.iter().take(if families.len() > 20 {
        0
    } else {
        families.len()
    }) {
        let surprisal = family["statistic"].as_str() == Some("surprisal");
        let points: Vec<(f64, f64)> = family["scored"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|entry| {
                let outcome = &entry["outcome"];
                let key = if surprisal { "delta_s" } else { "delta_z" };
                let delta = outcome[key].as_f64()?;
                Some((as_number(&outcome["delta_total"]), delta))
            })
            .filter(|(x, y)| x.is_finite() && y.is_finite())
            .collect();
        if points.is_empty() {
            continue;
        }
        panels.push_str(&quadrant(
            &format!(
                "{} [{}]",
                family["family"].as_str().unwrap_or(""),
                family["statistic"].as_str().unwrap_or("")
            ),
            &family["verdict"].as_str().unwrap_or("").to_uppercase(),
            &points,
        ));
    }

    let mut counterexamples = String::new();
    for family in families {
        let rows: String = family["counterexamples"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|entry| {
                let outcome = &entry["outcome"];
                let trial = &outcome["trial"];
                format!(
                    "<tr><td>{:+.4}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td>\
                     <td>{:+.4}</td><td class=\"marks\">{}<br>{}</td></tr>",
                    as_number(&entry["value"]),
                    as_integer(&trial["seed"]),
                    as_integer(&trial["core_len"]),
                    as_integer(&trial["context_len"]),
                    as_integer(&trial["gap_ratio"]),
                    as_number(&outcome["delta_total"]),
                    escape(&join_marks(&outcome["a_marks"])),
                    escape(&join_marks(&outcome["b_marks"])),
                )
            })
            .collect();
        counterexamples.push_str(&format!(
            "<h3>{} [{}] &mdash; worst counterexamples <span class=\"axis\">{}</span></h3>\
             <table><thead><tr><th>value</th><th>seed</th><th>core</th><th>context</th>\
             <th>ratio</th><th>&Delta;total</th><th>spans</th></tr></thead><tbody>{rows}</tbody>\
             </table>",
            escape(family["family"].as_str().unwrap_or("")),
            escape(family["statistic"].as_str().unwrap_or("")),
            escape(family["quantity"].as_str().unwrap_or("")),
        ));
    }

    format!(
        "<section class=\"card\">\
         <h2>Controlled synthetic validation</h2>\
         <p class=\"role\">{}</p>\
         <p class=\"meta\">{} trials, {} order-null realizations each. The metric, the null, and \
         the boundary search are frozen; the gauntlet only calls them. Scoring is one rule applied \
         to every family alike: PASS when the median has the expected sign and at least two thirds \
         of trials agree.</p>\
         <table><thead><tr><th>family</th><th>statistic</th><th>expectation</th><th>trials</th>\
         <th>undef</th><th>frac</th><th>median</th><th>median &Delta;total</th><th>result</th>\
         </tr></thead>\
         <tbody>{scorecard}</tbody></table>\
         <p class=\"meta\">Each panel below plots &Delta; raw distance against &Delta; surprise. \
         The shaded quadrant is the phenomenon under test: <strong>raw agreement worsens while \
         surprise improves</strong>.</p>\
         <div class=\"panels\">{panels}</div>\
         {counterexamples}\
         </section>",
        escape(document["role"].as_str().unwrap_or("")),
        as_integer(&document["trials"]),
        as_integer(&document["realizations"]),
    )
}

fn join_marks(value: &serde_json::Value) -> String {
    value
        .as_array()
        .map(|marks| {
            marks
                .iter()
                .filter_map(|mark| mark.as_str())
                .map(|mark| mark.rsplit('/').next().unwrap_or(mark))
                .collect::<Vec<_>>()
                .join(" · ")
        })
        .unwrap_or_default()
}

/// A Δraw-against-Δsurprise scatter with the four quadrants distinguished.
fn quadrant(title: &str, verdict: &str, points: &[(f64, f64)]) -> String {
    let extent = |values: Vec<f64>| {
        let low = values
            .iter()
            .copied()
            .fold(f64::INFINITY, f64::min)
            .min(0.0);
        let high = values
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max)
            .max(0.0);
        let pad = ((high - low) * 0.08).max(1e-6);
        (low - pad, high + pad)
    };
    let (x0, x1) = extent(points.iter().map(|(x, _)| *x).collect());
    let (y0, y1) = extent(points.iter().map(|(_, y)| *y).collect());
    let px = |x: f64| 6.0 + 88.0 * (x - x0) / (x1 - x0);
    let py = |y: f64| 92.0 - 84.0 * (y - y0) / (y1 - y0);

    let (zero_x, zero_y) = (px(0.0), py(0.0));
    let marks: String = points
        .iter()
        .map(|(x, y)| {
            let class = if *x > 0.0 && *y > 0.0 { "hit" } else { "miss" };
            format!(
                "<circle class=\"{class}\" cx=\"{:.2}\" cy=\"{:.2}\" r=\"1.1\"/>",
                px(*x),
                py(*y)
            )
        })
        .collect();

    format!(
        "<figure class=\"panel\"><figcaption>{} <span class=\"axis\">{} &middot; x \
         &Delta;total {:+.3} to {:+.3} &middot; y &Delta;z {:+.3} to {:+.3}</span></figcaption>\
         <svg viewBox=\"0 0 100 100\" preserveAspectRatio=\"none\" role=\"img\">\
         <rect class=\"interesting\" x=\"{:.2}\" y=\"8\" width=\"{:.2}\" height=\"{:.2}\"/>\
         <line class=\"rule\" x1=\"6\" y1=\"{zero_y:.2}\" x2=\"94\" y2=\"{zero_y:.2}\"/>\
         <line class=\"rule\" x1=\"{zero_x:.2}\" y1=\"8\" x2=\"{zero_x:.2}\" y2=\"92\"/>\
         {marks}</svg></figure>",
        escape(title),
        escape(verdict),
        x0,
        x1,
        y0,
        y1,
        zero_x,
        (94.0 - zero_x).max(0.0),
        (zero_y - 8.0).max(0.0),
    )
}

/// The sprint:15 adversarial commissioning card.
///
/// Selected when a document carries `adversarial_families`. Every number is read
/// from the computed result; the page scores nothing.
fn adversarial_card(document: &serde_json::Value) -> String {
    let empty = Vec::new();
    let families = document["adversarial_families"]
        .as_array()
        .unwrap_or(&empty);

    let mut scorecard = String::new();
    for family in families {
        let verdict = family["verdict"].as_str().unwrap_or("");
        scorecard.push_str(&format!(
            "<tr class=\"{}\"><td>{}</td><td class=\"stat\">{}</td>\
             <td class=\"strong\">{}</td><td>{}</td><td class=\"marks\">{}</td></tr>",
            verdict.to_lowercase(),
            escape(family["name"].as_str().unwrap_or("")),
            escape(&family["predicted"].as_str().unwrap_or("").to_uppercase()),
            escape(&verdict.to_uppercase()),
            family["points"].as_array().map(|p| p.len()).unwrap_or(0),
            escape(family["boundary"].as_str().unwrap_or("—")),
        ));
    }

    let mut detail = String::new();
    for family in families {
        let rows: String = family["points"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|point| {
                format!(
                    "<tr class=\"{}\"><td class=\"marks\">{}</td><td>{:.3}</td><td>{:.3}</td>\
                     <td>{}</td></tr>",
                    if point["holds"].as_bool() == Some(true) {
                        ""
                    } else {
                        "fail"
                    },
                    escape(point["params"].as_str().unwrap_or("")),
                    as_number(&point["weaker"]),
                    as_number(&point["stronger"]),
                    if point["holds"].as_bool() == Some(true) {
                        "yes"
                    } else {
                        "NO"
                    },
                )
            })
            .collect();
        detail.push_str(&format!(
            "<h3>{}</h3><p class=\"meta\">{}</p><p class=\"truth\">invariant: {}</p>\
             <p class=\"meta\">risk: {}</p>\
             <table><thead><tr><th>point</th><th>weaker</th><th>stronger</th><th>holds</th>\
             </tr></thead><tbody>{rows}</tbody></table>",
            escape(family["name"].as_str().unwrap_or("")),
            escape(family["construction"].as_str().unwrap_or("")),
            escape(family["invariant"].as_str().unwrap_or("")),
            escape(family["mechanism"].as_str().unwrap_or("")),
        ));
    }

    format!(
        "<section class=\"card\"><h2>Adversarial commissioning</h2>\
         <p class=\"role\">{}</p>\
         <p class=\"meta\">Statistic under test: <code>{}</code>. Frozen for the round and not \
         adopted by it. sprint:12's gauntlet is discovery evidence and runs as regression; these \
         families are the fresh evidence, built against inverse-frequency weighting's own failure \
         modes. Every expectation, sweep, and prediction was fixed before any family ran.</p>\
         <table><thead><tr><th>family</th><th>predicted</th><th>result</th><th>points</th>\
         <th>first failing point</th></tr></thead><tbody>{scorecard}</tbody></table>\
         {detail}</section>",
        escape(document["role"].as_str().unwrap_or("")),
        escape(document["under_test"].as_str().unwrap_or("")),
    )
}

/// The sprint:16 operating-envelope card.
///
/// Selected when a document carries `candidates`. sprint:17, task:27.
///
/// Presentation only: every number is read from the computed comparison, and
/// the page decides nothing. In particular it does not rank the candidates —
/// task:27 §I forbids selection by aggregate pass count, and a table that
/// totalled the ticks would be doing exactly that.
fn repair_card(document: &serde_json::Value) -> String {
    let empty = Vec::new();
    let contract = document["contract"].as_array().unwrap_or(&empty);
    let witnesses = document["crossing_witnesses"].as_array().unwrap_or(&empty);
    let matrix = document["family_matrix"].as_array().unwrap_or(&empty);
    let envelope = document["envelope"].as_array().unwrap_or(&empty);
    let candidates = document["candidates"].as_array().unwrap_or(&empty);

    let mut interpretations = String::new();
    for entry in candidates {
        interpretations.push_str(&format!(
            "<tr><td><strong>{}</strong></td><td class=\"mono\">{}</td><td>{}</td></tr>",
            escape(entry["name"].as_str().unwrap_or("")),
            escape(entry["formula"].as_str().unwrap_or("")),
            escape(entry["interpretation"].as_str().unwrap_or("")),
        ));
    }

    let mut clauses = String::new();
    let mut headers = String::new();
    if let Some(first) = contract
        .first()
        .and_then(|report| report["clauses"].as_array())
    {
        for clause in first {
            let free = clause["free_by_construction"].as_bool().unwrap_or(false);
            headers.push_str(&format!(
                "<th>{}{}</th>",
                escape(clause["clause"].as_str().unwrap_or("")),
                if free {
                    " <span class=\"muted\">(free)</span>"
                } else {
                    ""
                }
            ));
        }
    }
    for report in contract {
        let mut cells = String::new();
        for clause in report["clauses"].as_array().unwrap_or(&empty) {
            let ok = clause["satisfied"].as_bool().unwrap_or(false);
            cells.push_str(&format!(
                "<td class=\"{}\">{}</td>",
                if ok { "held" } else { "broken" },
                if ok { "ok" } else { "NO" }
            ));
        }
        clauses.push_str(&format!(
            "<tr><td>{}</td>{cells}</tr>",
            escape(report["candidate"].as_str().unwrap_or(""))
        ));
    }

    let mut violations = String::new();
    for report in contract {
        for clause in report["clauses"].as_array().unwrap_or(&empty) {
            if clause["satisfied"].as_bool().unwrap_or(false) {
                continue;
            }
            violations.push_str(&format!(
                "<li><strong>{}</strong> violates {}: {} = {:+.4} <span class=\"muted\">[{}]</span></li>",
                escape(report["candidate"].as_str().unwrap_or("")),
                escape(clause["clause"].as_str().unwrap_or("")),
                escape(clause["quantity"].as_str().unwrap_or("")),
                clause["value"].as_f64().unwrap_or(f64::NAN),
                escape(clause["witness"].as_str().unwrap_or("")),
            ));
        }
    }

    let mut crossings = String::new();
    for witness in witnesses {
        let crossed = witness["crossed"].as_bool().unwrap_or(false);
        crossings.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{:.3}</td><td>{:.3}</td>\
             <td class=\"{}\">{}</td></tr>",
            escape(witness["candidate"].as_str().unwrap_or("")),
            witness["fewer"].as_u64().unwrap_or(0),
            witness["more"].as_u64().unwrap_or(0),
            witness["fewer_score"].as_f64().unwrap_or(f64::NAN),
            witness["more_score"].as_f64().unwrap_or(f64::NAN),
            if crossed { "broken" } else { "held" },
            if crossed { "yes" } else { "no" },
        ));
    }

    let mut families = String::new();
    for row in matrix {
        let mut cells = String::new();
        for verdict in row["verdicts"].as_array().unwrap_or(&empty) {
            let text = verdict["verdict"].as_str().unwrap_or("");
            cells.push_str(&format!(
                "<td class=\"{}\">{}</td>",
                match text {
                    "PASS" => "held",
                    "FAIL" => "broken",
                    _ => "mixed",
                },
                escape(text)
            ));
        }
        families.push_str(&format!(
            "<tr><td>{}</td>{cells}</tr>",
            escape(row["family"].as_str().unwrap_or(""))
        ));
    }

    let mut candidate_headers = String::new();
    for entry in candidates {
        candidate_headers.push_str(&format!(
            "<th>{}</th>",
            escape(
                entry["name"]
                    .as_str()
                    .unwrap_or("")
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
            )
        ));
    }

    let mut real = String::new();
    for row in envelope {
        real.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{} / {}</td><td>{:.3}</td><td>{:.3}</td>\
             <td>{}</td><td>{} / {}</td><td>{} / {}</td></tr>",
            escape(row["candidate"].as_str().unwrap_or("")),
            row["pairs"].as_u64().unwrap_or(0),
            row["exchange_invariant_pairs"].as_u64().unwrap_or(0),
            row["pairs"].as_u64().unwrap_or(0),
            row["median_delta"].as_f64().unwrap_or(f64::NAN),
            row["max_delta"].as_f64().unwrap_or(f64::NAN),
            row["crossings"].as_u64().unwrap_or(0),
            row["picks_moved"].as_u64().unwrap_or(0),
            row["candidate_sets"].as_u64().unwrap_or(0),
            row["orders_reversed"].as_u64().unwrap_or(0),
            row["orders_compared"].as_u64().unwrap_or(0),
        ));
    }

    let shared = document["shared_marginal_points"].as_u64().unwrap_or(0);
    let total = document["total_family_points"].as_u64().unwrap_or(0);

    let real_section = if real.is_empty() {
        "<p class=\"muted\">The real operating envelope was not replayed in this document. \
         That is an absence, not a zero.</p>"
            .to_owned()
    } else {
        format!(
            "<h3>The real operating envelope</h3>\
             <p class=\"muted\">sprint:16's exact candidate sets, same machinery, same parameters. \
             Counts and margins only — decision:8 forbids publishing recording contents.</p>\
             <table><thead><tr><th>candidate</th><th>pairs</th><th>S(A,B) = S(B,A)</th>\
             <th>median δ</th><th>max δ</th><th>crossings</th><th>picks moved</th>\
             <th>orders reversed</th></tr></thead><tbody>{real}</tbody></table>"
        )
    };

    format!(
        "<section class=\"card\">\
         <h2>Candidate repairs, compared against a semantic contract</h2>\
         <p class=\"muted\">sprint:17, task:27. <strong>Nothing here is adopted.</strong> The \
         statistic under test is still <code>{}</code>. Candidates were derived from what the \
         statistic is supposed to mean, not from the tests they face.</p>\
         <h3>What each candidate claims to be</h3>\
         <table><thead><tr><th>candidate</th><th>formula</th><th>interpretation</th></tr></thead>\
         <tbody>{interpretations}</tbody></table>\
         <h3>The semantic contract</h3>\
         <p class=\"muted\">Clauses marked <em>(free)</em> hold by construction for every pooled \
         candidate, so satisfying one is evidence of correct code and nothing more. They confer no \
         eligibility.</p>\
         <table><thead><tr><th>candidate</th>{headers}</tr></thead><tbody>{clauses}</tbody></table>\
         <ul>{violations}</ul>\
         <h3>The crossing theorem, exhibited</h3>\
         <p class=\"muted\">Any statistic summing a per-mark weight over agreeing positions admits \
         a candidate with strictly fewer agreements outscoring one with more, whenever that weight \
         is non-constant — which is exactly what &ldquo;rare agreement is more informative&rdquo; \
         demands. Accumulation is a consequence of rarity weighting, not a defect of one statistic.\
         </p>\
         <table><thead><tr><th>candidate</th><th>fewer k</th><th>more k</th><th>fewer</th>\
         <th>more</th><th>crosses</th></tr></thead><tbody>{crossings}</tbody></table>\
         <h3>The ten sprint:15 families, constructions unchanged</h3>\
         <table><thead><tr><th>family</th>{candidate_headers}</tr></thead>\
         <tbody>{families}</tbody></table>\
         <p class=\"muted\">The frozen statistic and the pooled sum are numerically identical at \
         {shared} of {total} family points, because nine of the ten families build both recordings \
         from one set of marginals. Those {shared} points distinguish nothing.</p>\
         {real_section}\
         </section>",
        escape(document["under_test"].as_str().unwrap_or("")),
    )
}

/// Selected when a document carries `profiles`. Presentation only: every number
/// is read from the computed study, and the page measures nothing.
fn envelope_card(document: &serde_json::Value) -> String {
    let empty = Vec::new();
    let profiles = document["profiles"].as_array().unwrap_or(&empty);
    let approaches = document["approaches"].as_array().unwrap_or(&empty);
    let asymmetry = document["asymmetry"].as_array().unwrap_or(&empty);
    let orderings = document["orderings"].as_array().unwrap_or(&empty);
    let crossings = document["crossings"].as_array().unwrap_or(&empty);

    let mut envelope = String::new();
    for profile in profiles {
        let top = profile["frequencies"]
            .as_array()
            .and_then(|marks| marks.first())
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        envelope.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{:.4}</td><td>{}</td>\
             <td class=\"marks\">{}</td></tr>",
            escape(profile["session"].as_str().unwrap_or("")),
            as_integer(&profile["events"]),
            as_integer(&profile["vocabulary"]),
            as_integer(&profile["max_count"]),
            as_number(&top["frequency"]),
            as_integer(&profile["singletons"]),
            escape(top["mark"].as_str().unwrap_or("")),
        ));
    }

    let mut surface = String::new();
    for entry in approaches {
        let session = entry["session"].as_str().unwrap_or("");
        for approach in entry["approaches"].as_array().into_iter().flatten() {
            let crossed = approach["constructible"].as_bool() == Some(true);
            surface.push_str(&format!(
                "<tr class=\"{}\"><td>{}</td><td>{}</td><td>{:.1}</td><td>{}</td>\
                 <td>{:+.1}</td><td>{:.2}</td><td>{}</td><td class=\"strong\">{}</td></tr>",
                if crossed { "fail" } else { "" },
                escape(session),
                as_integer(&approach["k"]),
                as_number(&approach["boundary"]),
                as_integer(&approach["max_count"]),
                as_number(&approach["absolute_margin"]),
                as_number(&approach["relative_margin"]),
                as_integer(&approach["marks_above"]),
                if crossed { "YES" } else { "no" },
            ));
        }
    }

    let deltas: Vec<f64> = asymmetry
        .iter()
        .map(|sample| as_number(&sample["delta"]))
        .filter(|delta| delta.is_finite())
        .collect();
    let zero = deltas.iter().filter(|delta| **delta < 1e-12).count();
    let worst = deltas.iter().copied().fold(0.0f64, f64::max);
    let moved = orderings
        .iter()
        .filter(|check| check["pick_changed"].as_bool() == Some(true))
        .count();

    let mut moves = String::new();
    for check in orderings
        .iter()
        .filter(|check| check["pick_changed"].as_bool() == Some(true))
    {
        moves.push_str(&format!(
            "<tr class=\"fail\"><td class=\"marks\">{}</td><td>{}</td><td>#{}</td>\
             <td>#{}</td></tr>",
            escape(check["origin"].as_str().unwrap_or("")),
            as_integer(&check["candidates"]),
            as_integer(&check["forward_pick"]),
            as_integer(&check["backward_pick"]),
        ));
    }

    let mut crossing_rows = String::new();
    for crossing in crossings {
        crossing_rows.push_str(&format!(
            "<tr class=\"fail\"><td class=\"marks\">{}</td><td>{}</td><td>{:.3}</td>\
             <td>{}</td><td>{:.3}</td><td class=\"strong\">{:+.3}</td></tr>",
            escape(crossing["origin"].as_str().unwrap_or("")),
            as_integer(&crossing["fewer_agreements"]),
            as_number(&crossing["fewer_score"]),
            as_integer(&crossing["more_agreements"]),
            as_number(&crossing["more_score"]),
            as_number(&crossing["margin"]),
        ));
    }

    format!(
        "<section class=\"card\"><h2>Operating envelope</h2>\
         <p class=\"role\">{}</p>\
         <p class=\"meta\">Statistic under study: <code>{}</code>. Frozen. This measures exposure \
         to two known failure surfaces, not accuracy — these recordings have no known true motif \
         boundaries and are not ground truth for anything.</p>\
         <h3>corpus</h3>\
         <table><thead><tr><th>session</th><th>events</th><th>vocab</th><th>max count</th>\
         <th>max freq</th><th>singletons</th><th>commonest mark</th></tr></thead>\
         <tbody>{envelope}</tbody></table>\
         <h3>accumulation surface <span class=\"axis\">a singleton beats a k-motif when \
         c &gt; N^((k−1)/k)</span></h3>\
         <table><thead><tr><th>session</th><th>k</th><th>boundary</th><th>max count</th>\
         <th>abs margin</th><th>rel</th><th>above</th><th>constructible</th></tr></thead>\
         <tbody>{surface}</tbody></table>\
         <h3>observed crossings <span class=\"axis\">fewer agreements outscoring more, from the \
         unmodified machinery</span></h3>\
         <table><thead><tr><th>candidate set</th><th>fewer</th><th>scored</th><th>more</th>\
         <th>scored</th><th>margin</th></tr></thead><tbody>{crossing_rows}</tbody></table>\
         <h3>A/B asymmetry</h3>\
         <p class=\"meta\">delta = 0 in <strong>{} of {}</strong> real candidate pairs; largest \
         <strong>{:.3}</strong> nats; designated pick moved in <strong>{} of {}</strong> candidate \
         sets.</p>\
         <table><thead><tr><th>candidate set</th><th>candidates</th><th>forward pick</th>\
         <th>backward pick</th></tr></thead><tbody>{moves}</tbody></table>\
         </section>",
        escape(document["role"].as_str().unwrap_or("")),
        escape(document["under_study"].as_str().unwrap_or("")),
        zero,
        deltas.len(),
        worst,
        moved,
        orderings.len(),
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

    let (panels, evidence_table, histograms) = null_sections(document, truth_a, truth_b);

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
         {panels}{evidence_table}{histograms}\
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
.panels { display: grid; gap: .8rem; margin-top: 1rem; }
.panel, .hist { margin: 0; }
figcaption { font-size: .85em; opacity: .8; margin-bottom: .25rem; }
.panel svg { width: 100%; height: 9rem; border: 1px solid var(--line); border-radius: 3px; }
.hist svg { width: 100%; height: 6rem; border: 1px solid var(--line); border-radius: 3px; }
.hists { display: grid; gap: .8rem; margin-top: 1rem; }
circle.cloud { fill: var(--ink); opacity: .18; }
circle.front { fill: var(--pick); }
circle.truthmark { fill: var(--truth); stroke: var(--paper); stroke-width: .5; }
circle.notemark { fill: #2ea043; stroke: var(--paper); stroke-width: .5; }
circle.rawmark { fill: #d1495b; stroke: var(--paper); stroke-width: .5; }
line.rule { stroke: var(--line); stroke-width: .4; }
rect.bin { fill: var(--ink); opacity: .45; }
line.observed { stroke: #d1495b; stroke-width: 1; }
.legend { font-size: .85em; opacity: .85; }
.key { display: inline-block; width: .7rem; height: .7rem; border-radius: 50%; margin: 0 .2rem 0 .8rem;
       vertical-align: -1px; }
.key.cloud { background: var(--ink); opacity: .25; }
.key.front { background: var(--pick); }
.key.truthmark { background: var(--truth); }
.key.notemark { background: #2ea043; }
.key.rawmark { background: #d1495b; }
tr.planted td { color: var(--truth); }
tr.noted td { color: #2ea043; }
tr.rawbest td { color: #d1495b; }
circle.hit { fill: #2ea043; }
circle.miss { fill: var(--ink); opacity: .3; }
rect.interesting { fill: #2ea043; opacity: .07; }
tr.pass td { color: #2ea043; }
tr.mixed td { color: var(--truth); }
tr.fail td { color: #d1495b; }
h3 { font-size: .95rem; margin: 1.2rem 0 .3rem; }
td.marks { font-size: .78em; opacity: .8; text-align: left; }
td.held { color: #2ea043; }
td.broken { color: #d1495b; font-weight: 600; }
td.mixed { color: var(--truth); }
.muted { font-size: .88em; opacity: .8; }
.mono { font-family: ui-monospace, SFMono-Regular, Menlo, monospace; font-size: .85em; }
td.stat { opacity: .7; }
tr.fail td { color: #d1495b; }
";

/// Render one page from specimen documents.
pub fn render(documents: &[serde_json::Value]) -> String {
    let cards: String = documents
        .iter()
        .map(|document| {
            if document.get("candidates").is_some() {
                repair_card(document)
            } else if document.get("profiles").is_some() {
                envelope_card(document)
            } else if document.get("adversarial_families").is_some() {
                adversarial_card(document)
            } else if document.get("families").is_some() {
                gauntlet_card(document)
            } else {
                specimen_card(document)
            }
        })
        .collect();
    format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\">\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
         <title>WitnessGlass — boundary evidence (experimental)</title>\
         <style>{PAGE_STYLE}</style></head><body>\
         <h1>WitnessGlass — boundary evidence</h1>\
         <div class=\"intro\">\
         <p>sprint:10 to sprint:12. A disposable experiment, not a product surface. Every number on \
         this page was computed by <code>event-motif</code> and read from its JSON; nothing here \
         measures anything.</p>\
         <p>A <strong>controlled synthetic validation</strong> card, where the answer is known by \
         construction, is scored against expectations recorded before any trial ran. Cards below it \
         are <strong>observations on real specimens</strong>, which carry no ground truth about \
         boundary correctness and cannot be scored.</p>\
         <p>Each specimen shows the seed boundaries, the designated pick, the planted boundaries \
         where a fixture has them, and the Pareto frontier over retained events against total \
         distance. A row on the frontier retains fewer events than the row above it and scores \
         strictly better, so what each discarded event bought is visible.</p>\
         <p>Marks are the delivered event kind and tool-name string, verbatim. No figure here is \
         given a workflow name; the detector does not know one.</p>\
         </div>{cards}</body></html>"
    )
}
