---
id: tsk_01KZ2CDMWHMAW5ZYPD5KPBTB2V
sequence: 8
kind: task
status: pending
sprint: spr_01KZ2CCWX3JTRFXRG957Y8A2DR
created: 2026-08-02
---

# Project raw recordings into a receipt-bearing inspection model

## Objective

Build the pure Rust inspection projection that sits between a replayed recording and everything
the sprint renders, and record the raw/projection/browser boundary as an accepted decision once
that boundary is concrete and tested.

**Prerequisites: none.** This is the sprint's first task, and the other three depend on it.

The projection is a derived, disposable view under decision:5 — rebuildable from the raw stream,
never rewriting it, safe to delete. Its distinguishing property is that every derived entity
carries the raw sequence numbers supporting it, so a reader can always get from a claim back to
the records that licensed it. **A derived claim that cannot produce its receipts is asserting,
not deriving.**

What it may claim is fixed by task:4's measurements rather than by taste, and the temptation
this task has to resist is the pleasant one: filling a gap with a plausible value because the
gap is inconvenient to render.

## Acceptance criteria

- The projection is computed from a replayed recording and holds no fact the raw stream cannot
  regenerate. It never rewrites, overwrites, or tidies raw evidence, and discarding it loses
  nothing (decision:5, condition 1).
- Both the raw records and their canonical append sequence survive into the projection.
  `sequence` remains the only total order; no timestamp is sorted on (decision:3).
- Correlation is limited to relationships the evidence licenses, principally `tool_use_id` — the
  one identifier whose semantics this project has actually tested. Nothing derives parentage,
  causality, concurrency, or execution duration from containment, adjacency, or timing.
- Every derived tool lifecycle, anomaly, aggregate, and grouping retains the raw sequence numbers
  that support it.
- Paired and unpaired tool lifecycle evidence is represented without rejecting survivable
  anomalies. A request with no observed outcome, an outcome with no observed request, conflicting
  outcomes for one `tool_use_id`, and unmatched subagent boundaries are all first-class states
  rather than errors. decision:3 deferred pairing to a projection precisely so that a missing
  half is recorded as a blind spot; decision:4 made the fateless request first-class; task:4
  measured a real subagent stop with no observed start.
- Supplied agent attribution is represented exactly as supplied. An absent `agent_id` is
  "identity not supplied", never "root agent", and no parent identity is invented.
- Missing boundaries, unmatched requests and outcomes, conflicting outcomes, and unmatched
  subagent boundaries are detected and exposed rather than smoothed over.
- Channel, event-kind, adapter, mechanism, and supplied-agent aggregates are computed with
  wording that describes **records** rather than unseen reality: what was recorded, not what
  happened. Zero `tool_failed` records means no failure record was observed (task:4, dragon:1).
- v1, v2, complete, empty, and truncated recordings all project. v1's `observed_tool_started` is
  not silently equated with v2's `tool_requested`; decision:4 froze v1 exactly because the two
  mean different things. Truncation is carried into the projection as a first-class state.
- Prompt and turn grouping, and any execution-duration claim, are refused by construction rather
  than by convention (dragon:3; task:4 measured zero `duration_ms` across 82 completions).
- An accepted decision records the boundary: what the raw stream owns, what the projection may
  derive, what a browser may render, and — explicitly, as decision:5 asked rather than deferred
  by convenience — whether and how a projection may be served over a local HTTP port. It is
  written after the boundary is concrete and tested, and it does not restate the sprint plan.
- Focused tests cover, with synthetic fixtures only: equal and backward recorder timestamps;
  absent `duration_ms`; absent parentage; the duplicated reported description decision:4
  documents and task:4 measured 65-for-65; orphaned lifecycle records in both directions;
  conflicting outcomes for one `tool_use_id`; an unmatched subagent stop; and v1/v2 schema
  differences.
- The projection may be serializable for the local API, but it is not advertised as a stable
  public interchange format yet.
- No recording in `.witnessglass/` is read, listed, or copied by this task or its tests.
- `scripts/check.sh` passes, the slice is committed, and dragons 1–3 stay open.
