---
id: spr_01KZCA9DMMGRAEBF9G31VKMWN5
sequence: 19
kind: sprint
status: closed
created: 2026-08-06
closed: 2026-08-06
---

# Datum

## Goal

Calibrate the **complete R1-based search procedure** against the existing order null: does that
procedure produce more exceptional maxima on observed event sequences than the same procedure produces
after sequential structure is destroyed?

## Rationale

sprint:18 ended with R1 as self-information under a stated model and explicitly *not* calibrated
evidence, for one reason above the others: every real score is read at a maximum that a search chose.
No amount of further work on the statistic addresses that. Only a null that runs the same search does.

Two premises this round depends on turned out to be false as stated, and both were found in the code
rather than assumed. The search does not rank by R1 at all — it ranks by alignment cost and R1 is a
readout applied afterwards. And the existing null-referenced machinery holds candidate boundaries
fixed and permutes only identity, which is precisely the non-search-aware comparison this round exists
to avoid. Both are recorded before anything is preregistered, and neither is worked around quietly.

The round may well conclude that the order null is inadequate for this purpose. Agent event streams
carry near-deterministic local adjacency by construction — the schema correlates a tool request with
its own outcome — and a null that destroys that will separate from observation whether or not any
reusable motif exists. Establishing that would be a real result, not a failure.

## Success criteria

- Premises verified from code; every discrepancy recorded before preregistration.
- `T` derived from the implemented machinery, not from an assumed pipeline, with every stage of
  candidate generation, ranking, deduplication and selection named.
- Observed and null paths compute the identical `T`, with every data-dependent stage rerun inside each
  replicate. No boundary chosen on observed data and rescored after permutation.
- The order null documented property by property as preserved or destroyed, and the hypothesis it
  represents stated exactly.
- A selection-effect demonstration on this project's own machinery, visualized.
- Positive and negative controlled fixtures before any observational specimen is touched.
- `B` and every seed chosen before execution, with the computational rationale recorded.
- A propagation pass covering every numerical rule — PASS, FAIL and verdict branch alike.
- Specimen-level results reported; no corpus-wide verdict forced over a disagreement.

## Non-goals

No change to R1, the null, the search, the alignment, or the representation. No new score, no second
detector, no aggregate summary statistic invented to tidy a result. No adoption and no promotion to
production. No significance threshold chosen after seeing data. No interpretation of recording
contents, no workflow names assigned to discovered spans, and no treatment of observational recordings
as ground truth. Nothing pushed.
