---
id: spr_01KZEXTYYH9AQX2M7JM5YCA63M
sequence: 21
kind: sprint
status: closed
created: 2026-08-07
closed: 2026-08-07
---

# Eyepiece

## Goal

Build and run a repeatable, local-only corpus-analysis workflow over a directory of recordings, and
produce a field report that a technically capable reader who has not internalized sprint:19 and
sprint:20 can act on — while preserving, rather than spending, the epistemic honesty those rounds
earned.

## Rationale

Twenty sprints have produced a detector, three nulls, a calibration, and a collapse. What they have not
produced is an answer to the question an operator actually asks: *what does this pile of recordings
contain?*

The gap is not statistical. It is that every surface reads one recording, and that the only projection
a human can look at is the raw event stream, whose grammar is the recorder's rather than the agent's —
`tool_requested` followed by `tool_succeeded`, over and over, with `Bash` on more than half of them.
A reader handed that stream learns about the hook protocol.

This round is deliberately **exploratory engineering, not a preregistered experiment.** decision:7
governs how experimental criteria are written; nothing here is a criterion, no verdict partition is
declared in advance, and no claim of this round may be cited as an experimental result. What it must
do instead is state, in plain English and at the point of use, which of its outputs are descriptions
and which are calibrated — and never let the second borrow authority from a round that measured a
different projection.

sprint:20 is the specific hazard. Its collapse was measured on the raw event projection under the exact
doublet null, and the workflow projection this round introduces is a different sequence over a
different vocabulary. It gets its own null or its output is a description.

## Success criteria

- decision:9 recorded before the workflow exists, reconciling a directory-reading, cross-recording,
  report-emitting command with §6's standing non-goals without widening any of them.
- One documented command produces `report.md`, `facts.json` and `manifest.json` from an arbitrary
  compatible recordings directory; a second run on the same inputs produces byte-identical
  deterministic outputs, verified mechanically rather than asserted.
- Discovery, replay and validation go through `replay_file` and `inspection`. No second reader of raw
  NDJSON exists.
- Every discovered file is accounted for: included, or skipped with a reason, by opaque identity.
  Nothing disappears silently.
- A named, versioned workflow projection — the observed tool-action stream — that correlates an
  observed request with its observed terminal outcome and preserves incomplete, denied, failed and
  ambiguous cases instead of manufacturing a clean step. The raw event projection is left exactly as
  sprint:8 through sprint:20 left it.
- Candidates discovered by the existing search machinery, not by a frequency table, and retained with
  enough coordinates, receipts, alignment and scores to be inspected locally.
- Prevalence counted primarily by distinct eligible sessions, with non-overlapping occurrence counts
  reported beside it and never in place of it.
- A calibration measured on the workflow projection itself, against the exact first-order null, or an
  explicit statement that a lane is descriptive.
- The raw event projection run as an instrument-grammar control, and shapes it explains quarantined
  rather than reported as findings.
- Synthetic fixtures proving both that a planted cross-session shape is recovered and that
  request→outcome alternation and a dominant repeated tool are not promoted into findings.
- `scripts/check.sh` green and unweakened. No recording, and no real-corpus output, committed.

## Non-goals

No daemon, watcher, index, cache, or anything outliving the command. No network, upload, export,
share affordance or hosted mode. No language model in the pipeline. No new product subcommand and no
dependency from the product on `crate::experiment`. No new specimen admitted to decision:8. No change
to `Mark`, to the raw projection, to `cross_pairs`, `dedupe_overlapping`, `align`, `Observation`, R1,
`complete_search`, or to any null construction. No new statistic proposed as a repair to R1. No
retroactive reinterpretation of sprint:19's or sprint:20's conclusions. No verdict partition, no
threshold defended as preregistered, and no claim that this round establishes anything about agent
behaviour in general. No recording content in any artifact.
