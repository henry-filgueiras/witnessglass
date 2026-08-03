---
id: drg_01KZ1SQTYZE1MQBGP6WQSHSYXX
sequence: 1
kind: dragon
status: open
created: 2026-08-02
---

# Claude integration may not expose complete tool ground truth

## Context

WitnessGlass intends to record a real Claude coding session as its first adapter, and the
whole value proposition rests on the recording being an honest account of what happened.

At bootstrap time, nobody here has measured what a cooperative Claude integration actually
exposes. The plausible sensor surfaces — agent hooks fired around tool use, or an explicit
semantic emission API the agent calls — have not been enumerated against a real session,
let alone tested for coverage. Assumptions about them are currently just assumptions.

Several things are unknown in ways that matter:

- Which lifecycle points are observable at all, and whether both the start and the
  completion of a tool call are visible, or only one side.
- Whether the payload carries enough to identify the work (command, arguments, exit status,
  affected paths) or only enough to know that *something* happened.
- Whether a hook can be dropped, coalesced, reordered, or silently skipped under load,
  interruption, or cancellation.
- Whether the same surface exists across the CLI, IDE integrations, and other hosts, or
  varies per host and per platform.
- Whether a session that ends abnormally leaves a truncated tail and what a reader is
  entitled to conclude from one.

The alternative some tools reach for — attach to the agent's process, trace descendants,
reconstruct everything from syscalls — is not a promise this project can keep. It is not
portable across macOS, Linux, and Windows; it requires privileges that many environments
will not grant; it degrades badly under sandboxing and containerization; and even where it
works it produces process facts that cannot recover intent. Shipping it as a v0 claim would
mean shipping a claim that fails silently on someone else's machine, which is precisely the
failure this project exists to make visible.

## Question

What does a cooperative Claude integration actually let WitnessGlass observe, how much of a
real session does that cover, and where are its blind spots?

## Constraints

- Cooperative hooks or an explicit semantic emission API are the primary sensor. Reported
  intent can only come from cooperation; there is no other channel for it.
- OS/process observation is optional, secondary, and corroborating. It is not the v0
  mechanism.
- v0 must not claim it can attach to an arbitrary agent process and observe every
  descendant process.
- Whatever the integration turns out to be, reported and observed events stay in separate
  channels with source and fidelity provenance intact.
- No completeness claim without measurement. "We did not see this" must be a supported,
  publishable result.
- Portability matters: an adapter that works only on one platform must say so rather than
  degrading quietly.

## Candidate direction

Cooperative hooks first, documented corroboration second.

Build the smallest adapter against one supported cooperative path. Record what it emits,
compare that against what the session demonstrably did, and write the gap down as the
adapter's declared fidelity — including the specific things it cannot see. Only after that
gap is characterized does it make sense to ask whether limited process-level corroboration
is worth adding, and it would then be additive evidence in its own channel, never a
retroactive patch over a hole in the cooperative record.

## Resolution criteria

This dragon is resolved when:

- At least one cooperative integration path has been exercised against a real Claude
  session, not a synthetic fixture.
- The adapter's coverage is stated in measured terms: which lifecycle events were observed,
  which fields were populated, and against what the comparison was made.
- The adapter's blind spots are enumerated explicitly, including behavior under abnormal
  termination and any host- or platform-specific variation encountered.
- The recording produced under that adapter carries fidelity provenance a reader can act
  on, such that no consumer can mistake partial coverage for complete observation.
- The decision about whether to pursue process-level corroboration is made on the basis of
  that measured gap rather than on speculation.

## Findings from first contact (task:4)

One real Claude Code session was recorded through the command-hook adapter and characterized
in a separate, unrecorded archivist session. Scope of every statement below: **one** session,
**one** macOS host, Claude Code **2.1.220**, 17 minutes of ordinary repository work, 234
records. This does not generalize to another host, version, or session shape, and the dragon
stays open partly for that reason.

**The dragon's central question now has a partial answer.** A cooperative command-hook
integration exposed considerably more than "something happened": complete tool lifecycle
pairing, full tool input and response, session boundaries at both ends, and — the result that
was genuinely open — a subagent's own tool calls attributable to the subagent. It exposed
materially less than "everything the session did".

### What was observable

- **Both lifecycle sides of a tool call, reliably.** 82 `tool_requested` and 82
  `tool_succeeded`, correlated by `tool_use_id`, with **zero** unmatched records in either
  direction. The open question "are both the start and the completion visible, or only one
  side" is answered yes for the success path.
- **Enough payload to identify the work.** Tool name, full input, and the tool's reported
  response. Not exit status as a distinct field, and not affected paths as a distinct field —
  both are only inside opaque tool payloads.
- **Both session boundaries, including the exit,** with `reason: "prompt_input_exit"`.
- **A subagent's interior.** `SubagentStart` fired once with the child's `agent_id` and
  `agent_type`; the subagent's 27 tool calls produced 81 records carrying `context.agent_id`
  and `context.agent_type` naming the child. A subagent is **not** an opaque pair of boundary
  events on this path. This was the largest single unknown and it resolved favourably.
- **`prompt_id`,** populated on every record except `session_started`. Populated is not the
  same as meaningful: only two distinct values appeared across the whole session, and with
  `UserPromptSubmit` uncaptured there is no record for the identifier to refer to. What it
  delimits is now dragon:3, which is a question about the *meaning* of a delivered
  identifier rather than about its availability, and therefore sits outside this dragon.

### What was not observable, measured rather than assumed

- **Parentage.** `parent_agent_id` and `parent_agent_type` are documented and arrived on
  **none** of the three subagent records. Nothing in the recording links a subagent to the
  tool call or agent that spawned it. **A causal parent/child overlay is not buildable
  honestly from a recording of this shape** — not because the project refuses to infer it,
  but because the identifiers were never delivered. That is a direct answer to a question
  decision:4 deferred, and it is a negative one.
- **`duration_ms`.** Documented as optional; supplied zero times in 82 completions. A derived
  causal view has no duration input from this path.
- **Descendant effects.** Demonstrated twice, not asserted: a shell redirect wrote content
  into a tracked repository file and a formatter run from `Bash` rewrote another, and the
  recording contains the commands and their reported output but **no mutation event for
  either file**. Anyone reconstructing "which files did this session change" from tool events
  gets the wrong answer.
- **Parallel dispatch.** The session reports having issued tool calls in deliberate parallel
  batches. The recording contains no overlapping sibling tool-call spans, no non-monotonic
  timestamps, and no interleaving whatsoever; the only overlap in the file is the one `Agent`
  call containing the subagent's calls. The single surviving trace is a timing artifact — 10
  of 80 inter-call gaps under 200 ms against a 4-second median — which is consistent with
  batched requests whose hooks serialized and equally consistent with other explanations.
  **The finding is that this integration's record does not distinguish parallel dispatch from
  serial dispatch at all.** That is a coverage gap in its own right, and it means the
  "`sequence` is acquisition order, not causal order" caveat remains a documented hazard that
  has still never been demonstrated.

### An anomaly the dragon should carry

**A `subagent_stopped` arrived with no `subagent_started`.** One of the two stop records named
an agent id appearing nowhere else in the file, with an **empty-string** `agent_type`, 33
seconds after the last tool record and 7.5 seconds before `session_ended`. No tool call is
attributable to it. Subagent boundary events are therefore **not guaranteed to pair**, which
directly touches this dragon's "whether a hook can be dropped, coalesced, reordered, or
silently skipped" unknown — here it is a hook arriving *without its counterpart*, which is the
same class of problem from the other side. What that agent was is not answerable from the
recording and is not answered.

### Reported against observed, exercised on real data

The subject session wrote a witness statement before the recording was consulted, and the
archivist characterized the recording before reading the statement. Two disagreements
resulted: the statement says one subagent was spawned where the recording holds one start and
two stops, and the statement expected parallel-hook ordering evidence that the recording does
not contain. Both are preserved in task:4 with each claim attributed to its channel. This is
the first time the project's core invariant has been exercised against something other than a
fixture, and it held: the disagreements were findings, and neither channel was rewritten to
agree with the other.

A subtler result belongs here too. **Silence in the observed channel agreed with the
statement's "no failures or denials", and that agreement is nearly worthless** — it is exactly
what the record would look like if the denial hook simply never fires. Two silences agreeing
is not corroboration, and a projection that reported "no denials occurred" from this recording
would be making a claim the evidence does not support.

### Surfaces still unexercised, which is not the same as working

`PostToolUseFailure` (no tool failed), `PermissionDenied` (no denial provoked), `interrupted`,
abnormal termination, resumption with hooks armed, input rewriting (all 82 requested inputs
were identical to their effective inputs), and a `tool_requested` with no completion (none
occurred). Any of these could be broken and this session would look identical.

### Why this dragon stays open

Its resolution criteria are substantially met for one path on one host: a real session was
exercised, coverage is stated in measured terms, blind spots are enumerated, and the recording
carries per-record `provenance.mechanism` a reader can act on. Two criteria are not met.

1. **Behaviour under abnormal termination is still unmeasured,** and so is host and platform
   variation — one macOS host, one version, one session shape.
2. **The process-corroboration decision cannot yet be made on measured grounds.** The measured
   gap says descendant effects are invisible and parallel dispatch is indistinguishable, which
   is an argument *for* corroboration; it also says the cooperative path already delivers
   subagent-level attribution, which narrows what corroboration would need to add. That
   trade-off deserves a second recorded session — ideally one that fails, is denied, and is
   interrupted on purpose — before anything is decided.

## Follow-up: the absent `duration` was confirmed by payload inspection, and one tool self-reports

Appended after sprint:2 closed, from a question Henry asked about whether the missing duration
was a WitnessGlass defect rather than a coverage gap. It was worth asking, and answering it
sharpened the finding in two directions. No new session was recorded; this is a re-reading of the
first-contact recording.

### The absence is the integration's, not the adapter's

task:4 reported `duration_ms` as supplied zero times in 82 completions. That was a
field-population count, which is consistent with an adapter that reads the wrong key. Three
checks rule that out:

- The adapter deserializes `duration`, which is the key the hooks reference documents on
  `PostToolUse`.
- The hook payload struct ignores unknown fields but is strict about types. A `duration` arriving
  as a float or a string would have failed the whole translation, the record would not have been
  written, and the recording would hold fewer completions than requests. It holds exactly 82 for
  82, so nothing failed and the field was absent rather than misread.
- A recursive scan of every captured payload for any duration-shaped key found no hook-level
  field anywhere in the recording.

So the finding stands at full strength, and now rests on inspection rather than on a count.

### One tool carries its own duration, in its response body

The same scan found `totalDurationMs` on exactly one completion: the single `Agent` call, inside
its `tool_response`, alongside `totalTokens`, `totalToolUseCount`, `toolStats`, and
`resolvedModel`. `Bash` (64), `Read` (7), `Write` (5), and `Edit` (5) carry nothing of the kind.

This is worth the dragon carrying, for two reasons.

**It is a different measurement from a different source.** `duration_ms` on the envelope means
the capture mechanism reported how long a call took. `totalDurationMs` inside a response means
one tool's output format happens to include a self-reported number. The adapter deliberately does
not lift the second into the first: doing so would make one tool appear to have hook-level timing
that it does not have, and would make `duration_ms` mean two different things depending on which
tool a reader is looking at. That is the silent promotion decision:2 forbids, arriving from a
direction nobody was watching — not from a merge, but from a field name that reads the same.

**It is not a rescue for a timeline.** One completion in 82, on one tool, is 1.2% coverage. Any
derived view wanting per-call timing still has no input from this recording, and the only
elapsed quantity computable for all 82 calls remains the interval between two recorder writes,
which contains hook spawn, permission wait, queueing, and execution without separating them.

### What this does not settle

Whether the hook-level `duration` is absent by version, by host, by tool, or by defect. One
macOS host, one Claude Code version, one session. A newer version may populate it, and
re-measuring is cheap: arm, run a session with a single tool call, disarm, and check. That
measurement should record the Claude Code version beside the result, because this one is scoped
to a version that was never written down alongside the finding — which is itself a small lesson
for the next pass.

### Correction: the version was recoverable, and it is 2.1.220

The follow-up above closed by saying the finding "is scoped to a version that was never written
down". That was true of the archaeology and false of the machine, which still held enough install
metadata to recover it. Recovered before a later update could overwrite it:

| fact | value |
| --- | --- |
| `~/.local/bin/claude` symlink target | `versions/2.1.220`, a native (non-npm, non-Homebrew) install |
| `2.1.220` installed and symlink repointed | 2026-07-25 |
| only update attempt between then and the recording | 2026-08-02, **failed** (`install_failed`, `version_to: null`) |
| first-contact recording written | 2026-08-02 |

No successful version change occurred between the symlink being repointed and the recording being
written, and the failed attempt names `2.1.220` as the version it was updating *from*. So
**task:4's measurements are scoped to Claude Code 2.1.220 on macOS arm64**, and every count in
this dragon's findings section inherits that scope.

`claude update` subsequently reported 2.1.220 as the latest available version. Two consequences
follow, and the second is the one that matters:

- The absent hook-level `duration` is not a stale-install artifact. It is the behaviour of the
  current release.
- **A newly recorded session cannot answer the version question**, because there is no newer
  version to compare against. What a second recording can still answer is whether `duration`
  arrives on hooks that have never fired here — `PostToolUseFailure` and `PermissionDenied` each
  document their own optional `duration`, and neither has ever been observed — and whether the
  absence is per-hook, per-tool, or total.

The install metadata that made this recoverable is transient: `.last-update-result.json` is
overwritten by the next update attempt, and old version binaries are eventually reaped. The
durable lesson is the cheap one — **record the integration version at arm time, not at analysis
time**. `scripts/arm.sh` already writes a sentinel recording the WitnessGlass version, the binary
path, and its checksum; it records nothing about the agent being recorded, which is the more
consequential of the two.
