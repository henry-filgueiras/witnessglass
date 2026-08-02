---
id: tsk_01KZ1SR1012J6W4MBRJ80NNX2J
sequence: 4
kind: task
status: pending
sprint: spr_01KZ1SQTZ730K3VJMH127NMXNS
created: 2026-08-02
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
