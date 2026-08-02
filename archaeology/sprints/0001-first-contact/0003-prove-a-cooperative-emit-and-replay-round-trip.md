---
id: tsk_01KZ1SR0ZWB01W5T3DEM05YQXZ
sequence: 3
kind: task
status: closed
sprint: spr_01KZ1SQTZ730K3VJMH127NMXNS
created: 2026-08-02
closed: 2026-08-02
---

# Prove a cooperative emit-and-replay round trip

## Objective

Prove the smallest useful recording kernel end to end: emit an append-only raw session
stream, then replay it deterministically in chronological order.

The stream must be able to carry, at minimum, a session boundary, one reported-intent
event, and one observed tool lifecycle event, with the two channels remaining
distinguishable and retaining source and fidelity provenance across the round trip.

The point of doing this before anything else is that a working round trip forces the
questions that a design document lets you defer. Event ordering, timestamp source and
resolution, schema versioning, and what a reader may conclude from a truncated tail each
have to become explicit decisions with tests behind them. Answer them by building the
smallest thing that must handle them — not by designing a framework first.

## Acceptance criteria

- An append-only raw stream can be written and read back containing a session boundary, at
  least one reported-intent event, and at least one observed tool lifecycle event.
- Raw events are immutable and append-oriented; nothing in the write or read path rewrites
  or reconciles an already-written event.
- Reported and observed events remain distinguishable end to end, and each event carries
  source and fidelity provenance that survives the round trip.
- Replay is deterministic and chronological, and is asserted by tests rather than
  demonstrated by hand.
- Ordering behavior is explicit: what defines the canonical order, and what happens when
  two events carry the same timestamp.
- Timestamp behavior is explicit: which clock, what resolution, and whether clock movement
  can violate the ordering guarantee.
- Schema versioning is explicit: how a version travels with the stream and what a reader
  does with a version it does not recognize.
- Truncated-tail behavior is explicit and tested: a stream cut mid-record is readable up to
  the truncation, the truncation is visible to the reader, and no partial record is
  silently accepted as complete.
- Any fixture committed is synthetic; no real recording enters the repository.
- No storage abstraction, adapter framework, or plugin surface is introduced. One concrete
  path only.
- `scripts/check.sh` passes and the slice is committed.

## Result

The round trip holds. A session recording can be emitted a record at a time by independent
processes and replayed deterministically, with reported and observed claims still
distinguishable at the other end.

### Implemented contract

Full contract in decision:3. In summary:

- **Format.** UTF-8 NDJSON, one newline-terminated line per complete record, one file per
  session. Records are appended, never rewritten. A record whose `session_id` disagrees with
  the recording is refused at both ends.
- **Envelope.** `schema_version`, `session_id`, `sequence`, `recorded_at`, `provenance`,
  `event`.
- **Provenance.** `{channel, adapter, mechanism}` where channel is `reported`, `observed`,
  or `recorder`. No numeric fidelity score — the adapter and capture point are what let a
  reader judge coverage, and a number would imply a measurement nobody has taken.
- **Vocabulary.** `session_started`, `session_ended`, `reported_intent`,
  `observed_tool_started`, `observed_tool_finished`. Tool events carry `tool_call_id`, which
  maps to the Claude hook `tool_use_id`; `reported_intent` may carry the same id to correlate
  a claim with a call without merging the two.
- **Channel coherence is enforced by the format.** `reported_intent` may only arrive
  `reported`; tool events may only arrive `observed`; boundaries may be `recorder` or
  `observed`, never `reported`. Refused at emit, corruption on read. The epistemic error is
  unrepresentable rather than merely discouraged.
- **Order.** Canonical order is physical append order, carried by `sequence` (starts at 1,
  +1 each record). Replay never consults a timestamp. Equal timestamps need no tie-breaker;
  a backward-moving clock cannot reorder anything.
- **Concurrency.** The whole read-tail / decide-sequence / write-record transaction is
  serialized by an advisory exclusive lock on the recording file (`std::fs::File::lock`,
  stable since Rust 1.89), with the file opened in append mode. No daemon, no database.
- **Versioning.** Every record carries `schema_version: 1`; any other value is an explicit
  unsupported-version error. Unknown fields are rejected, so additions require a version bump.
- **Damage.** Corruption (a whole record that cannot be understood) fails loudly with a line
  number. A truncated tail (unterminated final fragment) returns the valid prefix, reports
  the recording incomplete with offset and length, and never parses the fragment. Appending
  onto a truncated tail is refused outright. Duplicate, decreasing, and skipped sequences are
  all rejected — a gap is indistinguishable from a deletion.
- **Durability, precisely.** A successful append means written plus `sync_data` returned. It
  is not claimed atomic against power loss; a crash mid-write can still leave a fragment,
  which is why the truncated tail is a readable state rather than an error.

### Files and dependencies

- `src/record.rs` — envelope, provenance, event vocabulary, channel coherence.
- `src/append.rs` — the serialized append transaction and backward tail scan.
- `src/replay.rs` — parsing, sequence and session validation, truncation vs corruption.
- `src/error.rs` — the error taxonomy that keeps those states distinct.
- `src/lib.rs` — library surface (`#![forbid(unsafe_code)]`, `#![warn(missing_docs)]`).
- `src/main.rs` — CLI: `append --recording <PATH>` (one JSON emission on stdin) and
  `replay --recording <PATH>`. Exit 0 complete, 2 truncated tail, 1 error.
- `tests/common/mod.rs`, `tests/round_trip.rs`, `tests/ordering.rs`, `tests/damage.rs`,
  `tests/concurrency.rs`.
- `README.md` — status rewritten; it no longer claims there is no recorder, states exactly
  what the kernel does, shows a synthetic invocation, and keeps the privacy warning.

Dependencies added, all justified individually: `serde` 1.0.229 (derive) and `serde_json`
1.0.151 for NDJSON, `jiff` 0.2.35 (serde) for RFC 3339 timestamps, and `tempfile` 3.27.0 as
a dev-dependency so tests write outside the repository. No CLI framework — argument parsing
for two verbs and one flag is hand-rolled. No async runtime, no storage abstraction, no
adapter framework, no plugin surface. `publish = false` retained.

### Tests

30 tests, all passing, across four integration targets:

- `round_trip.rs` (6) — full session/intent/tool-start/tool-finish round trip; one record per
  line with earlier bytes untouched; reported vs observed provenance preserved; lifecycle
  correlation by id without collapse; an orphan finish accepted as delivered; empty recording.
- `ordering.rs` (9) — determinism (value equality and byte-stable re-render); equal
  timestamps; backward-moving clock; duplicate, decreasing, skipped, and non-1 starting
  sequences; one-recording-one-session on both read and append.
- `damage.rs` (13) — unsupported schema version at first and later records; malformed
  complete record; non-JSON line; blank line; impossible channel on read and on emit
  (both directions); truncated tail returning the valid prefix; a parseable-but-unterminated
  final record still treated as truncated; a recording that is only a fragment; append onto a
  truncated tail refused without touching the file; invalid UTF-8.
- `concurrency.rs` (2) — 16 threads × 4 appends, and 8 genuinely separate short-lived
  processes invoking the built binary. Both assert sequences are exactly 1..=n and that every
  emission landed exactly once.

The correlation test additionally asserts the agent's words appear only in the reported
record, so a future adapter cannot quietly fuse a `Bash` payload's `description` into the
observed event.

All fixtures are synthetic and obviously so: `sess-synthetic-0001`, `toolu_synthetic_0001`,
`SyntheticTool`, `/synthetic/example`. No real recording, prompt, source tree, credential, or
machine path is committed.

### Manual verification

Exercised the built CLI against a throwaway recording outside the repository: appended a
recorder boundary, a reported intent, an observed start, and an observed failure, then
replayed all four in append order (exit 0). Then, on a copy, appended a partial line and
confirmed replay printed the four-record prefix and reported a 42-byte unterminated fragment
at byte 1186 with exit 2; that appending onto it was refused; that a `schema_version: 2`
record was refused by name; and that emitting intent on the observed channel was refused
before the file was created. `scripts/check.sh` passed before and after closing this task.

### Deviations and limitations

- **Beyond the literal ask, deliberately.** Channel/event coherence enforcement, session-id
  consistency, and refusing to append onto a truncated tail were not named in the task. Each
  falls out of an acceptance criterion — the first two from keeping the channels
  distinguishable and one-recording-one-session, the third from "no partial record is ever
  presented as complete" — and each is small and tested. Flagging them so the scope creep is
  visible rather than silent.
- **Session boundaries are not required.** A recording need not open with `session_started`.
  Enforcing it would pressure an adapter that never observed a boundary into fabricating one.
  The cost is that a recording is not guaranteed self-describing.
- **Lifecycle pairing is not validated.** A finish with no start is accepted as delivered,
  because a missing start is a capture blind spot to preserve, not an error to reject.
- **Replay reads the whole file into memory.** Honest at session scale, and a deferral rather
  than a claim about large recordings. Appending, by contrast, is O(final record).
- **The lock is advisory.** The concurrency guarantee covers WitnessGlass appenders honoring
  it; a foreign writer is out of scope and undetected.
- **Object key order is normalized** inside `arguments` and `result` when a record is
  serialized. This happens before the bytes become evidence, but the stored record is not
  byte-identical to the emitter's input.
- **No adapter, and no claim of one.** Hook shapes were read from the current official
  documentation only to avoid designing something that would preclude task:4. Nothing was
  configured, and no real session was recorded.

### Scarp desire paths

**idea:1 recurred for the third time.** Recording this result again required writing it to a
temporary file and appending it to the artifact with a shell redirect before `scarp close
task:3`, because Scarp still has no command that records a task outcome. Three occurrences in
three closed tasks is now the whole sample: the workaround is not incidental, it is the
workflow. Already captured; no new idea needed.

**Observed once, not yet promoted: no way to record a relationship between artifacts.**
decision:3 arose from this task and bears directly on dragon:1, and there is no affordance to
express either link. The only edge-creating flag in 0.2.0 is `scarp close --resolved-by`,
which requires closing a dragon — and dragon:1 is not resolved, since nothing has been
measured against a real session yet. `scarp resolve` suggests prose references are the
intended mechanism, so this may well be by design rather than a gap. Cross-references were
therefore written as prose. Noting it as a single observation and deliberately not promoting
it to an idea on one occurrence, per the rule against inflating desire-path evidence; if it
recurs in task:4 it earns an idea.
