# The Claude Code adapter

WitnessGlass records a Claude Code session through Claude's own **command hooks**. This
document states four things separately, and they must stay separate:

1. what Claude's current documentation promises;
2. what this implementation maps;
3. what each recorded session **measured**, one subsection per session;
4. what remains unmeasured.

Section 4 is not a disclaimer. It is the honest scope of every claim in sections 1 and 2 that
section 3 has not yet reached.

The adapter **has now been run against three live Claude Code sessions**, on one macOS host,
across two Claude Code versions, and section 3 states what each produced separately rather
than merged into a single verdict. Anything not in section 3 and not marked as tested is read
off documentation, not measured.

| | when | version | shape | what it produced |
| --- | --- | --- | --- | --- |
| **first contact** | 2026-08-02 | 2.1.220 | 17 minutes of ordinary repository work | 234 records, no failure, no denial, no interruption |
| **pass 2** | 2026-08-03 | 2.1.220 | hostile, two turns, probe on 3 hooks | 40 records, 2 deliberate failures, 14 raw payloads |
| **pass 3** | 2026-08-04 | 2.1.221 | hostile, two turns, probe on 5 hooks | 39 records, 2 deliberate failures, 1 refused call, 16 raw payloads |

Reference used: <https://code.claude.com/docs/en/hooks>, **re-read 2026-08-04**. The previous
reading of it was 2026-08-02, and section 1 records where the two readings differ. Claude Code
versions this document's measurements come from: **2.1.220** and **2.1.221**.

**Documentation agreeing with an observation is not the observation's evidence.** Several
things this project measured the hard way are now stated outright in the reference. Where that
happens it is written down as two independent facts that agree, because a reference can be
wrong — this adapter dropped the delivered timing field for two sprints on the strength of one
that was.

---

## 1. What Claude's documentation promises

### The hooks this adapter subscribes to

As of the 2026-08-04 reading:

| Hook | Documented firing condition | Documented payload beyond the common fields |
| --- | --- | --- |
| `SessionStart` | A session begins or resumes | `source` ∈ {`startup`, `resume`, `clear`, `compact`, `fork`}, optional `model` — the only hook documented to receive `model`, and "not guaranteed to be present" |
| `PreToolUse` | After the model constructs a tool request, **before the call is processed** | `tool_name`, `tool_input`, `tool_use_id` |
| `PostToolUse` | After a tool call **succeeds** | `tool_name`, `tool_input`, `tool_use_id`, `tool_response`, optional `duration_ms` |
| `PostToolUseFailure` | After a tool that **started executing** fails | `tool_name`, `tool_input`, `tool_use_id`, `error`, optional `is_interrupt`, optional `duration_ms` |
| `PermissionDenied` | When the **auto mode classifier** denies a tool call | `tool_name`, `tool_input`, `tool_use_id`, `reason` |
| `SubagentStart` | A subagent is spawned via the Agent tool | `agent_id`, `agent_type` |
| `SubagentStop` | A subagent finishes responding | `stop_hook_active`, `agent_id`, `agent_type`, `agent_transcript_path`, `last_assistant_message`, `background_tasks`, `session_crons` |
| `SessionEnd` | A session terminates | `reason` ∈ {`clear`, `resume`, `logout`, `prompt_input_exit`, `bypass_permissions_disabled`, `other`} |

Common fields documented across hooks include `session_id`, `hook_event_name`,
`transcript_path`, `cwd`, `permission_mode`, `effort`, an optional `prompt_id`, and — inside a
subagent — `agent_id` and `agent_type`.

`duration_ms` is documented as "tool execution time in milliseconds", explicitly **excluding**
time spent in permission prompts and `PreToolUse` hooks. So it is not wall-clock time from
request to completion, and a projection must not present it as one.

#### What changed between the 2026-08-02 reading and the 2026-08-04 one

The reference is not a fixed target, and four of these differences matter:

- **The timing and interruption fields are now documented under the names that are actually
  delivered** — `duration_ms` and `is_interrupt`, where the earlier reading gave `duration` and
  `interrupted`. This adapter believed the earlier reading for two sprints and dropped both
  fields unread. The reference now agrees with what the probe observed; the observation is what
  established it.
- **`PermissionDenied`'s firing condition is now explicit about what it excludes**: "This hook
  only fires in auto mode: it doesn't run when you manually deny a permission dialog, when a
  `PreToolUse` hook blocks a call, or when a `deny` rule matches." Pass 3 measured exactly that
  and measured it first. It also documents a `reason` field, which no session here has seen.
- **`parent_agent_id` and `parent_agent_type` no longer appear in the reference at all.**
  The 2026-08-02 reading recorded them as documented-and-optional on `SubagentStart`, and this
  document's table said so. They are absent from the current text, and pass 3 observed neither
  on any raw payload. Documented-then-undocumented is not the same as never-documented, and the
  earlier reading is left recorded above rather than erased.
- **`prompt_id` carries a stronger claim than "absent until the first input"**: "UUID
  identifying the user prompt currently being processed. Matches the `prompt.id` attribute on
  OpenTelemetry events… Requires Claude Code v2.1.196 or later." That is a claim about meaning,
  not just availability. It is a new claim to test, not a resolution of dragon:3 — see the note
  there.

Two more current statements are worth recording because they corroborate blind spots this
project found by running into them:

- `PostToolUseFailure` "doesn't fire for tool calls rejected before execution: an unknown tool
  name, input that fails schema or tool-specific validation, or a permission denial. Validation
  rejections … fire neither `PreToolUse` nor `PostToolUseFailure`." Pass 2 discovered this with
  a `sleep` that vanished from the recording *and* from the independent probe.
- "Permission denials fire `PreToolUse` but not this event." That is the shape of the dangling
  request pass 3 recorded: a `tool_requested` with no terminal record of any kind.
- `permission_mode`'s documented values are `default`, `plan`, `acceptEdits`, `auto`,
  `dontAsk`, `bypassPermissions`, and "the mode labeled **Manual** arrives as `default`, never
  as `manual`". Anyone reading a captured `permission_mode` should know that before concluding
  which mode a session ran in.

**The table above is the documentation, and the documentation is not the wire.** Where this
document says "documented", read it as a claim about the reference; where it says "observed" or
"delivered", read it as a claim about payloads this project has actually captured. Only the
second kind is evidence, and the second kind has twice been the thing that corrected the first.

### The four documented facts that determined the design

**A pre-tool payload is a request, not an execution.** `PreToolUse` fires after the model
constructs a tool request and before the call is processed. The request may then be
modified (a hook may return `updatedInput`), denied, escalated, deferred, or never executed
at all.

**Completion, failure, and denial are three separate events.** `PostToolUse` fires only on
success and carries the input *actually sent* plus the response. `PostToolUseFailure`
carries the effective input, an error, and an optional `is_interrupt` flag. `PermissionDenied`
fires on a classifier denial, where nothing executed — and, as the current reference now says
and pass 3 measured, on nothing else. A human refusing a permission prompt fires no hook of
any kind.

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

**`tool_denied` has never been reachable by the route a human would take.** The mapping is
implemented and tested against synthetic payloads, and no live session has produced one:
pass 3's interactive refusal fired no `PermissionDenied` hook at all (§3.3). The row above
says what the adapter would record if the hook fired, not that anything has ever fired it.

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

That leniency has a cost, and this project paid it: the delivered `duration_ms` was dropped
unread for two sprints because the adapter modelled `duration`. Silence is the correct default
and a bad permanent posture, so the adapter now names what it drops.

**Every top-level payload field is accounted for in one of two places**, both compile-time:

- the `HookPayload` struct, for fields that reach a record;
- a `DELIBERATELY_UNRECORDED` list, for fields **seen on a captured payload** and dropped on
  purpose — `cwd`, `transcript_path`, `agent_transcript_path`, `permission_mode`, `effort`,
  `last_assistant_message`, `stop_hook_active`, `background_tasks`, `session_crons`. Each entry
  carries its reason. Most are privacy (CLAUDE.md §5); `permission_mode` is a gap dragon:1
  argues should be closed by a schema decision.

**A field is listed only after being observed, never on the strength of documentation.** The
first version of that list broke the rule twice, adding `model` and `stop_reason` from the
hooks reference. Pass 3 then captured both `SubagentStop` payloads and `stop_reason` was not on
either — the same class of error as `duration`, committed inside the list written to prevent
it. Both entries were removed. A documented field nobody here has seen belongs in neither list,
so the canary fires the first time it arrives; `model` on `SessionStart` and `reason` on
`PermissionDenied` are currently in exactly that position.

`--strict-json-validation` refuses any payload carrying a field in neither list, naming the
fields. `WITNESSGLASS_STRICT_JSON=1` does the same and is the usable form, because a hook is
spawned by Claude from a settings file rather than by a human with a command line.

**It is a canary for one session, not a setting to leave on.** A refused payload is a record
that was never written, which is exactly the failure the lenient default exists to prevent. Run
it deliberately to ask "has the wire moved since anybody looked", read the answer, turn it off.
It has fired in earnest once: pass 3's two `SubagentStop` payloads carried four fields nobody
had looked at, and were refused.

#### What strict mode detects, exactly

Its granularity is one thing, and it is narrower than "the wire has changed":

> A **top-level field name** on a payload that is in neither the struct nor
> `DELIBERATELY_UNRECORDED`.

That is all. It does **not** detect:

- a field it already knows **moving to a different hook** — `duration_ms` appearing on
  `SubagentStop`, say, would be accounted for and silent;
- an optional field **disappearing**, because a field that stops arriving is indistinguishable
  from one that was never sent on that payload;
- a known field **changing shape** — a `duration_ms` delivered as a string or an object would
  fail deserialization as a type error, not as drift, and a `tool_input` whose interior schema
  changed is opaque JSON either way;
- a known field **becoming populated** where it was always empty, which is precisely the case
  dragon:2 is watching for `background_tasks` and `session_crons`;
- anything **nested** below the top level, anywhere.

Widening any of that means a schema model of somebody else's payloads, maintained by hand,
which is the thing `#[serde(flatten)]` exists here to avoid. It is not attempted, and this
paragraph is the statement of what a quiet canary is therefore worth.

Strict mode would have caught the `duration_ms` mismatch on day one — the adapter modelled
`duration`, so the delivered `duration_ms` was in neither list, and there is a test asserting
exactly that. What actually caught it was `scripts/probe.sh`, two sprints later. The two are
complements rather than substitutes:

| | strict mode | the probe |
| --- | --- | --- |
| asks | has the wire moved past *this adapter's model*? | what is on the wire? |
| reports | the names of fields the adapter cannot account for | every key, per hook, with counts |
| costs | the records it refuses | captured payloads as sensitive as a recording |
| fails independently of | nothing; it *is* the adapter's model | the adapter — it shares no code, no field model, and no translation with it |
| fails when | the adapter is stale | a payload is lost, a hook is not installed on, or `show` cannot parse a capture |

That last row is the correction this document owes a reader: it previously said the probe
"fails when: never; it has no model to be wrong". **Independent failure modes are not absent
ones.** The probe can be installed on the wrong hooks and capture nothing, lose a payload if a
hook process is killed mid-write, and — before the spool — corrupt its own capture when two
large payloads were appended concurrently. What it cannot do is fail in the *same way* the
adapter fails, and that is the entire property it was built for.

`scripts/probe.sh show` deserves the same care: it is **a parser and a summary, not raw
evidence**. It has a model of the payload, and a payload it cannot parse is reported by name
and count rather than folded in. The raw evidence is the captured files. Before drawing a
conclusion from a probe run — especially a negative one, of the form "the integration never
sent X" — check that the capture is complete and parseable: `show` prints the number of
completed captures, the number of incomplete ones left by hooks that did not finish, and every
capture that failed to parse. A negative finding taken from a capture with holes in it is
exactly the kind of confident wrong answer this project already published once.

Each captured payload is **one file** under `.witnessglass/probe/payloads/`, written under
`payloads/incomplete/` and renamed into place once stdin has been consumed — so a file in the
spool is a whole payload and a file left in `incomplete/` is a partial one, distinguishable
without parsing either. It is one file per invocation rather than one line appended to a shared
capture because Claude runs matching hooks in parallel: measured on this host, eight concurrent
512 KiB payloads appended with `cat >>` produced **four lines in total — two that parsed and two
that did not — leaving six of the eight payloads unrecoverable**. `tests/probe_capture.rs` runs the same
concurrent shape against the spool and asserts every capture is independently parseable, byte
for byte what the hook was handed, and that `show` prints none of it.

Strict mode is cheap enough to run on a whole session and tells you *that* something moved. The
probe tells you *what*. Neither is a substitute for reading a recording.

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
supplied `duration_ms` can support a derived causal view. Raw replay never reorders.

---

## 3. What the recorded sessions measured

Three sessions, on one macOS host, across two Claude Code versions. **Each is reported under
its own heading, and none of them is edited to agree with a later one.** A measurement taken
during first contact is a fact about that session and that adapter build; where a later session
measured the same thing differently, the later result is added beside it and the difference is
named. Nothing here generalizes to another host, another version, or another session shape.

### 3.1 First contact — 2026-08-02, Claude Code 2.1.220, 234 records

Scope of the measurement: **one** session, recorded end to end on **one** macOS host running
Claude Code **2.1.220**, doing ordinary repository work for 17 minutes and producing 234
records. Everything here is what that recording demonstrably contained. Each item says what it
saw rather than what it concludes. See task:4 for the full comparison, including the places
where the recording and the session's own self-report disagree.

**This session was recorded by an adapter now known to be defective in one respect** — it read
`duration`, and the integration sends `duration_ms` — so the timing result below is a fact
about this recording rather than about the integration. It is left as measured.

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

**`parent_agent_id` and `parent_agent_type` did not arrive.** Documented at the time, and
absent on every subagent record in this session. The adapter recorded what was supplied and
invented nothing, so the recording contains **no expressible link** between a subagent and the
tool call or agent that spawned it. A causal parent/child overlay is not buildable honestly
from a recording like this one.

*Measured here from adapter output, which cannot tell a withheld field from a dropped one.
Confirmed independently in §3.3, upstream of the adapter, on the hooks where parentage would
appear.*

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

**The hook-level duration is absent on all 82 completions, and that was this adapter's fault.**
The measurement was reported twice as an integration coverage gap, and confirmed once by an
inspection that could not have detected the actual cause: the adapter read `duration`, the
integration sends `duration_ms`, unknown fields are ignored by design, and the confirming scan
ran over the recording — which is the adapter's own output, downstream of the field selection
under test. A later session captured raw payloads with an independent probe and found
`duration_ms` populated on **every** completion of both hooks that fired. The 82 completions
here are unrecoverable; nothing retroactively fills them, and they stay absent. See dragon:1.

**One tool self-reports a duration in its response body, and the adapter deliberately does not
lift it.** The single `Agent` completion carries `totalDurationMs` inside its `tool_response`,
alongside `totalTokens`, `totalToolUseCount`, `toolStats`, and `resolvedModel`. `Bash`, `Read`,
`Write`, and `Edit` carry nothing of the kind. That value is preserved exactly where it arrived —
inside `response`, as delivered — and is **not** promoted into the envelope's `duration_ms`.
Promoting it would make one tool appear to have hook-level timing it does not have, and would
make `duration_ms` mean two different things depending on which tool a reader is looking at.
A reader wanting it must go to the response payload and know which tool produced it.

Either way, a derived view intending to use per-call tool duration has no usable input from
**this recording**: one completion in 82, on one tool. That is a fact about a recording written
by a defective adapter, not about the integration. See dragon:1 for the full follow-up.

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

### 3.2 Pass 2 — 2026-08-03, Claude Code 2.1.220, 40 records, 14 raw payloads

The first deliberately hostile session, run to the protocol in
[`hostile-recording.md`](hostile-recording.md), with `scripts/probe.sh` capturing raw payloads
from `PostToolUse`, `PostToolUseFailure`, and `PermissionDenied` alongside the recording. Two
turns, submitted separately. Findings in dragon:1, dragon:2, and dragon:3.

**Failure capture is exercised, in both shapes.** A non-zero shell exit (`cat` on a missing
file) and a tool-level error (`Read` on a missing file) both arrived on `PostToolUseFailure`,
neither on `PostToolUse`. The hook does not distinguish them; the `error` string is the only
discriminator. It arrives as delivered, **including terminal colour escape sequences** — the
error inside the shell failure is wrapped in a red-foreground SGR sequence — so a renderer
treating `error` as plain text will show escape codes to a reader.

**The delivered timing field was found, upstream of the adapter.** All 12 `PostToolUse` and
both `PostToolUseFailure` raw payloads carried a populated top-level `duration_ms`. The adapter
was reading `duration` and discarding it; the same mismatch existed for `is_interrupt` against
`interrupted`. Both fixed in `1b655ac`. This is the measurement that established the timing
field arrives at all, and it was taken by an observer sharing no code with the adapter.

**A call the harness refuses before dispatch is invisible to every hook.** A `sleep 120` was
rejected at the tool level and produced no `PreToolUse`, no failure, and no denial — **not in
the recording and not in the independent probe**. A recording cannot distinguish "the agent
tried something the harness would not run" from "the agent did not try". This was not one of
the session's five questions; it is the most useful thing the session found.

**Denial was not staged.** Every payload reported `permission_mode: "auto"` and the deletion the
protocol expected to be refused was auto-approved with no prompt. `PermissionDenied` was armed
and did not fire, which measures nothing about denial — only about the mode the session ran in.

**`prompt_id` changed at the turn boundary**, one value per turn, exactly where the human
pressed enter — reproduced in pass 3 on a different version. Necessary for a turn identifier and
not sufficient. What a reader may take from it, at full strength:

> A reader may conclude that two records with **different** `prompt_id` values were not produced
> by the same submission. A reader may **not** conclude that two records sharing one belong to
> the same turn, that a recording contains as many turns as it has distinct values, or that any
> span between changes is a unit of work.

The current reference calls it "a UUID identifying the user prompt currently being processed",
which is a stronger claim than anything measured here and does not explain first contact's
`session_ended` carrying a value no other record carried. It is a claim to test. dragon:3 is
open, and no projection segments by this field.

**`SubagentStop` fired three times against one `SubagentStart`** — once for the real subagent,
and once near the end of each turn with an `agent_id` seen nowhere else and an empty
`agent_type`. Recorded as an observation. It is not a turn boundary and must not become one.

### 3.3 Pass 3 — 2026-08-04, Claude Code 2.1.221, 39 records, 16 raw payloads

Run to [`hostile-recording-pass-3.md`](hostile-recording-pass-3.md), in `manual` permission
mode, with the probe extended to five hooks including both subagent hooks for the first time.
Note the version: pass 2 ran on 2.1.220 and Claude updated in between, so every pass-2 to
pass-3 comparison is cross-version.

**An interactive denial fires no hook at all.** The operator was prompted for
`rm -rf /tmp/wg-probe/scratch` and refused it. No `PermissionDenied` payload reached the probe,
which was armed for it; no record of any kind reached the recording. The arithmetic states it
exactly: **14 `tool_requested`, 11 `tool_succeeded`, 2 `tool_failed` — one request with no
terminal record.** From inside the file, a denied call is indistinguishable from one that was
interrupted, one that crashed the harness, and one that is still running. Two consequences,
and they are separate:

- the **event** is unobservable, measured upstream of the adapter, so it is the integration's
  boundary rather than this adapter's defect;
- the **absence** is unrenderable, because `permission_mode` — the one field that would tell a
  reader denials were even possible — is dropped. dragon:1 carries the argument for a schema
  decision about it.

The current reference now says the same thing about the hook's scope (§1). The observation came
first and does not depend on it.

**A denial ends the turn**, so the interruption the same protocol asked for never started.
Interruption is now unmeasured for a third time and a third distinct reason, and a denial and
an interruption cannot be staged in one turn.

**Parentage is genuinely absent, confirmed independently.** `SubagentStart` on 2.1.221 carried
exactly `agent_id`, `agent_type`, `cwd`, `hook_event_name`, `prompt_id`, `session_id`,
`transcript_path` — with **zero occurrences of `parent_agent_id` or `parent_agent_type` across
all 16 captured payloads**. The standing refusal to infer parentage now rests on an observation
taken upstream of the adapter rather than on the adapter's own output.

**`duration_ms` is populated on all 13 completions** — 430 ms on the first `Bash`, 3943 ms on
the `Agent`, 38 ms and 10 ms on the two deliberate failures. This is the first recording this
project has written that carries per-call timing, and it is the adapter's first output after
the field-name fix. Whether `PermissionDenied` carries one is unknown and now looks
unanswerable by this route, since the hook does not fire.

**`SubagentStop` carries four fields nobody had looked at** — `agent_transcript_path`,
`background_tasks`, `session_crons`, `stop_hook_active` — and strict mode refused both payloads
on its first live session, which is what it was built to do. `stop_reason`, which the earlier
`DELIBERATELY_UNRECORDED` list had taken from documentation, **did not arrive**.

**`SessionEnd` fired**, with `reason: "prompt_input_exit"`, on the first recording that runs
boundary to boundary *and* contains failures.

---

## 4. What remains unmeasured

Everything in this section is a **provisional blind spot**: not measured against a live
session, and not to be described as characterized until it has been. An unexercised surface is
not a working one. Three sessions have now moved several items out of this list, and section 3
is where they went; what is left is what three sessions did not reach.

- **Interruption has never been observed**, for three distinct reasons, and it is the largest
  remaining hole. First contact never provoked one; pass 2's `sleep 120` was refused by the
  harness before dispatch, invisibly; pass 3 never reached the command, because the denial in
  the same turn ended the turn. `is_interrupt` is modelled and has never arrived with a value.
  A denial and an interruption cannot be staged in one turn, and whatever runs next should put
  the interruption first, in its own turn.
- **Behaviour under abnormal termination is unmeasured.** All three sessions ended cleanly
  through a documented exit. Nothing here says what a recording looks like after a crash or a
  kill.
- **Whether a resumed session appends to the same recording is unmeasured.** Every session so
  far produced one `session_started` with `source: "startup"` and nothing after its
  `session_ended`. No resume with hooks armed has been observed, and neither has `compact`,
  `clear`, or `fork`.
- **Parallel dispatch is not distinguishable from serial dispatch.** First contact reports
  having issued tool calls in parallel batches; the recording contains no overlapping tool-call
  spans, no non-monotonic timestamps, and no interleaving of any kind. A batch whose hooks
  serialize is indistinguishable in the record from a sequence of separate calls. The ordering
  caveat below therefore remains a documented hazard rather than a demonstrated one, and the
  absence of parallel evidence is a coverage gap rather than a finding that nothing ran in
  parallel.
- **`@` file references may bypass `Read` tool hooks.** File content can enter a session
  without any tool event, so a recording can be missing files the session demonstrably read.
  Documented, never exercised here.
- **Appender order under parallel hooks is recorder order.** See above. Do not read
  `sequence` as proof that one tool call happened before another.
- **Total hook latency is unmeasured.** Eight hook surfaces are configured as synchronous
  command hooks, each a process spawn plus a lock-protected append. `async: true` was
  deliberately not used: during first contact, a complete recording and a visible failure
  matter more than shaving hook latency. One recorded session bounded the append transaction
  alone at a median of 5.0 ms (§3.1); process spawn, parsing, and Claude's own overhead are not
  in that number and have not been measured.
- **`PermissionDenied` has never fired**, so nothing is known about its payload from
  observation: not whether it carries a duration, not what its documented `reason` field
  contains, and not whether `tool_denied` renders correctly from a real one. Pass 3 established
  that an interactive refusal does not reach it; the auto-mode classifier path it documents has
  never been exercised here. **Absence of `tool_denied` records is not absence of denials**, and
  after pass 3 it is known to be compatible with a denial having happened.
- **Host and version coverage is one macOS host and two Claude Code versions**, 2.1.220 and
  2.1.221, recorded a day apart. Linux is untested, Windows is not claimed, and no result here
  should be read as stable across versions — the reference itself changed between two readings
  two days apart.
- **Recordings remain sensitive and unsafe to share.** A recording contains prompts,
  commands, absolute paths, file contents, tool output, and any credential that passed
  through any of them. Nothing is redacted. See dragon:2. *This is measured rather than
  feared: one 17-minute session of ordinary repository work produced 580 KB, of which 58% was
  tool response bodies and 24% was tool input, with the host's home-directory path present in
  a quarter of all records.*

Measured and no longer provisional, with the detail in section 3: failure capture on both
shapes (§3.2), hook-level timing on every completion (§3.2, §3.3), parentage genuinely absent
(§3.3), an interactive denial firing nothing and leaving a request with no terminal record
(§3.3), and a harness-refused call leaving no trace in any hook (§3.2). That last one is worth
naming as its own limit: **a request the harness rejects before dispatch is invisible to every
hook this adapter subscribes to**, so a recording cannot distinguish it from a call the agent
never attempted.

Also deliberately **not captured** in this slice, and therefore invisible in any recording:
`UserPromptSubmit`, streamed assistant messages, transcript contents, `SessionStart.model`,
`SubagentStop.last_assistant_message`, `agent_transcript_path`, `background_tasks`,
`session_crons`, `stop_hook_active`, `permission_mode`, `cwd`, `effort`, and every other
lifecycle hook Claude offers.

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
