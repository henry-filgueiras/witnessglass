//! The committed synthetic fixtures, and source-level guards over the bundled
//! browser assets.
//!
//! **What these tests are.** Two things, and neither is a substitute for the
//! other:
//!
//! 1. Real tests over the fixtures. They replay and project through the same
//!    code path the viewer uses, and assert that each anomaly the fixtures exist
//!    to demonstrate is actually present. If a fixture stops reproducing the
//!    shape it was built for, these fail.
//! 2. Guards over `viewer.js` and `viewer.html`, asserting the absence of the
//!    unsafe constructions this workbench is forbidden to use.
//!
//! **What the guards are not.** They are not UI testing. They read source text;
//! they do not run a browser, render anything, click anything, or observe a
//! single pixel. A guard proves that `innerHTML` does not appear in the file. It
//! cannot prove the page renders correctly, that focus is visible, or that the
//! event map positions a mark where it belongs — all of which have to be looked
//! at, and all of which are on the manual smoke checklist in `docs/viewer.md`.
//!
//! Headless-browser infrastructure was considered and declined; `docs/viewer.md`
//! records that trade and what it costs.

use std::path::Path;

use witnessglass::inspection::{AnomalyKind, CorrelationId, CoveredField, GroupShape, inspect};
use witnessglass::{Snapshot, replay_bytes};

const COMPLETE: &str = "fixtures/synthetic-first-light.ndjson";
const TRUNCATED: &str = "fixtures/synthetic-truncated.ndjson";

fn read(path: &str) -> Vec<u8> {
    std::fs::read(path).unwrap_or_else(|err| panic!("fixture {path} should be readable: {err}"))
}

fn asset(name: &str) -> String {
    let path = format!("src/assets/{name}");
    std::fs::read_to_string(&path).unwrap_or_else(|err| panic!("{path} should be readable: {err}"))
}

// ---------------------------------------------------------------------------
// The fixtures reproduce first contact's shapes, without any of its content
// ---------------------------------------------------------------------------

#[test]
fn the_complete_fixture_is_synthetic_and_obviously_so() {
    let text = String::from_utf8(read(COMPLETE)).expect("the fixture is UTF-8");
    assert!(text.contains("sess-synthetic-first-light"));
    for line in text.lines() {
        assert!(
            line.contains("synthetic"),
            "every record should be self-evidently synthetic: {line}"
        );
    }
    // Nothing resembling a real machine, user, or repository.
    for leak in [
        "/Users/",
        "/home/",
        "github.com",
        "witnessglass/.witnessglass",
    ] {
        assert!(!text.contains(leak), "fixture mentions {leak:?}");
    }
}

#[test]
fn the_complete_fixture_reproduces_every_shape_it_exists_to_demonstrate() {
    let replay = replay_bytes(&read(COMPLETE)).expect("the fixture should replay");
    let inspection = inspect(&replay);

    assert_eq!(inspection.schema_version, Some(2));
    assert_eq!(inspection.record_count(), 34);
    assert!(!inspection.scope.is_truncated());

    let kinds: Vec<AnomalyKind> = inspection
        .anomalies
        .iter()
        .map(|anomaly| anomaly.kind.clone())
        .collect();

    // A request whose fate was never observed.
    assert!(kinds.contains(&AnomalyKind::OpeningWithoutOutcome {
        id: CorrelationId::V2ToolUseId("toolu_synthetic_0003")
    }));
    // An outcome with no request — here a denial.
    assert!(kinds.contains(&AnomalyKind::OutcomeWithoutOpening {
        id: CorrelationId::V2ToolUseId("toolu_synthetic_0004")
    }));
    // Duplicate requests, not greedily paired.
    assert!(kinds.contains(&AnomalyKind::DuplicateOpenings {
        id: CorrelationId::V2ToolUseId("toolu_synthetic_0006")
    }));
    // One id, two delivered tool names, neither canonical.
    assert!(kinds.contains(&AnomalyKind::DivergentToolNames {
        id: CorrelationId::V2ToolUseId("toolu_synthetic_0007")
    }));
    // Outcomes that disagree.
    assert!(kinds.contains(&AnomalyKind::ConflictingOutcomes {
        id: CorrelationId::V2ToolUseId("toolu_synthetic_0008")
    }));
    // The unmatched subagent stop task:4 measured for real.
    assert!(kinds.contains(&AnomalyKind::SubagentStopWithoutStart {
        agent_id: "agent-synthetic-orphan-0002"
    }));

    // Absent duration on all but one completion.
    let duration = inspection
        .coverage
        .iter()
        .find(|c| c.field == CoveredField::V2DurationMs)
        .expect("duration coverage");
    assert_eq!(duration.present.count(), 1);
    assert!(duration.absent.count() > 1);

    // Parentage never supplied, on any subagent boundary.
    let parent = inspection
        .coverage
        .iter()
        .find(|c| c.field == CoveredField::V2SuppliedParentAgent)
        .expect("parent coverage");
    assert_eq!(parent.present.count(), 0);
    assert_eq!(parent.population.count(), 3);
    for subagent in &inspection.subagents {
        assert!(subagent.supplied_parents.is_empty());
    }

    // A duplicated reported description, correlated and not merged.
    let paired = inspection
        .tool_groups
        .iter()
        .find(|g| g.id == CorrelationId::V2ToolUseId("toolu_synthetic_0001"))
        .expect("the ordinary group");
    assert_eq!(paired.shape, GroupShape::PairedLifecycle);
    assert_eq!(paired.reported_intents.count(), 1);

    // A subagent's work attributable by context.agent_id, with the Agent call's
    // request and outcome bracketing it in sequence and creating no parentage.
    let child = inspection
        .current_agents
        .supplied
        .iter()
        .find(|t| t.value == "agent-synthetic-child-0001")
        .expect("the child agent's records");
    assert_eq!(child.records.count(), 5);
    let agent_call = inspection
        .tool_groups
        .iter()
        .find(|g| g.id == CorrelationId::V2ToolUseId("toolu_synthetic_0009"))
        .expect("the Agent call");
    let interval = agent_call
        .paired_interval
        .expect("one request, one outcome");
    assert!(
        interval.opening < 21 && interval.outcome > 25,
        "the child's records fall inside this interval"
    );

    // A clock that moved backwards, with append order intact.
    let timestamps = inspection.timestamps.as_ref().expect("extrema");
    assert_eq!(timestamps.non_monotonic.count(), 1);
    let sequences: Vec<u64> = inspection.ledger.iter().map(|e| e.sequence).collect();
    assert_eq!(sequences, (1..=34).collect::<Vec<u64>>());

    // A payload deliberately shaped like markup, preserved exactly.
    let hostile = inspection
        .ledger
        .iter()
        .find(|e| e.sequence == 28)
        .expect("the hostile-payload record");
    let rendered = serde_json::to_string(hostile.record).expect("serialize");
    assert!(
        rendered.contains("onerror=alert(1)"),
        "the payload survives as data"
    );
}

#[test]
fn the_truncated_fixture_serves_its_valid_prefix() {
    let replay = replay_bytes(&read(TRUNCATED)).expect("the valid prefix should replay");
    let inspection = inspect(&replay);
    assert!(inspection.scope.is_truncated());
    assert_eq!(inspection.record_count(), 33);

    // The session_ended record is in the missing tail, so its absence is scoped
    // to the prefix rather than to a complete recording.
    let missing_end = inspection
        .anomalies
        .iter()
        .find(|a| a.kind == AnomalyKind::MissingSessionEnd)
        .expect("the closing boundary is beyond the truncation");
    assert!(missing_end.scope.is_truncated());
}

#[test]
fn both_fixtures_load_through_the_viewers_own_entry_point() {
    let complete = Snapshot::load(Path::new(COMPLETE)).expect("load the complete fixture");
    assert_eq!(complete.records(), 34);
    assert!(!complete.is_truncated());

    let truncated = Snapshot::load(Path::new(TRUNCATED)).expect("load the truncated fixture");
    assert_eq!(truncated.records(), 33);
    assert!(truncated.is_truncated());
}

// ---------------------------------------------------------------------------
// Source-level guards. Not UI testing — see the module documentation.
// ---------------------------------------------------------------------------

#[test]
fn guard_the_script_never_turns_recording_text_into_markup() {
    let script = asset("viewer.js");
    for unsafe_path in [
        "innerHTML",
        "outerHTML",
        "insertAdjacentHTML",
        "document.write",
        "eval(",
        "new Function",
        "createContextualFragment",
        "srcdoc",
        "javascript:",
    ] {
        assert!(
            !script.contains(unsafe_path),
            "viewer.js uses {unsafe_path:?}, which can turn a payload into markup"
        );
    }
    // The one sanctioned way content reaches the page.
    assert!(script.contains("node.textContent = String(options.text)"));
}

#[test]
fn guard_nothing_is_persisted_and_nothing_leaves_the_machine() {
    let script = asset("viewer.js");
    for persistence in [
        "localStorage",
        "sessionStorage",
        "indexedDB",
        "document.cookie",
        "serviceWorker",
        "navigator.sendBeacon",
        "XMLHttpRequest",
        "WebSocket",
        "EventSource",
    ] {
        assert!(
            !script.contains(persistence),
            "viewer.js uses {persistence:?}"
        );
    }
    // Exactly one network call, same-origin and relative.
    assert_eq!(script.matches("fetch(").count(), 1);
    assert!(script.contains("fetch(`/projection.json?c=${encodeURIComponent(CAPABILITY)}`"));
    assert!(script.contains("cache: \"no-store\""));
    assert!(script.contains("credentials: \"omit\""));

    for asset_name in ["viewer.js", "viewer.html", "viewer.css"] {
        let text = asset(asset_name);
        for remote in ["http://", "https://", "//cdn.", "@import url("] {
            assert!(!text.contains(remote), "{asset_name} references {remote:?}");
        }
    }
}

#[test]
fn guard_the_capability_is_never_written_anywhere_durable() {
    let script = asset("viewer.js");
    // Read once from the URL, held in one constant, and used in one fetch.
    assert_eq!(
        script.matches("location.search").count(),
        1,
        "the capability should be read exactly once"
    );
    // Declared once, used once. Nothing else touches it.
    assert_eq!(script.matches("CAPABILITY").count(), 2);
    assert!(!script.contains("history.pushState"));
    assert!(!script.contains("history.replaceState"));
}

#[test]
fn guard_the_page_declares_no_inline_script_and_no_remote_asset() {
    let page = asset("viewer.html");
    // Exactly one script element, external, from this origin.
    assert_eq!(page.matches("<script").count(), 1);
    assert!(page.contains(r#"<script type="module" src="/viewer.js?c={{CAPABILITY}}"></script>"#));
    assert!(page.contains(r#"<link rel="stylesheet" href="/viewer.css?c={{CAPABILITY}}">"#));
    assert!(page.contains(r#"<meta name="referrer" content="no-referrer">"#));
    // A noscript path that does not pretend the workbench is working.
    assert!(page.contains("<noscript>"));
    // The sensitive-recording warning is in the markup, not conjured by script.
    assert!(page.contains("Not redacted."));
    assert!(page.contains("Rendering is not redacting."));
    for handler in ["onclick=", "onload=", "onerror=", "javascript:"] {
        assert!(!page.contains(handler), "viewer.html uses {handler:?}");
    }
}

#[test]
fn guard_absences_are_phrased_as_absences() {
    let script = asset("viewer.js");
    // The exact wording the sprint requires, wherever the interface reports a
    // missing half or an unsupplied field.
    for phrasing in [
        "outcome not observed",
        "agent identity not supplied",
        "stop without observed start",
        "no session_ended record observed",
        "parent identity not supplied",
    ] {
        assert!(
            script.contains(phrasing),
            "viewer.js should say {phrasing:?}"
        );
    }
    // And the readings it must never offer.
    for forbidden in [
        "still running",
        "root agent\"",
        "execution duration of",
        "turns",
    ] {
        assert!(!script.contains(forbidden), "viewer.js says {forbidden:?}");
    }
    // Colour is never the only cue: every channel carries a glyph and a word.
    assert!(script.contains("CHANNEL_GLYPH"));
    assert!(script.contains("Channel marker: a glyph *and* a word"));
}

#[test]
fn guard_the_perspectives_are_declared_as_real_tabs() {
    let page = asset("viewer.html");
    assert!(page.contains(r#"role="tablist""#));
    for perspective in ["events", "coverage", "provenance"] {
        assert!(page.contains(&format!(r#"id="tab-{perspective}""#)));
        assert!(page.contains(&format!(r#"aria-controls="panel-{perspective}""#)));
        assert!(page.contains(&format!(r#"id="panel-{perspective}""#)));
    }
    // Events is the initial perspective, in the markup as well as at runtime.
    assert!(page.contains(
        r#"id="tab-events" aria-controls="panel-events"
              aria-selected="true" tabindex="0""#
    ));
    assert!(page.contains(
        r#"id="panel-coverage" role="tabpanel" aria-labelledby="tab-coverage" tabindex="-1" hidden"#
    ));

    // Roving tabindex and the arrow keys that go with it.
    let script = asset("viewer.js");
    assert!(script.contains(r#"tab.setAttribute("tabindex", on ? "0" : "-1")"#));
    for key in ["ArrowRight", "ArrowLeft", "Home", "End"] {
        assert!(
            script.contains(key),
            "tab and map navigation should handle {key}"
        );
    }
}

#[test]
fn guard_receipts_are_collapsed_but_never_deleted() {
    let script = asset("viewer.js");
    // A long receipt list becomes a disclosure naming its size, and builds its
    // buttons on open. Collapsed is not deleted.
    assert!(script.contains("supporting records"));
    assert!(script.contains("INLINE_RECEIPT_LIMIT"));
    assert!(script.contains(r#"el("details", { class: "receipt-set" }"#));
    // Every path still ends at a receiptButton, which selects a raw record.
    assert!(script.contains("holder.appendChild(receiptButton(sequence))"));
    assert!(
        script.contains(
            "for (const sequence of sequences) wrap.appendChild(receiptButton(sequence))"
        )
    );
}

#[test]
fn guard_the_summary_reads_rusts_counts_and_invents_no_rollup() {
    let script = asset("viewer.js");
    // Lifecycle kinds are named per schema and rendered individually. Nothing
    // sums them: a derived aggregate without receipts belongs in Rust, or
    // nowhere.
    assert!(script.contains("const LIFECYCLE_KINDS"));
    assert!(script.contains("never adds them together into an invented \"outcomes\" total"));
    for forbidden in ["totalOutcomes", "outcomeTotal", "sumOf", "reduce("] {
        assert!(
            !script.contains(forbidden),
            "viewer.js computes {forbidden:?} itself"
        );
    }
    // The summary carries completeness, so moving the recording panel to
    // Provenance cannot hide damage.
    assert!(script.contains(
        r#"statCell(
      "completeness","#
    ));
    assert!(script.contains("ends mid-record"));
}

#[test]
fn guard_the_map_stays_one_point_per_record() {
    let script = asset("viewer.js");
    let css = asset("viewer.css");
    // No binning, no aggregation, no spans: one mark per ledger entry.
    assert!(script.contains("for (const entry of laneEntries)"));
    assert!(script.contains("One mark per record, positioned by append sequence"));
    assert!(script.contains("DERIVED VIEW"));
    for forbidden in [
        "bucket",
        "bin(",
        "aggregateMarks",
        "span-bar",
        "duration-bar",
    ] {
        assert!(!script.contains(forbidden), "the map uses {forbidden:?}");
    }
    // Channel survives without colour: a circle, a diamond, a square.
    assert!(css.contains(".mark-observed .mark-dot"));
    assert!(css.contains(".mark-reported .mark-dot"));
    assert!(css.contains("transform: rotate(45deg)"));
    // Marks stay individually operable, with a real hit target.
    assert!(css.contains("width: 15px"));
}

#[test]
fn guard_the_stylesheet_respects_reduced_motion_and_shows_focus() {
    let css = asset("viewer.css");
    assert!(css.contains("@media (prefers-reduced-motion: reduce)"));
    assert!(css.contains(":focus-visible"));
    assert!(css.contains("outline: 2px solid var(--focus)"));
    // A light-scheme override exists, so the workbench is not dark-only.
    assert!(css.contains("@media (prefers-color-scheme: light)"));
    // Payloads are collapsed by default: that is a <details> in the script, and
    // the stylesheet must not force them open.
    assert!(!css.contains("details[open]"));
    assert!(asset("viewer.js").contains("el(\"details\", { class: \"payload\" })"));
}
