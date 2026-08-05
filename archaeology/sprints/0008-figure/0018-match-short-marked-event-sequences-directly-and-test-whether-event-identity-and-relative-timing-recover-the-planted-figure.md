---
id: tsk_01KZ9R3J2103D0DXC335S113K2
sequence: 18
kind: task
status: pending
sprint: spr_01KZ9R11YTDBKH39X3FDP68EZR
created: 2026-08-05
---

# Match short marked event sequences directly and test whether event identity and relative timing recover the planted figure

## Objective

Build the smallest event-native sequence representation and distance that could recover the planted
recurring figure, and run it at a preregistered event-count ladder against both synthetic fixtures,
two deterministic nulls, and — exploratorily — the untracked real recording.

Everything below the `### Preregistration` heading was written and committed to disk **before the
matcher was run against any fixture or recording**. The figure sizes it cites are read from the
committed fixture generators' own constants, which is inspection of ground truth rather than a
result.

## Acceptance criteria

- An event-native representation preserving licensed event identity, order, and relative timing,
  built from `crate::Inspection` like every other projection in this repository, borrowing rather
  than owning, reading no file and consulting no clock.
- A sequence distance whose decomposition is inspectable: substitutions, insertions, deletions,
  aligned gap pairs, event cost, timing cost, and combined distance all reported.
- Identity, symmetry, and non-negativity asserted where the metric intends them; indel behaviour
  asserted on unequal-length sequences; timing-jitter behaviour asserted.
- Known-answer microtests over hand-checkable sequences, passing before any fixture is evaluated.
- Both fixtures run at the preregistered ladder below, with the trivial-match exclusion policy below.
- Both nulls below run through the identical path.
- A perturbation sweep, only if basic recovery is earned.
- A direct comparison against sprint:6's sampled Matrix Profile result, and a supported/mixed/
  falsified verdict kept separate from any opinion of the implementation.
- Future legally derivable similarity facets encountered but not used, inventoried with exact
  provenance.
- `scripts/check.sh` passes unweakened; no existing test changed.

### Preregistration

#### 1. The representation

One marked event per record, in canonical order:

```text
MarkedEvent {
    mark:                 (schema-tagged event kind, delivered tool name or none)
    offset_ms:            recorded_at minus the sequence origin
    gap_from_previous_ms: recorded_at minus the previous retained event's, or none for the first
}
```

The mark is a pair of raw fields. The kind is `crate::inspection`'s schema-tagged kind, so a v1 and
a v2 kind sharing a name are different marks. The tool name is the byte string the integration
delivered, compared byte for byte, never normalized, lower-cased, stemmed, or grouped. No third
component: no channel (the kind already carries it), no path, no payload size, no correlation id, no
`prompt_id`, no reported text, no hierarchy, no duration.

A window of `k` events carries `k − 1` **within-window** gaps: the first event's gap points outside
the window and is not used. A window's timing is therefore translation-invariant — it does not depend
on when the window starts — which is the property that makes two occurrences of a figure comparable.

**Channel scope is a parameter, and the primary run is the harder one.** The planted figure in both
fixtures opens with a `reported_intent` record, and in the sparse fixture that mark appears *nowhere
except inside the figure*. Including the reported channel would hand the matcher a unique marker and
make recovery a result about the presence of a rare mark rather than about sequence structure. So:

- **primary: observed records only.** The figure must be recovered from observed tool lifecycle alone.
- **secondary: all channels**, reported beside it.

Neither scope promotes one channel into the other: every mark keeps its own kind, no reported mark is
ever equal to an observed mark, and no output describes a reported claim as an observed fact.

#### 2. The planted figure, from the committed fixture constants

| fixture | records/instance | observed-only | period | instances A / B | jitter in B |
|---|---|---|---|---|---|
| legible (`synthetic-behavioral-oracle.ndjson`, 196 records) | 9 | **8** | 8 s | 4 / 4 | start ≤700 ms, offsets ≤150 ms, one `tool_succeeded` → `tool_failed` |
| sparse (`synthetic-behavioral-oracle-sparse.ndjson`, 365 records) | 5 | **4** | 8 s | 30 / 15 | the same |

Region A and region B, as half-open millisecond intervals from each fixture's own origin:

- legible: A = `[60000, 90000)`, B = `[210000, 240000)`
- sparse: A = `[300000, 540000)`, B = `[1080000, 1200000)`

Regions are supplied to the matcher by the caller and never guessed from a fixture inside the
library, exactly as sprint:6 did it.

#### 3. The event-count ladder

Fixed number of **events**, not of wall-clock. The rule, applied to the figure length `n` in the
scope being analysed:

```text
ladder(n) = sorted unique of { 3 } ∪ { n−2, n−1, n, n+1, n+2 }, dropping anything below 3
```

which gives, for the four (fixture, scope) combinations:

| fixture | scope | n | ladder |
|---|---|---|---|
| legible | observed | 8 | **3, 6, 7, 8, 9, 10** |
| legible | all | 9 | **3, 7, 8, 9, 10, 11** |
| sparse | observed | 4 | **3, 4, 5, 6** |
| sparse | all | 5 | **3, 4, 5, 6, 7** |

Each ladder contains at least one window shorter than the figure, the figure's exact length, and two
longer. The constant `3` is a deliberate short control: it is the shortest window that has two
within-window gaps, and it is short enough that a fragment of the figure is expected to be
indistinguishable from a fragment of the baseline. No length outside these ladders is searched.

Because every window in one scan has the same `k`, an insertion is always paired with a deletion.
Genuine unequal-length indel behaviour is exercised by the microtests, not by the fixture scans.

#### 4. The distance

A weighted global alignment (Needleman–Wunsch) over marked events. Every constant below is fixed
here and is not tuned against any result.

| term | cost |
|---|---|
| aligned pair, same mark | `0` |
| aligned pair, different mark | `SUBSTITUTION = 1.0` |
| insertion or deletion | `GAP = 1.0` each |
| aligned pair, timing | `TIMING_WEIGHT × t(g_a, g_b)`, `TIMING_WEIGHT = 0.5` |

`SUBSTITUTION ≤ 2 × GAP`, so an alignment never prefers a delete-plus-insert over a substitution,
and the maximum event cost of aligning two sequences is `max(len_a, len_b)`.

Timing applies only where **both** aligned events have a within-window predecessor. The first
position of each window contributes identity but no timing, which is the same statement as "a window
has `k − 1` gaps".

#### 5. The timing policy

Bounded symmetric log-ratio, floored:

```text
t(g_a, g_b) = min( 1, | ln((g_a + FLOOR) / (g_b + FLOOR)) | / ln(RATIO_FULL) )
FLOOR = 100 ms      RATIO_FULL = 4
```

Multiplicative rather than absolute, on the reasoning the brief for this round states and this
policy is chosen to satisfy: a factor matters, a millisecond count does not.

| comparison | `t` | why this is the wanted answer |
|---|---|---|
| 1.0 s vs 1.2 s | ≈ 0.12 | modest, as a 20% difference should be |
| 1.0 s vs 8.0 s | 1.00 (clamped) | a different figure, not a jittered one |
| 100 s vs 100.2 s | ≈ 0.001 | a 200 ms discrepancy at 100 s is nothing |
| 0.1 s vs 0.3 s | ≈ 0.50 | the same 200 ms at small scale is a lot |

`FLOOR` damps the sub-100 ms region where a cooperative hook adapter's own latency lives; without it,
1 ms against 10 ms would be a full-scale disagreement about nothing. `RATIO_FULL = 4` sets a 4×
difference as maximally different. Both are chosen from the shape of the data before it is measured,
and neither is revisited after results are seen.

#### 6. Normalization and what is reported

With `L = max(len_a, len_b)` and `P` = aligned pairs where both sides carry a within-window gap:

```text
event_norm  = event_cost  / L                                    ∈ [0, 1]
timing_norm = timing_cost / (TIMING_WEIGHT × P), or 0 when P = 0  ∈ [0, 1]
total       = (event_cost + timing_cost) / (L + TIMING_WEIGHT × (L − 1))   ∈ [0, 1]
```

`total` is the ranking quantity. Since every window in a scan has the same `k`, its denominator is a
constant and ranking by `total` is ranking by raw alignment cost, so the alignment the dynamic
program minimizes is the alignment `total` is read from.

Reported for every pair: substitutions, insertions, deletions, matched pairs, `P`, `event_cost`,
`timing_cost`, `event_norm`, `timing_norm`, `total`. The three distances are never collapsed into
one number in any output.

#### 7. Trivial-match exclusion, stated explicitly

Two windows starting at event indices `i` and `j` are comparable only when `|i − j| ≥ k`: they share
no event. This is stricter than the `ceil(m/4)` exclusion zone sprint:6 inherited from `motif-rs`,
and it is stricter on purpose — a window overlapping itself by one event is not a second occurrence
of anything.

#### 8. The diagnostic that replaces `occupancy`

sprint:6's decisive column was occupancy: a matched window holding **one** non-empty bucket is a lone
impulse, not a figure. The event-native analogue is **distinct marks in the window**. A window whose
events are all one mark, or which alternates two, is a degenerate figure however perfectly it
repeats. Reported per matched window, and a window with `≤ 2` distinct marks is labelled degenerate.

Expected values: a planted legible occurrence at `k = 8` has **8** distinct marks; a legible baseline
window has **2**; a legible elevated-regime window has **4**. A planted sparse occurrence at `k = 4`
has **4**; a sparse baseline window has **2**.

#### 9. The nulls

Two, both deterministic, both run through the identical matcher, each answering a different question.

- **Order null** — permute the marks across the whole sequence with a fixed-seed Fisher–Yates,
  leaving every gap where it is. Preserves the mark multiset and the entire timing profile; destroys
  which event happened when. *Does event order carry information?*
- **Timing null** — permute the gaps across the whole sequence with the same generator, leaving every
  mark where it is, and recompute offsets cumulatively so the result is still a timeline. Preserves
  the event sequence exactly; destroys relative timing. *Does timing contribute anything beyond
  identity?*

The comparison metric, fixed before any result is seen. For each fixture, scope, and `k`:

```text
separation_order  = d_query_top1(order null)  − d_query_top1(real)
separation_timing = d_query_top1(timing null) − d_query_top1(real)
```

where the query window is the first window lying entirely inside region A, at the same event index in
the real and nulled sequences, and `d_query_top1` is that window's best distance to any window
sharing no event with it. Global best distances over all pairs are reported beside these, since a
query-anchored number and a global one answer different questions and sprint:6 conflated nothing only
because it reported both.

#### 10. Predictions

**P1 — exact repeats of degenerate figures will dominate the global ranking, at distance 0.** Both
baselines and both regime blocks are exactly periodic in mark and in gap, so non-overlapping pairs
inside them are identical and score 0. This is predicted as a certainty, not a risk. It is *not*
sprint:6's failure mode: those are genuine multi-event exact repetitions, not coincidences of two
lone events. §8's distinct-marks column is what tells the two apart, and the global top-K is expected
to be full of 1–2-mark windows.

**P2 — the planted query's top-1 non-overlapping neighbour will be another planted occurrence**, at
`k = n`, on both fixtures, in the observed-only scope. This is the criterion sprint:6 failed and it is
the one that matters.

**P3 — recovery will hold at `k = n` and `k = n ± 1` and fail at `k = 3`.** Three events is a
fragment, and the legible figure's first three observed events are `tool_requested`/`tool_succeeded`
on one tool name followed by a request on another — close enough to a baseline pair that the short
control is expected to lose the distinction.

**P4 — event identity will carry nearly all of the discrimination and timing almost none.** The
planted jitter is ≤700 ms on instance start (which is outside the window's own gaps) and ≤150 ms on
each offset, against gaps of 200–400 ms; `t` at that scale is small. Timing is predicted to be a
tie-breaker, not a discriminator, on these fixtures.

**P5 — the order null will destroy recovery and the timing null will not.** `separation_order`
positive and material; `separation_timing` near zero. This is predicted *in advance* so that a small
`separation_timing` is read as "identity is doing the work here", which is a finding, rather than as
a failure of the round.

**P6 — the injected failing call bounds the cross-region distance away from zero.** The recurrence's
third legible instance has one `tool_succeeded` replaced by `tool_failed`, which is exactly one
substitution, so any cross-region pair involving that instance has `event_norm ≥ 1/k`. The other
three instances differ only in timing. Predicted, so that a non-zero best cross-region distance is
not mistaken for a defect.

**P7 — the real recording may produce nothing.** 234 records, one session, no ground truth. A null
result there is an acceptable outcome and is not evidence against the representation.

#### 11. Success, mixed, and falsification, fixed before any result is seen

**Supported** requires all four, on **both** fixtures, in the observed-only scope, at `k = n`:

- **S1** the planted query window's top-1 non-overlapping neighbour is another planted occurrence
  (in region A or region B);
- **S2** the best cross-region A↔B planted pair appears within the query window's top-5 neighbours;
- **S3** `separation_order ≥ 0.05`;
- **S4** the planted windows in the query's top-5 are distinguishable from degenerate windows by the
  distinct-marks diagnostic — that is, the recovery is not an artefact of matching one repeated mark.

**Falsified** if either: at every `k` in the ladder the planted query's nearest neighbour is a
non-planted window; or the order null reaches within `0.05` of the real sequence's query distance,
which would say the apparent match is explained by the mark multiset and the timing profile alone.

**Mixed** is anything else — including recovery that depends on the exact `k`, recovery that survives
only in the all-channel scope, or serious false matches beside a correct one.

`separation_timing` is deliberately **not** part of any criterion. P5 predicts it is near zero by
construction of the fixtures, and a criterion that a prediction expects to fail is not a criterion.

#### 12. What this task will not do, whatever it finds

No corpus accumulation, no motif families, no variable-length discovery, no MinHash, no path or
payload or extension or edit-delta facets, no semantic categorization, no learned representation, no
Spectroscope change, no `MotifDetector` abstraction, no public motif schema, no change to the raw
format, the schema, the recorder, `inspection`, the viewer, or the product CLI. No real recording
committed or copied. Nothing pushed.
