# The Claude Code adapter

WitnessGlass records a Claude Code session through Claude's own **command hooks**. This
document states four things separately, and they must stay separate:

1. what Claude's current documentation promises;
2. what this implementation maps;
3. what one real recorded session **measured**;
4. what remains unmeasured.

Section 4 is not a disclaimer. It is the honest scope of every claim in sections 1 and 2 that
section 3 has not yet reached.

The adapter **has now been run against one live Claude Code session** — one session, one
macOS host, one Claude Code version — and section 3 states what that produced. Anything not
in section 3 and not marked as tested is read off documentation, not measured.

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

## 3. What one real session measured

Scope of the measurement: **one** session, recorded end to end on **one** macOS host running
Claude Code **2.1.220**, doing ordinary repository work for 17 minutes and producing 234
records. Everything here is what that recording demonstrably contained. Nothing here
generalizes to another host, another version, or another kind of session, and each item says
what it saw rather than what it concludes. See task:4 for the full comparison, including the
places where the recording and the session's own self-report disagree.

**The recording survived and is structurally intact.** 234 records, schema v2 throughout,
`sequence` 1..234 with no gap, duplicate, or decrease, `recorded_at` monotonic, final newline
present. `scripts/check-recording.sh` returned 0. The session ended through an interactive
exit and the configuration was removed underneath it afterwards; neither damaged the file.

**Both session boundaries were captured, including the exit.** One `session_started` with
`source: "startup"` and one `session_ended` with `reason: "prompt_input_exit"`. The exit is
not a blind spot on this path.

**Tool lifecycle pairing was complete.** 82 requests, 82 successful completions, correlated
by `tool_use_id` with zero unmatched records in either direction.

**`requested_input` and `effective_input` were identical in all 82 cases.** Claude rewrote no
input in this session. The distinction is preserved and was not exercised.

**A subagent's own tool calls are visible, and are attributable to it.** This was the open
empirical question. The recorded subagent was not an opaque pair of boundary events: its 27
tool calls produced 81 records — request, reported intent, and completion — each carrying
`context.agent_id` and `context.agent_type` identifying the child. A subagent's work is
recorded at the same fidelity as its parent's and can be separated from it by identifier
rather than by adjacency.

**`parent_agent_id` and `parent_agent_type` did not arrive.** Documented, and absent on every
subagent record in this session. The adapter recorded what was supplied and invented nothing,
so the recording contains **no expressible link** between a subagent and the tool call or
agent that spawned it. A causal parent/child overlay is not buildable honestly from a
recording like this one.

**A `subagent_stopped` arrived with no `subagent_started`.** One of the two stop records named
an agent id that appears nowhere else in the recording, with an empty `agent_type`, and no
tool call attributable to it. Subagent boundary events are not guaranteed to pair.

**`prompt_id` arrived populated** on every record except `session_started`, which carried no
`context` at all — matching the documented "absent until the first input". It is populated but
**unanchored**: `UserPromptSubmit` is not captured, so nothing in a recording says what a
`prompt_id` delimits. Only two distinct values appeared in the whole session — one covering
232 records including every tool call by both the parent agent and the subagent, and one on
`session_ended` alone. Do not segment a recording by it, and do not describe a recording as
containing N turns. See dragon:3.

**The hook-level `duration` never arrived.** Documented as optional on `PostToolUse`; supplied
zero times in 82 completions, so `duration_ms` is absent on every record. This was later
confirmed by inspection rather than by field count: the adapter reads the documented key, its
payload struct is strict about types so a malformed value would have failed the translation
outright rather than silently, and a recursive scan of every captured payload found no
hook-level duration anywhere. The absence is the integration's, not this adapter's.

**One tool self-reports a duration in its response body, and the adapter deliberately does not
lift it.** The single `Agent` completion carries `totalDurationMs` inside its `tool_response`,
alongside `totalTokens`, `totalToolUseCount`, `toolStats`, and `resolvedModel`. `Bash`, `Read`,
`Write`, and `Edit` carry nothing of the kind. That value is preserved exactly where it arrived —
inside `response`, as delivered — and is **not** promoted into the envelope's `duration_ms`.
Promoting it would make one tool appear to have hook-level timing it does not have, and would
make `duration_ms` mean two different things depending on which tool a reader is looking at.
A reader wanting it must go to the response payload and know which tool produced it.

Either way, a derived view intending to use per-call tool duration has no usable input from this
path: one completion in 82, on one tool. See dragon:1 for the full follow-up.

**Reported intent behaved exactly as documented, and the duplication is real.** 65
`reported_intent` records, all on the `reported` channel, from 64 `Bash` calls and one `Agent`
call. In all 65, the reported text was identical to the `description` still present in the
same call's `requested_input`: a reader counting occurrences of such a string across a
recording will find it exactly twice. 17 calls carried no `description` and produced no
reported record.

**A `Bash` command's file mutations are invisible, demonstrated rather than asserted.** The
recorded session wrote content into a tracked repository file with a shell redirect and
reformatted another file with a formatter run from `Bash`. The recording contains the commands
and the tools' reported output and **no mutation event for either file**. Reconstructing
"which files did this session change" from tool events alone gives the wrong answer. This is
the general limit made concrete: completed hooks expose Claude's tool-level input and response,
never what a descendant process spawned, wrote, or read. There is no process tracing here and
none is planned for v0.

**Tool-level evidence must be read with its arguments.** The recording contains an execution of
`scripts/arm.sh` that was a `--help` invocation during a test of help-flag behaviour. Matching
on command name alone would have produced a false claim that the session armed the repository.

**One number for recorder cost, and it is a narrow one.** Within a single `PreToolUse` hook
process, the interval between its two appends — one complete lock / tail-scan / write /
`sync_data` transaction — was median 5.0 ms across 65 samples (min 3.3, p90 5.9, max 8.4). This
excludes process spawn, JSON parsing, and Claude's own overhead. Total hook latency is still
unmeasured.

---

## 4. What remains unmeasured

Everything in this section is a **provisional blind spot**. None of it has been measured
against a live session, and none of it may be described as characterized until it has been.
Where the one recorded session left a surface deliberately unexercised, it says so — an
unexercised surface is not a working one.

- **Pre-tool evidence is a request, not proof of execution.** A `tool_requested` record
  with no matching completion means WitnessGlass did not see what happened next. It does
  not mean nothing happened, and it does not mean the call was blocked. *Unexercised: the
  recorded session produced zero unmatched requests, so what one looks like in practice, and
  under what conditions one arises, is still unknown.*
- **Failure capture is unexercised.** `PostToolUseFailure` did not fire once in the recorded
  session, because no tool call failed. Zero `tool_failed` records is not evidence that the
  surface works, and `interrupted` has never been observed at all.
- **Behaviour under interruption and abnormal termination is unmeasured.** The one recorded
  session ended cleanly through a documented exit. Nothing here says what a recording looks
  like after a crash, a kill, or a cancelled tool call.
- **Whether a resumed session appends to the same recording is unmeasured.** The recorded
  session produced one `session_started` with `source: "startup"` and nothing after its
  `session_ended`. No resume with hooks armed has been observed.
- **Parallel dispatch is not distinguishable from serial dispatch.** The recorded session
  reports having issued tool calls in parallel batches; the recording contains no overlapping
  tool-call spans, no non-monotonic timestamps, and no interleaving of any kind. A batch whose
  hooks serialize is indistinguishable in the record from a sequence of separate calls. So the
  ordering caveat below is still a documented hazard rather than a demonstrated one — and the
  absence of parallel evidence is itself a coverage gap, not a finding that nothing ran in
  parallel.
- **Validation failures can escape the selected lifecycle hooks.** A request rejected by
  input validation may fire neither `PreToolUse` nor `PostToolUseFailure`, leaving no trace
  in the recording whatsoever.
- **Permission-denial coverage depends on this host and version.** `PermissionDenied` is
  documented as firing when a call is denied *by the auto mode classifier*. Whether a denial
  at an interactive permission prompt, under a different permission mode, or via a deny rule
  also fires it is unknown here. Absence of `tool_denied` records must not be read as
  absence of denials. *Unexercised: no denial was provoked in the recorded session, so it
  tests this neither way.*
- **`@` file references may bypass `Read` tool hooks.** File content can enter a session
  without any tool event, so a recording can be missing files the session demonstrably read.
- **Appender order under parallel hooks is recorder order.** See above. Do not read
  `sequence` as proof that one tool call happened before another.
- **Total hook latency is unmeasured.** Eight hook surfaces are configured as synchronous
  command hooks, each a process spawn plus a lock-protected append. `async: true` was
  deliberately not used: during first contact, a complete recording and a visible failure
  matter more than shaving hook latency. One recorded session bounded the append transaction
  alone at a median of 5.0 ms (section 3); process spawn, parsing, and Claude's own overhead
  are not in that number and have not been measured.
- **Recordings remain sensitive and unsafe to share.** A recording contains prompts,
  commands, absolute paths, file contents, tool output, and any credential that passed
  through any of them. Nothing is redacted. See dragon:2. *This is now measured rather than
  feared: one 17-minute session of ordinary repository work produced 580 KB, of which 58% was
  tool response bodies and 24% was tool input, with the host's home-directory path present in
  a quarter of all records.*

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
./scripts/arm.sh
```

That rebuilds the binary — the hooks invoke it directly rather than through `cargo run`, so a
stale build would quietly record a real session using old code — then runs the adapter
against a synthetic payload in a throwaway directory and **refuses to arm** if it fails, or
if it writes anything to stdout. Only then does it copy the example into
`.claude/settings.local.json`.

Then start a **fresh** Claude session. Arming mid-session produces a partial recording with
no `SessionStart`, which is worse than no recording for a first-contact experiment.

Recordings appear at `.witnessglass/recordings/<session-id>.ndjson` (gitignored). Replay one
with:

```sh
witnessglass replay --recording .witnessglass/recordings/<session-id>.ndjson
```

### Checking a recording without reading it

`replay` prints every record, which makes it the wrong tool for the first question anyone has
about a fresh recording: did the recorder survive the session? `scripts/check-recording.sh`
asks exactly that and throws the answer's body away.

```sh
./scripts/check-recording.sh .witnessglass/recordings/<session-id>.ndjson
```

It runs the same `replay`, so there is still exactly one implementation of what a recording
says, and it preserves replay's exit status: **0** complete, **2** a valid prefix with a
truncated tail, **1** corruption, an unreadable or missing recording, an invalid invocation, a
missing binary, or replay reaching no verdict at all. Replay's NDJSON stdout is discarded
whole; its payload-free summary stays on stderr.

Payload-silent means event bodies reach neither stream, and it has **one measured exception**.
Ordinary diagnostics — line numbers, byte offsets, schema versions, sequence numbers, session
ids — are not payloads and are not hidden. But a *corrupt* record's diagnostic comes from the
parser itself and can quote the bytes it rejected, and those bytes may be part of a payload: a
record whose `sequence` holds a string produces `invalid type: string "…"` with the string in
full. A recording that checks as corrupt is therefore the one not to check on a shared
terminal. `tests/check_recording.rs` pins that limit with a test asserting the leak, so it
cannot widen unnoticed; suppressing the diagnostic in the script instead would make the check a
second opinion about what a recording says, which is the one thing it must not become.

Checking never alters the recording, and never arms, disarms, or builds anything. It does not
make a recording safe to share — nothing is redacted, and the warnings above stand unchanged.

To disarm:

```sh
./scripts/disarm.sh
```

### What the scripts guarantee

Re-running `arm.sh` while already armed disarms first and re-arms from scratch, so "armed"
always means armed with the current build and the current example. A deleted sentinel does
not strand an armed configuration: `arm.sh` recognises its own settings file and cleans up
regardless.

`arm.sh` writes a sentinel at `.witnessglass/armed`. It is deliberately **not** a second copy
of "am I armed" — `.claude/settings.local.json` is already that, and a duplicate flag would
only drift from it. It is a record of what arming *did*: the binary and its SHA-256, the hash
of the settings file as written, and whether a pre-existing settings file was displaced. That
is what lets disarming undo exactly what arming did.

Two rules cover the destructive edges:

- **`disarm.sh` never deletes a file it did not write byte-for-byte.** It removes the settings
  file only when it matches either the sentinel's recorded hash or the committed example. An
  edited configuration is moved to `.claude/settings.local.json.disarmed.<timestamp>` instead,
  and a settings file that is not a WitnessGlass configuration at all is left exactly where it
  is.
- **Recordings survive a disarm.** Disarming stops recording; it does not discard evidence
  already captured. `disarm.sh` reports how many recordings are being kept, and that they are
  not safe to share.

Both scripts are covered by `tests/arm_disarm.rs`, which exercises them against a throwaway
directory shaped like the repository. The test suite never arms the real repository.

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
