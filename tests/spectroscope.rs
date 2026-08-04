//! The sprint:7 Behavioral Spectroscope: its derived document, and guards over
//! its assets.
//!
//! **Disposable.** Deleted with the experiment.
//!
//! **What matters most here.** One test, and it is the one the whole page rests
//! on: that planted ground truth comes from the fixture's generator constants
//! and could not have been reverse-engineered out of the signal. A visualization
//! that discovered its own annotations would look identical to one that read
//! them, and would be worthless. So the test feeds a recording carrying a
//! fixture's session id and *entirely different content*, and asserts the
//! annotations do not move.
//!
//! The rest: index-to-time conversion, the shape of the document, the
//! attachment transport, and source-level guards over the new assets matching
//! the ones `tests/workbench.rs` already holds over the workbench's.

#![cfg(feature = "experiment-matrix-profile")]

use witnessglass::experiment::oracle;
use witnessglass::experiment::spectroscope::{self, Class, RegionKind};
use witnessglass::record::{Channel, Provenance, v2};
use witnessglass::replay_bytes;
use witnessglass::view::{Attachment, Snapshot, Viewer};

const LEGIBLE: &str = "fixtures/synthetic-behavioral-oracle.ndjson";
const SPARSE: &str = "fixtures/synthetic-behavioral-oracle-sparse.ndjson";

fn read(path: &str) -> Vec<u8> {
    std::fs::read(path).unwrap_or_else(|err| panic!("fixture {path} should be readable: {err}"))
}

fn asset(name: &str) -> String {
    let path = format!("src/experiment/assets/{name}");
    std::fs::read_to_string(&path).unwrap_or_else(|err| panic!("{path} should be readable: {err}"))
}

macro_rules! document {
    ($bytes:expr) => {{
        let replay = replay_bytes($bytes).expect("the recording should replay");
        spectroscope::project(&replay).expect("the recording has records")
    }};
}

// ---------------------------------------------------------------------------
// Ground truth is read, never discovered
// ---------------------------------------------------------------------------

#[test]
fn planted_regions_equal_the_generator_constants() {
    let document = document!(&read(LEGIBLE));
    let truth = document
        .ground_truth
        .expect("the legible oracle is a known fixture");

    assert_eq!(truth.class, Class::Planted);
    assert_eq!(truth.motif_period_ms, oracle::MOTIF_PERIOD_MS);
    assert_eq!(
        truth.motif_only_dimension,
        format!("tool_name:{}", oracle::TOOL_SEARCHER)
    );

    let motif = truth
        .regions
        .iter()
        .find(|region| region.kind == RegionKind::Motif)
        .expect("the fixture plants a motif");
    assert_eq!(motif.start_ms, oracle::FIRST_MOTIF_START_MS);
    assert_eq!(motif.end_ms, oracle::FIRST_MOTIF_END_MS);

    let recurrence = truth
        .regions
        .iter()
        .find(|region| region.kind == RegionKind::Recurrence)
        .expect("the fixture plants a recurrence");
    assert_eq!(recurrence.start_ms, oracle::SECOND_MOTIF_START_MS);
    assert_eq!(recurrence.end_ms, oracle::SESSION_END_MS);

    let regime = truth
        .regions
        .iter()
        .find(|region| region.kind == RegionKind::Regime)
        .expect("the fixture plants a regime");
    assert_eq!(regime.start_ms, oracle::REGIME_CHANGE_MS);
    assert_eq!(regime.end_ms, oracle::ELEVATED_END_MS);

    // The regions must tile the recording without gaps or overlaps, or the band
    // would imply quiet stretches the generator did not leave.
    let mut cursor = 0;
    for region in &truth.regions {
        assert_eq!(region.start_ms, cursor, "regions must be contiguous");
        cursor = region.end_ms;
    }
    assert_eq!(cursor, oracle::SESSION_END_MS);
}

#[test]
fn planted_regions_do_not_move_when_the_signal_does() {
    // The test the page depends on. This recording carries the legible oracle's
    // session id and nothing else of it: three records, one tool, no motif, no
    // regime, no recurrence, a span of four seconds rather than four minutes.
    //
    // If the annotations were derived from the signal in any way, they would
    // collapse or move. They must not: they come from the generator's constants
    // and the session id is only how the fixture is recognised.
    let recording = recording_with_session(oracle::SESSION_ID);
    let document = document!(recording.as_bytes());
    let truth = document
        .ground_truth
        .expect("the session id is the fixture's, so the constants apply");

    let reference = document!(&read(LEGIBLE))
        .ground_truth
        .expect("the fixture itself");
    assert_eq!(
        truth.regions, reference.regions,
        "ground truth must come from the generator constants, not from the signal"
    );
    assert_eq!(truth.motif_period_ms, reference.motif_period_ms);

    // And the observed side did move, which is what makes the comparison mean
    // something.
    assert_eq!(document.provenance.records, 3);
    assert!(document.provenance.span_ms < 10_000);
}

#[test]
fn an_unknown_session_gets_no_ground_truth_at_all() {
    let recording = recording_with_session("sess-not-a-fixture-at-all");
    let document = document!(recording.as_bytes());
    assert!(
        document.ground_truth.is_none(),
        "absent, not empty and not guessed"
    );
    // And the narrative says so rather than staying silent about it.
    assert!(
        document
            .narrative
            .iter()
            .any(|step| step.heading.contains("No ground truth")),
        "a reader must be told the page knows nothing about this recording"
    );
}

#[test]
fn both_fixtures_are_recognised_and_are_not_confused_for_one_another() {
    let legible = document!(&read(LEGIBLE)).ground_truth.expect("known");
    let sparse = document!(&read(SPARSE)).ground_truth.expect("known");
    assert!(legible.fixture.contains("legible"));
    assert!(sparse.fixture.contains("sparse"));
    assert_ne!(legible.regions, sparse.regions);
    assert_eq!(
        sparse.motif_only_dimension,
        format!("tool_name:{}", oracle::sparse::TOOL_SEARCHER)
    );
}

// ---------------------------------------------------------------------------
// The document's shape
// ---------------------------------------------------------------------------

#[test]
fn every_scale_is_a_full_matrix_and_none_of_them_is_the_canonical_one() {
    let document = document!(&read(LEGIBLE));
    assert_eq!(
        document.scales.len(),
        spectroscope::DISPLAY_SCALES_MS.len(),
        "the scrubber's stops are precomputed, not derived in the browser"
    );
    for scale in &document.scales {
        assert_eq!(
            scale.rows.len(),
            document.dimensions.len(),
            "one row per dimension at every aggregation"
        );
        for row in &scale.rows {
            assert_eq!(row.len(), scale.samples, "no ragged rows");
        }
    }
    // Coarser aggregations really are coarser, so the control teaches something.
    let first = document.scales.first().expect("at least one scale");
    let last = document.scales.last().expect("at least one scale");
    assert!(last.samples < first.samples / 4);
}

#[test]
fn a_match_span_converts_to_the_time_it_actually_covers() {
    // Off by one here would relocate every highlight on the page.
    let document = document!(&read(LEGIBLE));
    let bucket = document.provenance.base_bucket_ms;
    for profile in &document.profiles {
        for window in &profile.windows {
            for found in &window.matches {
                assert_eq!(
                    found.a_end_ms - found.a_start_ms,
                    window.window_ms,
                    "a span is exactly one window wide"
                );
                assert_eq!(found.a_start_ms % bucket, 0, "spans start on a bucket");
                assert!(found.a_end_ms <= document.provenance.span_ms + window.window_ms);
                assert_eq!(
                    found.trivial,
                    found.a_occupancy <= 2 && found.b_occupancy <= 2,
                    "trivial means what the page says it means"
                );
            }
            assert_eq!(
                window.profile.len(),
                window.subsequences,
                "the drawn curve has one entry per candidate, gaps included"
            );
        }
    }
}

#[test]
fn the_profiled_and_unprofiled_dimensions_account_for_all_of_them() {
    let document = document!(&read(LEGIBLE));
    assert!(document.profiles.len() <= spectroscope::MAX_PROFILED_DIMENSIONS);
    assert_eq!(
        document.profiles.len() + document.unprofiled.len(),
        document.dimensions.len(),
        "a dimension left out must be named, not silently missing"
    );

    // The fixture's own motif-carrying dimension is profiled, because ranking by
    // occupancy alone excluded exactly the sparse columns the fixture was built
    // to be interesting in.
    let truth = document.ground_truth.expect("known fixture");
    assert!(
        document
            .profiles
            .iter()
            .any(|profile| profile.label == truth.motif_only_dimension),
        "the planted motif's own dimension must get a profile"
    );
}

#[test]
fn the_narrative_carries_all_three_kinds_of_claim_and_names_the_failure() {
    let document = document!(&read(LEGIBLE));
    let classes: Vec<Class> = document.narrative.iter().map(|step| step.class).collect();
    assert!(classes.contains(&Class::Planted));
    assert!(classes.contains(&Class::Observed));
    assert!(classes.contains(&Class::Interpretation));

    // The trivial-match exhibit is the round's strongest result and must be in
    // the sequence whenever the data contains one.
    let has_trivial = document.profiles.iter().any(|profile| {
        profile
            .windows
            .iter()
            .any(|window| window.matches.first().is_some_and(|found| found.trivial))
    });
    if has_trivial {
        let step = document
            .narrative
            .iter()
            .find(|step| step.heading.contains("arithmetic"))
            .expect("the trivial-match explanation");
        assert_eq!(step.class, Class::Interpretation);
    }
}

#[test]
fn haar_levels_carry_their_null_beside_them() {
    let document = document!(&read(LEGIBLE));
    for view in &document.haar {
        assert_eq!(view.class, Class::Observed);
        for level in &view.levels {
            assert!(level.magnitude.iter().all(|value| *value >= 0.0));
            assert!(
                level.impulse_null_share > 0.0,
                "the null is always available"
            );
            assert!(level.scale_ms >= document.provenance.base_bucket_ms * 2);
        }
    }
}

// ---------------------------------------------------------------------------
// The transport
// ---------------------------------------------------------------------------

#[test]
fn an_attachment_cannot_replace_a_built_in_route() {
    let replay = replay_bytes(&read(LEGIBLE)).expect("replays");
    let attach = |route: &'static str| {
        Viewer::bind_with(
            Snapshot::from_replay(&replay).expect("projects"),
            vec![Attachment {
                route,
                content_type: "text/plain; charset=utf-8",
                body: "x".to_owned(),
            }],
        )
    };
    for route in ["/", "/viewer.css", "/viewer.js", "/projection.json"] {
        assert!(attach(route).is_err(), "{route} must stay the viewer's own");
    }
    assert!(attach("spectroscope").is_err(), "a route must be absolute");
    let viewer = attach("/spectroscope.json").expect("a fresh route binds");
    assert_eq!(viewer.attachment_routes(), vec!["/spectroscope.json"]);
}

// ---------------------------------------------------------------------------
// Source-level guards over the new assets
// ---------------------------------------------------------------------------

#[test]
fn guard_the_script_never_turns_recording_text_into_markup() {
    let script = asset("spectroscope.js");
    for forbidden in [
        "innerHTML",
        "outerHTML",
        "insertAdjacentHTML",
        "document.write",
        "eval(",
        "new Function",
    ] {
        assert!(
            !script.contains(forbidden),
            "spectroscope.js uses {forbidden:?}"
        );
    }
}

#[test]
fn guard_nothing_is_persisted_and_nothing_leaves_the_machine() {
    for name in ["spectroscope.js", "spectroscope.css", "spectroscope.html"] {
        let text = asset(name);
        for forbidden in [
            "localStorage",
            "sessionStorage",
            "indexedDB",
            "document.cookie",
            "history.pushState",
            "history.replaceState",
            "navigator.sendBeacon",
            "XMLHttpRequest",
            "WebSocket",
        ] {
            assert!(!text.contains(forbidden), "{name} uses {forbidden:?}");
        }
        for remote in ["http://", "https://", "//cdn", "@import"] {
            // The one permitted absolute URL is the SVG namespace, which is an
            // identifier rather than a fetch.
            let stripped = text.replace("http://www.w3.org/2000/svg", "");
            assert!(!stripped.contains(remote), "{name} references {remote:?}");
        }
    }
}

#[test]
fn guard_the_page_declares_no_inline_script_and_no_inline_handler() {
    let page = asset("spectroscope.html");
    assert!(page.contains("src=\"/spectroscope.js"));
    for handler in [
        "onclick=",
        "onload=",
        "onerror=",
        "onchange=",
        "oninput=",
        "javascript:",
    ] {
        assert!(
            !page.contains(handler),
            "spectroscope.html uses {handler:?}"
        );
    }
    // The script tag must stay a reference, never a body.
    assert!(!page.contains("<script>"));
}

#[test]
fn guard_the_three_claim_classes_are_never_carried_by_colour_alone() {
    // The stylesheet's own stated rule, and the reason it is stated: a reader who
    // cannot see the hue must still be able to tell a generated fact from a
    // measured one. Each class carries a glyph and the word as well.
    let page = asset("spectroscope.html");
    for word in ["planted", "observed", "interpretation"] {
        assert!(page.contains(word), "the legend must name {word:?}");
    }
    let script = asset("spectroscope.js");
    assert!(
        script.contains("planted: \"■\"") && script.contains("observed: \"▲\""),
        "each class needs a shape, not only a colour"
    );
}

#[test]
fn guard_the_page_says_it_is_experimental_and_not_redacted() {
    let page = asset("spectroscope.html");
    assert!(page.contains("experimental"));
    assert!(page.contains("Not redacted"));
    assert!(page.contains("Rendering is not redacting"));
    // No export affordance anywhere. Checked as mechanisms rather than as words,
    // because the page's own warning has to be free to *say* the words while the
    // document stays free of the things that do it.
    for name in ["spectroscope.html", "spectroscope.js"] {
        let text = asset(name);
        for mechanism in [
            "download=",
            "href=\"data:",
            "href=\"blob:",
            "createObjectURL",
            "navigator.clipboard",
            "execCommand",
            "showSaveFilePicker",
        ] {
            assert!(!text.contains(mechanism), "{name} offers {mechanism:?}");
        }
    }
}

#[test]
fn guard_the_stylesheet_respects_reduced_motion_and_shows_focus() {
    let css = asset("spectroscope.css");
    assert!(css.contains("prefers-reduced-motion: no-preference"));
    assert!(css.contains(":focus-visible"));
    assert!(css.contains("prefers-color-scheme: light"));
}

// ---------------------------------------------------------------------------
// A small hand-built recording
// ---------------------------------------------------------------------------

/// Three records under a chosen session id, sharing nothing else with any
/// fixture.
fn recording_with_session(session_id: &str) -> String {
    let origin: jiff::Timestamp = "2026-05-01T00:00:00Z".parse().expect("valid");
    let mut out = String::new();
    for index in 0..3u64 {
        let record = v2::Record {
            schema_version: 2,
            session_id: session_id.to_owned(),
            sequence: index + 1,
            recorded_at: origin + jiff::SignedDuration::from_millis(index as i64 * 2_000),
            context: v2::Context::default(),
            provenance: Provenance {
                channel: Channel::Observed,
                adapter: "synthetic-test-adapter".to_owned(),
                mechanism: "synthetic:test".to_owned(),
            },
            event: v2::Event::ToolRequested(v2::ToolRequested {
                tool_use_id: format!("id-{index}"),
                tool_name: "SyntheticUnrelated".to_owned(),
                requested_input: serde_json::json!({}),
            }),
        };
        out.push_str(&serde_json::to_string(&record).expect("serializes"));
        out.push('\n');
    }
    out
}
