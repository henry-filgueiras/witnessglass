---
id: ide_01KZ2414BPGN4NXW1D8GHVB86Q
sequence: 3
kind: idea
status: parked
created: 2026-08-02
---

# Scarp: record relationships between artifacts as validated edges

## Problem

Scarp artifacts refer to each other constantly — a task implements a decision, a decision
informs a dragon, a task repairs the conformance of an earlier one — but 0.2.0 has no way to
record any of those relationships as data. The single edge-creating affordance is
`scarp close --resolved-by`, which applies only to dragons and only at the moment of
closing.

So cross-references end up in prose. `scarp resolve` can turn `decision:3` in body text into
a stable id, which suggests prose is the intended mechanism, and for reading that is fine.
The gap is that nothing validates it. A prose reference is not checked by `scarp doctor`, is
not visible from the other end, and does not survive being wrong: if the referenced artifact
is renumbered, superseded, or simply mistyped, the reference silently points at nothing and
no command will ever say so.

That matters most for exactly the artifacts worth keeping. A conformance repair whose link
to the decision it repairs has quietly rotted is archaeology that has lost the thing it was
recording.

## Sketch

An explicit edge, created the same way everything else is:

```sh
scarp link task:5 --conforms-to decision:3
```

with a small closed vocabulary rather than free-form labels — something like `refines`,
`implements`, `conforms-to`, `supersedes`, alongside the existing `resolved-by`. Stored as
stable ids in front matter, so `scarp doctor` can verify both ends still exist and `scarp
show` can display the reverse direction.

The narrower version, if a link command is too much: teach `scarp doctor` to check prose
references. It would catch the rot without adding a vocabulary or any new state.

## Boundaries

- Not a general graph database, and not a dependency scheduler. Edges are for reading
  history, not for computing anything.
- A closed vocabulary or none. Free-form edge labels would decay into the same unvalidated
  prose this is meant to replace.
- Should not require edges. Most artifacts do not need one, and demanding them would turn a
  useful affordance into ceremony.
- No opinion on whether edges can be added after creation, though a conformance repair is
  usually recognized only in hindsight, which argues for yes.

## Evidence

Observed twice in consecutive tasks during WitnessGlass work on Scarp 0.2.0.

In task:3, decision:3 was created by the task and bears directly on dragon:1; neither link
could be expressed, and `close --resolved-by` was unavailable because dragon:1 is not
resolved — nothing has been measured against a real session yet. Recorded at the time as a
single observation, deliberately not promoted.

In task:5, the task exists purely as a conformance repair to decision:3. That relationship
is the entire reason the task exists, and it survives only as a sentence in the objective.
Second occurrence, so promoted here rather than noted a second time.
