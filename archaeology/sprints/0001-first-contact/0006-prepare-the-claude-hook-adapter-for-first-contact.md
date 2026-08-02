---
id: tsk_01KZ26S6Y7PE6WRJEJS6S2DFDV
sequence: 6
kind: task
status: closed
sprint: spr_01KZ1SQTZ730K3VJMH127NMXNS
created: 2026-08-02
closed: 2026-08-02
---

# Prepare the Claude hook adapter for first contact

## Objective

Build the passive Claude Code command-hook adapter that task:4 will be run through, and get
the raw stream honest about what a Claude hook payload actually proves — before a real
session is recorded rather than after.

This is a prerequisite, not the first-contact exercise itself. No real session is recorded
here, no hook configuration is activated, and dragon:1 stays open: nothing in this task
measures anything against a live Claude process, and every coverage statement it produces is
read off Claude's documentation.

The forcing problem is that raw stream v1 cannot represent Claude's tool lifecycle honestly.
`PreToolUse` fires after the model constructs a request and before the call is processed, and
that request may then be modified, denied, deferred, or never executed. Mapping it to v1's
`observed_tool_started` would record a request as an execution; v1's single `arguments` field
would collapse requested and effective input; and v1's `Succeeded | Failed` outcome would
file a permission denial as an execution failure. Each is exactly the silent promotion
decision:2 forbids, so the schema changes rather than the truth.

## Acceptance criteria

- An accepted decision records the versioning change, why v1 sufficed for the synthetic
  kernel but not for first contact, the compatibility consequences, and what is deliberately
  deferred. decision:3 and task:3 are not rewritten.
- Raw stream v2 keeps every envelope invariant from decision:3 and adds only causal context
  the integration actually supplies: optional `prompt_id`, optional current `agent_id` and
  `agent_type`, and `tool_use_id` on tool events. No root agent id, parent agent id, span id,
  or hierarchy is invented.
- The v2 vocabulary represents session started (with source), session ended (with reason),
  tool requested, tool succeeded, tool failed, tool denied, subagent started, subagent
  stopped, and reported intent — with requested and effective input distinct, and denial
  distinct from execution failure.
- Existing schema v1 recordings still replay. A recording uses one schema version throughout,
  and appending across versions is refused at both the replay and append boundaries.
- One narrowly named CLI entry point, `witnessglass claude-hook --recordings-dir <DIR>`,
  reads exactly one hook payload from stdin, selects `<DIR>/<safe-session-id>.ndjson`,
  appends through the existing serialized transaction, prints nothing to stdout on success,
  exits 0 on success and 1 on failure, and never exits 2.
- The adapter returns no permission decision, no updated tool input or output, and no
  additional context; never reads the transcript; and never executes or interpolates any
  value from the payload.
- `session_id` is validated before use as a filename, so path traversal is impossible.
- Exactly eight hook surfaces are supported: `SessionStart`, `PreToolUse`, `PostToolUse`,
  `PostToolUseFailure`, `PermissionDenied`, `SubagentStart`, `SubagentStop`, `SessionEnd`.
  Unknown JSON fields at the adapter boundary are ignored; an unknown `hook_event_name` is
  refused rather than guessed at.
- An explicit agent-supplied description in a tool request may produce a separate
  `reported_intent` record correlated by `tool_use_id`. Intent is never manufactured from a
  command, path, prompt, tool name, temporal adjacency, or result.
- No active hook configuration is committed. `.witnessglass/` and
  `.claude/settings.local.json` are gitignored, a clearly inert example configuration exists,
  and activation and disarming instructions are user-facing.
- Tests use only synthetic payloads and temporary directories, and cover every supported hook
  event, the requested/effective distinction, description-becomes-reported-intent,
  success/failure/denial staying distinct, absent optional fields, preserved context
  identifiers, no invented parentage, `SubagentStart` filing its id as the child, a
  post-event without a pre-event, unsafe session ids, malformed and unknown hooks appending
  nothing, concurrent hook processes, v1 and v2 replay, refusal to mix versions, the example
  settings parsing as JSON, and success producing no stdout. No real Claude process is
  invoked.
- A focused Claude adapter document states separately what Claude's documentation promises,
  what this implementation maps, and what remains unmeasured until task:4, with the
  provisional blind spots prominent.
- `scripts/check.sh` passes and the slice is committed. `publish = false` is retained. The
  configuration is not activated during the authoring session, task:4 stays pending, and both
  dragons stay open.

## Result

The adapter exists, the schema is honest about what a hook payload proves, and none of it
has met a real Claude session. That last clause is the important one: this task deliberately
produced no measurement, and every coverage statement it wrote down is read off Claude's
documentation and marked provisional. task:4 remains pending and both dragons remain open.

Claude Code on the authoring host: **2.1.220**, obtained with `claude --version`, which
returned without touching the `CLAUDECODE` nesting guard (still `=1` throughout). No nested
Claude process was launched and no hook configuration was activated.

### The versioning decision

decision:4 introduces raw stream v2 and freezes v1. The forcing evidence came from reading
the current hooks reference *before* writing the adapter, which is the only reason the
mismatch was caught at design time rather than after a recording existed.

v1 was adequate for what it was built against — a synthetic emitter — and inadequate for
Claude in three specific ways, each of which would have been a silent promotion of the kind
decision:2 forbids:

- **`PreToolUse` is not a start.** It fires after the model constructs a request and before
  the call is processed; the request may then be modified, denied, deferred, or never
  executed. Mapping it to `observed_tool_started` would have asserted, on the observed
  channel, something no capture point ever witnessed.
- **One `arguments` field cannot hold two inputs.** Claude delivers a requested input before
  execution and an effective input after, and documents that they can differ. Collapsing them
  destroys the only evidence that what ran was not what was asked for.
- **`Succeeded | Failed` has no room for a denial.** Folding `PermissionDenied` into `Failed`
  makes "the agent was stopped" indistinguishable from "the agent tried and it broke".

The alternative — keep v1 and paper over the gap in the adapter — was rejected on the same
reasoning decision:3 used for channel coherence: the schema should make the error
unrepresentable rather than leave an adapter to remember not to make it.

Compatibility: v1 recordings still replay and round-trip byte-for-byte through replay and
re-rendering. Only v2 is written. A recording uses one schema version throughout, enforced at
**both** ends — replay refuses a mixed recording, and the appender refuses to add a v2 record
to a v1 recording, leaving it byte-for-byte unchanged. Catching it only at replay would have
meant the appender had already destroyed a readable recording by writing to it.

### Event mapping

| Hook | Record | Channel |
| --- | --- | --- |
| `SessionStart` | `session_started` (+ `source`) | observed |
| `PreToolUse` | `tool_requested` (`tool_use_id`, `tool_name`, `requested_input`) | observed |
| `PreToolUse` with a non-blank `tool_input.description` | *additionally* `reported_intent` | **reported** |
| `PostToolUse` | `tool_succeeded` (`effective_input`, `response`, optional `duration_ms`) | observed |
| `PostToolUseFailure` | `tool_failed` (`effective_input`, `error`, optional `interrupted`, `duration_ms`) | observed |
| `PermissionDenied` | `tool_denied` (`requested_input`, no error, no effective input) | observed |
| `SubagentStart` / `SubagentStop` | `subagent_started` / `subagent_stopped` | observed |
| `SessionEnd` | `session_ended` (+ `reason`) | observed |

`prompt_id`, and `agent_id`/`agent_type` where supplied, go into the envelope's `context`,
omitted entirely when the integration supplied none. `provenance.mechanism` names the
originating hook (`command-hook:PostToolUse`), so a reader can always tell which capture
point produced a claim and therefore what it could see.

A description becomes a *second* record on the reported channel with mechanism
`command-hook:PreToolUse#tool_input.description`, correlated by `tool_use_id`. It is
**duplicated, not moved** — the full requested input stays whole as source-delivered
evidence — and the duplication is documented, because a reader counting occurrences of a
string across a recording needs to know it appears twice. Nothing else produces intent: a
command, path, prompt, tool name, result, and temporal adjacency are all not the agent
saying anything, and a blank description is not a claim either.

### A divergence from the task brief, resolved in favour of the evidence

The brief stated that the documented hook payload does not establish a parent identifier. The
reference as read on 2026-08-02 against 2.1.220 **does** document `parent_agent_id` and
`parent_agent_type` on `SubagentStart` and `SubagentStop`.

Recording a supplied identifier is not inference, and discarding delivered evidence would be
its own defect, so the adapter carries those fields through **exactly when Claude supplies
them** and leaves them absent otherwise. The prohibition the brief was actually protecting is
untouched and tested: nothing synthesizes a root id, a parent id, a span id, or a hierarchy,
and parentage is never derived from timing or from adjacency in the recording. Preserving a
supplied identifier and refusing to invent a missing one are the same rule applied in both
directions.

Whether those fields are ever actually populated in practice is unmeasured, and the adapter
document says so rather than assuming it.

`SubagentStart.agent_id` is filed as the **child's** identity in the event payload, not in
`context.agent_id`, which would have claimed it was the identity of the agent that emitted
the event.

### Supported hook surfaces and passivity

Exactly eight: `SessionStart`, `PreToolUse`, `PostToolUse`, `PostToolUseFailure`,
`PermissionDenied`, `SubagentStart`, `SubagentStop`, `SessionEnd`. `UserPromptSubmit`,
streamed assistant messages, transcript contents, and every other lifecycle hook are out of
scope for this slice and therefore invisible in any recording.

`witnessglass claude-hook --recordings-dir <DIR>` reads exactly one payload from stdin,
selects `<DIR>/<safe-session-id>.ndjson`, and appends through the existing serialized
transaction. Passivity is structural, not conventional:

- **Nothing on stdout on success.** Claude reads a hook's stdout for permission decisions,
  `updatedInput`, `updatedToolOutput`, and `additionalContext`. Writing nothing there is what
  makes influence impossible rather than merely unintended.
- **Never exits 2** — the code that blocks a `PreToolUse` call. Only 0 or 1, and there is a
  test that holds it to that. Exit 1 is documented as non-blocking for all eight configured
  hooks, so a broken recorder stops recording without stopping the session.
- **Never reads the transcript**, though `transcript_path` is in every payload.
- **Never executes or interpolates any payload value.** Every value is either compared
  against a fixed set of names or stored as opaque JSON.

The two boundaries have deliberately opposite strictness. Unknown JSON fields in a *payload*
are ignored, because a harmless Claude field addition must not silently disable recording on
a host; unknown fields in a *record* are still rejected. Strictness belongs on the evidence,
not on somebody else's wire format. An unknown `hook_event_name`, by contrast, is refused by
name rather than guessed at — inventing a meaning for an unrecognized lifecycle point would
put evidence in a recording that nothing generated.

`session_id` is validated against `[A-Za-z0-9_-]`, non-empty, ≤128 bytes, before becoming a
filename. `.` is excluded from the set entirely, which makes `.` and `..` unrepresentable and
removes the traversal question rather than answering it. An id outside the set is refused
loudly rather than escaped by an encoding whose inverse nobody has defined.

### Tests and manual verification

**71 tests passing, up from 35.** Two new suites, four updated.

- `tests/claude_hook.rs` (27) — every supported hook surface; requested input staying distinct
  from effective input across two real process invocations; an explicit description producing
  reported, not observed, intent while the requested input keeps it too; success, failure, and
  denial as three different records with the denial carrying no invented error and no
  effective input; absent `prompt_id`/`agent_id`/`agent_type`/`duration`/`interrupted` staying
  absent in the serialized record rather than defaulting (`false` and "not stated" are
  different claims); supplied context identifiers preserved; no root or parent identity
  invented; a supplied parent identifier recorded as delivered; `SubagentStart` filing its id
  as the child with `context.agent_id` left empty; a completion with no captured request
  accepted rather than rejected; ten hostile session ids refused with nothing written
  anywhere; malformed, empty, double-object, and field-missing payloads appending nothing;
  six unknown hook events refused; unknown payload fields ignored; twelve concurrent hook
  processes producing intact uniquely-sequenced records; no stdout on any of the eight
  surfaces; never exiting 2; and the example settings file parsing as JSON with exactly the
  eight supported surfaces, all synchronous, none setting `async`.
- `tests/schema_versions.rs` (7) — v1 replay, v1 byte-for-byte round trip, v2 replay, mixing
  refused in both directions, appending v2 to a v1 recording refused with the file unchanged,
  and the same refusal reached through the adapter binary.
- Updated: `round_trip.rs` (8), `damage.rs` (18), `ordering.rs` (9), `concurrency.rs` (2).
  New in `round_trip.rs`: a request is not recorded as an execution (asserted against the
  serialized line, not just the type), and requested/effective input staying distinct.

The concurrency test deliberately asserts *which* records landed and not *in what order*.
Under parallel hooks the sequence is recorder acquisition order, and asserting a causal order
there would be the exact mistake the test exists to avoid making.

No real Claude process is invoked anywhere in the suite. Every payload is synthetic and every
recording is written to a temporary directory.

**Manual verification** against a throwaway directory outside the repository, using the built
binary: all eight surfaces plus a description-bearing `PreToolUse` produced nine records in
one recording, replaying complete as schema v2 — including a `PostToolUse` whose
`effective_input` deliberately differed from the earlier `requested_input`, and an unknown
`unknown_future_field` that was ignored without incident. Ten refusal cases (three traversal
shapes, an absolute path, a dotted name, an empty id, two unknown hooks, non-JSON, empty
stdin, and a missing required field) each exited 1 with zero stdout, a specific stderr
message, and **no file created anywhere** — verified by listing both the target directory and
its parent. `.gitignore` coverage was verified by creating `.witnessglass/recordings/` and
`.claude/settings.local.json`, confirming git ignored both, and deleting them again.

### Fidelity and blind spots — all provisional

`docs/claude-adapter.md` states separately what Claude's documentation promises, what this
implementation maps, and what remains unmeasured, and the README links to it. Nothing in it
is described as characterized, because nothing has been measured. The prominent ones:

- **Pre-tool evidence is a request, not proof of execution.** A `tool_requested` with no
  matching completion means WitnessGlass did not see what happened next — not that nothing
  happened, and not that the call was blocked.
- **Completed hooks expose Claude's tool-level input and response, not descendant syscalls.**
  A `Bash` record shows the command and the tool's reported output. What the shell actually
  spawned, wrote, or read is not observed at all.
- **Validation failures can escape the selected hooks entirely,** firing neither `PreToolUse`
  nor `PostToolUseFailure` and leaving no trace whatsoever.
- **Permission-denial coverage depends on this host and version.** `PermissionDenied` is
  documented as firing when a call is denied *by the auto mode classifier*. Whether an
  interactive prompt denial, a deny rule, or another permission mode also fires it is unknown,
  so absence of `tool_denied` records must not be read as absence of denials.
- **`@` file references may bypass `Read` tool hooks,** so file content can enter a session
  with no tool event at all.
- **Parentage is not invented when ids are absent,** so a recording of a session with
  subagents may contain no expressible hierarchy. That is the correct output, not a gap.
- **Appender order under parallel hooks is recorder order,** not causal order.
- **The hooks add synchronous latency,** unmeasured. `async: true` was deliberately not used:
  during first contact a complete recording and a visible failure matter more than shaving
  hook latency.
- **Recordings remain sensitive and unsafe to share.** Nothing is redacted. See dragon:2.

### Files and configuration

`src/record.rs` became a module directory: shared `Channel`/`Provenance`/`AnyRecord` plus
frozen `v1` and current `v2`. New `src/claude.rs`. `src/append.rs`, `src/replay.rs`,
`src/error.rs`, `src/main.rs`, `src/lib.rs` updated. New `docs/claude-adapter.md`. README
updated; `CLAUDE.md` §3 had the same stale "likely JSONL/NDJSON" wording task:5 reported and
left alone, and it is now corrected to point at decision:3 and decision:4.

**No dependencies were added.** The authority allowed the smallest justified additions and
none turned out to be justified. `publish = false` retained.

Opt-in configuration: `.claude/settings.witnessglass.example.json` is committed and inert —
Claude reads `settings.json` and `settings.local.json`, both now gitignored, so a clone
records nothing and cannot silently begin executing hooks. `.witnessglass/` is gitignored.
The example invokes the already-built debug binary through `${CLAUDE_PROJECT_DIR}` in exec
form, stores recordings under `.witnessglass/recordings/`, configures all eight surfaces as
synchronous command hooks, and carries activation and disarming instructions in a comment
block; the same instructions are in the README and the adapter document, which also scope the
configuration to macOS and Linux and state plainly that Windows is untested. `cargo build` is
required before activation and is documented as such.

The configuration was **not activated during this session**. Arming it after `SessionStart`
would have manufactured a partial recording with no session boundary and defeated the point
of the fresh-session experiment.

### Concerns that should alter the task:4 experiment

- **`PermissionDenied` may well never fire.** Its documented trigger is the auto mode
  classifier. If task:4 is run in a permission mode where denials come from an interactive
  prompt, the recording will contain no `tool_denied` records and that must be reported as
  "not exercised", not as "no denials occurred". Worth deliberately provoking a denial and
  seeing which way it goes — that is a measurement dragon:1 wants.
- **`prompt_id` is documented as absent until the first input,** so `SessionStart` will
  almost certainly carry none. Expected, not a defect.
- **Subagent parentage is the open empirical question.** The fields are documented; whether
  they arrive populated is exactly what first contact should check, and it decides whether a
  causal overlay is buildable honestly or not at all.
- **A resumed session appends to an existing recording.** With `source: "resume"` the same
  `session_id` reopens the same file, so a recording can contain more than one
  `session_started`. Replay accepts this — decision:3 deliberately does not require a
  recording to open with a boundary — but a reader should expect it.
- **Eight synchronous hooks on every tool call is a real latency cost** that nobody has
  measured. If first contact feels slow, that is data worth writing down rather than a reason
  to quietly switch to `async`.

### Scarp desire paths

**idea:1 recurred for the fifth time.** This result was written to a temporary file and
appended with a shell redirect before `scarp close task:6`. Five for five across every task
this project has closed. Nothing to add; the sample was already unambiguous at four.

**idea:3 recurred for the third time, and more sharply than before.** This task created
decision:4, implements it, and refines decision:3 — three edges, none expressible. Worse,
`docs/claude-adapter.md` and `README.md` now both link to decision:4 *by file path*, so the
prose reference has escaped the Scarp corpus entirely and `scarp doctor` could not check it
even in principle. Already promoted; recorded here only because the failure mode it predicts
(a reference that rots silently) now has more surface than it did in task:5.

**idea:2 did not recur, and that is informative.** Creating a decision and a task by
`--body-file` needed the section headings again, but this time the corpus already contained
examples of both collections, so reading an existing artifact answered it in seconds. The gap
idea:2 describes is specifically a *first-use* gap per collection, and it closes by accretion
once a project has one of everything. That narrows the idea rather than strengthening it: the
affordance would be most valuable to a new Scarp repository and progressively less valuable
to an established one.
