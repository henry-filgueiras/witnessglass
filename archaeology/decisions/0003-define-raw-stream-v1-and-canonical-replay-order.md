---
id: dec_01KZ1VGT89P0Q3FZEZTDK1Y02H
sequence: 3
kind: decision
status: accepted
created: 2026-08-02
---

# Define raw stream v1 and canonical replay order

## Context

Task:3 required the smallest recording kernel that actually works, on the reasoning that
building one forces decisions a design document lets you defer. That turned out to be
correct: ordering, timestamps, versioning, damage, and concurrency each had to be settled
before the round trip would hold together.

The forcing constraint came from the intended first sensor. Claude Code delivers hooks as
separate, short-lived processes, and its documentation states plainly that matching hooks
run in parallel. So the recorder cannot assume a single writer, cannot assume a live
in-memory sequence counter, and cannot assume an emitter will survive long enough to clean
up after itself. Whatever the format is, independent processes must be able to append to
the same recording safely and then exit.

The hook payload also settles the correlation question. Both `PreToolUse` and `PostToolUse`
carry `tool_use_id`, which is a stable identifier for one tool call, alongside `session_id`,
`tool_name`, and `tool_input`. That gives a real correlation key rather than an invented
one. It also produces the exact epistemic trap decision:2 warns about: a `Bash` call's
`tool_input` contains a `description` field that the agent wrote about its own intentions,
sitting inside the same payload as the command that was actually run. One payload, two
kinds of claim.

## Decision

Adopt raw stream v1.

**Format.** A recording is UTF-8 NDJSON. One newline-terminated line is exactly one complete
record. One file is one session; a record whose `session_id` disagrees with the recording is
rejected. Records are appended and never rewritten.

**Envelope.** Every record carries, in this order: `schema_version`, `session_id`,
`sequence`, `recorded_at`, `provenance`, `event`.

**Provenance.** `provenance` is `{channel, adapter, mechanism}`. `channel` is `reported`,
`observed`, or `recorder`. `adapter` names the integration and `mechanism` names the capture
point within it, which together let a reader work out what that source could see. There is
deliberately no numeric fidelity score: a made-up number implies a precision nobody has
measured.

**Event vocabulary.** Five kinds: `session_started`, `session_ended`, `reported_intent`,
`observed_tool_started`, `observed_tool_finished`. Tool events carry `tool_call_id`, which
maps to the hook's `tool_use_id`. `reported_intent` may carry the same `tool_call_id`, which
correlates a claim with a tool call without merging them.

**Channel coherence is enforced.** `reported_intent` may only arrive on `reported` — no
mechanism observes intent. Tool events may only arrive on `observed` — an agent describing a
tool call is making a statement, not witnessing one. Session boundaries may be `recorder` or
`observed`, never `reported`. Violations are refused at emit and treated as corruption on
read, so the format itself makes the epistemic error unrepresentable rather than merely
discouraged.

**Canonical order is physical append order,** represented by `sequence`: starts at 1,
increases by exactly 1. Replay yields records in that order and never consults a timestamp
for ordering.

**Timestamps are descriptive metadata.** `recorded_at` is the recorder's wall clock at the
moment of writing. Equal timestamps need no tie-breaker because sequence already decides.
A backward-moving clock produces descending timestamps and a perfectly intact order.

**Concurrent appends are serialized** by an advisory exclusive lock on the recording file
(`std::fs::File::lock`), held across the whole read-tail / decide-sequence / write-record
transaction. The file is opened in append mode. Two racing emitters therefore cannot claim
the same sequence number or interleave partial lines. No daemon and no database.

**Schema version.** Every record carries `schema_version: 1`. A reader that meets any other
value fails with an explicit unsupported-version error rather than guessing. Unknown fields
are rejected, so any additive change requires a version bump.

**Damage has two distinct meanings.** A newline-terminated record that cannot be understood
is *corruption*: it was written whole and is wrong, so replay fails loudly with the line
number. A final fragment with no terminating newline is a *truncated tail*: the recording
stops mid-record. Replay returns the valid prefix, reports the recording as incomplete with
the fragment's offset and length, and never parses or presents the fragment as an event. The
CLI signals this with exit code 2, distinct from both success and error. Appending to a
recording with a truncated tail is refused outright, because splicing a new record onto a
partial one would manufacture one corrupt line out of two honest halves and destroy the
evidence that the recording was cut short.

**Ambiguity is rejected, not repaired.** A duplicate, decreasing, or skipped sequence all
fail. A gap in particular cannot be distinguished from a deletion, and a canonical history
that might have had records removed from it is not canonical.

**Durability is claimed only as far as it holds.** A successful append means the record was
written and `sync_data` returned. It is not claimed to be atomic against power loss. A crash
mid-write can still leave an unterminated fragment — which is exactly why the truncated tail
is a first-class readable state.

## Consequences

- The two channels survive the round trip intact. A recording can hold "the agent claimed
  the check passed" next to "the tool reported failure" and preserve the disagreement, which
  is the most valuable thing it can do.
- Channel coherence being enforced by the format means a future adapter physically cannot
  file an agent's `description` string as an observation. When the Claude adapter meets a
  `Bash` payload it must split it into a reported record and an observed record, or drop the
  semantics. It cannot quietly fuse them.
- The lock is advisory, so the guarantee covers WitnessGlass appenders honoring it. A
  foreign process writing to the recording is out of scope and would not be detected.
- Appending is O(size of the final record) rather than O(file), because the appender scans
  backwards for the last complete line instead of reading the whole recording. Replay,
  however, reads the entire file into memory — fine at session scale, and a deliberate
  deferral rather than a claim about large recordings.
- `deny_unknown_fields` means the format cannot be extended without a version bump. That is
  the intended cost: silent forward compatibility is how a reader ends up quietly discarding
  evidence it did not recognize.
- JSON object keys inside `arguments` and `result` are normalized to sorted order when the
  record is serialized. This happens before the bytes become evidence, and it makes
  re-rendering byte-stable, but it does mean the stored record is not byte-identical to the
  emitter's input.
- Rejecting a gapped sequence means a recording that loses a middle record becomes
  unreplayable rather than partially readable. That is the deliberate trade: a loud failure
  over a quiet, plausible, wrong history.

### Deliberately deferred

- **Originating time.** Only the recorder's write time is stored. A hook knows when the
  event actually happened, and that may deserve a separate field — but adding it costs a
  version bump and it is not needed until a real adapter exists.
- **The recorder's own identity.** Records say which adapter produced them, not which
  WitnessGlass build wrote them.
- **Whether a recording must open with a session boundary.** Not enforced. Requiring it
  would pressure an adapter that never observed a boundary into fabricating one, which is
  the wrong failure. The cost is that a recording is not guaranteed self-describing.
- **Lifecycle pairing.** A `observed_tool_finished` with no matching start is accepted
  exactly as delivered. Refusing it would delete the only evidence that the call existed,
  and a missing start is a capture blind spot to record, not an error to reject. Pairing is
  a derived projection's job.
- **Rotation, compaction, indexing, and streaming reads.** None exist.
- **Redaction and export.** Nothing is redacted. Recordings remain unsafe to share, per
  dragon:2.
