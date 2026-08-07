//! sprint:21, task:31. The local corpus report.
//!
//! Every corpus here is synthetic and obviously so: the sessions are named
//! `synthetic-*`, the commands are inert strings written in this file, and
//! nothing is derived from a real session, prompt, source tree, or machine.

use std::path::{Path, PathBuf};

use witnessglass::experiment::calibration;
use witnessglass::experiment::corpus::{
    self, Analysis, Category, Facts, Outcome, Projection, Quarantine, Request, SkipReason,
    classify_shell, render_comparison, render_report,
};
use witnessglass::experiment::event_sequence::{ChannelScope, project};
use witnessglass::inspection::inspect;
use witnessglass::{
    Channel, Context, Emission, Event, Provenance, ReportedIntent, SessionStarted, ToolDenied,
    ToolFailed, ToolRequested, ToolSucceeded, append, replay_file,
};

// ---------------------------------------------------------------------------
// A synthetic corpus generator
// ---------------------------------------------------------------------------

const ADAPTER: &str = "synthetic-corpus-adapter";
const MECHANISM: &str = "synthetic-corpus-harness";

/// One step a synthetic session takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Step {
    tool: &'static str,
    command: Option<&'static str>,
    outcome: Outcome,
}

const fn step(tool: &'static str) -> Step {
    Step {
        tool,
        command: None,
        outcome: Outcome::Succeeded,
    }
}

const fn shell(command: &'static str) -> Step {
    Step {
        tool: "Bash",
        command: Some(command),
        outcome: Outcome::Succeeded,
    }
}

const fn failing(command: &'static str) -> Step {
    Step {
        tool: "Bash",
        command: Some(command),
        outcome: Outcome::Failed,
    }
}

fn provenance(channel: Channel) -> Provenance {
    Provenance {
        channel,
        adapter: ADAPTER.to_owned(),
        mechanism: MECHANISM.to_owned(),
    }
}

fn emit(directory: &Path, session: &str, at: &mut i64, channel: Channel, event: Event) {
    let path = directory.join(format!("{session}.ndjson"));
    let recorded_at = jiff::Timestamp::from_second(1_800_000_000 + *at).expect("in range");
    *at += 7;
    append(
        &path,
        &Emission {
            session_id: session.to_owned(),
            context: Context::default(),
            provenance: provenance(channel),
            event,
        },
        recorded_at,
    )
    .expect("synthetic append should succeed");
}

/// Write one synthetic recording: a session boundary, then one request record
/// and one outcome record per step.
fn write_session(directory: &Path, session: &str, steps: &[Step]) {
    let mut at = 0i64;
    emit(
        directory,
        session,
        &mut at,
        Channel::Recorder,
        Event::SessionStarted(SessionStarted { source: None }),
    );
    for (index, step) in steps.iter().enumerate() {
        let id = format!("toolu_{session}_{index:04}");
        let input = match step.command {
            Some(command) => serde_json::json!({ "command": command }),
            None => serde_json::json!({ "path": "synthetic" }),
        };
        emit(
            directory,
            session,
            &mut at,
            Channel::Observed,
            Event::ToolRequested(ToolRequested {
                tool_use_id: id.clone(),
                tool_name: step.tool.to_owned(),
                requested_input: input.clone(),
            }),
        );
        let outcome = match step.outcome {
            Outcome::Succeeded => Some(Event::ToolSucceeded(ToolSucceeded {
                tool_use_id: id.clone(),
                tool_name: step.tool.to_owned(),
                effective_input: input.clone(),
                response: serde_json::json!("synthetic"),
                duration_ms: None,
            })),
            Outcome::Failed => Some(Event::ToolFailed(ToolFailed {
                tool_use_id: id.clone(),
                tool_name: step.tool.to_owned(),
                effective_input: input.clone(),
                error: "synthetic failure".to_owned(),
                interrupted: None,
                duration_ms: None,
            })),
            Outcome::Denied => Some(Event::ToolDenied(ToolDenied {
                tool_use_id: id.clone(),
                tool_name: step.tool.to_owned(),
                requested_input: input.clone(),
            })),
            Outcome::NoOutcomeObserved | Outcome::Disagreeing => None,
        };
        if let Some(event) = outcome {
            emit(directory, session, &mut at, Channel::Observed, event);
        }
    }
}

/// A tiny deterministic generator, so a synthetic background is reproducible.
struct Lcg(u64);

impl Lcg {
    fn next_below(&mut self, bound: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((self.0 >> 33) as usize) % bound
    }
}

/// The background vocabulary every synthetic corpus draws from.
///
/// Eight steps over seven categories, so an accidental four-step agreement
/// between two independent sessions is rare and a planted one has somewhere to
/// stand out from.
const BACKGROUND: [Step; 8] = [
    step("Read"),
    step("Edit"),
    step("Write"),
    step("WebFetch"),
    step("Task"),
    shell("echo hello"),
    shell("git status"),
    shell("ls -la"),
];

/// The shape planted in the positive corpus. Every one of its steps occurs in
/// the background too, so it is recoverable only as a *sequence*.
const PLANTED: [Step; 4] = [step("Read"), step("Edit"), shell("npm test"), step("Read")];

/// A second planted shape, for the A/B comparison's "gained" case.
const PLANTED_SECOND: [Step; 4] = [
    step("Write"),
    shell("git status"),
    shell("npm test"),
    step("Edit"),
];

/// Write `sessions` synthetic recordings whose background is drawn from
/// [`BACKGROUND`], planting each shape into the first `count` of them.
fn corpus_with(directory: &Path, prefix: &str, sessions: usize, plants: &[(&[Step], usize)]) {
    for session in 0..sessions {
        let mut rng = Lcg(0x51ED_2701 ^ (session as u64 + 1).wrapping_mul(0x9E37_79B9));
        let mut steps: Vec<Step> = Vec::new();
        for block in 0..12 {
            for _ in 0..5 {
                steps.push(BACKGROUND[rng.next_below(BACKGROUND.len())]);
            }
            for (plant, count) in plants {
                if session < *count && block % 2 == 0 {
                    steps.extend_from_slice(plant);
                }
            }
        }
        write_session(directory, &format!("{prefix}-{session:02}"), &steps);
    }
}

fn positive_corpus(directory: &Path, sessions: usize) {
    corpus_with(
        directory,
        "synthetic-pos",
        sessions,
        &[(&PLANTED, sessions)],
    );
}

/// A corpus with no planted shape and one overwhelmingly dominant tool.
fn dominant_corpus(directory: &Path, sessions: usize) {
    let vocabulary = [shell("echo one"), step("Read")];
    for session in 0..sessions {
        let mut rng = Lcg(0x00BA_D1DE ^ (session as u64 + 1).wrapping_mul(0xD1B5_4A32));
        let steps: Vec<Step> = (0..40)
            .map(|_| {
                // Nine parts shell to one part read: a dominant tool and nothing
                // else.
                if rng.next_below(10) == 0 {
                    vocabulary[1]
                } else {
                    vocabulary[0]
                }
            })
            .collect();
        write_session(directory, &format!("synthetic-dom-{session:02}"), &steps);
    }
}

fn analyse(directory: &Path, label: &str, replicates: usize) -> Analysis {
    corpus::analyze(&Request {
        directory: directory.to_path_buf(),
        label: label.to_owned(),
        replicates,
    })
    .expect("a readable directory should analyse")
}

fn scratch() -> tempfile::TempDir {
    tempfile::tempdir().expect("a temporary directory")
}

fn lead_names(facts: &Facts) -> Vec<String> {
    facts
        .leads
        .iter()
        .filter_map(|id| {
            facts
                .workflow
                .families
                .iter()
                .find(|family| family.id == *id)
        })
        .map(|family| family.name.clone())
        .collect()
}

// ---------------------------------------------------------------------------
// The category vocabulary
// ---------------------------------------------------------------------------

#[test]
fn shell_classification_reads_only_the_leading_program() {
    assert_eq!(classify_shell("npm test"), Category::Verify);
    assert_eq!(classify_shell("npm run test:unit"), Category::Verify);
    assert_eq!(classify_shell("npm run build"), Category::Shell);
    assert_eq!(classify_shell("cargo test --all"), Category::Verify);
    assert_eq!(classify_shell("cargo build"), Category::Shell);
    assert_eq!(classify_shell("node --test t.js"), Category::Verify);
    assert_eq!(classify_shell("npx tsc --noEmit"), Category::Verify);
    assert_eq!(classify_shell("git status"), Category::VersionControl);
    assert_eq!(classify_shell("/usr/bin/git log"), Category::VersionControl);
    assert_eq!(classify_shell("cat file"), Category::Inspect);
    assert_eq!(classify_shell("echo hello"), Category::Shell);
    assert_eq!(classify_shell(""), Category::Shell);
    // The one concession to how commands are written, and no more parsing than
    // that: a later stage of a pipeline is never looked at.
    assert_eq!(
        classify_shell("cd /somewhere && npm test"),
        Category::Verify
    );
    assert_eq!(classify_shell("cat a | sed -i s/x/y/ b"), Category::Inspect);
}

#[test]
fn a_tool_name_alone_decides_every_non_shell_category() {
    assert_eq!(corpus::categorize(Some("Read"), None), Category::Inspect);
    assert_eq!(corpus::categorize(Some("Edit"), None), Category::Modify);
    assert_eq!(
        corpus::categorize(Some("WebFetch"), None),
        Category::Research
    );
    assert_eq!(corpus::categorize(Some("Task"), None), Category::Delegate);
    // Nothing is guessed at. An unmapped tool is `Other`, not a plausible label.
    assert_eq!(
        corpus::categorize(Some("SomeNewTool"), None),
        Category::Other
    );
    assert_eq!(corpus::categorize(None, None), Category::Other);
    // A shell call with no command recorded is `Shell`, never a category the
    // absent command might have justified.
    assert_eq!(corpus::categorize(Some("Bash"), None), Category::Shell);
}

// ---------------------------------------------------------------------------
// The workflow projection
// ---------------------------------------------------------------------------

#[test]
fn reported_intent_contributes_no_action_and_is_counted() {
    let scratch = scratch();
    let session = "synthetic-reported-00";
    let mut at = 0i64;
    emit(
        scratch.path(),
        session,
        &mut at,
        Channel::Recorder,
        Event::SessionStarted(SessionStarted { source: None }),
    );
    // A claim citing an id no observed record ever mentions.
    emit(
        scratch.path(),
        session,
        &mut at,
        Channel::Reported,
        Event::ReportedIntent(ReportedIntent {
            text: "synthetic claim".to_owned(),
            tool_use_id: Some("toolu_orphan".to_owned()),
        }),
    );
    let replay = replay_file(&scratch.path().join(format!("{session}.ndjson")))
        .expect("synthetic recording replays");
    let inspection = inspect(&replay);
    let (actions, reported_only) = corpus::action_stream(&inspection);

    assert!(
        actions.is_empty(),
        "a reported-only correlation id must contribute no observed action"
    );
    assert_eq!(reported_only, 1, "and must be counted rather than dropped");
}

#[test]
fn an_action_keeps_its_outcome_and_its_receipts() {
    let scratch = scratch();
    write_session(
        scratch.path(),
        "synthetic-outcomes-00",
        &[
            step("Read"),
            failing("npm test"),
            Step {
                tool: "Bash",
                command: Some("rm -rf /"),
                outcome: Outcome::Denied,
            },
            Step {
                tool: "Write",
                command: None,
                outcome: Outcome::NoOutcomeObserved,
            },
        ],
    );
    let replay = replay_file(&scratch.path().join("synthetic-outcomes-00.ndjson"))
        .expect("synthetic recording replays");
    let inspection = inspect(&replay);
    let (actions, _) = corpus::action_stream(&inspection);

    let outcomes: Vec<Outcome> = actions.iter().map(|action| action.outcome).collect();
    assert_eq!(
        outcomes,
        vec![
            Outcome::Succeeded,
            Outcome::Failed,
            Outcome::Denied,
            Outcome::NoOutcomeObserved,
        ],
        "every observed outcome class survives; none is turned into a clean step"
    );
    let categories: Vec<Category> = actions.iter().map(|action| action.category).collect();
    assert_eq!(
        categories,
        vec![
            Category::Inspect,
            Category::Verify,
            Category::Shell,
            Category::Modify,
        ]
    );
    for action in &actions {
        assert!(
            action.first_sequence <= action.last_sequence,
            "an action must carry a receipt range into the raw stream"
        );
    }
    // A request with no outcome keeps its opening receipt and nothing is
    // invented to sit beside it.
    let unpaired = actions.last().expect("four actions");
    assert_eq!(unpaired.first_sequence, unpaired.last_sequence);
}

// ---------------------------------------------------------------------------
// The search, and its agreement with the established one
// ---------------------------------------------------------------------------

#[test]
fn the_retained_search_agrees_with_the_established_complete_search() {
    let scratch = scratch();
    positive_corpus(scratch.path(), 2);
    let paths: Vec<PathBuf> = corpus::discover(scratch.path()).expect("a readable directory");
    let replays: Vec<_> = paths
        .iter()
        .map(|path| replay_file(path).expect("synthetic recording replays"))
        .collect();
    let inspections: Vec<_> = replays.iter().map(inspect).collect();
    let sequences: Vec<_> = inspections
        .iter()
        .map(|inspection| project(inspection, ChannelScope::Observed).expect("a projection"))
        .collect();

    for k in [3usize, 4, 6] {
        let established = calibration::complete_search(&sequences[0], &sequences[1], k);
        let retained =
            corpus::retained_search("a", "b", &sequences[0], &sequences[1], k, calibration::KEEP);
        let best = retained
            .iter()
            .filter_map(|candidate| candidate.r1)
            .fold(f64::NEG_INFINITY, f64::max);
        match established.t {
            Some(value) => assert!(
                (best - value).abs() < 1e-12,
                "at k={k} the retained search must read the same maximum R1 as \
                 `complete_search`: {best} against {value}"
            ),
            None => assert!(retained.is_empty()),
        }
        assert_eq!(retained.len(), established.kept);
    }
}

// ---------------------------------------------------------------------------
// Recovery, and the negative controls
// ---------------------------------------------------------------------------

#[test]
fn a_planted_cross_session_shape_is_recovered_named_and_ranked() {
    let scratch = scratch();
    positive_corpus(scratch.path(), 5);
    let analysis = analyse(scratch.path(), "synthetic-positive", 49);
    let facts = &analysis.facts;

    assert_eq!(facts.discovered, 5);
    assert_eq!(facts.eligible_sessions, 5);

    let planted = vec![
        "Inspect".to_owned(),
        "Modify".to_owned(),
        "Verify".to_owned(),
        "Inspect".to_owned(),
    ];
    let found = facts
        .workflow
        .families
        .iter()
        .find(|family| family.pipeline == planted)
        .expect("the planted shape must be among the families the search found");

    assert_eq!(
        found.sessions, 5,
        "the planted shape was written into every session and must be found in every session"
    );
    assert_eq!(found.eligible, 5, "the denominator must be stated");
    assert!(
        found.occurrences >= 5 * 6,
        "six plants per session across five sessions"
    );
    assert!(
        found.quarantine.is_empty(),
        "a four-step shape over three distinct kinds of step is not instrument grammar"
    );
    assert_eq!(found.name, "Inspect–Modify–Verify–Inspect loop");
    assert!(
        found.tool_sequence.as_deref() == Some(&["Read", "Edit", "Bash", "Read"].map(String::from)),
        "the underlying delivered tool names must survive beside the categories: {:?}",
        found.tool_sequence
    );
    for support in found.support.iter().filter(|entry| entry.occurrences > 0) {
        assert!(
            !support.receipts.is_empty(),
            "every occurrence must carry raw sequence receipts"
        );
    }

    // And it is a lead a human is shown, not merely a row in the facts.
    let report = render_report(facts);
    assert!(
        report.contains("Inspect → Modify → Verify → Inspect"),
        "the planted pipeline must appear in the report"
    );
    // The four-step plant is itself flagged as subsumed by the five-step shape
    // that contains it and holds exactly the same sessions — the same finding
    // with less information. What must reach a human is a lead containing it.
    assert!(found.subsumed_by.is_some(), "{:?}", found.subsumed_by);
    let leads = lead_names(facts);
    assert!(
        leads
            .iter()
            .any(|name| name.contains("Inspect–Modify–Verify")),
        "a lead must carry the planted shape: {leads:?}"
    );
}

#[test]
fn a_dominant_repeated_tool_is_not_promoted_into_a_finding() {
    let scratch = scratch();
    dominant_corpus(scratch.path(), 5);
    let analysis = analyse(scratch.path(), "synthetic-dominant", 49);
    let facts = &analysis.facts;

    assert_eq!(facts.eligible_sessions, 5);
    assert!(
        !facts.workflow.families.is_empty(),
        "the search does find shapes here — the point is what happens to them"
    );
    for family in &facts.workflow.families {
        assert!(
            !family.quarantine.is_empty(),
            "every shape over a two-symbol vocabulary must be quarantined, not reported: {:?}",
            family.pipeline
        );
    }
    assert!(
        facts.leads.is_empty(),
        "a corpus with one dominant tool must produce no lead"
    );
    assert!(
        facts.exceptional.is_empty(),
        "and nothing in it may clear the calibration"
    );

    let report = render_report(facts);
    assert!(report.contains("Top investigation leads"));
    assert!(
        report.contains("None. Every shape the search found across sessions was quarantined"),
        "the report must say plainly that it found nothing"
    );
}

#[test]
fn request_outcome_alternation_is_quarantined_in_the_raw_control() {
    let scratch = scratch();
    positive_corpus(scratch.path(), 4);
    let analysis = analyse(scratch.path(), "synthetic-raw-control", 0);
    let raw = &analysis.facts.raw;

    assert!(
        !raw.families.is_empty(),
        "the raw projection finds plenty; that is the problem it exists to demonstrate"
    );
    let alternating = raw
        .families
        .iter()
        .filter(|family| {
            family
                .quarantine
                .contains(&Quarantine::RequestOutcomeAlternation)
        })
        .count();
    assert!(
        alternating > 0,
        "the recorder's request-then-outcome protocol must be recognised as protocol"
    );
    for family in &raw.families {
        assert_eq!(family.projection, Projection::Raw);
    }
    assert!(
        analysis
            .facts
            .leads
            .iter()
            .all(|id| id.starts_with("workflow-action")),
        "the raw projection is a control and never supplies a lead"
    );
}

// ---------------------------------------------------------------------------
// Eligibility
// ---------------------------------------------------------------------------

#[test]
fn every_discovered_file_is_accounted_for() {
    let scratch = scratch();
    positive_corpus(scratch.path(), 3);

    // Empty: replays, holds no complete record.
    std::fs::write(scratch.path().join("synthetic-empty.ndjson"), b"").expect("write");
    // Corrupt: does not replay at all.
    std::fs::write(
        scratch.path().join("synthetic-corrupt.ndjson"),
        b"{ this is not a record }\n",
    )
    .expect("write");
    // Tiny: one boundary record and nothing else.
    let mut at = 0i64;
    emit(
        scratch.path(),
        "synthetic-tiny",
        &mut at,
        Channel::Recorder,
        Event::SessionStarted(SessionStarted { source: None }),
    );
    // Too small a vocabulary: enough actions, one kind of step.
    write_session(
        scratch.path(),
        "synthetic-monotone",
        [step("Read"); 20].as_slice(),
    );
    // Truncated: a valid prefix followed by half a record.
    write_session(
        scratch.path(),
        "synthetic-truncated",
        &[
            step("Read"),
            step("Edit"),
            shell("npm test"),
            step("Read"),
            step("Write"),
            shell("git status"),
            step("Read"),
            step("Edit"),
            shell("ls"),
            shell("npm test"),
            step("Read"),
            step("Edit"),
            step("Write"),
        ],
    );
    {
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(scratch.path().join("synthetic-truncated.ndjson"))
            .expect("open");
        file.write_all(b"{\"schema_version\":2,\"session")
            .expect("write");
    }
    // A file that is not a recording at all is not discovered.
    std::fs::write(scratch.path().join("notes.txt"), b"ignored").expect("write");

    let analysis = analyse(scratch.path(), "synthetic-eligibility", 0);
    let manifest = &analysis.manifest;

    assert_eq!(
        manifest.discovered, 8,
        "three good, four bad, one truncated"
    );
    assert_eq!(manifest.included + manifest.skipped, manifest.discovered);
    assert_eq!(
        manifest.inputs.len(),
        manifest.discovered,
        "nothing may disappear silently"
    );

    let reason = |identity: &str| -> Option<SkipReason> {
        manifest
            .inputs
            .iter()
            .find(|input| input.identity == identity)
            .and_then(|input| input.skipped.clone())
    };
    assert_eq!(
        reason("synthetic"),
        None,
        "no identity collides by accident"
    );
    assert_eq!(
        reason("syntheti"),
        None,
        "identities are the first eight characters and these all share them"
    );

    // Identities are session prefixes, so these synthetic names collide by
    // design; check the classifications by their position in file-name order
    // instead, which is what the manifest is ordered by.
    let by_file: Vec<Option<SkipReason>> = manifest
        .inputs
        .iter()
        .map(|input| input.skipped.clone())
        .collect();
    assert!(
        by_file.iter().filter(|skip| skip.is_some()).count() >= 4,
        "the empty, corrupt, tiny and monotone recordings must all be skipped"
    );
    let reasons: Vec<SkipReason> = by_file.into_iter().flatten().collect();
    assert!(reasons.contains(&SkipReason::ReplayFailed), "{reasons:?}");
    assert!(
        reasons.contains(&SkipReason::NoCompleteRecords),
        "{reasons:?}"
    );
    assert!(reasons.contains(&SkipReason::TooFewActions), "{reasons:?}");
    assert!(
        reasons.contains(&SkipReason::VocabularyTooSmall),
        "{reasons:?}"
    );

    // The truncated recording's valid prefix is admitted, and says so.
    let truncated = manifest
        .inputs
        .iter()
        .find(|input| input.truncated)
        .expect("one truncated input");
    assert!(
        truncated.skipped.is_none(),
        "a truncated recording's valid prefix is evidence and is included"
    );
    assert_eq!(analysis.facts.truncated_included, 1);
    assert!(render_report(&analysis.facts).contains("end mid-record"));

    // Every input carries a fingerprint, and no input carries a path.
    for input in &manifest.inputs {
        assert!(!input.fingerprint.is_empty() || input.skipped.is_some());
        assert!(!input.identity.contains('/'));
    }
}

#[test]
fn a_duplicate_session_identity_is_skipped_rather_than_double_counted() {
    let scratch = scratch();
    let steps: Vec<Step> = (0..20)
        .map(|index| BACKGROUND[index % BACKGROUND.len()])
        .collect();
    write_session(scratch.path(), "synthetic-dup-a", &steps);
    // Same session id, different file name: the recorder writes one file per
    // session, so this can only be a copy, and a copy is not a second sample.
    let source = scratch.path().join("synthetic-dup-a.ndjson");
    std::fs::copy(&source, scratch.path().join("synthetic-dup-z.ndjson")).expect("copy");
    write_session(scratch.path(), "synthetic-dup-b", &steps);

    let analysis = analyse(scratch.path(), "synthetic-duplicate", 0);
    assert_eq!(analysis.manifest.discovered, 3);
    assert_eq!(analysis.manifest.included, 2);
    assert!(
        analysis
            .manifest
            .inputs
            .iter()
            .any(|input| input.skipped == Some(SkipReason::DuplicateIdentity))
    );
}

// ---------------------------------------------------------------------------
// Determinism
// ---------------------------------------------------------------------------

#[test]
fn two_runs_over_the_same_corpus_produce_identical_documents() {
    let scratch = scratch();
    positive_corpus(scratch.path(), 4);

    let first = analyse(scratch.path(), "synthetic-determinism", 99);
    let second = analyse(scratch.path(), "synthetic-determinism", 99);

    let facts_a = serde_json::to_string_pretty(&first.facts).expect("serializes");
    let facts_b = serde_json::to_string_pretty(&second.facts).expect("serializes");
    assert_eq!(facts_a, facts_b, "facts.json must be byte-identical");

    let manifest_a = serde_json::to_string_pretty(&first.manifest).expect("serializes");
    let manifest_b = serde_json::to_string_pretty(&second.manifest).expect("serializes");
    assert_eq!(
        manifest_a, manifest_b,
        "manifest.json must be byte-identical"
    );

    assert_eq!(
        render_report(&first.facts),
        render_report(&second.facts),
        "report.md must be byte-identical"
    );

    // And the manifest carries no clock: a document that changes between two
    // identical runs is not a reproducibility record.
    assert!(!manifest_a.contains("generated_at"));
}

#[test]
fn the_report_is_a_function_of_the_facts_document_alone() {
    let scratch = scratch();
    positive_corpus(scratch.path(), 3);
    let analysis = analyse(scratch.path(), "synthetic-render", 49);

    let direct = render_report(&analysis.facts);
    let text = serde_json::to_string(&analysis.facts).expect("serializes");
    let restored: Facts = serde_json::from_str(&text).expect("round-trips");
    let rendered = render_report(&restored);

    assert_eq!(
        direct, rendered,
        "re-rendering from a stored facts.json must reproduce the report exactly"
    );
}

#[test]
fn ranking_is_total_so_no_two_runs_can_disagree_on_order() {
    let scratch = scratch();
    positive_corpus(scratch.path(), 4);
    let analysis = analyse(scratch.path(), "synthetic-ranking", 0);
    let families = &analysis.facts.workflow.families;

    for pair in families.windows(2) {
        let (left, right) = (&pair[0], &pair[1]);
        let ordered = (right.sessions, right.occurrences, right.k)
            <= (left.sessions, left.occurrences, left.k)
            || left.sessions > right.sessions;
        assert!(
            ordered || left.sessions == right.sessions,
            "families must be ranked by a total order"
        );
    }
    let ids: Vec<&String> = families.iter().map(|family| &family.id).collect();
    let mut unique = ids.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(ids.len(), unique.len(), "family ids must be unique");
}

// ---------------------------------------------------------------------------
// Privacy
// ---------------------------------------------------------------------------

#[test]
fn no_command_text_reaches_any_document() {
    let scratch = scratch();
    // A command carrying a string that appears nowhere else, so finding it in an
    // output can only mean the payload leaked.
    let steps: Vec<Step> = (0..20)
        .map(|index| match index % 4 {
            0 => step("Read"),
            1 => step("Edit"),
            2 => shell("npm test --filter zzqqxx-secret-marker"),
            _ => shell("git commit -m zzqqxx-secret-marker"),
        })
        .collect();
    write_session(scratch.path(), "synthetic-privacy-00", &steps);
    write_session(scratch.path(), "synthetic-privacy-01", &steps);

    let analysis = analyse(scratch.path(), "synthetic-privacy", 49);
    let facts = serde_json::to_string(&analysis.facts).expect("serializes");
    let manifest = serde_json::to_string(&analysis.manifest).expect("serializes");
    let report = render_report(&analysis.facts);

    for (name, document) in [
        ("facts", &facts),
        ("manifest", &manifest),
        ("report", &report),
    ] {
        assert!(
            !document.contains("zzqqxx-secret-marker"),
            "{name} must not contain command text"
        );
        assert!(!document.contains("npm test"), "{name} leaked a command");
        assert!(!document.contains("git commit"), "{name} leaked a command");
        assert!(
            !document.contains("synthetic failure"),
            "{name} leaked an error string"
        );
    }
    // The categories the command text produced did survive, which is the whole
    // point of reading it.
    assert!(report.contains("Verify"));
    assert!(report.contains("VersionControl"));
}

// ---------------------------------------------------------------------------
// Comparison
// ---------------------------------------------------------------------------

#[test]
fn comparison_reports_a_gained_and_a_strengthened_shape() {
    let before_dir = scratch();
    let after_dir = scratch();

    // A: three sessions, two of which carry the first planted shape. A shape has
    // to appear in two sessions to be discovered at all — a cross-recording
    // search cannot produce a candidate from one recording.
    corpus_with(before_dir.path(), "synthetic-before", 3, &[(&PLANTED, 2)]);
    // B: four sessions, all of which carry it, plus a second shape A never has.
    corpus_with(
        after_dir.path(),
        "synthetic-after",
        4,
        &[(&PLANTED, 4), (&PLANTED_SECOND, 4)],
    );

    let before = analyse(before_dir.path(), "corpus-a", 0);
    let after = analyse(after_dir.path(), "corpus-b", 0);
    let comparison = render_comparison(&before.facts, &after.facts);

    assert!(comparison.contains("## Gained"));
    assert!(comparison.contains("## Strengthened"));
    assert!(comparison.contains("## Lost"));
    assert!(comparison.contains("## Weakened"));
    assert!(comparison.contains("## Unchanged"));
    assert!(comparison.contains("corpus-a") && comparison.contains("corpus-b"));

    let planted_line = comparison
        .lines()
        .find(|line| line.contains("`Inspect → Modify → Verify → Inspect`"))
        .expect("the planted shape must appear somewhere in the comparison");
    assert!(
        planted_line.contains("2 of 3 sessions in `corpus-a`, 4 of 4 in `corpus-b`"),
        "both denominators must be printed: {planted_line}"
    );

    let gained = comparison
        .split("## Gained")
        .nth(1)
        .and_then(|rest| rest.split("## ").next())
        .expect("a Gained section");
    assert!(
        gained.contains("`Modify → VersionControl → Verify → Modify`"),
        "the shape planted only in B must be reported as gained: {gained}"
    );
    let strengthened = comparison
        .split("## Strengthened")
        .nth(1)
        .and_then(|rest| rest.split("## ").next())
        .expect("a Strengthened section");
    assert!(
        strengthened.contains("`Inspect → Modify → Verify → Inspect`"),
        "the shape present in both must be reported as strengthened: {strengthened}"
    );
    assert!(comparison.contains("not evidence that anything changed"));
}

// ---------------------------------------------------------------------------
// What the report says about itself
// ---------------------------------------------------------------------------

#[test]
fn the_report_states_its_own_limits() {
    let scratch = scratch();
    positive_corpus(scratch.path(), 4);
    let analysis = analyse(scratch.path(), "synthetic-limits", 99);
    let report = render_report(&analysis.facts);

    for required in [
        "not safe to share",
        "Corpus at a glance",
        "Top investigation leads",
        "Common background grammar",
        "What the calibrated matcher did not find",
        "Limitations, in plain English",
        "How to reproduce this report",
        "What an A/B corpus comparison would mean",
        "A category is a label this analyser applied",
        "An action is a correlation, not an execution",
        "Nothing here reads reported intent",
        "exploratory engineering round",
        "not** sprint:19's `T`",
    ] {
        assert!(
            report.contains(required),
            "the report must say {required:?}"
        );
    }
    // Nothing may claim the report is safe to pass on.
    for forbidden in [
        "redacted and safe",
        "sanitized and safe",
        "safe to share with",
    ] {
        assert!(!report.contains(forbidden));
    }
}

#[test]
fn zero_replicates_produce_no_calibration_rather_than_a_fabricated_one() {
    let scratch = scratch();
    positive_corpus(scratch.path(), 3);
    let analysis = analyse(scratch.path(), "synthetic-descriptive", 0);

    assert!(analysis.facts.workflow.null.is_empty());
    for family in &analysis.facts.workflow.families {
        assert!(
            family.calibration.is_none(),
            "with no null there is no tail, and none is invented"
        );
    }
    assert!(analysis.facts.exceptional.is_empty());
}
