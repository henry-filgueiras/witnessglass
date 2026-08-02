---
id: tsk_01KZ1SR0ZWB01W5T3DEM05YQXZ
sequence: 3
kind: task
status: pending
sprint: spr_01KZ1SQTZ730K3VJMH127NMXNS
created: 2026-08-02
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
