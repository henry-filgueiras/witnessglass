---
id: spr_01KZ1SQTZ730K3VJMH127NMXNS
sequence: 1
kind: sprint
status: closed
created: 2026-08-02
closed: 2026-08-02
---

# First contact

## Goal

Stand up WitnessGlass as a real public project and prove the smallest useful recording
kernel: an append-only raw stream that can carry a session boundary, a reported intent, and
one observed tool lifecycle, replayed deterministically in chronological order — then
exercise that kernel exactly once against a real Claude session and write down honestly
what it did and did not see.

## Rationale

The project's entire claim is evidentiary: that a recording tells you what actually
happened, including where it could not see. That claim cannot be established by design
work. It is settled by one end-to-end round trip and one confrontation with reality.

Sequencing follows from that. The foundation and the public repository come first because
they are what makes the rest reviewable. The emit-and-replay round trip comes next because
it is the smallest thing that forces the genuinely hard questions into the open — event
ordering, timestamp source and resolution, schema versioning, and what a reader may
conclude from a truncated tail. Those questions are much better answered by a working
kernel that must handle them than by a document that anticipates them.

The real Claude session comes last, and it is the point. Until an adapter has run against
a live session, every statement about coverage is speculation, and the two open dragons —
what a cooperative integration actually exposes, and what a recording actually contains —
cannot begin to close. The deliverable of that task is as much the list of blind spots as
the recording itself.

Nothing in this sprint is allowed to become a framework. Two adapters do not exist, so
there is nothing to generalize over yet.

## Success criteria

- The public repository exists with the foundation, the engineering contract, a healthy
  Scarp corpus, and CI running the same gate as local development.
- An append-only raw stream can be written and read back containing, at minimum: a session
  boundary, one reported-intent event, and one observed tool lifecycle event.
- Replay is deterministic and chronological, and is covered by tests rather than
  demonstrated by hand.
- Ordering, timestamp, schema-version, and truncated-tail behavior are each explicit and
  tested, not left implicit in the implementation.
- Reported and observed events remain distinguishable end to end, with source and fidelity
  provenance surviving the round trip.
- One real Claude session has been recorded through one supported cooperative integration
  path.
- That session's promised coverage and actually captured evidence are compared in writing,
  with omissions recorded as omissions.

## Non-goals

- Dashboards, TUI, web UI, or any visualization layer.
- A generalized adapter or plugin framework, or a second integration.
- MCP, a daemon, or a background collector.
- OS-wide tracing or attaching to arbitrary PIDs.
- Crate publication, tags, or releases.
- A redaction or safe-sharing implementation; the constraint for this sprint is simply that
  real recordings are not committed.
- Performance work, storage compaction, or long-term retention policy.

## Outcome

The sprint's goal was to stand up a real public project, prove the smallest useful recording
kernel, then exercise it exactly once against a real Claude session and write down honestly
what it did and did not see. All three happened, in that order, and the third produced a
result worth more than the first two: **the recording and the recorded session's own account
of itself disagree in two places, and both accounts survived.**

Seven tasks, all closed. Four accepted decisions. Two dragons carrying measured findings and
still open, plus a third the evidence forced into existence. 100 tests behind a single gate
that CI and local development share.

### Success criteria, against evidence

- **Public repository, foundation, engineering contract, Scarp corpus, CI on the same gate.**
  task:1, task:2. `.github/workflows/ci.yml` runs `./scripts/check.sh`, the same script local
  work runs, so the two cannot drift.
- **A raw stream carrying a session boundary, a reported intent, and an observed tool
  lifecycle, written and read back.** task:3, decision:3. Then done for real: the first
  contact recording holds all three from a live session.
- **Deterministic, chronological replay, covered by tests.** task:3, task:5.
- **Ordering, timestamps, schema versioning, and truncated tails each explicit and tested.**
  task:3, task:5, decision:3. Damage has two distinct meanings and both are first-class
  readable states. The truncated-tail path was exercised against a real recording's *shape*
  in task:7's tests, and the real recording itself came back complete.
- **Reported and observed distinguishable end to end, with provenance surviving the round
  trip.** decision:2, enforced by the format rather than by convention, and now demonstrated
  on live data: 169 observed records and 65 reported records in one recording, each naming the
  hook that produced it, with the two channels disagreeing about the session and neither
  overwriting the other.
- **One real Claude session recorded through one supported cooperative path.** task:4. Command
  hooks only. No process attachment, no descendant tracing, no OS-wide observation.
- **Promised coverage and captured evidence compared in writing, omissions recorded as
  omissions.** task:4, written in a separate archivist session that characterized the recording
  *before* reading the subject's witness statement, so the comparison had two independent ends.

### What first contact actually settled

The cooperative path delivered more than the sprint assumed and less than a naive reader would
claim. Both session boundaries including the exit reason; 82 tool requests paired with 82
completions and no unmatched record in either direction; a subagent's own tool calls recorded
and attributable to it by `agent_id`, which was the largest open unknown and resolved
favourably. Against that: `parent_agent_id` never arrived, so no causal parent/child link is
expressible; `duration_ms` never arrived; a `subagent_stopped` arrived with no matching start;
a file written by a shell command produced no mutation event at all; and parallel dispatch
turned out to be indistinguishable from serial dispatch in the record.

The last one is the sprint's sharpest lesson about its own methodology. The subject session
*expected* to demonstrate the "sequence is acquisition order, not causal order" caveat and
believed it had issued parallel batches. The recording contains no interleaving whatsoever.
Neither channel is wrong; the integration simply does not expose the distinction. A project
willing to promote either channel would have published a confident and unfounded claim here,
in its first week.

### What the sprint deliberately did not build

Every non-goal held. No dashboard, TUI, web UI, or visualization layer. No second adapter and
no framework to generalize over one. No MCP, daemon, or background collector. No OS-wide
tracing. No crate publication, tag, or release. No redaction implementation — and, matching
the sprint's actual constraint, no real recording was committed at any point, in any form,
including during the task:4 autopsy that had to read one closely.

### Dragons

**dragon:1 and dragon:2 stay open,** both now carrying measured findings rather than
speculation. dragon:1's resolution criteria are substantially met for one path on one host;
what holds it open is behaviour under abnormal termination, host and platform variation, and a
process-corroboration decision that deserves a second, deliberately hostile recording first.
dragon:2 has its first enumeration of sensitive surfaces measured against a real recording,
including the finding that the recording contains a string deliberately shaped like a
credential which is in fact a synthetic test marker — a false positive sitting next to
potentially unrecognizable true ones, which is the dragon's own argument arriving as evidence.

**dragon:3 was filed from the evidence,** not from planning: `prompt_id` arrived populated on
233 of 234 records and turned out to delimit nothing a projection can currently rely on. It is
the first dragon this project found by measuring rather than by anticipating, which is the
sprint's rationale working as intended.

### Scarp desire paths

idea:1 recurred every single time — seven for seven, once per closed task, plus this sprint
close. The sample stopped being informative several tasks ago. idea:2 recurred in the narrowed
form task:6 predicted and narrowed once more here: closing a sprint needed its section shape
discovered by reading, and no closed sprint existed to read from, which is the first-use gap
one level up from the one idea:2 already describes. idea:3 recurred as prose references to
decisions escaped the Scarp corpus into `docs/` and `README.md` by file path. All three stay
parked; none was sent upstream, and nothing was invented to fill a quota.

### What the next sprint inherits

A recording kernel that has met reality once, an adapter whose fidelity section now separates
what was measured from what is still provisional, and three open dragons — one of which,
dragon:3, constrains any projection built next: **no derived view may segment work by
`prompt_id`, and no recording may be described as containing N turns, until that dragon is
settled.** The other standing constraint is `CLAUDE.md` §6, which lists a dashboard and any UI
among the bootstrap non-goals; whether first contact's result is enough to lift that is a
decision for whoever opens the next sprint, and it should be made explicitly rather than
assumed by starting to build one.
