---
id: spr_01KZ1SQTZ730K3VJMH127NMXNS
sequence: 1
kind: sprint
status: active
created: 2026-08-02
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
