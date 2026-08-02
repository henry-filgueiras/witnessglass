---
id: tsk_01KZ1SR1012J6W4MBRJ80NNX2J
sequence: 4
kind: task
status: closed
sprint: spr_01KZ1SQTZ730K3VJMH127NMXNS
created: 2026-08-02
closed: 2026-08-02
---

# Record a real Claude session and document blind spots

## Objective

Record one real Claude session through one supported cooperative integration path, then
compare what that path promised to expose against what was actually captured, and write the
difference down honestly.

The deliverable is as much the list of blind spots as the recording. This is the task that
begins to close the dragon about whether a cooperative integration exposes complete tool
ground truth, and it can only do that if the omissions are recorded as omissions rather
than papered over with inference, reconstruction, or optimistic wording.

## Acceptance criteria

- Exactly one supported cooperative integration path is used. No process attachment, no
  descendant-process tracing, no OS-wide observation.
- The session recorded is a real working session, not a synthetic or scripted one.
- The integration path's promised coverage is stated before the comparison: which lifecycle
  points it claims to expose and which fields it claims to populate.
- The captured evidence is compared against what the session demonstrably did, with the
  basis of comparison stated.
- Gaps are enumerated explicitly: lifecycle events not observed, fields not populated,
  behavior under interruption or abnormal termination, and any host- or platform-specific
  variation encountered.
- No gap is filled by inference. Where the recording did not see something, the record says
  so, and no derived view implies otherwise.
- The adapter's fidelity and blind spots are documented where a user will encounter them,
  not only in archaeology.
- The recording itself is not committed. Any illustrative excerpt is synthetic or
  demonstrably free of sensitive material, consistent with the standing constraint that
  redaction is not implemented and safe sharing is not yet claimed.
- Findings are fed back into the relevant dragon rather than left only in this task.
## Result

One real Claude Code session was recorded through the cooperative command-hook adapter and
compared, in a separate archivist session, against what `docs/claude-adapter.md` promised it
would expose. The recording is complete and its structure is now measured rather than
assumed. Six things the adapter document listed as unmeasured are now measured; five remain
unmeasured because this session did not exercise them; and the comparison against the
subject session's own witness statement produced two genuine disagreements, both preserved
below rather than reconciled.

The subject session did the work of task:7. It was armed by Henry before it started, ran
from baseline `3095e2d` to `c3dfd3e`, and was disarmed after it exited. This archivist
session was **not** recorded: the repository was disarmed throughout, `.witnessglass/armed`
and `.claude/settings.local.json` were both absent at every point, and nothing here armed
anything. The recording was read and never written, moved, trimmed, or committed.

### 1. Structural verdict, taken before anything was displayed

`scripts/check-recording.sh` was run against the recording before any of it was read. This
was the tool's first use against a real recording.

```
replayed 234 record(s) in append order (schema v2); recording is complete
exit 0
```

Complete. No truncated tail, no corruption, a final newline present, `sequence` running
1..234 with no gap, duplicate, or decrease, and `recorded_at` monotonically non-decreasing
across the whole file. A recording of a session that ended by an interactive exit and was
then disarmed underneath survived intact.

**The tool earned its place, and its payload silence did not get in the way.** The recording
is 580 KB; `replay` would have put all of it on the terminal to answer a question that took
one line of stderr. The one-line summary was sufficient to decide that everything after it
was worth doing, and its exception — a corrupt record quoting the bytes it rejected — never
fired, because nothing was corrupt. That exception remains the case where the tool is least
useful and it is still correctly documented as such.

What the check does *not* do, and what this task had to do by hand, is say anything about
content. Every count, population, and absence below came from `jq` over the raw file, not
from the check. That is the intended division and it held.

### 2. What the path promised, stated before the comparison

From `docs/claude-adapter.md` §1 and §2, read off Claude's documentation and this adapter's
mapping, before the recording was characterized.

**Lifecycle points claimed to be exposed** — eight hook surfaces, mapping to nine record
kinds: `SessionStart` → `session_started` (+`source`); `PreToolUse` → `tool_requested`
(+ optionally a second `reported_intent` record when `tool_input.description` is a non-blank
string); `PostToolUse` → `tool_succeeded`; `PostToolUseFailure` → `tool_failed`;
`PermissionDenied` → `tool_denied`; `SubagentStart` → `subagent_started`; `SubagentStop` →
`subagent_stopped`; `SessionEnd` → `session_ended` (+`reason`).

**Fields claimed to be populated** — `tool_use_id`, `tool_name`, `requested_input` on a
request; `effective_input`, `response`, and an **optional** `duration_ms` on a completion;
`effective_input`, `error`, optional `interrupted` and `duration_ms` on a failure;
`requested_input` and no error on a denial; `agent_id`, optional `agent_type`, and optional
supplied `parent_agent_id`/`parent_agent_type` on subagent boundaries; and in the envelope's
`context`, optional `prompt_id` and the current `agent_id`/`agent_type` where a payload
supplies them. Every record carries `provenance.{channel, adapter, mechanism}` naming the
originating hook.

**What was explicitly *not* claimed**: that `duration` arrives, that `parent_agent_id`
arrives populated, that `PermissionDenied` fires outside the auto-mode classifier, that a
subagent's own tool calls are visible at all, or that `sequence` is a causal order.

### 3. What the recording actually contains

234 records, schema v2 throughout, one `session_id`, one adapter (`claude-code`), spanning
17 minutes 34 seconds of wall clock.

| Record kind | Count |
| --- | --- |
| `tool_requested` | 82 |
| `tool_succeeded` | 82 |
| `reported_intent` | 65 |
| `subagent_stopped` | **2** |
| `subagent_started` | **1** |
| `session_started` | 1 |
| `session_ended` | 1 |
| `tool_failed` | **0** |
| `tool_denied` | **0** |

169 records on the `observed` channel, 65 on `reported`. Mechanisms present:
`command-hook:PreToolUse` (82), `command-hook:PostToolUse` (82),
`command-hook:PreToolUse#tool_input.description` (65), and one each of `SessionStart`,
`SubagentStart`, `SessionEnd`, plus two `SubagentStop`. No record carried a mechanism from
`PostToolUseFailure` or `PermissionDenied`, because neither hook fired.

**Session boundaries.** Exactly one `session_started`, at sequence 1, `source: "startup"`.
Exactly one `session_ended`, at sequence 234, `reason: "prompt_input_exit"`. **The exit was
captured.** The session's last act is in the file, on the observed channel, with the reason
Claude supplied.

**Nothing landed after the disarm.** The last record in the file is the `session_ended`, and
the file's mtime is that record's timestamp to the second. There is no second
`session_started`, no `source: "resume"`, and no second recording in the directory. So the
recording carries no evidence that any process resumption appended anything.

That absence is worth stating precisely, because it is the exact shape of claim this project
exists to refuse. The file distinguishes none of: hooks were unconfigured by the disarm and
so could not fire; a resumed process ran but used no tool; or no resumption occurred at all.
Silence is what the recording says, and silence is all it says. task:6 left "does a resumed
session append to the same recording" unmeasured, and it is **still unmeasured** — this
session did not produce a resume with hooks armed.

**Tool lifecycle pairing is complete: 82 requests, 82 completions, zero orphans in either
direction.** Every `tool_use_id` that appears in a request appears in exactly one
completion, and no completion lacked a request. The first-class state decision:4 introduced
— a request whose fate was never observed — did not occur once in this session. Every
completion was a success.

**Requested and effective input were identical in all 82 cases**, byte-for-byte after the
recorder's key normalization. Claude never rewrote an input in this session. The schema's
capacity to record a rewrite is therefore intact and **unexercised**; this is not evidence
that rewrites do not happen.

**Tools used:** `Bash` 64, `Read` 7, `Write` 5, `Edit` 5, `Agent` 1.

**Subagents — the largest positive result, and one anomaly.**

*The subagent was not opaque.* `subagent_started` fired once, naming child
`agent_id` with `agent_type: "Plan"`. Its 27 tool calls **did** produce hook records, and
those records carry a distinguishable `context.agent_id` and `context.agent_type` matching
the subagent: 27 `tool_requested`, 27 `tool_succeeded`, 27 `reported_intent`, 81 records in
all, every one attributable to the child by identifier rather than by adjacency. The
main-agent/subagent split of the 82 tool calls is 55 / 27. This answers task:6's open
empirical question in the good direction: a subagent's work is recorded at the same fidelity
as the parent's, and is attributable.

*Parentage is still not expressible.* `parent_agent_id` and `parent_agent_type` were
**absent on all three subagent records**. They are documented; they did not arrive. Nothing
in the recording links the subagent to the `Agent` tool call that spawned it, or to any
parent agent. The subagent's records sit inside the parent `Agent` call's sequence span
(request at 74, completion at 159, subagent records at 76–158), but containment in the
sequence is temporal adjacency, and decision:4 forbids deriving parentage from it. The
adapter behaved correctly: it recorded what was supplied and invented nothing. The
consequence is that **a causal parent/child overlay is not buildable honestly from this
recording** — the identifiers to build it with were not delivered.

*One `subagent_stopped` has no `subagent_started`.* At sequence 233, 33 seconds after the
last tool record and 7.5 seconds before `session_ended`, a second `subagent_stopped` arrived
carrying a **different** `agent_id` and an **empty-string** `agent_type`. No
`subagent_started` ever named that id, no record anywhere carries it in `context.agent_id`,
and no tool call is attributable to it. The recording says a stop was reported for an agent
whose start was never observed and whose work, if any, was never seen. What that agent was
is not answerable from this file, and is not answered here.

**Reported intent behaved exactly as decision:4 documents.** 65 `reported_intent` records,
64 from `Bash` and one from `Agent`. Every one is on the `reported` channel with mechanism
`command-hook:PreToolUse#tool_input.description`. For all 65, the `reported_intent.text` is
**identical** to the `description` field still present inside the same call's
`requested_input`: the duplication is real, is 65-for-65, and a reader counting occurrences
of any such string across this recording will find it exactly twice. 17 calls carried no
`description` field at all and produced no reported record; **zero** carried a blank one.
Each `reported_intent` landed at exactly its `tool_requested`'s sequence + 1, in all 65
cases, because one hook process writes both appends in order.

**Envelope `context`.** `prompt_id` **arrived populated** on 233 of 234 records. The sole
record without any `context` object at all is `session_started` — matching the
documentation's statement that `prompt_id` is absent until the first input, and confirming
task:6's prediction. 81 records carried `agent_id` and `agent_type` (the subagent's work,
above). Two distinct `prompt_id` values appear: one on 232 records covering the entire
working session, and a second on `session_ended` alone.

**`duration_ms` never arrived.** All 82 `tool_succeeded` records lack it. The field is
documented as optional on `PostToolUse` and this host's version supplied it zero times. Any
derived causal view that intended to use tool duration has no input.

**`sequence` shows no sign of parallel-hook acquisition.** This was the caveat the subject
session expected to exercise, and the recording does not contain the evidence.

- No two sibling tool-call spans overlap. The only 27 overlapping pairs in the whole file are
  the single `Agent` call containing the subagent's calls.
- Maximum simultaneously-open requests: 2, and that 2 is only ever the `Agent` call plus one
  subagent call.
- Every completion landed at its own request's sequence + 1 (17 calls without a description)
  or + 2 (64 calls with one). No third value except the `Agent` call's + 85.
- `recorded_at` never moved backwards relative to `sequence`.

The one trace that survives is a timing artifact: of 80 same-agent gaps between a completion
and the next request, the median is 4.0 seconds, but **10 are under 200 ms and 9 are under
50 ms** — consecutive calls dispatched with no interval for a model round trip. That is
consistent with batched requests whose hooks nonetheless serialized, and it is also
consistent with other explanations. It is not proof of either, and no conclusion is drawn
from it here.

**One measurement of recorder cost.** Within a single `PreToolUse` hook process, the interval
between the `tool_requested` append and the `reported_intent` append is one complete
serialized transaction — lock, tail scan, write, `sync_data`, unlock. Across 65 samples:
min 3.3 ms, median 5.0 ms, p90 5.9 ms, max 8.4 ms. This is **not** total hook latency: it
excludes process spawn, JSON parse, and Claude's own overhead, and no measurement here
bounds those. It is the first number this project has for anything, and it is a lower bound
on the per-append cost only.

### 4. The basis of comparison

Three channels were used, and kept distinct:

1. **The recording** — the observed channel plus the adapter's reported channel, read with
   `jq` over the raw NDJSON.
2. **The subject session's witness statement** — task:7's self-report, written before the
   recording was consulted, and deliberately read only *after* section 3 above was complete.
   This ordering was the point: reading it first would have turned a comparison into a
   description.
3. **Repository state** — git history and the working tree, an independent third party to
   both.

### 5. Gaps, enumerated as gaps

Measured absences. None of these is filled by inference, and none is read as evidence that
the underlying thing did not happen.

- **`PostToolUseFailure` never fired.** Zero `tool_failed` records. No tool call failed in
  this session, so the surface is **unexercised**, not proven working and not proven broken.
- **`PermissionDenied` never fired.** Zero `tool_denied` records. No denial was provoked, so
  task:6's open question — whether it fires outside the auto-mode classifier — is untouched.
  Absence of `tool_denied` here means nothing about denial coverage.
- **`duration_ms` was never populated**, on any of 82 completions.
- **`parent_agent_id` / `parent_agent_type` were never populated**, on any of 3 subagent
  records, despite being documented.
- **`interrupted` was never observed**, since no failure occurred. Behaviour under
  interruption and abnormal termination remains entirely unmeasured — this session ended
  cleanly through the documented exit path.
- **Parallel-hook acquisition ordering was not exercised**, so the `sequence`-is-not-causal
  caveat remains a documented hazard rather than a demonstrated one.
- **Resumption was not exercised.** No `source: "resume"`, no second `session_started`.
- **Input rewriting was not exercised.** 82/82 identical requested and effective inputs.
- **File mutation caused by a `Bash` command is invisible**, and this recording demonstrates
  it concretely rather than asserting it. A shell append wrote content into a tracked
  repository file, and the recording contains the command and the tool's reported output but
  **no mutation event for the file** — the only `Write`/`Edit` events are the 10 that went
  through those tools. A reader reconstructing "which files did this session change" from
  tool events alone would get the wrong answer. Likewise `cargo fmt` rewrote a file that no
  event shows being written.
- **No prompt is in the recording.** `UserPromptSubmit` is not captured, so `prompt_id` is
  populated but unanchored: nothing in the file says what a `prompt_id` delimits, and the
  two observed values cannot be interpreted.
- **Host and platform variation is unmeasured.** One session, one host (macOS), one Claude
  Code version. Nothing here says anything about Linux, about IDE hosts, or about any other
  version.

### 6. Reported against observed, on real data for the first time

The witness statement was read only after section 3 was written. Where the two channels
disagree, both are recorded and neither is promoted.

**Disagreement 1 — subagent count.** The statement says: "one read-only review subagent was
requested, started, and completed... No other subagent was spawned." The recording holds
**one** `subagent_started` and **two** `subagent_stopped`, the second naming an agent id
that appears nowhere else in the file, with an empty `agent_type`. Reported: one subagent.
Observed: one start, two stops. Both are in the record. The reported channel cannot be
promoted into ground truth about how many agents existed, and the observed channel cannot
explain what the second stop was. This is precisely the class of disagreement decision:2
exists to preserve, and it appeared in the first real recording.

**Disagreement 2 — parallelism.** The statement says: "Several tool calls were deliberately
issued as parallel batches. If parallel hooks are what task:6 predicts, this recording
should contain records whose `sequence` is recorder acquisition order rather than causal
order." The recording contains **no** overlapping sibling spans, **no** non-monotonic
timestamps, and a maximum of two simultaneously-open requests, which is only ever the
subagent nesting. The predicted evidence is absent. Reported: parallel batches were issued.
Observed: strictly serial, non-overlapping tool-call spans. The recording cannot corroborate
the claim, and it also cannot refute it — a batch dispatched in parallel whose hooks
serialize would look exactly like this. **The finding is that this integration's recording
does not distinguish parallel dispatch from serial dispatch at all**, which is a coverage
gap the adapter document did not previously state.

**A near-disagreement that was not one.** The statement enumerates `./scripts/check.sh` as
run "once at the start... once after implementation... and once after this task was closed",
and separately notes a `cargo fmt --all` "after the first post-implementation gate failed".
The recording holds **four** invocations. Counting the three enumerated ones and treating
the acknowledged failed gate as the fourth reconciles it exactly. A naive count would have
manufactured a disagreement here; it is recorded as a caution about comparing a narrative
enumeration against a machine count.

**Where the recording corroborates the statement.**

- **Baseline commit.** Stated `3095e2d…94e86`; git confirms it as `c3dfd3e`'s parent. Note
  this came from the *third* channel, not the recording — the recording contains no git
  state except as opaque command payloads.
- **Files intentionally changed.** The statement names two created and two modified
  repository files plus the task's archaeology file. The recording's `Write`/`Edit` events
  target exactly those five, plus two scratch files outside the repository. `src/` and both
  Cargo manifests appear in no mutation event.
- **`cargo fmt --all` run once.** One matching command in the recording.
- **The recordings directory was never opened.** The statement claims it; the recording shows
  no `Read` targeting it and no command referencing it. Every `check-recording.sh` invocation
  in the recording — and there are many — points at synthetic recordings in a scratch
  directory. Corroborated within the observed channel's limits.
- **No failures, denials, or interruptions "noticed".** Zero `tool_failed`, zero
  `tool_denied`, 82/82 successes. The channels agree — but the agreement is weak evidence,
  because the observed channel's silence here is exactly what it would look like if the
  denial hook simply never fires. Two silences agreeing is not corroboration.
- **Reported intent plentiful and duplicated.** Predicted; measured at 65 records, 65/65
  duplicated.
- **The `cargo fmt` blind spot.** Predicted as "a clean, concrete instance of a blind spot
  task:4 can demonstrate rather than assert". Demonstrated, and a second instance was found
  that the statement did not predict: the shell append of the statement's own containing
  file.

**One observed fact that reads wrong without care.** The recording contains an execution of
`scripts/arm.sh` — with `--help`, as part of testing a help-flag convention. A reader
grepping the observed channel for "did this session arm the repository" would find a hit.
The session did not arm anything. Tool-level evidence needs its arguments read, and a
projection that matched on command name alone would produce a false and serious claim.

### 7. What changed outside this task

`docs/claude-adapter.md` gains a measured section, and the items this session actually
measured moved out of the provisional list into it. Everything not measured stayed in the
provisional list, unchanged, with the ones this session proved *unexercised* now marked as
such. The document's opening no longer says the adapter has not been run against a live
session, because it has.

dragon:1 and dragon:2 each carry a first-contact findings section. Neither is closed.
dragon:1's resolution criteria are now substantially met for one path on one host, and the
two things holding it open are named there. dragon:2 gains its first enumeration of
sensitive surfaces measured against a real recording rather than an imagined one.

### Scarp desire paths

**idea:1 recurred for the seventh time.** This result was written to a temporary file and
appended with a shell redirect before `scarp close task:4`. Seven for seven. Nothing to add
beyond the count, and there is a small irony in the fact that the previous instance of this
same workaround is the one the recording proves is invisible to tool-event capture.

**No new idea is filed.** The friction in this session was in reading a 580 KB NDJSON file
without displaying it, which is a WitnessGlass problem and not a Scarp one. `scarp doctor`,
`scarp show`, and `scarp close` did what was needed without incident, and appending a
findings section to an open dragon needed no affordance that did not exist.

### Follow-up: dragon:3 filed for what `prompt_id` delimits

Added after this task closed, at Henry's request. The conclusions above are unchanged; this
records where the new dragon came from, because a dragon with no provenance is a dragon
nobody can retire.

Section 3 reported `prompt_id` as arriving populated on 233 of 234 records, and filed that
under measured coverage — correctly, as a statement about availability. The distribution is
the part that does not fit under coverage at all: **two** distinct values across the entire
session, one spanning 232 records including every tool call by both the parent agent and the
subagent, and one appearing on `session_ended` alone. With `UserPromptSubmit` deliberately
uncaptured, there is no record anywhere in a WitnessGlass recording for that identifier to
refer to. It is populated and unanchored.

That is not a coverage gap and it is not a privacy question, so neither open dragon was the
right home for it. dragon:1 asks what is observable; the field *is* observable. dragon:2 asks
what is safe to share. Neither asks what a delivered identifier **means**, and that meaning is
a precondition for every derived projection this project intends to build: grouping tool calls
by prompt, counting turns, segmenting a timeline. Attaching a unit of work to `prompt_id` on
the strength of it being populated would be decision:2's silent promotion committed from a new
direction — not by fabricating a value, but by attaching a meaning to a real one.

dragon:3 records that, notes that `tool_use_id` is the only identifier whose semantics this
project has actually tested (82 correlated pairs), and proposes the cheap experiment — a
recorded multi-turn session — before the expensive one. Capturing `UserPromptSubmit` would
give the identifier an anchor and would also put user prompt text into the raw stream, which
is a dragon:2 decision and is explicitly not the first move.

No projection may segment by `prompt_id` until that dragon is settled, and this task's own
result is written to comply: it reports counts of records and tool calls, and never a count of
turns.
