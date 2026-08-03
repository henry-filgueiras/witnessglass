//! The passive Claude Code command-hook adapter.
//!
//! Every payload here is synthetic. No real Claude process is invoked, no real
//! session id is used, and no recording leaves a temporary directory. First
//! contact with a live session is task:4; these tests establish only that the
//! translation is honest about what a hook payload does and does not prove.

mod common;

use std::collections::BTreeSet;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Output, Stdio};

use common::*;
use witnessglass::claude::{self, HookError};
use witnessglass::{Channel, Event, Tail, replay_file};

const HOOK_SESSION: &str = "e7c1a0f2-0000-4000-8000-synthetic0001";

/// Run the adapter as Claude would: a short-lived process, one payload on
/// stdin, nothing else.
fn run_hook(dir: &Path, payload: &str) -> Output {
    run_hook_with(dir, payload, &[], &[])
}

/// As above, with extra arguments and environment. Claude spawns a hook from a
/// settings file, so both are ways a real deployment configures one.
fn run_hook_with(dir: &Path, payload: &str, args: &[&str], env: &[(&str, &str)]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_witnessglass"));
    command
        .arg("claude-hook")
        .arg("--recordings-dir")
        .arg(dir)
        .args(args);
    // Cleared so an operator's own setting cannot decide a test's outcome.
    command.env_remove("WITNESSGLASS_STRICT_JSON");
    for (key, value) in env {
        command.env(key, value);
    }
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn adapter");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(payload.as_bytes())
        .expect("write payload");
    child.wait_with_output().expect("wait")
}

/// Translate a payload, expecting success.
fn translate(payload: serde_json::Value) -> claude::Translation {
    claude::translate(&payload.to_string()).expect("payload should translate")
}

/// The single emission a payload produced.
fn only_event(payload: serde_json::Value) -> Event {
    let translation = translate(payload);
    assert_eq!(
        translation.emissions.len(),
        1,
        "expected exactly one emission"
    );
    translation.emissions[0].event.clone()
}

fn pre_tool_use(description: Option<&str>) -> serde_json::Value {
    let mut input = serde_json::json!({ "command": "echo synthetic", "timeout": 120000 });
    if let Some(text) = description {
        input["description"] = serde_json::json!(text);
    }
    serde_json::json!({
        "hook_event_name": "PreToolUse",
        "session_id": HOOK_SESSION,
        "prompt_id": "prompt-synthetic-0001",
        "tool_use_id": "toolu_synthetic_0001",
        "tool_name": "Bash",
        "tool_input": input,
    })
}

// ---------------------------------------------------------------------------
// Every supported hook surface
// ---------------------------------------------------------------------------

#[test]
fn session_start_records_the_documented_source() {
    let event = only_event(serde_json::json!({
        "hook_event_name": "SessionStart",
        "session_id": HOOK_SESSION,
        "source": "startup",
    }));
    let Event::SessionStarted(started) = event else {
        panic!("expected session_started");
    };
    assert_eq!(started.source.as_deref(), Some("startup"));
}

#[test]
fn session_end_records_the_documented_reason() {
    let event = only_event(serde_json::json!({
        "hook_event_name": "SessionEnd",
        "session_id": HOOK_SESSION,
        "reason": "prompt_input_exit",
    }));
    let Event::SessionEnded(ended) = event else {
        panic!("expected session_ended");
    };
    assert_eq!(ended.reason.as_deref(), Some("prompt_input_exit"));
}

#[test]
fn pre_tool_use_records_a_request_not_an_execution() {
    // The load-bearing mapping. Claude documents this hook as firing after the
    // model constructs a request and before the call is processed, and the
    // request may still be modified, denied, deferred, or never executed. The
    // record must say "requested" and nothing stronger.
    let translation = translate(pre_tool_use(None));
    assert_eq!(translation.emissions.len(), 1);

    let Event::ToolRequested(requested) = &translation.emissions[0].event else {
        panic!("expected tool_requested");
    };
    assert_eq!(requested.tool_use_id, "toolu_synthetic_0001");
    assert_eq!(requested.tool_name, "Bash");
    assert_eq!(requested.requested_input["command"], "echo synthetic");

    assert_eq!(
        translation.emissions[0].provenance.mechanism,
        "command-hook:PreToolUse"
    );
    assert_eq!(translation.emissions[0].event.kind(), "tool_requested");
}

#[test]
fn post_tool_use_records_effective_input_and_response() {
    let event = only_event(serde_json::json!({
        "hook_event_name": "PostToolUse",
        "session_id": HOOK_SESSION,
        "tool_use_id": "toolu_synthetic_0001",
        "tool_name": "Bash",
        "tool_input": { "command": "echo synthetic-effective" },
        "tool_response": { "content": [{ "type": "text", "text": "synthetic output" }] },
        "duration_ms": 5000,
    }));
    let Event::ToolSucceeded(succeeded) = event else {
        panic!("expected tool_succeeded");
    };
    assert_eq!(
        succeeded.effective_input["command"],
        "echo synthetic-effective"
    );
    assert_eq!(succeeded.response["content"][0]["text"], "synthetic output");
    assert_eq!(succeeded.duration_ms, Some(5000));
}

#[test]
fn post_tool_use_failure_records_error_and_interruption() {
    let event = only_event(serde_json::json!({
        "hook_event_name": "PostToolUseFailure",
        "session_id": HOOK_SESSION,
        "tool_use_id": "toolu_synthetic_0001",
        "tool_name": "Bash",
        "tool_input": { "command": "echo synthetic" },
        "error": "Command failed with exit code 1",
        "duration_ms": 3000,
        "is_interrupt": false,
    }));
    let Event::ToolFailed(failed) = event else {
        panic!("expected tool_failed");
    };
    assert_eq!(failed.error, "Command failed with exit code 1");
    assert_eq!(failed.interrupted, Some(false));
    assert_eq!(failed.duration_ms, Some(3000));
}

#[test]
fn timing_and_interruption_are_read_from_the_wire_names_not_the_documented_ones() {
    // The hooks reference documents `duration` and `interrupted`. Observed
    // payloads from Claude Code 2.1.220 carry `duration_ms` and `is_interrupt`
    // instead, on every completion. The adapter read the documented names from
    // the day it was written, ignored the real ones as unknown fields, and
    // dragon:1 recorded the resulting emptiness as an integration coverage gap
    // for two sprints.
    //
    // This test pins the spelling that was actually observed. If the
    // integration ever sends the documented names as well, that is a new
    // observation to record — not a reason to quietly swap these back.
    let event = only_event(serde_json::json!({
        "hook_event_name": "PostToolUseFailure",
        "session_id": HOOK_SESSION,
        "tool_use_id": "toolu_synthetic_0001",
        "tool_name": "Bash",
        "tool_input": { "command": "echo synthetic" },
        "error": "synthetic failure",
        "duration": 111,
        "interrupted": true,
    }));
    let Event::ToolFailed(failed) = event else {
        panic!("expected tool_failed");
    };
    assert_eq!(
        failed.duration_ms, None,
        "`duration` is not the delivered key and must not populate duration_ms"
    );
    assert_eq!(
        failed.interrupted, None,
        "`interrupted` is not the delivered key and must not populate interrupted"
    );
}

#[test]
fn permission_denied_records_a_denial() {
    let event = only_event(serde_json::json!({
        "hook_event_name": "PermissionDenied",
        "session_id": HOOK_SESSION,
        "tool_use_id": "toolu_synthetic_0001",
        "tool_name": "Bash",
        "tool_input": { "command": "echo synthetic-denied" },
    }));
    let Event::ToolDenied(denied) = event else {
        panic!("expected tool_denied");
    };
    assert_eq!(denied.requested_input["command"], "echo synthetic-denied");
}

#[test]
fn subagent_start_and_stop_are_recorded() {
    let started = only_event(serde_json::json!({
        "hook_event_name": "SubagentStart",
        "session_id": HOOK_SESSION,
        "agent_id": "subagent-synthetic-0001",
        "agent_type": "SyntheticExplorer",
    }));
    let Event::SubagentStarted(started) = started else {
        panic!("expected subagent_started");
    };
    assert_eq!(started.agent_id, "subagent-synthetic-0001");
    assert_eq!(started.agent_type.as_deref(), Some("SyntheticExplorer"));

    let stopped = only_event(serde_json::json!({
        "hook_event_name": "SubagentStop",
        "session_id": HOOK_SESSION,
        "agent_id": "subagent-synthetic-0001",
        "agent_type": "SyntheticExplorer",
    }));
    let Event::SubagentStopped(stopped) = stopped else {
        panic!("expected subagent_stopped");
    };
    assert_eq!(stopped.agent_id, "subagent-synthetic-0001");
}

// ---------------------------------------------------------------------------
// The distinctions v2 exists to preserve
// ---------------------------------------------------------------------------

#[test]
fn requested_input_stays_distinct_from_effective_input() {
    // Claude documents that a request may be modified before execution. If the
    // adapter filed both under one field, a recording could never show that
    // what ran was not what was asked for.
    let dir = tempfile::tempdir().expect("temp dir");

    let requested = run_hook(dir.path(), &pre_tool_use(None).to_string());
    assert!(requested.status.success());

    let effective = run_hook(
        dir.path(),
        &serde_json::json!({
            "hook_event_name": "PostToolUse",
            "session_id": HOOK_SESSION,
            "tool_use_id": "toolu_synthetic_0001",
            "tool_name": "Bash",
            "tool_input": { "command": "echo modified-before-execution" },
            "tool_response": { "content": [] },
        })
        .to_string(),
    );
    assert!(effective.status.success());

    let replay = replay_file(&dir.path().join(format!("{HOOK_SESSION}.ndjson"))).expect("replay");
    assert_eq!(
        kinds(&replay.records),
        vec!["tool_requested", "tool_succeeded"]
    );

    let Event::ToolRequested(request) = v2_event(&replay.records[0]) else {
        panic!("expected a request");
    };
    let Event::ToolSucceeded(success) = v2_event(&replay.records[1]) else {
        panic!("expected a success");
    };
    assert_eq!(request.requested_input["command"], "echo synthetic");
    assert_eq!(
        success.effective_input["command"],
        "echo modified-before-execution"
    );
    assert_ne!(request.requested_input, success.effective_input);
    assert_eq!(request.tool_use_id, success.tool_use_id);
}

#[test]
fn an_explicit_description_becomes_reported_intent_not_an_observation() {
    let translation = translate(pre_tool_use(Some("Run the synthetic check")));
    assert_eq!(translation.emissions.len(), 2);

    // The observation stays an observation, on the observed channel.
    assert_eq!(
        translation.emissions[0].provenance.channel,
        Channel::Observed
    );
    assert_eq!(translation.emissions[0].event.kind(), "tool_requested");

    // The agent's own words become a reported claim, correlated by id and
    // classified as what they are.
    let intent = &translation.emissions[1];
    assert_eq!(intent.provenance.channel, Channel::Reported);
    assert_eq!(
        intent.provenance.mechanism,
        "command-hook:PreToolUse#tool_input.description"
    );
    let Event::ReportedIntent(reported) = &intent.event else {
        panic!("expected reported_intent");
    };
    assert_eq!(reported.text, "Run the synthetic check");
    assert_eq!(
        reported.tool_use_id.as_deref(),
        Some("toolu_synthetic_0001")
    );

    // The description is duplicated, not moved: the requested input is still
    // preserved whole as source-delivered evidence.
    let Event::ToolRequested(requested) = &translation.emissions[0].event else {
        panic!("expected tool_requested");
    };
    assert_eq!(
        requested.requested_input["description"],
        "Run the synthetic check"
    );
}

#[test]
fn intent_is_never_manufactured_from_a_command_or_a_tool_name() {
    // No description field, so nothing the agent said about itself exists. The
    // command is not a statement of intent and must not become one.
    let translation = translate(pre_tool_use(None));
    assert_eq!(translation.emissions.len(), 1);
    assert!(
        translation
            .emissions
            .iter()
            .all(|e| e.provenance.channel != Channel::Reported)
    );

    // A blank description is not a claim either.
    let blank = translate(pre_tool_use(Some("   ")));
    assert_eq!(blank.emissions.len(), 1);
}

#[test]
fn success_failure_and_denial_are_three_different_records() {
    let dir = tempfile::tempdir().expect("temp dir");
    let session = "three-outcomes-synthetic";

    for payload in [
        serde_json::json!({
            "hook_event_name": "PostToolUse",
            "session_id": session,
            "tool_use_id": "toolu_a",
            "tool_name": "Bash",
            "tool_input": { "command": "true" },
            "tool_response": { "content": [] },
        }),
        serde_json::json!({
            "hook_event_name": "PostToolUseFailure",
            "session_id": session,
            "tool_use_id": "toolu_b",
            "tool_name": "Bash",
            "tool_input": { "command": "false" },
            "error": "exit 1",
        }),
        serde_json::json!({
            "hook_event_name": "PermissionDenied",
            "session_id": session,
            "tool_use_id": "toolu_c",
            "tool_name": "Bash",
            "tool_input": { "command": "rm -rf /synthetic" },
        }),
    ] {
        assert!(run_hook(dir.path(), &payload.to_string()).status.success());
    }

    let replay = replay_file(&dir.path().join(format!("{session}.ndjson"))).expect("replay");
    assert_eq!(
        kinds(&replay.records),
        vec!["tool_succeeded", "tool_failed", "tool_denied"]
    );

    // A denial is not a failure. The denied call never ran, and no error field
    // was invented for it.
    let denied = serde_json::to_string(&replay.records[2]).expect("serialize");
    assert!(
        !denied.contains("\"error\""),
        "denial invented an error: {denied}"
    );
    assert!(
        !denied.contains("effective_input"),
        "denial claimed execution: {denied}"
    );
    assert!(denied.contains("requested_input"));
}

#[test]
fn a_completion_without_a_captured_request_is_representable() {
    // Validation rejections can fire neither the pre hook nor the failure hook,
    // and a hook process can simply die. A completion arriving with no recorded
    // request is a capture blind spot to record, not an error to reject.
    let dir = tempfile::tempdir().expect("temp dir");
    let output = run_hook(
        dir.path(),
        &serde_json::json!({
            "hook_event_name": "PostToolUse",
            "session_id": HOOK_SESSION,
            "tool_use_id": "toolu_never_seen_before",
            "tool_name": "Bash",
            "tool_input": { "command": "echo synthetic" },
            "tool_response": { "content": [] },
        })
        .to_string(),
    );
    assert!(output.status.success());

    let replay = replay_file(&dir.path().join(format!("{HOOK_SESSION}.ndjson"))).expect("replay");
    assert_eq!(kinds(&replay.records), vec!["tool_succeeded"]);
}

// ---------------------------------------------------------------------------
// Optional context: absent means absent
// ---------------------------------------------------------------------------

#[test]
fn supplied_context_identifiers_are_preserved() {
    let translation = translate(serde_json::json!({
        "hook_event_name": "PostToolUse",
        "session_id": HOOK_SESSION,
        "prompt_id": "prompt-synthetic-0007",
        "agent_id": "agent-synthetic-0007",
        "agent_type": "SyntheticExplorer",
        "tool_use_id": "toolu_synthetic_0001",
        "tool_name": "Bash",
        "tool_input": { "command": "echo synthetic" },
        "tool_response": { "content": [] },
    }));
    let context = &translation.emissions[0].context;
    assert_eq!(context.prompt_id.as_deref(), Some("prompt-synthetic-0007"));
    assert_eq!(context.agent_id.as_deref(), Some("agent-synthetic-0007"));
    assert_eq!(context.agent_type.as_deref(), Some("SyntheticExplorer"));
}

#[test]
fn absent_optional_fields_stay_absent_rather_than_being_defaulted() {
    // prompt_id is documented as absent until the first input, agent fields
    // only exist inside subagents, and duration and interrupted are optional.
    // A default value here would be a fact nobody supplied.
    let translation = translate(serde_json::json!({
        "hook_event_name": "PostToolUseFailure",
        "session_id": HOOK_SESSION,
        "tool_use_id": "toolu_synthetic_0001",
        "tool_name": "Bash",
        "tool_input": { "command": "echo synthetic" },
        "error": "synthetic failure",
    }));

    let emission = &translation.emissions[0];
    assert!(emission.context.is_empty());
    let Event::ToolFailed(failed) = &emission.event else {
        panic!("expected tool_failed");
    };
    assert_eq!(failed.interrupted, None);
    assert_eq!(failed.duration_ms, None);

    // Absent in the serialized record too, not present-and-null. `false` and
    // "not stated" are different claims about whether a call was interrupted.
    let line = serde_json::to_string(emission).expect("serialize");
    assert!(!line.contains("interrupted"), "{line}");
    assert!(!line.contains("duration_ms"), "{line}");
    assert!(!line.contains("prompt_id"), "{line}");
    assert!(!line.contains("context"), "{line}");
}

#[test]
fn no_root_or_parent_agent_identity_is_invented() {
    // Nothing supplies a parent here, so nothing may claim one — not from the
    // session id, not from a default, and above all not from the fact that
    // another record happens to sit next to this one in the recording.
    let dir = tempfile::tempdir().expect("temp dir");
    let session = "no-invented-parent-synthetic";

    for payload in [
        serde_json::json!({
            "hook_event_name": "SessionStart",
            "session_id": session,
            "source": "startup",
        }),
        serde_json::json!({
            "hook_event_name": "SubagentStart",
            "session_id": session,
            "agent_id": "subagent-synthetic-0001",
            "agent_type": "SyntheticExplorer",
        }),
    ] {
        assert!(run_hook(dir.path(), &payload.to_string()).status.success());
    }

    let recording =
        std::fs::read_to_string(dir.path().join(format!("{session}.ndjson"))).expect("read");
    assert!(!recording.contains("parent_agent_id"), "{recording}");
    assert!(!recording.contains("parent_agent_type"), "{recording}");
    assert!(!recording.contains("root_agent"), "{recording}");
    assert!(!recording.contains("span"), "{recording}");
}

#[test]
fn a_supplied_parent_identifier_is_recorded_as_delivered() {
    // The other half of the same rule. Parentage is never inferred, and it is
    // never discarded either: where the integration states it, it is evidence
    // and it is kept.
    let event = only_event(serde_json::json!({
        "hook_event_name": "SubagentStart",
        "session_id": HOOK_SESSION,
        "agent_id": "subagent-synthetic-child",
        "agent_type": "SyntheticExplorer",
        "parent_agent_id": "subagent-synthetic-parent",
        "parent_agent_type": "SyntheticGeneral",
    }));
    let Event::SubagentStarted(started) = event else {
        panic!("expected subagent_started");
    };
    assert_eq!(
        started.parent_agent_id.as_deref(),
        Some("subagent-synthetic-parent")
    );
    assert_eq!(
        started.parent_agent_type.as_deref(),
        Some("SyntheticGeneral")
    );
}

#[test]
fn subagent_start_files_its_identifier_as_the_child() {
    // `SubagentStart.agent_id` identifies the subagent being started. Putting it
    // in the envelope's causal context would claim it was the agent that emitted
    // the event, which is the opposite of what it means.
    let translation = translate(serde_json::json!({
        "hook_event_name": "SubagentStart",
        "session_id": HOOK_SESSION,
        "prompt_id": "prompt-synthetic-0001",
        "agent_id": "subagent-synthetic-child",
        "agent_type": "SyntheticExplorer",
    }));
    let emission = &translation.emissions[0];

    assert_eq!(emission.context.agent_id, None);
    assert_eq!(emission.context.agent_type, None);
    assert_eq!(
        emission.context.prompt_id.as_deref(),
        Some("prompt-synthetic-0001")
    );

    let Event::SubagentStarted(started) = &emission.event else {
        panic!("expected subagent_started");
    };
    assert_eq!(started.agent_id, "subagent-synthetic-child");
}

// ---------------------------------------------------------------------------
// Refusals
// ---------------------------------------------------------------------------

#[test]
fn unsafe_session_ids_cannot_escape_the_recordings_directory() {
    let dir = tempfile::tempdir().expect("temp dir");
    let outside = dir.path().join("outside");
    std::fs::create_dir(&outside).expect("create");
    let recordings = dir.path().join("recordings");

    for hostile in [
        "../escaped",
        "../../escaped",
        "..",
        ".",
        "sub/dir",
        "/absolute",
        "",
        "with space",
        "nul\0byte",
        "dotted.name",
    ] {
        assert!(
            matches!(
                claude::recording_file_name(hostile),
                Err(HookError::UnsafeSessionId(_))
            ),
            "session id {hostile:?} should have been refused"
        );

        let payload = serde_json::json!({
            "hook_event_name": "SessionStart",
            "session_id": hostile,
            "source": "startup",
        });
        let output = run_hook(&recordings, &payload.to_string());
        assert_eq!(
            output.status.code(),
            Some(1),
            "session id {hostile:?} should exit 1"
        );
        assert!(output.stdout.is_empty());
    }

    // Nothing was written anywhere: not beside the recordings directory, and
    // not inside it either.
    assert!(std::fs::read_dir(&outside).expect("read").next().is_none());
    let written: Vec<_> = std::fs::read_dir(&recordings)
        .map(|entries| entries.map(|e| e.expect("entry").file_name()).collect())
        .unwrap_or_default();
    assert!(written.is_empty(), "unexpected recordings: {written:?}");
}

#[test]
fn an_overlong_session_id_is_refused() {
    let long = "a".repeat(129);
    assert!(matches!(
        claude::recording_file_name(&long),
        Err(HookError::UnsafeSessionId(_))
    ));
    assert_eq!(
        claude::recording_file_name(&"a".repeat(128)).expect("at the limit"),
        format!("{}.ndjson", "a".repeat(128))
    );
}

#[test]
fn a_malformed_payload_fails_without_appending() {
    let dir = tempfile::tempdir().expect("temp dir");

    for payload in [
        "",
        "not json",
        "[]",
        "{}",
        "{\"hook_event_name\":\"SessionStart\"}",
        "{\"session_id\":\"abc\"}",
        // Two objects: exactly one is the contract.
        "{\"hook_event_name\":\"SessionEnd\",\"session_id\":\"abc\"}{\"a\":1}",
    ] {
        let output = run_hook(dir.path(), payload);
        assert_eq!(output.status.code(), Some(1), "payload {payload:?}");
        assert!(output.stdout.is_empty(), "payload {payload:?} wrote stdout");
        assert!(!output.stderr.is_empty(), "payload {payload:?} was silent");
    }

    assert!(
        std::fs::read_dir(dir.path())
            .expect("read")
            .next()
            .is_none()
    );
}

#[test]
fn an_unknown_hook_event_is_refused_rather_than_guessed_at() {
    let dir = tempfile::tempdir().expect("temp dir");

    // Real Claude hooks this adapter deliberately does not support, plus one
    // that does not exist. Guessing at any of them would put evidence in a
    // recording that nothing generated.
    for name in [
        "UserPromptSubmit",
        "Stop",
        "PreCompact",
        "PermissionRequest",
        "Notification",
        "SomeHookInventedLater",
    ] {
        let payload = serde_json::json!({
            "hook_event_name": name,
            "session_id": HOOK_SESSION,
        });
        assert!(matches!(
            claude::translate(&payload.to_string()),
            Err(HookError::UnsupportedHookEvent(_))
        ));

        let output = run_hook(dir.path(), &payload.to_string());
        assert_eq!(output.status.code(), Some(1), "hook {name}");
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains(name));
    }

    assert!(
        std::fs::read_dir(dir.path())
            .expect("read")
            .next()
            .is_none()
    );
}

#[test]
fn a_supported_hook_missing_a_required_field_appends_nothing() {
    let dir = tempfile::tempdir().expect("temp dir");

    // PreToolUse with no tool_use_id: the correlation key the whole record
    // depends on. Half a record is not better than none.
    let output = run_hook(
        dir.path(),
        &serde_json::json!({
            "hook_event_name": "PreToolUse",
            "session_id": HOOK_SESSION,
            "tool_name": "Bash",
            "tool_input": { "command": "echo synthetic" },
        })
        .to_string(),
    );
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(
        std::fs::read_dir(dir.path())
            .expect("read")
            .next()
            .is_none()
    );
}

#[test]
fn unknown_payload_fields_are_ignored_so_recording_survives_a_claude_update() {
    // Claude adds fields to hook payloads over time. Rejecting an unrecognized
    // one would mean a harmless upstream addition silently switched off
    // recording for every session on the host.
    let event = only_event(serde_json::json!({
        "hook_event_name": "PostToolUse",
        "session_id": HOOK_SESSION,
        "tool_use_id": "toolu_synthetic_0001",
        "tool_name": "Bash",
        "tool_input": { "command": "echo synthetic" },
        "tool_response": { "content": [] },
        "transcript_path": "/synthetic/transcript.jsonl",
        "cwd": "/synthetic/project",
        "permission_mode": "default",
        "effort": { "level": "medium" },
        "a_field_claude_adds_in_2027": { "nested": true },
    }));
    assert_eq!(event.kind(), "tool_succeeded");
}

// ---------------------------------------------------------------------------
// Process behavior
// ---------------------------------------------------------------------------

#[test]
fn success_writes_nothing_to_stdout() {
    // Claude reads a hook's stdout for permission decisions, updated tool input
    // and output, and additional context. Writing nothing there is what makes
    // this adapter incapable of influencing the session it records.
    let dir = tempfile::tempdir().expect("temp dir");

    for payload in [
        serde_json::json!({"hook_event_name": "SessionStart", "session_id": HOOK_SESSION, "source": "startup"}),
        pre_tool_use(Some("Run the synthetic check")),
        serde_json::json!({
            "hook_event_name": "PostToolUse", "session_id": HOOK_SESSION,
            "tool_use_id": "toolu_synthetic_0001", "tool_name": "Bash",
            "tool_input": {"command": "echo synthetic"}, "tool_response": {"content": []},
        }),
        serde_json::json!({
            "hook_event_name": "PostToolUseFailure", "session_id": HOOK_SESSION,
            "tool_use_id": "toolu_synthetic_0002", "tool_name": "Bash",
            "tool_input": {"command": "false"}, "error": "exit 1",
        }),
        serde_json::json!({
            "hook_event_name": "PermissionDenied", "session_id": HOOK_SESSION,
            "tool_use_id": "toolu_synthetic_0003", "tool_name": "Bash",
            "tool_input": {"command": "echo denied"},
        }),
        serde_json::json!({"hook_event_name": "SubagentStart", "session_id": HOOK_SESSION, "agent_id": "sub-1"}),
        serde_json::json!({"hook_event_name": "SubagentStop", "session_id": HOOK_SESSION, "agent_id": "sub-1"}),
        serde_json::json!({"hook_event_name": "SessionEnd", "session_id": HOOK_SESSION, "reason": "clear"}),
    ] {
        let output = run_hook(dir.path(), &payload.to_string());
        assert!(
            output.status.success(),
            "hook failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            output.stdout.is_empty(),
            "hook wrote stdout: {:?}",
            String::from_utf8_lossy(&output.stdout)
        );
    }

    // All eight surfaces landed in one recording, with the description of the
    // pre-tool payload producing a ninth, reported record.
    let replay = replay_file(&dir.path().join(format!("{HOOK_SESSION}.ndjson"))).expect("replay");
    assert_eq!(replay.tail, Tail::Complete);
    assert_eq!(
        kinds(&replay.records),
        vec![
            "session_started",
            "tool_requested",
            "reported_intent",
            "tool_succeeded",
            "tool_failed",
            "tool_denied",
            "subagent_started",
            "subagent_stopped",
            "session_ended",
        ]
    );
}

#[test]
fn a_hook_never_exits_two() {
    // Exit 2 is how a hook blocks. A recorder that can block is not passive,
    // and a recorder that blocks by accident is worse than no recorder.
    let dir = tempfile::tempdir().expect("temp dir");

    for payload in [
        "not json",
        "{\"hook_event_name\":\"NotAHook\",\"session_id\":\"abc\"}",
        "{\"hook_event_name\":\"SessionStart\",\"session_id\":\"../escape\"}",
        "{\"hook_event_name\":\"SessionStart\",\"session_id\":\"fine\",\"source\":\"startup\"}",
    ] {
        let code = run_hook(dir.path(), payload).status.code();
        assert!(
            matches!(code, Some(0) | Some(1)),
            "payload {payload:?} exited {code:?}"
        );
    }
}

#[test]
fn concurrent_hook_processes_produce_intact_uniquely_sequenced_records() {
    // Claude runs matching hooks in parallel, and parallel tool completions can
    // launch concurrent hook processes against the same recording.
    const PROCESSES: usize = 12;

    let dir = tempfile::tempdir().expect("temp dir");
    let session = "concurrent-hooks-synthetic";

    let mut children = Vec::new();
    for index in 0..PROCESSES {
        let mut child = Command::new(env!("CARGO_BIN_EXE_witnessglass"))
            .arg("claude-hook")
            .arg("--recordings-dir")
            .arg(dir.path())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn");
        let payload = serde_json::json!({
            "hook_event_name": "PostToolUse",
            "session_id": session,
            "tool_use_id": format!("toolu_synthetic_{index:02}"),
            "tool_name": "Bash",
            "tool_input": { "command": format!("echo {index:02}") },
            "tool_response": { "content": [] },
        })
        .to_string();
        child
            .stdin
            .take()
            .expect("stdin")
            .write_all(payload.as_bytes())
            .expect("write");
        children.push(child);
    }

    for child in children {
        let output = child.wait_with_output().expect("wait");
        assert!(
            output.status.success(),
            "hook failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stdout.is_empty());
    }

    let replay = replay_file(&dir.path().join(format!("{session}.ndjson"))).expect("replay");
    assert_eq!(replay.tail, Tail::Complete);
    assert_eq!(replay.records.len(), PROCESSES);
    assert_eq!(
        sequences(&replay.records),
        (1..=PROCESSES as u64).collect::<Vec<_>>()
    );

    // Every hook landed exactly once. Which order they landed in is the
    // recorder's acquisition order and is deliberately not asserted: under
    // parallel hooks it is not a causal order, and pretending otherwise is the
    // mistake this test exists to avoid making.
    let seen: BTreeSet<String> = replay
        .records
        .iter()
        .map(|record| match v2_event(record) {
            Event::ToolSucceeded(succeeded) => succeeded.tool_use_id.clone(),
            other => panic!("expected tool_succeeded, got {}", other.kind()),
        })
        .collect();
    let expected: BTreeSet<String> = (0..PROCESSES)
        .map(|i| format!("toolu_synthetic_{i:02}"))
        .collect();
    assert_eq!(seen, expected);
}

#[test]
fn the_example_settings_file_parses_as_json() {
    // The example is the thing a user copies. If it does not parse, activation
    // fails in a way that is confusing rather than obvious.
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(".claude")
        .join("settings.witnessglass.example.json");
    let text = std::fs::read_to_string(&path).expect("example settings file should exist");
    let parsed: serde_json::Value =
        serde_json::from_str(&text).expect("example settings should be valid JSON");

    let hooks = parsed["hooks"]
        .as_object()
        .expect("example should configure hooks");

    // Exactly the surfaces this adapter supports, and nothing else.
    let configured: BTreeSet<&str> = hooks.keys().map(String::as_str).collect();
    let supported: BTreeSet<&str> = BTreeSet::from([
        "SessionStart",
        "PreToolUse",
        "PostToolUse",
        "PostToolUseFailure",
        "PermissionDenied",
        "SubagentStart",
        "SubagentStop",
        "SessionEnd",
    ]);
    assert_eq!(configured, supported);

    // Synchronous command hooks, and no `async`. Recording completion and
    // visible failure matter more than shaving hook latency during first
    // contact.
    for (event, matchers) in hooks {
        for matcher in matchers.as_array().expect("matcher list") {
            for hook in matcher["hooks"].as_array().expect("hook list") {
                assert_eq!(hook["type"], "command", "{event}");
                assert!(hook.get("async").is_none(), "{event} sets async");
                assert!(
                    hook.get("asyncRewake").is_none(),
                    "{event} sets asyncRewake"
                );
                assert!(
                    hook["command"]
                        .as_str()
                        .expect("command")
                        .contains("${CLAUDE_PROJECT_DIR}"),
                    "{event} does not use ${{CLAUDE_PROJECT_DIR}}"
                );
                let args: Vec<&str> = hook["args"]
                    .as_array()
                    .expect("args")
                    .iter()
                    .map(|a| a.as_str().expect("arg"))
                    .collect();
                assert_eq!(args[0], "claude-hook", "{event}");
                assert_eq!(args[1], "--recordings-dir", "{event}");
                assert!(args[2].contains(".witnessglass/recordings"), "{event}");
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Strict mode: a drift canary for fields the adapter cannot name.
// ---------------------------------------------------------------------------

/// Translate with unmodelled fields refused rather than dropped.
fn translate_strict(payload: serde_json::Value) -> Result<claude::Translation, HookError> {
    claude::translate_with(&payload.to_string(), claude::UnmodelledFields::Reject)
}

/// The exact top-level key set Claude Code 2.1.220 sent on `PostToolUse`,
/// taken from raw payloads captured by the probe rather than from the docs.
fn observed_post_tool_use_payload() -> serde_json::Value {
    serde_json::json!({
        "hook_event_name": "PostToolUse",
        "session_id": HOOK_SESSION,
        "cwd": "/synthetic/workdir",
        "transcript_path": "/synthetic/transcript.jsonl",
        "permission_mode": "default",
        "effort": { "level": "synthetic" },
        "prompt_id": "prompt-synthetic-0001",
        "tool_use_id": "toolu_synthetic_0001",
        "tool_name": "Bash",
        "tool_input": { "command": "echo synthetic" },
        "tool_response": { "content": [] },
        "duration_ms": 7,
    })
}

#[test]
fn strict_mode_accepts_the_field_set_the_integration_actually_sends() {
    // This is the canary's calibration. Every key here was observed on the wire;
    // each is either modelled or listed as deliberately unrecorded, so a correct
    // adapter must accept the whole payload with nothing left over.
    //
    // When this fails, one of two things happened: someone narrowed the adapter,
    // or the field set moved. Both are worth a person looking.
    translate_strict(observed_post_tool_use_payload())
        .expect("every field 2.1.220 sends should be accounted for");

    let mut failure = observed_post_tool_use_payload();
    let object = failure.as_object_mut().expect("object");
    object.insert("hook_event_name".into(), "PostToolUseFailure".into());
    object.remove("tool_response");
    object.insert("error".into(), "synthetic failure".into());
    object.insert("is_interrupt".into(), false.into());
    translate_strict(failure).expect("the failure hook's field set too");
}

#[test]
fn strict_mode_refuses_a_field_the_adapter_can_neither_model_nor_name() {
    let mut payload = observed_post_tool_use_payload();
    payload.as_object_mut().expect("object").insert(
        "synthetic_future_field".into(),
        serde_json::Value::from(1234),
    );

    let error = translate_strict(payload).expect_err("strict mode should refuse it");
    let HookError::UnmodelledFields {
        hook_event_name,
        fields,
    } = &error
    else {
        panic!("expected UnmodelledFields, got {error:?}");
    };
    assert_eq!(hook_event_name, "PostToolUse");
    assert_eq!(fields, &["synthetic_future_field".to_owned()]);

    // The message has to name the field, or the canary reports only that
    // something changed and leaves the reader to diff the payload by hand.
    assert!(
        error.to_string().contains("synthetic_future_field"),
        "{error}"
    );
}

#[test]
fn strict_mode_would_have_caught_the_key_that_went_unread_for_two_sprints() {
    // The historical failure, from the direction it would now be detected. The
    // adapter modelled `duration`; the wire sent `duration_ms`. Whichever of the
    // two the adapter is wrong about, the other one shows up as a field it
    // cannot name — so strict mode reports the mismatch rather than producing a
    // recording that quietly lacks timing.
    let mut payload = observed_post_tool_use_payload();
    payload
        .as_object_mut()
        .expect("object")
        .insert("duration".into(), serde_json::Value::from(5000));

    let error = translate_strict(payload).expect_err("the documented spelling is not modelled");
    assert!(error.to_string().contains("duration"), "{error}");
}

#[test]
fn deliberately_unrecorded_fields_do_not_trip_the_canary() {
    // `cwd` arrives on every payload and is dropped on purpose, for privacy. If
    // it tripped strict mode the canary would fire on every hook of every
    // session and be worth nothing. "Dropped on purpose" and "never heard of"
    // must stay different facts.
    let payload = observed_post_tool_use_payload();
    assert!(
        payload.get("cwd").is_some(),
        "the fixture must actually carry one"
    );
    translate_strict(payload).expect("a deliberately unrecorded field is accounted for");
}

#[test]
fn capturing_unmodelled_fields_did_not_change_the_lenient_default() {
    // `unknown_payload_fields_are_ignored_so_recording_survives_a_claude_update`
    // states the policy. This states that collecting those fields into a map,
    // instead of letting serde discard them, left the policy and the parse
    // intact: the payload still translates, the modelled value still lands, and
    // the captured field reaches no record.
    let mut payload = observed_post_tool_use_payload();
    payload
        .as_object_mut()
        .expect("object")
        .insert("synthetic_future_field".into(), 1234.into());

    let translation = translate(payload);
    assert_eq!(translation.emissions.len(), 1);
    let Event::ToolSucceeded(succeeded) = &translation.emissions[0].event else {
        panic!("expected tool_succeeded");
    };
    assert_eq!(succeeded.duration_ms, Some(7));

    // And the dropped field reaches no record, in any spelling.
    let line = serde_json::to_string(&translation.emissions[0]).expect("serialize");
    assert!(!line.contains("synthetic_future_field"), "{line}");
}

#[test]
fn strict_mode_refuses_before_writing_anything() {
    // A refused payload must leave no partial record behind. The adapter either
    // records a hook completely or records none of it, and strict mode is not an
    // exception to that.
    let dir = tempfile::tempdir().expect("temp dir");
    let mut payload = observed_post_tool_use_payload();
    payload
        .as_object_mut()
        .expect("object")
        .insert("synthetic_future_field".into(), 1234.into());

    let output = run_hook_with(
        dir.path(),
        &payload.to_string(),
        &["--strict-json-validation"],
        &[],
    );
    assert_eq!(output.status.code(), Some(1), "exit 1 is non-blocking");
    assert!(output.stdout.is_empty(), "stdout is read as a decision");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("synthetic_future_field"), "{stderr}");
    assert!(
        !dir.path().join(format!("{HOOK_SESSION}.ndjson")).exists(),
        "a refused payload must not create a recording"
    );
}

#[test]
fn the_environment_variable_enables_strict_mode_for_hooks_claude_spawns() {
    // A hook is launched by Claude from a settings file this project's arm.sh
    // writes, so there is no command line to add a flag to. The environment is
    // the only path that reaches it.
    let dir = tempfile::tempdir().expect("temp dir");
    let mut payload = observed_post_tool_use_payload();
    payload
        .as_object_mut()
        .expect("object")
        .insert("synthetic_future_field".into(), 1234.into());

    let output = run_hook_with(
        dir.path(),
        &payload.to_string(),
        &[],
        &[("WITNESSGLASS_STRICT_JSON", "1")],
    );
    assert_eq!(output.status.code(), Some(1));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("synthetic_future_field"),
        "the environment variable should refuse the same payload the flag does"
    );

    // Unset, the same payload records.
    let dir = tempfile::tempdir().expect("temp dir");
    let output = run_hook(dir.path(), &payload.to_string());
    assert_eq!(output.status.code(), Some(0));
}
