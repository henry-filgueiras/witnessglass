---
id: dec_01KZ26QZDWXSQ7RWB41B5YK6XS
sequence: 4
kind: decision
status: accepted
created: 2026-08-02
---

# Represent requested and effective Claude tool evidence separately

## Context

Decision:3 defined raw stream v1 against a synthetic emitter. It was adequate for what it
was built for: the kernel had to settle ordering, timestamps, versioning, damage, and
concurrency, and it did, with tests that still hold. What it did not have was a real hook
payload in front of it. The v1 tool vocabulary — `observed_tool_started` and
`observed_tool_finished` with a success/failure outcome — was written from a reasonable
guess about the shape of a cooperative integration, not from the integration.

Reading Claude Code's current hooks reference before building the adapter showed the guess
was wrong in a way that matters. The documented evidence is:

- `PreToolUse` fires after the model constructs a tool request and **before the call is
  processed**. The request may subsequently be modified — a hook may return `updatedInput` —
  denied, escalated, deferred, or never executed at all.
- `PostToolUse` fires **only** after successful execution, and carries the input *actually
  sent*, the response, and an optional duration.
- `PostToolUseFailure` fires after an executing tool fails, and carries the effective input,
  error information, an optional interruption flag, and an optional duration.
- `PermissionDenied` fires on a denial. Nothing executed.
- Validation rejection may fire neither `PreToolUse` nor `PostToolUseFailure`, so a request
  can leave no trace at all.
- Matching hooks run in parallel, so parallel tool completions can launch concurrent hook
  processes writing to the same recording.
- Common fields include `session_id`, an optional `prompt_id`, and inside a subagent
  `agent_id` and `agent_type`. Tool events carry `tool_use_id`.
- `SubagentStart.agent_id` identifies the child.

Mapping that onto v1 would have required three quiet lies, each of exactly the kind
decision:2 exists to forbid.

**A request would have been recorded as an execution.** `PreToolUse` → `observed_tool_started`
reads as "the machinery witnessed this call begin". No Claude hook witnesses a call
beginning. A recording full of `observed_tool_started` records would have been asserting,
on the observed channel, something no capture point ever saw.

**Requested and effective input would have collapsed into one field.** v1 has a single
`arguments`. Claude delivers a requested input before execution and an effective input
after, and documents that they can differ. One field would have destroyed the only evidence
that what ran was not what was asked for — which is precisely the disagreement this project
exists to preserve.

**Denial would have been filed as failure.** v1's `ToolOutcome` is `Succeeded | Failed`.
Folding `PermissionDenied` into `Failed` makes "the agent was stopped from doing this"
indistinguishable from "the agent did this and it broke". Those license different
conclusions about a session, and about the agent.

The alternative to a version bump was to keep v1 and paper over the gap in the adapter or in
documentation. That is worse: the schema would then permit the epistemic error, and the only
thing preventing it would be an adapter remembering not to make it. decision:3 made channel
coherence unrepresentable rather than merely discouraged, and the same reasoning applies
here.

## Decision

Introduce raw stream v2, narrowly, and freeze v1 rather than extending it.

**The envelope invariants from decision:3 are unchanged.** Schema version on every record;
one Claude session per recording; strict append sequence as canonical storage order;
recorder wall-clock timestamp as non-ordering descriptive metadata; concrete
`{channel, adapter, mechanism}` provenance; event payload. `deny_unknown_fields` still
applies to records.

**One addition to the envelope: `context`**, holding only causal identifiers the integration
actually supplies — optional `prompt_id`, and optional current `agent_id` and `agent_type`.
Omitted entirely when the integration supplied none. `tool_use_id` moves onto the tool
events themselves.

**Nothing beyond that is added.** No root agent id, no parent agent id, no span id, no
hierarchy. This is the empirical precursor to a possible agent-local causal overlay, not its
implementation.

**The v2 vocabulary is nine kinds:** `session_started` (with the documented startup source),
`session_ended` (with the documented reason), `tool_requested` (tool id, name, requested
input), `tool_succeeded` (same id, name, effective input, response, optional duration),
`tool_failed` (effective input, error, optional interruption state, optional duration),
`tool_denied` (kept distinct from execution failure), `subagent_started`, `subagent_stopped`,
and `reported_intent`.

**Channel coherence carries over.** `reported_intent` may only arrive on `reported`; tool and
subagent lifecycle only on `observed`; session boundaries on `recorder` or `observed`.

**An explicit agent-supplied description in a tool request may produce a separate
`reported_intent` record**, correlated by `tool_use_id`. The full requested input is
preserved unedited as source-delivered evidence; the description is duplicated into a
separately classified reported record rather than moved, and that duplication is documented.
Intent is never manufactured from a command, a path, a prompt, a tool name, temporal
adjacency, or a result.

**One recording uses one schema version throughout.** Both versions replay. Only v2 is
written. Replay refuses a recording that mixes versions, and the appender refuses to add a
v2 record to a v1 recording, leaving the older recording byte-for-byte unchanged.

**Physical append sequence records recorder acquisition order.** Under concurrent hook
processes it is not automatically a total causal order for Claude's execution. Per-tool
correlation and supplied durations may support derived views; raw replay never reorders.

### On supplied parent identifiers

The current hooks reference documents `parent_agent_id` and `parent_agent_type` on
`SubagentStart` and `SubagentStop`. Where Claude supplies them they are recorded exactly as
delivered; where it does not, the fields are absent and stay absent. Recording a supplied
identifier is not inference, and discarding delivered evidence would be its own defect.
Parentage is never derived from timing, from adjacency in the recording, or from anything
else. Whether those fields are actually populated in practice is unmeasured, and the
adapter's fidelity note says so.

`SubagentStart.agent_id` is filed as the child's identity in the event payload rather than in
the envelope's `context.agent_id`, which would claim it was the identity of the agent that
emitted the event.

## Consequences

- v1 is frozen: readable forever, never written again. A flight recorder that cannot read
  its own older recordings is not much of a flight recorder, so replay dispatches on the
  declared version and returns a version-tagged record. A reader must acknowledge which
  vocabulary it is holding, which is deliberate — v1's "tool started" means something v2
  refuses to claim.
- v1 recordings round-trip byte-for-byte through replay and re-rendering. Reading an old
  recording cannot quietly rewrite it into the current schema.
- Refusing mixed versions at the append boundary as well as at replay is load-bearing.
  Catching it only at replay would mean the appender had already destroyed a readable v1
  recording by writing to it.
- The cost of `deny_unknown_fields` is paid again: any further additive change needs v3.
  Accepted for the same reason as before — silent forward compatibility is how a reader ends
  up quietly discarding evidence it did not recognize.
- The recording is more verbose. Five kinds became nine, a request and a completion each
  carry their own input, and a `Bash` description appears twice. All of that is the point.
- A `tool_requested` record with no matching completion is now a first-class, meaningful
  state: the recorder saw a request and did not see what became of it. Under v1 that state
  was indistinguishable from a call that started and never finished.
- The adapter boundary and the record boundary have deliberately opposite strictness.
  Unknown fields in a hook payload are ignored, because a harmless Claude field addition
  must not silently disable recording on a host; unknown fields in a record are rejected.
  Strictness belongs on the evidence, not on somebody else's wire format.

### Deliberately deferred

- **Originating time.** Still only the recorder's write time. Hook payloads carry a duration
  but no event timestamp, so a separate originating-time field would be populated by
  inference on most events. Revisit after first contact.
- **A causal overlay.** `context` collects the identifiers Claude supplies and stops there.
  Whether a derived parent/child or prompt-scoped view can be built honestly is a question
  for after a real session has been recorded, and it belongs in a projection either way.
- **`SubagentStop.last_assistant_message` and `stop_reason`,** `SessionStart.model`,
  `permission_mode`, `cwd`, and `effort`. All available, none captured in this slice. The
  first is agent-authored prose and would need its own reported-channel treatment and a
  privacy justification.
- **Whether `PermissionDenied` covers denial at an interactive prompt** or only the
  documented auto-mode classifier path. Unmeasured, so absence of `tool_denied` records must
  not be read as absence of denials.
- **The latency cost of eight synchronous command hooks.** Unmeasured. `async: true` was
  deliberately not used for first contact, because recording completion and visible failure
  matter more than shaving hook latency.
- **Redaction and export.** Nothing is redacted. Recordings remain unsafe to share, per
  dragon:2.
