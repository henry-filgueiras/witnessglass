//! The inspection projection.
//!
//! Every recording here is synthetic and obviously so. Nothing in this file
//! reads, lists, copies, or is derived from a real recording.
//!
//! The projection's whole claim is that a derived statement can produce the raw
//! records that licensed it, so most of these tests assert on receipts rather
//! than on counts.

mod common;

use common::*;
use witnessglass::inspection::{
    AgentAttribution, AnomalyKind, CorrelationId, CoveredField, EventKind, ExaminedScope,
    GroupShape, Inspection, Receipts, SequenceInterval, ToolEvidence, V1Kind, V2Kind,
};
use witnessglass::record::v1;
use witnessglass::{Context, Event, Replay, Tail, inspect, replay_bytes};

fn replay_of(recording: &str) -> Replay {
    replay_bytes(recording.as_bytes()).expect("synthetic recording should replay")
}

/// A row with no causal context supplied.
fn row(recorded_at: &str, event: Event) -> V2Row<'_> {
    (recorded_at, Context::default(), event)
}

/// A row with causal context supplied.
fn row_ctx(recorded_at: &str, context: Context, event: Event) -> V2Row<'_> {
    (recorded_at, context, event)
}

fn group<'i, 'a>(
    inspection: &'i Inspection<'a>,
    id: CorrelationId<'_>,
) -> &'i witnessglass::inspection::ToolGroup<'a> {
    inspection
        .tool_groups
        .iter()
        .find(|group| group.id.as_str() == id.as_str() && group.id == id)
        .expect("expected a group for this correlation id")
}

fn kind_count(inspection: &Inspection<'_>, kind: EventKind) -> usize {
    inspection
        .aggregates
        .by_event_kind
        .iter()
        .find(|tally| tally.value == kind)
        .expect("the schema's whole vocabulary should be tallied")
        .records
        .count()
}

fn anomaly_kinds<'a>(inspection: &Inspection<'a>) -> Vec<AnomalyKind<'a>> {
    inspection
        .anomalies
        .iter()
        .map(|anomaly| anomaly.kind.clone())
        .collect()
}

/// Every receipt anywhere in the projection, so a test can assert that no
/// derived claim cites a record that does not exist.
fn all_receipts(inspection: &Inspection<'_>) -> Vec<u64> {
    fn take(out: &mut Vec<u64>, receipts: &Receipts) {
        out.extend_from_slice(receipts.sequences());
    }

    let out = &mut Vec::new();
    take(out, &inspection.session_boundaries.started.records);
    take(out, &inspection.session_boundaries.ended.records);
    for group in &inspection.tool_groups {
        take(out, &group.reported_intents.records);
        match &group.evidence {
            ToolEvidence::V1 {
                started,
                finished_succeeded,
                finished_failed,
            } => {
                take(out, started);
                take(out, finished_succeeded);
                take(out, finished_failed);
            }
            ToolEvidence::V2 {
                requested,
                succeeded,
                failed,
                denied,
            } => {
                take(out, requested);
                take(out, succeeded);
                take(out, failed);
                take(out, denied);
            }
        }
        for delivered in &group.delivered_tool_names {
            take(out, &delivered.receipts);
        }
        if let Some(SequenceInterval { opening, outcome }) = group.paired_interval {
            out.push(opening);
            out.push(outcome);
        }
        out.push(group.first_sequence);
    }
    for subagent in &inspection.subagents {
        take(out, &subagent.started.records);
        take(out, &subagent.stopped.records);
        for delivered in &subagent.delivered_types {
            take(out, &delivered.receipts);
        }
        for delivered in &subagent.supplied_parents {
            take(out, &delivered.receipts);
        }
        out.push(subagent.first_sequence);
    }
    for tally in &inspection.current_agents.supplied {
        take(out, &tally.records.records);
    }
    take(out, &inspection.current_agents.not_supplied.records);
    take(out, &inspection.current_agents.not_representable.records);
    for tally in &inspection.aggregates.by_channel {
        take(out, &tally.records.records);
    }
    for tally in &inspection.aggregates.by_event_kind {
        take(out, &tally.records.records);
    }
    for tally in &inspection.aggregates.by_adapter {
        take(out, &tally.records.records);
    }
    for tally in &inspection.aggregates.by_mechanism {
        take(out, &tally.records.records);
    }
    for coverage in &inspection.coverage {
        take(out, &coverage.population.records);
        take(out, &coverage.present.records);
        take(out, &coverage.absent.records);
    }
    if let Some(timestamps) = &inspection.timestamps {
        take(out, &timestamps.non_monotonic.records);
        out.push(timestamps.earliest.sequence);
        out.push(timestamps.latest.sequence);
    }
    for anomaly in &inspection.anomalies {
        take(out, &anomaly.receipts);
    }
    out.clone()
}

// ---------------------------------------------------------------------------
// Correlation, per schema, without flattening
// ---------------------------------------------------------------------------

#[test]
fn v2_evidence_correlates_through_tool_use_id() {
    let recording = v2_recording(vec![
        row("2026-01-01T00:00:00Z", ev_session_started(Some("startup"))),
        row(
            "2026-01-01T00:00:01Z",
            ev_tool_requested("toolu_a", "SyntheticTool"),
        ),
        row(
            "2026-01-01T00:00:02Z",
            ev_tool_succeeded("toolu_a", "SyntheticTool", None),
        ),
        row("2026-01-01T00:00:03Z", ev_session_ended(Some("exit"))),
    ]);
    let replay = replay_of(&recording);
    let inspection = inspect(&replay);

    assert_eq!(inspection.tool_groups.len(), 1);
    let group = group(&inspection, CorrelationId::V2ToolUseId("toolu_a"));
    assert_eq!(group.shape, GroupShape::PairedLifecycle);
    assert_eq!(
        group.paired_interval,
        Some(SequenceInterval {
            opening: 2,
            outcome: 3
        })
    );
    match &group.evidence {
        ToolEvidence::V2 {
            requested,
            succeeded,
            failed,
            denied,
        } => {
            assert_eq!(requested.sequences(), [2]);
            assert_eq!(succeeded.sequences(), [3]);
            assert!(failed.is_empty());
            assert!(denied.is_empty());
        }
        other => panic!("a v2 recording must produce v2 evidence, got {other:?}"),
    }
}

#[test]
fn v1_evidence_correlates_through_tool_call_id_without_adopting_v2_semantics() {
    let recording = v1_recording(vec![
        ("2026-01-01T00:00:00Z", v1::Event::SessionStarted),
        (
            "2026-01-01T00:00:01Z",
            ev1_tool_started("call-a", "SyntheticTool"),
        ),
        (
            "2026-01-01T00:00:02Z",
            ev1_tool_finished("call-a", v1::ToolOutcome::Succeeded),
        ),
        ("2026-01-01T00:00:03Z", v1::Event::SessionEnded),
    ]);
    let replay = replay_of(&recording);
    let inspection = inspect(&replay);

    assert_eq!(inspection.schema_version, Some(1));
    let group = group(&inspection, CorrelationId::V1ToolCallId("call-a"));
    match &group.evidence {
        ToolEvidence::V1 {
            started,
            finished_succeeded,
            finished_failed,
        } => {
            // v1's `observed_tool_started` claims a witnessed beginning. It is
            // filed as that claim, not as a v2 request.
            assert_eq!(started.sequences(), [2]);
            assert_eq!(finished_succeeded.sequences(), [3]);
            assert!(finished_failed.is_empty());
        }
        other => panic!("a v1 recording must produce v1 evidence, got {other:?}"),
    }

    // v1 has no denial vocabulary and no causal context, and the projection does
    // not lend it either.
    assert_eq!(inspection.aggregates.by_event_kind.len(), V1Kind::ALL.len());
    for tally in &inspection.aggregates.by_event_kind {
        assert_eq!(tally.value.schema_version(), 1);
        assert_ne!(tally.value.as_str(), "tool_denied");
    }
    for entry in &inspection.ledger {
        assert_eq!(entry.current_agent, AgentAttribution::NotRepresentable);
        assert_eq!(entry.prompt_id, None);
    }
    assert_eq!(
        inspection.current_agents.not_representable.count(),
        inspection.record_count()
    );
    assert!(inspection.current_agents.supplied.is_empty());
    assert!(inspection.current_agents.not_supplied.is_absent());
}

#[test]
fn a_v1_id_and_a_v2_id_spelled_the_same_are_not_the_same_key() {
    assert_ne!(
        CorrelationId::V1ToolCallId("shared"),
        CorrelationId::V2ToolUseId("shared")
    );
}

#[test]
fn reported_intent_stays_a_separate_record_beside_the_observed_evidence() {
    // decision:4 duplicates an agent-supplied description into its own reported
    // record; task:4 measured that duplication 65 for 65. The projection
    // correlates the two and fuses nothing.
    let recording = v2_recording(vec![
        row(
            "2026-01-01T00:00:00Z",
            ev_tool_requested("toolu_a", "SyntheticTool"),
        ),
        row(
            "2026-01-01T00:00:01Z",
            ev_reported_intent("check the synthetic thing", Some("toolu_a")),
        ),
        row(
            "2026-01-01T00:00:02Z",
            ev_tool_succeeded("toolu_a", "SyntheticTool", None),
        ),
    ]);
    let replay = replay_of(&recording);
    let inspection = inspect(&replay);

    let group = group(&inspection, CorrelationId::V2ToolUseId("toolu_a"));
    // The intent is correlated, and it is not counted as observed evidence.
    assert_eq!(group.reported_intents.records.sequences(), [2]);
    assert_eq!(group.shape, GroupShape::PairedLifecycle);
    assert_eq!(
        group.paired_interval,
        Some(SequenceInterval {
            opening: 1,
            outcome: 3
        })
    );

    // Three records in, three records out, on the channels they arrived on.
    assert_eq!(inspection.ledger.len(), 3);
    assert_eq!(
        inspection.ledger[1].channel,
        witnessglass::Channel::Reported
    );
    assert_eq!(
        inspection.ledger[0].channel,
        witnessglass::Channel::Observed
    );
    assert_eq!(
        inspection.ledger[2].channel,
        witnessglass::Channel::Observed
    );
    assert_eq!(
        inspection.ledger[1].facets.reported_text,
        Some("check the synthetic thing")
    );
}

// ---------------------------------------------------------------------------
// Cardinality
// ---------------------------------------------------------------------------

#[test]
fn a_request_with_no_observed_outcome_is_a_first_class_state() {
    let recording = v2_recording(vec![row(
        "2026-01-01T00:00:00Z",
        ev_tool_requested("toolu_a", "SyntheticTool"),
    )]);
    let replay = replay_of(&recording);
    let inspection = inspect(&replay);

    let group = group(&inspection, CorrelationId::V2ToolUseId("toolu_a"));
    assert_eq!(group.shape, GroupShape::OpeningWithoutOutcome);
    assert_eq!(group.paired_interval, None);
    assert!(
        anomaly_kinds(&inspection).contains(&AnomalyKind::OpeningWithoutOutcome {
            id: CorrelationId::V2ToolUseId("toolu_a")
        })
    );
}

#[test]
fn an_outcome_with_no_observed_request_is_a_first_class_state() {
    let recording = v2_recording(vec![row(
        "2026-01-01T00:00:00Z",
        ev_tool_succeeded("toolu_a", "SyntheticTool", None),
    )]);
    let replay = replay_of(&recording);
    let inspection = inspect(&replay);

    let group = group(&inspection, CorrelationId::V2ToolUseId("toolu_a"));
    assert_eq!(group.shape, GroupShape::OutcomeWithoutOpening);
    assert_eq!(group.paired_interval, None);
    assert!(
        anomaly_kinds(&inspection).contains(&AnomalyKind::OutcomeWithoutOpening {
            id: CorrelationId::V2ToolUseId("toolu_a")
        })
    );
}

#[test]
fn an_intent_citing_an_id_no_observation_carries_is_a_first_class_state() {
    let recording = v2_recording(vec![row(
        "2026-01-01T00:00:00Z",
        ev_reported_intent("a claim about a call nothing observed", Some("toolu_a")),
    )]);
    let replay = replay_of(&recording);
    let inspection = inspect(&replay);

    let group = group(&inspection, CorrelationId::V2ToolUseId("toolu_a"));
    assert_eq!(group.shape, GroupShape::ReportedIntentOnly);
    assert_eq!(group.reported_intents.records.sequences(), [1]);
    assert!(anomaly_kinds(&inspection).contains(
        &AnomalyKind::ReportedIntentWithoutObservedEvidence {
            id: CorrelationId::V2ToolUseId("toolu_a")
        }
    ));
}

#[test]
fn duplicate_requests_are_not_greedily_paired_with_the_first_outcome() {
    let recording = v2_recording(vec![
        row(
            "2026-01-01T00:00:00Z",
            ev_tool_requested("toolu_a", "SyntheticTool"),
        ),
        row(
            "2026-01-01T00:00:01Z",
            ev_tool_requested("toolu_a", "SyntheticTool"),
        ),
        row(
            "2026-01-01T00:00:02Z",
            ev_tool_succeeded("toolu_a", "SyntheticTool", None),
        ),
    ]);
    let replay = replay_of(&recording);
    let inspection = inspect(&replay);

    let group = group(&inspection, CorrelationId::V2ToolUseId("toolu_a"));
    assert_eq!(group.shape, GroupShape::Ambiguous);
    // No interval: nothing was paired, so nothing may be presented as a pair.
    assert_eq!(group.paired_interval, None);
    assert_eq!(group.evidence.opening().sequences(), [1, 2]);

    let duplicate = inspection
        .anomalies
        .iter()
        .find(|anomaly| {
            anomaly.kind
                == AnomalyKind::DuplicateOpenings {
                    id: CorrelationId::V2ToolUseId("toolu_a"),
                }
        })
        .expect("duplicate openings should be reported");
    assert_eq!(duplicate.receipts.sequences(), [1, 2]);
}

#[test]
fn duplicate_outcomes_are_not_greedily_paired_with_the_request() {
    let recording = v2_recording(vec![
        row(
            "2026-01-01T00:00:00Z",
            ev_tool_requested("toolu_a", "SyntheticTool"),
        ),
        row(
            "2026-01-01T00:00:01Z",
            ev_tool_succeeded("toolu_a", "SyntheticTool", None),
        ),
        row(
            "2026-01-01T00:00:02Z",
            ev_tool_succeeded("toolu_a", "SyntheticTool", None),
        ),
    ]);
    let replay = replay_of(&recording);
    let inspection = inspect(&replay);

    let group = group(&inspection, CorrelationId::V2ToolUseId("toolu_a"));
    assert_eq!(group.shape, GroupShape::Ambiguous);
    assert_eq!(group.paired_interval, None);

    let duplicate = inspection
        .anomalies
        .iter()
        .find(|anomaly| {
            anomaly.kind
                == AnomalyKind::DuplicateOutcomes {
                    id: CorrelationId::V2ToolUseId("toolu_a"),
                }
        })
        .expect("duplicate outcomes should be reported");
    assert_eq!(duplicate.receipts.sequences(), [2, 3]);
    // Duplicated but not conflicting: two successes disagree about nothing.
    assert!(
        !anomaly_kinds(&inspection).contains(&AnomalyKind::ConflictingOutcomes {
            id: CorrelationId::V2ToolUseId("toolu_a")
        })
    );
}

#[test]
fn success_failure_and_denial_stay_distinct() {
    let recording = v2_recording(vec![
        row(
            "2026-01-01T00:00:00Z",
            ev_tool_succeeded("toolu_ok", "SyntheticTool", None),
        ),
        row(
            "2026-01-01T00:00:01Z",
            ev_tool_failed("toolu_bad", "SyntheticTool", Some(true)),
        ),
        row(
            "2026-01-01T00:00:02Z",
            ev_tool_denied("toolu_no", "SyntheticTool"),
        ),
    ]);
    let replay = replay_of(&recording);
    let inspection = inspect(&replay);

    for (id, expected) in [
        ("toolu_ok", "tool_succeeded"),
        ("toolu_bad", "tool_failed"),
        ("toolu_no", "tool_denied"),
    ] {
        let group = group(&inspection, CorrelationId::V2ToolUseId(id));
        match &group.evidence {
            ToolEvidence::V2 {
                succeeded,
                failed,
                denied,
                ..
            } => {
                let present: Vec<&str> = [
                    ("tool_succeeded", succeeded),
                    ("tool_failed", failed),
                    ("tool_denied", denied),
                ]
                .into_iter()
                .filter(|(_, receipts)| !receipts.is_empty())
                .map(|(name, _)| name)
                .collect();
                assert_eq!(present, vec![expected], "for {id}");
            }
            other => panic!("expected v2 evidence, got {other:?}"),
        }
    }

    assert_eq!(
        inspection.ledger[1].facets.interrupted,
        Some(true),
        "a delivered interruption flag survives"
    );
    assert_eq!(inspection.ledger[1].facets.error, Some("synthetic failure"));
    // A denial is not an execution, so it carries a requested input and no
    // effective one.
    assert!(inspection.ledger[2].facets.has_requested_input);
    assert!(!inspection.ledger[2].facets.has_effective_input);
}

#[test]
fn conflicting_outcomes_become_an_anomaly_carrying_every_receipt() {
    let recording = v2_recording(vec![
        row(
            "2026-01-01T00:00:00Z",
            ev_tool_requested("toolu_a", "SyntheticTool"),
        ),
        row(
            "2026-01-01T00:00:01Z",
            ev_tool_succeeded("toolu_a", "SyntheticTool", None),
        ),
        row(
            "2026-01-01T00:00:02Z",
            ev_tool_failed("toolu_a", "SyntheticTool", None),
        ),
        row(
            "2026-01-01T00:00:03Z",
            ev_tool_denied("toolu_a", "SyntheticTool"),
        ),
    ]);
    let replay = replay_of(&recording);
    let inspection = inspect(&replay);

    let group = group(&inspection, CorrelationId::V2ToolUseId("toolu_a"));
    assert_eq!(group.shape, GroupShape::Ambiguous);
    assert_eq!(group.evidence.outcome_classes(), 3);
    // Every outcome is kept. None is chosen.
    match &group.evidence {
        ToolEvidence::V2 {
            succeeded,
            failed,
            denied,
            ..
        } => {
            assert_eq!(succeeded.sequences(), [2]);
            assert_eq!(failed.sequences(), [3]);
            assert_eq!(denied.sequences(), [4]);
        }
        other => panic!("expected v2 evidence, got {other:?}"),
    }

    let conflict = inspection
        .anomalies
        .iter()
        .find(|anomaly| {
            anomaly.kind
                == AnomalyKind::ConflictingOutcomes {
                    id: CorrelationId::V2ToolUseId("toolu_a"),
                }
        })
        .expect("conflicting outcomes should be reported");
    assert_eq!(conflict.receipts.sequences(), [2, 3, 4]);
}

#[test]
fn a_v1_success_beside_a_v1_failure_also_conflicts() {
    let recording = v1_recording(vec![
        (
            "2026-01-01T00:00:00Z",
            ev1_tool_started("call-a", "SyntheticTool"),
        ),
        (
            "2026-01-01T00:00:01Z",
            ev1_tool_finished("call-a", v1::ToolOutcome::Succeeded),
        ),
        (
            "2026-01-01T00:00:02Z",
            ev1_tool_finished("call-a", v1::ToolOutcome::Failed),
        ),
    ]);
    let replay = replay_of(&recording);
    let inspection = inspect(&replay);

    let group = group(&inspection, CorrelationId::V1ToolCallId("call-a"));
    assert_eq!(group.shape, GroupShape::Ambiguous);
    assert!(
        anomaly_kinds(&inspection).contains(&AnomalyKind::ConflictingOutcomes {
            id: CorrelationId::V1ToolCallId("call-a")
        })
    );
}

#[test]
fn differing_tool_names_stay_delivered_evidence_rather_than_one_canonical_value() {
    let recording = v2_recording(vec![
        row(
            "2026-01-01T00:00:00Z",
            ev_tool_requested("toolu_a", "RequestedName"),
        ),
        row(
            "2026-01-01T00:00:01Z",
            ev_tool_succeeded("toolu_a", "DeliveredName", None),
        ),
    ]);
    let replay = replay_of(&recording);
    let inspection = inspect(&replay);

    let group = group(&inspection, CorrelationId::V2ToolUseId("toolu_a"));
    assert_eq!(group.delivered_tool_names.len(), 2);
    assert_eq!(group.delivered_tool_names[0].value, "RequestedName");
    assert_eq!(group.delivered_tool_names[0].receipts.sequences(), [1]);
    assert_eq!(group.delivered_tool_names[1].value, "DeliveredName");
    assert_eq!(group.delivered_tool_names[1].receipts.sequences(), [2]);

    let divergent = inspection
        .anomalies
        .iter()
        .find(|anomaly| {
            anomaly.kind
                == AnomalyKind::DivergentToolNames {
                    id: CorrelationId::V2ToolUseId("toolu_a"),
                }
        })
        .expect("divergent tool names should be reported");
    assert_eq!(divergent.receipts.sequences(), [1, 2]);
    // The cardinality is still one-to-one; the disagreement is about a field.
    assert_eq!(group.shape, GroupShape::PairedLifecycle);
}

// ---------------------------------------------------------------------------
// Agent identity
// ---------------------------------------------------------------------------

#[test]
fn current_agent_attribution_is_distinct_from_a_subagent_events_child_id() {
    let recording = v2_recording(vec![
        row_ctx(
            "2026-01-01T00:00:00Z",
            Context::default(),
            ev_subagent_started("agent-child", Some("Plan"), None, None),
        ),
        row_ctx(
            "2026-01-01T00:00:01Z",
            context(Some("prompt-1"), Some("agent-child"), Some("Plan")),
            ev_tool_requested("toolu_a", "SyntheticTool"),
        ),
        row_ctx(
            "2026-01-01T00:00:02Z",
            context(Some("prompt-1"), Some("agent-child"), Some("Plan")),
            ev_tool_succeeded("toolu_a", "SyntheticTool", None),
        ),
        row_ctx(
            "2026-01-01T00:00:03Z",
            Context::default(),
            ev_subagent_stopped("agent-child", Some("Plan"), None, None),
        ),
    ]);
    let replay = replay_of(&recording);
    let inspection = inspect(&replay);

    // The boundary records are *about* the child and were not delivered from it.
    assert_eq!(
        inspection.ledger[0].current_agent,
        AgentAttribution::NotSupplied { agent_type: None }
    );
    assert_eq!(
        inspection.ledger[0]
            .subject_agent
            .expect("a subagent boundary names its subject")
            .agent_id,
        "agent-child"
    );
    assert_eq!(inspection.ledger[1].subject_agent, None);
    assert_eq!(
        inspection.ledger[1].current_agent,
        AgentAttribution::Supplied {
            agent_id: "agent-child",
            agent_type: Some("Plan"),
        }
    );

    // The attribution aggregate counts only records delivered *from* an agent.
    assert_eq!(inspection.current_agents.supplied.len(), 1);
    assert_eq!(inspection.current_agents.supplied[0].value, "agent-child");
    assert_eq!(
        inspection.current_agents.supplied[0]
            .records
            .records
            .sequences(),
        [2, 3]
    );
    assert_eq!(
        inspection.current_agents.not_supplied.records.sequences(),
        [1, 4]
    );

    // The subagent index tracks the same id as a subject, separately.
    assert_eq!(inspection.subagents.len(), 1);
    assert_eq!(inspection.subagents[0].agent_id, "agent-child");
    assert_eq!(inspection.subagents[0].started.records.sequences(), [1]);
    assert_eq!(inspection.subagents[0].stopped.records.sequences(), [4]);
}

#[test]
fn absent_current_agent_identity_stays_unattributed() {
    let recording = v2_recording(vec![
        row("2026-01-01T00:00:00Z", ev_session_started(Some("startup"))),
        row(
            "2026-01-01T00:00:01Z",
            ev_tool_requested("toolu_a", "SyntheticTool"),
        ),
    ]);
    let replay = replay_of(&recording);
    let inspection = inspect(&replay);

    assert!(inspection.current_agents.supplied.is_empty());
    assert_eq!(
        inspection.current_agents.not_supplied.records.sequences(),
        [1, 2]
    );
    assert!(inspection.current_agents.not_representable.is_absent());
    for entry in &inspection.ledger {
        assert_eq!(
            entry.current_agent,
            AgentAttribution::NotSupplied { agent_type: None },
            "absent identity is absent, not a root agent"
        );
    }
}

#[test]
fn supplied_parent_identity_is_retained_and_absent_parent_is_never_inferred() {
    let recording = v2_recording(vec![
        row(
            "2026-01-01T00:00:00Z",
            ev_subagent_started(
                "agent-with",
                Some("Plan"),
                Some("agent-parent"),
                Some("Main"),
            ),
        ),
        row(
            "2026-01-01T00:00:01Z",
            ev_subagent_stopped(
                "agent-with",
                Some("Plan"),
                Some("agent-parent"),
                Some("Main"),
            ),
        ),
        row(
            "2026-01-01T00:00:02Z",
            ev_subagent_started("agent-without", Some("Plan"), None, None),
        ),
        row(
            "2026-01-01T00:00:03Z",
            ev_subagent_stopped("agent-without", Some("Plan"), None, None),
        ),
    ]);
    let replay = replay_of(&recording);
    let inspection = inspect(&replay);

    let with = &inspection.subagents[0];
    assert_eq!(with.agent_id, "agent-with");
    assert_eq!(with.supplied_parents.len(), 1);
    assert_eq!(
        with.supplied_parents[0].value.agent_id,
        Some("agent-parent")
    );
    assert_eq!(with.supplied_parents[0].value.agent_type, Some("Main"));
    assert_eq!(with.supplied_parents[0].receipts.sequences(), [1, 2]);

    let without = &inspection.subagents[1];
    assert_eq!(without.agent_id, "agent-without");
    assert!(
        without.supplied_parents.is_empty(),
        "no parent was delivered, so no parent exists in the projection"
    );

    let parent_coverage = inspection
        .coverage
        .iter()
        .find(|coverage| coverage.field == CoveredField::V2SuppliedParentAgent)
        .expect("parent coverage should be summarized");
    assert_eq!(parent_coverage.population.records.sequences(), [1, 2, 3, 4]);
    assert_eq!(parent_coverage.present.records.sequences(), [1, 2]);
    assert_eq!(parent_coverage.absent.records.sequences(), [3, 4]);
}

#[test]
fn one_agent_id_delivered_with_two_types_exposes_the_disagreement() {
    let recording = v2_recording(vec![
        row_ctx(
            "2026-01-01T00:00:00Z",
            context(None, Some("agent-a"), Some("Plan")),
            ev_tool_requested("toolu_a", "SyntheticTool"),
        ),
        row_ctx(
            "2026-01-01T00:00:01Z",
            context(None, Some("agent-a"), Some("Explore")),
            ev_tool_succeeded("toolu_a", "SyntheticTool", None),
        ),
    ]);
    let replay = replay_of(&recording);
    let inspection = inspect(&replay);

    let divergent = inspection
        .anomalies
        .iter()
        .find(|anomaly| {
            anomaly.kind
                == AnomalyKind::DivergentAgentTypes {
                    agent_id: "agent-a",
                }
        })
        .expect("divergent agent types should be reported");
    assert_eq!(divergent.receipts.sequences(), [1, 2]);
    // Both delivered values survive on the records themselves.
    assert_eq!(
        inspection.ledger[0].current_agent,
        AgentAttribution::Supplied {
            agent_id: "agent-a",
            agent_type: Some("Plan")
        }
    );
    assert_eq!(
        inspection.ledger[1].current_agent,
        AgentAttribution::Supplied {
            agent_id: "agent-a",
            agent_type: Some("Explore")
        }
    );
}

#[test]
fn a_subagent_id_delivered_with_two_types_keeps_both() {
    let recording = v2_recording(vec![
        row(
            "2026-01-01T00:00:00Z",
            ev_subagent_started("agent-child", Some("Plan"), None, None),
        ),
        row(
            "2026-01-01T00:00:01Z",
            ev_subagent_stopped("agent-child", Some(""), None, None),
        ),
    ]);
    let replay = replay_of(&recording);
    let inspection = inspect(&replay);

    let child = &inspection.subagents[0];
    assert_eq!(child.delivered_types.len(), 2);
    assert_eq!(child.delivered_types[0].value, Some("Plan"));
    assert_eq!(child.delivered_types[0].receipts.sequences(), [1]);
    assert_eq!(child.delivered_types[1].value, Some(""));
    assert_eq!(child.delivered_types[1].receipts.sequences(), [2]);
}

#[test]
fn sequence_containment_inside_an_agent_call_creates_no_parent_and_no_nesting() {
    // The shape task:4 measured: subagent-attributed records fell between an
    // `Agent` call's request and its outcome in append sequence. That is
    // containment in the append chain and nothing else — no parent, no child
    // relationship, no nested span.
    let recording = v2_recording(vec![
        row("2026-01-01T00:00:00Z", ev_session_started(Some("startup"))),
        row(
            "2026-01-01T00:00:01Z",
            ev_tool_requested("toolu_agent", "Agent"),
        ),
        row(
            "2026-01-01T00:00:02Z",
            ev_subagent_started("agent-child", Some("Plan"), None, None),
        ),
        row_ctx(
            "2026-01-01T00:00:03Z",
            context(Some("prompt-1"), Some("agent-child"), Some("Plan")),
            ev_tool_requested("toolu_inner", "SyntheticTool"),
        ),
        row_ctx(
            "2026-01-01T00:00:04Z",
            context(Some("prompt-1"), Some("agent-child"), Some("Plan")),
            ev_tool_succeeded("toolu_inner", "SyntheticTool", None),
        ),
        row(
            "2026-01-01T00:00:05Z",
            ev_subagent_stopped("agent-child", Some("Plan"), None, None),
        ),
        row(
            "2026-01-01T00:00:06Z",
            ev_tool_succeeded("toolu_agent", "Agent", None),
        ),
        row("2026-01-01T00:00:07Z", ev_session_ended(Some("exit"))),
    ]);
    let replay = replay_of(&recording);
    let inspection = inspect(&replay);

    let agent_call = group(&inspection, CorrelationId::V2ToolUseId("toolu_agent"));
    assert_eq!(agent_call.shape, GroupShape::PairedLifecycle);
    assert_eq!(
        agent_call.paired_interval,
        Some(SequenceInterval {
            opening: 2,
            outcome: 7
        }),
        "two canonical positions, not a span and not a containment"
    );

    // Nothing derived links the inner group or the child agent to the Agent
    // call. The only relationships in the projection are correlation ids.
    let inner = group(&inspection, CorrelationId::V2ToolUseId("toolu_inner"));
    assert_eq!(inner.shape, GroupShape::PairedLifecycle);
    assert!(inspection.subagents[0].supplied_parents.is_empty());
    assert!(!anomaly_kinds(&inspection).iter().any(|kind| matches!(
        kind,
        AnomalyKind::SubagentStartWithoutStop { .. } | AnomalyKind::SubagentStopWithoutStart { .. }
    )));

    // The `Agent` call's own records carry no agent identity, and the inner
    // records carry the child's. Neither is derived from the other.
    assert_eq!(
        inspection.ledger[1].current_agent,
        AgentAttribution::NotSupplied { agent_type: None }
    );
    assert_eq!(
        inspection.ledger[3].current_agent,
        AgentAttribution::Supplied {
            agent_id: "agent-child",
            agent_type: Some("Plan")
        }
    );
}

// ---------------------------------------------------------------------------
// Session and subagent boundaries
// ---------------------------------------------------------------------------

#[test]
fn missing_session_boundaries_are_reported_with_the_scope_they_were_missed_in() {
    let recording = v2_recording(vec![row(
        "2026-01-01T00:00:00Z",
        ev_tool_requested("toolu_a", "SyntheticTool"),
    )]);
    let replay = replay_of(&recording);
    let inspection = inspect(&replay);

    assert!(inspection.session_boundaries.started.is_absent());
    assert!(inspection.session_boundaries.ended.is_absent());
    assert_eq!(
        inspection.session_boundaries.started.scope,
        ExaminedScope::CompleteRecording { records: 1 }
    );

    let missing: Vec<_> = inspection
        .anomalies
        .iter()
        .filter(|anomaly| {
            matches!(
                anomaly.kind,
                AnomalyKind::MissingSessionStart | AnomalyKind::MissingSessionEnd
            )
        })
        .collect();
    assert_eq!(missing.len(), 2);
    for anomaly in missing {
        assert!(anomaly.receipts.is_empty());
        assert_eq!(
            anomaly.scope,
            ExaminedScope::CompleteRecording { records: 1 },
            "an absence carries what was searched"
        );
    }
}

#[test]
fn duplicate_session_boundaries_are_reported_with_every_receipt() {
    let recording = v2_recording(vec![
        row("2026-01-01T00:00:00Z", ev_session_started(Some("startup"))),
        row("2026-01-01T00:00:01Z", ev_session_started(Some("resume"))),
        row("2026-01-01T00:00:02Z", ev_session_ended(Some("exit"))),
        row("2026-01-01T00:00:03Z", ev_session_ended(Some("clear"))),
    ]);
    let replay = replay_of(&recording);
    let inspection = inspect(&replay);

    assert_eq!(
        inspection.session_boundaries.started.records.sequences(),
        [1, 2]
    );
    assert_eq!(
        inspection.session_boundaries.ended.records.sequences(),
        [3, 4]
    );
    let kinds = anomaly_kinds(&inspection);
    assert!(kinds.contains(&AnomalyKind::DuplicateSessionStart));
    assert!(kinds.contains(&AnomalyKind::DuplicateSessionEnd));
    // Both delivered sources survive on the records.
    assert_eq!(inspection.ledger[0].facets.session_source, Some("startup"));
    assert_eq!(inspection.ledger[1].facets.session_source, Some("resume"));
}

#[test]
fn an_unmatched_subagent_stop_and_an_unmatched_start_are_both_reported() {
    // task:4 measured a real `subagent_stopped` with no matching start.
    let recording = v2_recording(vec![
        row(
            "2026-01-01T00:00:00Z",
            ev_subagent_started("agent-open", Some("Plan"), None, None),
        ),
        row(
            "2026-01-01T00:00:01Z",
            ev_subagent_stopped("agent-orphan", Some(""), None, None),
        ),
    ]);
    let replay = replay_of(&recording);
    let inspection = inspect(&replay);

    assert_eq!(inspection.subagents.len(), 2);
    assert!(inspection.subagents[0].stopped.is_absent());
    assert!(inspection.subagents[1].started.is_absent());

    let start_without_stop = inspection
        .anomalies
        .iter()
        .find(|anomaly| {
            anomaly.kind
                == AnomalyKind::SubagentStartWithoutStop {
                    agent_id: "agent-open",
                }
        })
        .expect("an unmatched start should be reported");
    assert_eq!(start_without_stop.receipts.sequences(), [1]);

    let stop_without_start = inspection
        .anomalies
        .iter()
        .find(|anomaly| {
            anomaly.kind
                == AnomalyKind::SubagentStopWithoutStart {
                    agent_id: "agent-orphan",
                }
        })
        .expect("an unmatched stop should be reported");
    assert_eq!(stop_without_start.receipts.sequences(), [2]);
}

// ---------------------------------------------------------------------------
// Empty, truncated, and absent evidence
// ---------------------------------------------------------------------------

#[test]
fn an_empty_complete_recording_projects_without_inventing_a_vocabulary() {
    let replay = replay_of("");
    let inspection = inspect(&replay);

    assert_eq!(inspection.schema_version, None);
    assert_eq!(inspection.record_count(), 0);
    assert_eq!(inspection.session_id, None);
    assert_eq!(inspection.tail(), Tail::Complete);
    assert_eq!(
        inspection.scope,
        ExaminedScope::CompleteRecording { records: 0 }
    );
    assert!(inspection.ledger.is_empty());
    assert!(inspection.tool_groups.is_empty());
    assert!(inspection.timestamps.is_none());
    // No first record, so no schema version, so no vocabulary to enumerate.
    // Enumerating one would choose a schema the recording never declared.
    assert!(inspection.aggregates.by_event_kind.is_empty());
    // Channels are raw provenance and their vocabulary is fixed, so they are
    // still tallied — at zero.
    assert_eq!(inspection.aggregates.by_channel.len(), 3);
    for tally in &inspection.aggregates.by_channel {
        assert!(tally.records.is_absent());
    }
}

#[test]
fn a_truncated_recording_with_no_complete_record_projects_as_a_valid_prefix() {
    let replay = replay_of("{\"schema_version\":2,\"session_id\":\"sess-synth");
    let inspection = inspect(&replay);

    assert_eq!(inspection.schema_version, None);
    assert_eq!(inspection.record_count(), 0);
    assert!(inspection.scope.is_truncated());
    assert!(matches!(inspection.tail(), Tail::Truncated { .. }));
    match inspection.scope {
        ExaminedScope::ValidPrefix {
            records,
            fragment_byte_offset,
            fragment_bytes,
        } => {
            assert_eq!(records, 0);
            assert_eq!(fragment_byte_offset, 0);
            assert_eq!(fragment_bytes, 44);
        }
        other => panic!("expected a valid-prefix scope, got {other:?}"),
    }
}

#[test]
fn an_absence_in_a_truncated_recording_carries_that_it_is_a_valid_prefix() {
    let complete = v2_recording(vec![
        row("2026-01-01T00:00:00Z", ev_session_started(Some("startup"))),
        row(
            "2026-01-01T00:00:01Z",
            ev_tool_requested("toolu_a", "SyntheticTool"),
        ),
    ]);
    let truncated = format!("{complete}{{\"schema_version\":2,\"session");
    let replay = replay_of(&truncated);
    let inspection = inspect(&replay);

    assert_eq!(inspection.record_count(), 2);
    assert!(inspection.scope.is_truncated());

    // "No session_ended record" means something weaker here than in a complete
    // recording, and the scope is what says so.
    let missing_end = inspection
        .anomalies
        .iter()
        .find(|anomaly| anomaly.kind == AnomalyKind::MissingSessionEnd)
        .expect("a missing end should be reported");
    assert!(missing_end.scope.is_truncated());
    assert_eq!(missing_end.scope.records(), 2);

    // The same is true of the unresolved request.
    let open_request = inspection
        .anomalies
        .iter()
        .find(|anomaly| {
            anomaly.kind
                == AnomalyKind::OpeningWithoutOutcome {
                    id: CorrelationId::V2ToolUseId("toolu_a"),
                }
        })
        .expect("an unresolved request should be reported");
    assert!(open_request.scope.is_truncated());

    // Zero counts across the vocabulary carry the same scope.
    for tally in &inspection.aggregates.by_event_kind {
        assert!(tally.records.scope.is_truncated());
    }
}

#[test]
fn a_truncated_v1_recording_also_projects_as_a_valid_prefix() {
    let complete = v1_recording(vec![
        ("2026-01-01T00:00:00Z", v1::Event::SessionStarted),
        (
            "2026-01-01T00:00:01Z",
            ev1_tool_started("call-a", "SyntheticTool"),
        ),
    ]);
    let truncated = format!("{complete}{{\"schema_version\":1");
    let replay = replay_of(&truncated);
    let inspection = inspect(&replay);

    assert_eq!(inspection.schema_version, Some(1));
    assert!(inspection.scope.is_truncated());
    assert_eq!(inspection.aggregates.by_event_kind.len(), V1Kind::ALL.len());
    let group = group(&inspection, CorrelationId::V1ToolCallId("call-a"));
    assert_eq!(group.shape, GroupShape::OpeningWithoutOutcome);
    assert!(group.scope.is_truncated());
}

#[test]
fn a_kind_the_recording_contains_none_of_is_a_zero_count_with_a_scope() {
    // task:4's clean session: no failure record, no denial record. That is a
    // statement about records observed, and the scope is what makes it one.
    let recording = v2_recording(vec![
        row("2026-01-01T00:00:00Z", ev_session_started(Some("startup"))),
        row(
            "2026-01-01T00:00:01Z",
            ev_tool_requested("toolu_a", "SyntheticTool"),
        ),
        row(
            "2026-01-01T00:00:02Z",
            ev_tool_succeeded("toolu_a", "SyntheticTool", None),
        ),
        row("2026-01-01T00:00:03Z", ev_session_ended(Some("exit"))),
    ]);
    let replay = replay_of(&recording);
    let inspection = inspect(&replay);

    assert_eq!(
        inspection.aggregates.by_event_kind.len(),
        V2Kind::ALL.len(),
        "the whole vocabulary is tallied, including kinds with no records"
    );
    assert_eq!(
        kind_count(&inspection, EventKind::V2(V2Kind::ToolFailed)),
        0
    );
    assert_eq!(
        kind_count(&inspection, EventKind::V2(V2Kind::ToolDenied)),
        0
    );
    assert_eq!(
        kind_count(&inspection, EventKind::V2(V2Kind::ToolSucceeded)),
        1
    );
    for tally in &inspection.aggregates.by_event_kind {
        assert_eq!(
            tally.records.scope,
            ExaminedScope::CompleteRecording { records: 4 }
        );
    }
}

#[test]
fn absent_duration_is_coverage_rather_than_a_zero_duration() {
    // task:4 measured zero `duration_ms` across 82 completions. Absent means the
    // integration did not supply one.
    let recording = v2_recording(vec![
        row(
            "2026-01-01T00:00:00Z",
            ev_tool_succeeded("toolu_a", "SyntheticTool", None),
        ),
        row(
            "2026-01-01T00:00:01Z",
            ev_tool_succeeded("toolu_b", "SyntheticTool", Some(12)),
        ),
    ]);
    let replay = replay_of(&recording);
    let inspection = inspect(&replay);

    let coverage = inspection
        .coverage
        .iter()
        .find(|coverage| coverage.field == CoveredField::V2DurationMs)
        .expect("duration coverage should be summarized");
    assert_eq!(coverage.population.records.sequences(), [1, 2]);
    assert_eq!(coverage.absent.records.sequences(), [1]);
    assert_eq!(coverage.present.records.sequences(), [2]);
    assert_eq!(inspection.ledger[0].facets.duration_ms, None);
    assert_eq!(inspection.ledger[1].facets.duration_ms, Some(12));

    // No group carries a duration, and the paired interval is two positions.
    let group = group(&inspection, CorrelationId::V2ToolUseId("toolu_a"));
    assert_eq!(group.paired_interval, None);
}

#[test]
fn prompt_id_survives_as_context_with_a_presence_count_and_groups_nothing() {
    // dragon:3 is open: prompt_id delimits no unit of work, so it may be counted
    // and carried, and may not define a group.
    let recording = v2_recording(vec![
        row_ctx(
            "2026-01-01T00:00:00Z",
            Context::default(),
            ev_session_started(Some("startup")),
        ),
        row_ctx(
            "2026-01-01T00:00:01Z",
            context(Some("prompt-1"), None, None),
            ev_tool_requested("toolu_a", "SyntheticTool"),
        ),
        row_ctx(
            "2026-01-01T00:00:02Z",
            context(Some("prompt-2"), None, None),
            ev_tool_succeeded("toolu_a", "SyntheticTool", None),
        ),
    ]);
    let replay = replay_of(&recording);
    let inspection = inspect(&replay);

    let coverage = inspection
        .coverage
        .iter()
        .find(|coverage| coverage.field == CoveredField::V2PromptId)
        .expect("prompt_id coverage should be summarized");
    assert_eq!(coverage.population.records.sequences(), [1, 2, 3]);
    assert_eq!(coverage.present.records.sequences(), [2, 3]);
    assert_eq!(coverage.absent.records.sequences(), [1]);

    assert_eq!(inspection.ledger[1].prompt_id, Some("prompt-1"));
    assert_eq!(inspection.ledger[2].prompt_id, Some("prompt-2"));

    // Two prompt ids did not split the recording into two of anything.
    assert_eq!(inspection.tool_groups.len(), 1);
    assert_eq!(
        group(&inspection, CorrelationId::V2ToolUseId("toolu_a")).shape,
        GroupShape::PairedLifecycle
    );
}

// ---------------------------------------------------------------------------
// Order
// ---------------------------------------------------------------------------

#[test]
fn equal_and_backward_timestamps_leave_append_order_intact() {
    let recording = v2_recording(vec![
        row("2026-01-01T00:00:05Z", ev_session_started(Some("startup"))),
        row(
            "2026-01-01T00:00:05Z",
            ev_tool_requested("toolu_a", "SyntheticTool"),
        ),
        // The recorder's clock moves backwards. Order is unaffected.
        row(
            "2026-01-01T00:00:01Z",
            ev_tool_succeeded("toolu_a", "SyntheticTool", None),
        ),
        row("2026-01-01T00:00:09Z", ev_session_ended(Some("exit"))),
    ]);
    let replay = replay_of(&recording);
    let inspection = inspect(&replay);

    let ledger_order: Vec<u64> = inspection
        .ledger
        .iter()
        .map(|entry| entry.sequence)
        .collect();
    assert_eq!(ledger_order, [1, 2, 3, 4]);
    assert_eq!(
        inspection.ledger.iter().map(|e| e.kind).collect::<Vec<_>>(),
        [
            EventKind::V2(V2Kind::SessionStarted),
            EventKind::V2(V2Kind::ToolRequested),
            EventKind::V2(V2Kind::ToolSucceeded),
            EventKind::V2(V2Kind::SessionEnded),
        ]
    );

    let timestamps = inspection
        .timestamps
        .as_ref()
        .expect("a recording with records has extrema");
    // The earliest timestamp is at sequence 3, which says nothing about order.
    assert_eq!(timestamps.earliest.sequence, 3);
    assert_eq!(timestamps.latest.sequence, 4);
    assert_eq!(timestamps.non_monotonic.records.sequences(), [3]);

    // The pair is still request-then-outcome in the append chain.
    let group = group(&inspection, CorrelationId::V2ToolUseId("toolu_a"));
    assert_eq!(
        group.paired_interval,
        Some(SequenceInterval {
            opening: 2,
            outcome: 3
        })
    );
}

#[test]
fn a_tie_in_the_extrema_cites_the_earliest_record_in_canonical_order() {
    let recording = v2_recording(vec![
        row("2026-01-01T00:00:00Z", ev_session_started(Some("startup"))),
        row("2026-01-01T00:00:00Z", ev_session_ended(Some("exit"))),
    ]);
    let replay = replay_of(&recording);
    let inspection = inspect(&replay);

    let timestamps = inspection.timestamps.as_ref().expect("extrema exist");
    assert_eq!(timestamps.earliest.sequence, 1);
    assert_eq!(timestamps.latest.sequence, 1);
    assert!(timestamps.non_monotonic.is_absent());
}

#[test]
fn projection_output_is_deterministic() {
    let recording = v2_recording(vec![
        row("2026-01-01T00:00:00Z", ev_session_started(Some("startup"))),
        row_ctx(
            "2026-01-01T00:00:01Z",
            context(Some("prompt-1"), Some("agent-b"), Some("Plan")),
            ev_tool_requested("toolu_b", "SyntheticTool"),
        ),
        row_ctx(
            "2026-01-01T00:00:02Z",
            context(Some("prompt-1"), Some("agent-a"), Some("Explore")),
            ev_tool_requested("toolu_a", "OtherTool"),
        ),
        row_ctx(
            "2026-01-01T00:00:03Z",
            context(Some("prompt-1"), Some("agent-a"), Some("Explore")),
            ev_tool_succeeded("toolu_a", "OtherTool", None),
        ),
        row(
            "2026-01-01T00:00:04Z",
            ev_subagent_stopped("agent-z", None, None, None),
        ),
        row(
            "2026-01-01T00:00:05Z",
            ev_subagent_started("agent-a", Some("Explore"), None, None),
        ),
    ]);
    let replay = replay_of(&recording);

    let first = serde_json::to_string(&inspect(&replay)).expect("projection should serialize");
    let second = serde_json::to_string(&inspect(&replay)).expect("projection should serialize");
    assert_eq!(first, second);

    let inspection = inspect(&replay);
    // Groups, subagents, and anomalies are all in canonical record order.
    let group_order: Vec<u64> = inspection
        .tool_groups
        .iter()
        .map(|group| group.first_sequence)
        .collect();
    assert_eq!(group_order, [2, 3]);
    let subagent_order: Vec<&str> = inspection
        .subagents
        .iter()
        .map(|subagent| subagent.agent_id)
        .collect();
    assert_eq!(subagent_order, ["agent-z", "agent-a"]);
    // Agents in first-appearance order, not sorted by id.
    let agent_order: Vec<&str> = inspection
        .current_agents
        .supplied
        .iter()
        .map(|tally| tally.value)
        .collect();
    assert_eq!(agent_order, ["agent-b", "agent-a"]);

    let anomaly_receipts: Vec<Option<u64>> = inspection
        .anomalies
        .iter()
        .map(|anomaly| anomaly.receipts.first())
        .collect();
    let mut sorted = anomaly_receipts.clone();
    sorted.sort_by_key(|first| first.unwrap_or(u64::MAX));
    assert_eq!(anomaly_receipts, sorted);
}

// ---------------------------------------------------------------------------
// The projection's own contract
// ---------------------------------------------------------------------------

#[test]
fn every_receipt_refers_to_a_real_raw_sequence() {
    let recording = v2_recording(vec![
        row("2026-01-01T00:00:00Z", ev_session_started(Some("startup"))),
        row("2026-01-01T00:00:01Z", ev_session_started(Some("resume"))),
        row_ctx(
            "2026-01-01T00:00:02Z",
            context(Some("prompt-1"), Some("agent-a"), Some("Plan")),
            ev_tool_requested("toolu_a", "SyntheticTool"),
        ),
        row_ctx(
            "2026-01-01T00:00:03Z",
            context(Some("prompt-1"), Some("agent-a"), Some("Explore")),
            ev_tool_succeeded("toolu_a", "OtherTool", None),
        ),
        row(
            "2026-01-01T00:00:04Z",
            ev_tool_failed("toolu_a", "SyntheticTool", Some(false)),
        ),
        row(
            "2026-01-01T00:00:05Z",
            ev_reported_intent("a claim about nothing observed", Some("toolu_ghost")),
        ),
        row(
            "2026-01-01T00:00:06Z",
            ev_subagent_stopped("agent-orphan", None, None, None),
        ),
        row(
            "2026-01-01T00:00:07Z",
            ev_tool_denied("toolu_denied", "SyntheticTool"),
        ),
    ]);
    let replay = replay_of(&recording);
    let inspection = inspect(&replay);

    let valid: Vec<u64> = inspection
        .records
        .iter()
        .map(witnessglass::AnyRecord::sequence)
        .collect();
    assert_eq!(valid, (1..=8).collect::<Vec<u64>>());

    let receipts = all_receipts(&inspection);
    assert!(
        !receipts.is_empty(),
        "this recording should produce receipts"
    );
    for receipt in receipts {
        assert!(
            valid.contains(&receipt),
            "receipt {receipt} does not name a record in this recording"
        );
    }

    // And the recording did produce the anomalies it was built to produce.
    let kinds = anomaly_kinds(&inspection);
    assert!(kinds.contains(&AnomalyKind::DuplicateSessionStart));
    assert!(kinds.contains(&AnomalyKind::MissingSessionEnd));
    assert!(kinds.contains(&AnomalyKind::ConflictingOutcomes {
        id: CorrelationId::V2ToolUseId("toolu_a")
    }));
    assert!(kinds.contains(&AnomalyKind::DivergentToolNames {
        id: CorrelationId::V2ToolUseId("toolu_a")
    }));
    assert!(kinds.contains(&AnomalyKind::DivergentAgentTypes {
        agent_id: "agent-a"
    }));
    assert!(
        kinds.contains(&AnomalyKind::ReportedIntentWithoutObservedEvidence {
            id: CorrelationId::V2ToolUseId("toolu_ghost")
        })
    );
    assert!(kinds.contains(&AnomalyKind::SubagentStopWithoutStart {
        agent_id: "agent-orphan"
    }));
    assert!(kinds.contains(&AnomalyKind::OutcomeWithoutOpening {
        id: CorrelationId::V2ToolUseId("toolu_denied")
    }));
}

#[test]
fn projecting_does_not_mutate_the_replay_or_its_records() {
    let recording = v2_recording(vec![
        row("2026-01-01T00:00:00Z", ev_session_started(Some("startup"))),
        row(
            "2026-01-01T00:00:01Z",
            ev_tool_requested("toolu_a", "SyntheticTool"),
        ),
        row(
            "2026-01-01T00:00:02Z",
            ev_reported_intent("a claim", Some("toolu_a")),
        ),
        row(
            "2026-01-01T00:00:03Z",
            ev_tool_succeeded("toolu_a", "SyntheticTool", None),
        ),
        row("2026-01-01T00:00:04Z", ev_session_ended(Some("exit"))),
    ]);
    let replay = replay_of(&recording);
    let before = replay.clone();

    let inspection = inspect(&replay);

    assert_eq!(replay, before, "the replay is untouched");
    // The projection borrows the records; it does not hold copies it could
    // diverge from.
    assert_eq!(inspection.records, before.records.as_slice());
    assert_eq!(
        inspection
            .ledger
            .iter()
            .map(|entry| entry.record)
            .collect::<Vec<_>>(),
        before.records.iter().collect::<Vec<_>>()
    );

    // The projection is disposable: dropping it changes nothing.
    drop(inspection);
    assert_eq!(replay, before);

    // And re-rendering the raw records still produces the original recording.
    let rendered = ndjson(
        &replay
            .records
            .iter()
            .map(|record| {
                record
                    .as_v2()
                    .expect("a v2 recording holds v2 records")
                    .clone()
            })
            .collect::<Vec<_>>(),
    );
    assert_eq!(rendered, recording);
}

#[test]
fn aggregates_describe_records_by_raw_provenance_and_delivered_metadata() {
    let recording = v2_recording(vec![
        row("2026-01-01T00:00:00Z", ev_session_started(Some("startup"))),
        row(
            "2026-01-01T00:00:01Z",
            ev_tool_requested("toolu_a", "SyntheticTool"),
        ),
        row(
            "2026-01-01T00:00:02Z",
            ev_reported_intent("a claim", Some("toolu_a")),
        ),
        row(
            "2026-01-01T00:00:03Z",
            ev_tool_succeeded("toolu_a", "SyntheticTool", None),
        ),
    ]);
    let replay = replay_of(&recording);
    let inspection = inspect(&replay);

    let channels: Vec<(witnessglass::Channel, usize)> = inspection
        .aggregates
        .by_channel
        .iter()
        .map(|tally| (tally.value, tally.records.count()))
        .collect();
    assert_eq!(
        channels,
        vec![
            (witnessglass::Channel::Reported, 1),
            (witnessglass::Channel::Observed, 2),
            (witnessglass::Channel::Recorder, 1),
        ]
    );

    assert_eq!(inspection.aggregates.by_adapter.len(), 1);
    assert_eq!(inspection.aggregates.by_adapter[0].value, ADAPTER);
    assert_eq!(inspection.aggregates.by_adapter[0].records.count(), 4);
    assert_eq!(inspection.aggregates.by_mechanism.len(), 1);
    assert_eq!(inspection.aggregates.by_mechanism[0].value, MECHANISM);

    assert_eq!(inspection.session_id, Some(SESSION));
    assert_eq!(inspection.schema_version, Some(2));
}
