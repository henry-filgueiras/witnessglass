# The Claude Code adapter

WitnessGlass records a Claude Code session through Claude's own **command hooks**. This
document states three things separately, and they must stay separate:

1. what Claude's current documentation promises;
2. what this implementation maps;
3. what remains unmeasured.

Section 3 is not a disclaimer. It is the honest scope of every claim in sections 1 and 2,
and it stays there until a real session has been recorded and compared against what that
session demonstrably did (task:4).

The adapter has **not yet been run against a live Claude session**. Everything below that
is not marked as tested is read off documentation, not measured.

Reference used: <https://code.claude.com/docs/en/hooks>, read 2026-08-02.
Claude Code version present on the authoring host: **2.1.220**.

---

## 1. What Claude's documentation promises

### The hooks this adapter subscribes to

| Hook | Documented firing condition | Documented payload beyond the common fields |
| --- | --- | --- |
| `SessionStart` | A session begins or resumes | `source` ∈ {`startup`, `resume`, `clear`, `compact`, `fork`}, optional `model` |
| `PreToolUse` | After the model constructs a tool request, **before the call is processed** | `tool_name`, `tool_input`, `tool_use_id` |
| `PostToolUse` | After a tool call **succeeds** | `tool_name`, `tool_input`, `tool_use_id`, `tool_response`, optional `duration` |
| `PostToolUseFailure` | After a tool call **fails** | `tool_name`, `tool_input`, `tool_use_id`, `error`, optional `duration`, optional `interrupted` |
| `PermissionDenied` | When a tool call is denied by the auto mode classifier | `tool_name`, `tool_input`, `tool_use_id` |
| `SubagentStart` | A subagent is spawned | `agent_id`, `agent_type`, optional `parent_agent_id`, `parent_agent_type` |
| `SubagentStop` | A subagent finishes | as above, plus `last_assistant_message`, `stop_reason` |
| `SessionEnd` | A session terminates | `reason` ∈ {`clear`, `resume`, `logout`, `prompt_input_exit`, `bypass_permissions_disabled`, `other`} |

Common fields documented across hooks include `session_id`, `hook_event_name`,
`transcript_path`, `cwd`, `permission_mode`, an optional `prompt_id` (documented as absent
until the first input), and — inside a subagent — `agent_id` and `agent_type`.

### The four documented facts that determined the design

**A pre-tool payload is a request, not an execution.** `PreToolUse` fires after the model
constructs a tool request and before the call is processed. The request may then be
modified (a hook may return `updatedInput`), denied, escalated, deferred, or never executed
at all.

**Completion, failure, and denial are three separate events.** `PostToolUse` fires only on
success and carries the input *actually sent* plus the response. `PostToolUseFailure`
carries the effective input, an error, and an optional `interrupted` flag. `PermissionDenied`
fires on a denial, where nothing executed.

**Matching hooks run in parallel.** Parallel tool completions can therefore launch
concurrent hook processes writing to the same recording.

**Exit codes differ per hook.** For every hook this adapter subscribes to, exit 2 is
documented as non-blocking or ignored — but `PreToolUse` exit 2 *blocks the tool call*.
That asymmetry is why this adapter never exits 2 under any circumstance.

---

## 2. What this implementation maps

### Schema

Recordings are **raw stream v2**. See
[decision 4](../archaeology/decisions/0004-represent-requested-and-effective-claude-tool-evidence-separately.md)
for why v1 was not stretched to fit. Existing v1 recordings still replay; nothing writes v1
any more; a recording never mixes versions.

### Event mapping

| Hook | Record kind | Channel | Payload |
| --- | --- | --- | --- |
| `SessionStart` | `session_started` | observed | `source` |
| `PreToolUse` | `tool_requested` | observed | `tool_use_id`, `tool_name`, `requested_input` |
| `PreToolUse` (when `tool_input.description` is a non-blank string) | *additionally* `reported_intent` | **reported** | `text`, `tool_use_id` |
| `PostToolUse` | `tool_succeeded` | observed | `tool_use_id`, `tool_name`, `effective_input`, `response`, optional `duration_ms` |
| `PostToolUseFailure` | `tool_failed` | observed | `tool_use_id`, `tool_name`, `effective_input`, `error`, optional `interrupted`, optional `duration_ms` |
| `PermissionDenied` | `tool_denied` | observed | `tool_use_id`, `tool_name`, `requested_input` |
| `SubagentStart` | `subagent_started` | observed | `agent_id` (the child), optional `agent_type`, optional supplied parent ids |
| `SubagentStop` | `subagent_stopped` | observed | as above |
| `SessionEnd` | `session_ended` | observed | `reason` |

`prompt_id`, and `agent_id`/`agent_type` where the payload supplies them, go into the
record envelope's `context`. `tool_use_id` goes on the event.

Every record's `provenance.mechanism` names the hook it came from — `command-hook:PostToolUse`
— so a reader can always tell which capture point produced a claim, and therefore what that
capture point could see.

### The three mappings that carry the epistemic weight

**`PreToolUse` becomes `tool_requested`, never "started".** The record says a request
existed. It does not say the call ran, does not say it ran with this input, and does not
imply an outcome. A recording holding `tool_requested` and nothing else is evidence that a
request was constructed and that WitnessGlass never saw what became of it.

**Requested input and effective input are different fields.** `requested_input` on a
request, `effective_input` on a completion or failure. Claude documents that a request can
be rewritten before execution, so collapsing the two would destroy the only evidence that
what ran was not what was asked for.

**Denial is not failure.** `tool_denied` carries no error and no effective input, because
nothing executed and no error occurred. Filing a denial as a failure would make "the agent
was stopped" indistinguishable from "the agent tried and it broke".

### Reported intent

Claude's `Bash` tool input carries a `description` field the agent writes about its own
intentions — a claim, sitting inside the same payload as the command that was actually run.
When a `PreToolUse` payload's `tool_input` contains a non-blank top-level `description`
string, the adapter emits a **second, separate** record on the `reported` channel,
correlated by `tool_use_id`, with `mechanism` = `command-hook:PreToolUse#tool_input.description`.

The description is **duplicated, not moved**: the full `requested_input` is preserved whole
as source-delivered evidence. The duplication is deliberate and is recorded here because a
reader counting occurrences of a string across a recording needs to know it is there twice.

Nothing else produces intent. A command, a path, a prompt, a tool name, a result, and
temporal adjacency are all *not* the agent saying anything, and none of them is ever
promoted into a reported record.

### Agent identity

`SubagentStart.agent_id` names the **child** — the subagent being started — so it is filed
in the event payload, not in the envelope's `context.agent_id`, which would claim it was the
identity of the agent that emitted the event.

Where Claude supplies `parent_agent_id` / `parent_agent_type`, they are recorded exactly as
delivered. Where it does not, the fields are **absent and stay absent**. No root agent id,
no parent id, no span id, and no hierarchy is ever synthesized, and parentage is never
inferred from timing or from adjacency in the recording. Preserving a supplied identifier
and refusing to invent a missing one are the same rule, applied in both directions.

### Passivity

The adapter is passive by construction, not by convention:

- It prints **nothing** to stdout on success. Claude reads a hook's stdout for permission
  decisions, `updatedInput`, `updatedToolOutput`, and `additionalContext`; writing nothing
  there is what makes influence impossible rather than merely unintended.
- It **never exits 2**, the code that blocks a `PreToolUse` call. Only 0 or 1, and this is
  tested.
- It **never reads the transcript**, even though `transcript_path` is in every payload.
- It **never executes or interpolates any value** from the payload. Every value is either
  compared against a fixed set of names or stored as opaque JSON.
- Failures go to stderr with exit 1, which Claude documents as non-blocking for all eight
  configured hooks. A broken recorder stops recording; it does not stop the session.

### Boundary strictness

Unknown JSON fields in a hook payload are **ignored**. Claude adds fields over time, and
rejecting an unrecognized one would mean a harmless upstream addition silently switched off
recording for every session on the host. The strictness lives on the record written out,
which does reject unknown fields.

An unknown `hook_event_name` is **refused**, not guessed at. Inventing a meaning for an
unrecognized lifecycle point would put evidence in a recording that nothing generated.

A `session_id` is validated against `[A-Za-z0-9_-]`, non-empty, at most 128 bytes, before
becoming `<session-id>.ndjson`. `.` is excluded from the set entirely, which makes `.` and
`..` unrepresentable and removes the path-traversal question rather than answering it. An id
outside that set is refused loudly rather than escaped by an encoding whose inverse nobody
has defined.

### Ordering under parallel hooks

`sequence` is the recorder's **acquisition** order and the canonical storage order of the
recording. Because Claude runs matching hooks in parallel, it is *not* automatically a total
causal order for Claude's execution: two calls that complete concurrently land in whichever
order their hook processes won the file lock. Per-call correlation by `tool_use_id` and the
supplied `duration` can support a derived causal view. Raw replay never reorders.

---

## 3. What remains unmeasured until task:4

Everything in this section is a **provisional blind spot**. None of it has been measured
against a live session, and none of it may be described as characterized until it has been.

- **Pre-tool evidence is a request, not proof of execution.** A `tool_requested` record
  with no matching completion means WitnessGlass did not see what happened next. It does
  not mean nothing happened, and it does not mean the call was blocked.
- **Completed hooks expose Claude's tool-level input and response, not descendant
  syscalls.** A `Bash` call's record shows the command and the tool's reported output. What
  the shell actually spawned, wrote, or read is not observed at all. There is no process
  tracing here and none is planned for v0.
- **Validation failures can escape the selected lifecycle hooks.** A request rejected by
  input validation may fire neither `PreToolUse` nor `PostToolUseFailure`, leaving no trace
  in the recording whatsoever.
- **Permission-denial coverage depends on this host and version.** `PermissionDenied` is
  documented as firing when a call is denied *by the auto mode classifier*. Whether a denial
  at an interactive permission prompt, under a different permission mode, or via a deny rule
  also fires it is unknown here. Absence of `tool_denied` records must not be read as
  absence of denials.
- **`@` file references may bypass `Read` tool hooks.** File content can enter a session
  without any tool event, so a recording can be missing files the session demonstrably read.
- **Root and nested-agent parentage are not invented when ids are absent.** A recording of a
  session with subagents may therefore contain no expressible hierarchy at all. That is the
  correct output, not a gap to fill.
- **Appender order under parallel hooks is recorder order.** See above. Do not read
  `sequence` as proof that one tool call happened before another.
- **The hook adds synchronous latency.** Eight hook surfaces are configured as synchronous
  command hooks, each a process spawn plus a lock-protected append. `async: true` was
  deliberately not used: during first contact, a complete recording and a visible failure
  matter more than shaving hook latency. The cost has not been measured.
- **Recordings remain sensitive and unsafe to share.** A recording contains prompts,
  commands, absolute paths, file contents, tool output, and any credential that passed
  through any of them. Nothing is redacted. See dragon:2.

Also deliberately **not captured** in this slice, and therefore invisible in any recording:
`UserPromptSubmit`, streamed assistant messages, transcript contents, `SessionStart.model`,
`SubagentStop.last_assistant_message` and `stop_reason`, `permission_mode`, `cwd`, and every
other lifecycle hook Claude offers.

---

## Activating it

**Not active by default, and a clone records nothing.** Claude reads `.claude/settings.json`
and `.claude/settings.local.json`; both are gitignored here. Only the inert example
`.claude/settings.witnessglass.example.json` is committed.

```sh
# 1. Build first. The hooks invoke the built binary, not `cargo run`.
cargo build

# 2. Arm recording.
cp .claude/settings.witnessglass.example.json .claude/settings.local.json
```

Then start a **fresh** Claude session. Arming mid-session produces a partial recording with
no `SessionStart`, which is worse than no recording for a first-contact experiment.

Recordings appear at `.witnessglass/recordings/<session-id>.ndjson` (gitignored). Replay one
with:

```sh
witnessglass replay --recording .witnessglass/recordings/<session-id>.ndjson
```

To disarm:

```sh
rm .claude/settings.local.json
```

Scoped to macOS and Linux. The configured command path uses forward slashes and no
extension; **Windows is untested and is not claimed to work.**

## Exercising it without Claude

The adapter reads one hook payload from stdin, so it can be driven directly with synthetic
input:

```sh
echo '{"hook_event_name":"SessionStart","session_id":"synthetic-0001","source":"startup"}' \
  | witnessglass claude-hook --recordings-dir /tmp/wg-synthetic
```

Silence and exit 0 mean it worked.
