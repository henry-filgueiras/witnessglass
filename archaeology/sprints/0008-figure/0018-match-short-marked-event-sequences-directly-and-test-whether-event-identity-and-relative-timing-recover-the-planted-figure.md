---
id: tsk_01KZ9R3J2103D0DXC335S113K2
sequence: 18
kind: task
status: closed
sprint: spr_01KZ9R11YTDBKH39X3FDP68EZR
created: 2026-08-05
closed: 2026-08-05
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

## Result

Delivered. **Supported**, with one preregistered criterion that was mis-specified and is reported
both ways.

The short version: keeping events as events reverses sprint:6's central failure completely. On both
fixtures, in the harder channel scope, at every rung of both preregistered ladders, the planted
figure's nearest non-overlapping neighbour is another planted occurrence at distance exactly zero —
and with degenerate windows excluded, the global minimum over every disjoint pair in the whole
recording *is* the planted figure, with nobody pointing at a region. sprint:6 could not do this at
any window, on either fixture, in any dimension.

### 1. Implementation decision: written, not adopted, and no dependency added

`cargo search` was run for sequence-alignment and edit-distance crates. Several exist and several are
good. None was adopted, for a reason the task brief anticipated and this round agrees with: the
metric that had to be tested is about thirty lines of dynamic programming plus a decomposition, and
the decomposition is the part that matters. A library that returned one scalar would have hidden
exactly the thing the round was commissioned to measure — whether event identity or timing carries
the discrimination — and a library that returned an alignment would still have needed the timing
term, the normalization, and the reporting written on top of it.

So: no new dependency, no feature gate, no `Cargo.toml` change at all. `src/experiment/
event_sequence.rs` is ~700 lines including documentation, `examples/event-motif.rs` is the whole
invocation surface, `tests/event_sequence.rs` is eighteen tests. Deleting those three files and one
line of `src/experiment.rs` removes the experiment and leaves the crate exactly as sprint:7 left it.

This is a different judgement from sprint:6's, which adopted `motif-rs` after reading its source. The
difference is not principle but size: a Matrix Profile is 11,000 lines with STUMPY-validated
numerics and a documented constant-subsequence convention, and a weighted edit distance is a table.

### 2. The representation, exactly

One marked event per record, in canonical append order:

```text
MarkedEvent {
    sequence:             the record's position in the append chain, or none if hand-built
    mark:                 (schema-tagged event kind, delivered tool name or none)
    offset_ms:            recorded_at minus the inspection's earliest recorded_at
    gap_from_previous_ms: recorded_at minus the previous *retained* event's, or none for the first
}
```

The mark is two raw fields. Nothing else is in it — no channel (the kind carries it), no path, no
payload size, no correlation id, no `prompt_id`, no agent identity, no reported text, no hierarchy,
no `duration_ms`. A tool name is compared byte for byte and is never read as a category.

A window of `k` events carries `k − 1` within-window gaps; the first event's gap points outside the
window and is unused, so a window's timing is translation-invariant. `tests/event_sequence.rs`
asserts that a window whose first event carries a gap of 9,999,999 ms is at distance zero from the
same window whose first event carries none.

Sizes, measured:

| fixture | records | observed retained | filtered by channel |
|---|---|---|---|
| legible oracle | 196 | 186 | 10 |
| sparse oracle | 365 | 318 | 47 |
| real recording | 234 | 169 | 65 |

`clamped_gaps` is zero everywhere: no clock moved backwards in any of the three, which matches
sprint:4, sprint:5, and sprint:6. The clamp is implemented and tested and remains unexercised by real
evidence, and that is recorded rather than glossed.

### 3. The distance, and how to hand-check it

Weighted global alignment. Substitution `1.0`, insertion and deletion `1.0` each, timing
`0.5 × min(1, |ln((g_a + 100)/(g_b + 100))| / ln 4)` on every aligned pair where both sides have a
within-window predecessor.

At `k = 3` the denominator is `3 + 0.5 × 2 = 4`, which makes two microtests checkable on paper:

```text
A: Reader --1.0s--> Searcher --2.0s--> Editor
C: Reader --1.0s--> Shell    --2.0s--> Editor     one substitution -> 1.0 / 4        = 0.250
D: Reader --1.0s--> Searcher --8.0s--> Editor     one 4x stretch   -> 0.5x0.973 / 4  = 0.122
E: Reader ---------2.0s--------------> Editor     one deletion     -> 1.0 / 4        = 0.250
```

`d(A,A) = d(A,B) = 0`, `d(A,B) < d(A,C)`, `d(A,B) < d(A,D)`, `d(A,B) < d(A,E)`, all asserted. The
timing policy's four preregistered reference values hold: 1.0 s against 1.2 s is 0.12, against 8.0 s
is 1.00, 100 s against 100.2 s is under 0.005, and 0.1 s against 0.3 s is 0.50 — the same 200 ms
costing four hundred times more at small scale than at large, which is the whole reason the policy is
multiplicative and floored.

`d(A,C)` and `d(A,E)` coming out equal is a genuine property of unit costs, not a coincidence being
hidden: one substitution and one deletion cost the same. It is visible in the decomposition, where
one is `sub 1 / ins 0 / del 0` and the other `sub 0 / ins 0 / del 1`.

Identity, symmetry, non-negativity, and boundedness of all three normalized components are asserted
over a seven-sequence corpus, with insertions and deletions required to swap roles under exchange.
**The triangle inequality is not claimed and not tested.** The timing term is a bounded metric on
gaps, but it is attached to whichever alignment the dynamic program chose, and this round needs a
ranking rather than a metric space.

### 4. Known-answer microtests: all passing before the oracle was touched

Twelve tests — microtests, metric properties, the timing policy's four values, the ladder rule, the
exclusion policy, the projection's ordering and gap arithmetic, both nulls' invariants, and both
nulls' determinism — were written and passing before any fixture scan was run. Six more were added
afterwards to pin what the scans found.

One of the twelve is worth naming: `the_perturbation_base_agrees_with_the_committed_fixtures_first_
planted_occurrence`. The perturbation sweep's base figure is hand-built from the oracle's own
generator constants rather than carved out of the fixture, and that test asserts it agrees mark for
mark and gap for gap with the window the ordinary projection extracts. "Hand-built" therefore does
not mean "unverified".

### 5. Legible oracle, observed scope, figure length 8

`--figure 8 --region-a 60000:90000 --region-b 210000:240000`

```text
  global, over every pair of windows sharing no event:
    k  windows      pairs     best  best-nd  null-ord  null-tim
    3      184      16471    0.000    0.000     0.000     0.000
    6      181      15400    0.000    0.000     0.210     0.007
    7      180      15051    0.000    0.000     0.229     0.035
    8      179      14706    0.000    0.000     0.261     0.043
    9      178      14365    0.000    0.000     0.298     0.038
   10      177      14028    0.000    0.000     0.332     0.056

  anchored at the query window — the first window lying entirely inside region A:
    k  query   q-best     q-ev    q-tm  q-nbr  null-ord  null-tim  sep-ord  sep-tim
    3     20    0.000    0.000   0.000     28     0.145     0.023   +0.145   +0.023
    6     20    0.000    0.000   0.000     28     0.465     0.133   +0.465   +0.133
    7     20    0.000    0.000   0.000     28     0.431     0.146   +0.431   +0.146
    8     20    0.000    0.000   0.000     28     0.464     0.143   +0.464   +0.143
    9     20    0.000    0.000   0.000     36     0.464     0.150   +0.464   +0.150
   10     20    0.000    0.000   0.000     36     0.485     0.162   +0.485   +0.162
```

The query's neighbour list at `k = 8`, in full to rank 8:

```text
  1. idx  20 [ 60.1s +2.1s] marks 8 <-> idx  28 [ 68.1s +2.1s] marks 8  AA  ev 0.000 tm 0.000 tot 0.000
  2. idx  20                          <-> idx  36 [ 76.1s +2.1s] marks 8  AA  ev 0.000 tm 0.000 tot 0.000
  3. idx  20                          <-> idx  44 [ 84.1s +2.1s] marks 8  AA  ev 0.000 tm 0.000 tot 0.000
  4. idx  20                          <-> idx 162 [218.5s +2.1s] marks 8  AB  ev 0.000 tm 0.087 tot 0.026
  5. idx  20                          <-> idx 154 [210.4s +2.1s] marks 8  AB  ev 0.000 tm 0.098 tot 0.030
  6. idx  20                          <-> idx 178 [234.4s +2.2s] marks 8  AB  ev 0.000 tm 0.114 tot 0.035
  7. idx  20                          <-> idx 170 [226.4s +2.2s] marks 8  AB  ev 0.125 tm 0.107 tot 0.119  sub 1
  8. idx  20                          <-> idx  29 [ 68.3s +7.8s] marks 8  AA  ev 0.250 tm 0.000 tot 0.174  ins 1 del 1
```

**Ranks 1 to 7 are exactly the seven other planted occurrences** — three in region A, four in region
B, in ascending order of jitter — and rank 7 is the instance carrying the fixture's injected failing
call, at exactly one substitution. Rank 8 is the first window that is *not* occurrence-aligned: it
straddles two instances, needs an insertion and a deletion to line up, and jumps to 0.174.

### 6. Sparse oracle, observed scope, figure length 4

`--figure 4 --region-a 300000:540000 --region-b 1080000:1200000`

```text
  global, over every pair of windows sharing no event:
    k  windows      pairs     best  best-nd  null-ord  null-tim
    3      316      49141    0.000    0.000     0.000     0.000
    4      315      48516    0.000    0.000     0.015     0.000
    5      314      47895    0.000    0.000     0.019     0.000
    6      313      47278    0.000    0.000     0.118     0.006

  anchored at the query window:
    k  query   q-best     q-ev    q-tm  q-nbr  null-ord  null-tim  sep-ord  sep-tim
    3     20    0.000    0.000   0.000     24     0.125     0.000   +0.125   +0.000
    4     20    0.000    0.000   0.000     24     0.273     0.017   +0.273   +0.017
    5     20    0.000    0.000   0.000     28     0.286     0.085   +0.286   +0.085
    6     20    0.000    0.000   0.000     28     0.294     0.076   +0.294   +0.076
```

Same shape, at four times the length. The query's neighbour list at `k = 4`:

- ranks **1–29**: the twenty-nine other occurrences inside region A, every one at `tot 0.000`;
- ranks **30–43**: fourteen of the fifteen occurrences inside region B, `ev 0.000`, `tm 0.052` to
  `0.159`, `tot 0.014` to `0.043` — the jitter, and nothing but the jitter;
- rank **44**: the fifteenth, the one carrying the injected failure, `ev 0.250 tm 0.080 tot 0.204`;
- rank **45**: `tot 0.364`, a phase-shifted window needing an insertion and a deletion.

**The query's top forty-four neighbours are precisely the forty-four other planted occurrences**,
and the first thing that is not one costs nearly nine times the worst thing that is.

### 7. The result that needed no query at all

The strongest number in the round is unanchored. Excluding pairs where either window is degenerate —
the event-native analogue of sprint:6 masking constant subsequences, using §8's preregistered
diagnostic and no new definition — **the global minimum over every disjoint pair in the whole
sequence is a pair of planted occurrences at distance zero**, on both fixtures, in both channel
scopes.

```text
legible, observed, k=8, non-degenerate global rank 1:
  idx 20 [60.1s +2.1s] marks 8 <-> idx 28 [68.1s +2.1s] marks 8  AA  ev 0.000 tm 0.000 tot 0.000
sparse,  observed, k=4, non-degenerate global rank 1:
  idx 20 [300.1s +0.8s] marks 4 <-> idx 24 [308.1s +0.8s] marks 4  AA  ev 0.000 tm 0.000 tot 0.000
```

Nobody supplied a region. The detector was asked "what is the most similar pair of non-degenerate
windows anywhere in this recording" and it answered with the figure the fixture was built to contain.

### 8. Against the preregistered predictions

- **P1 held exactly.** The *unrestricted* global top pairs are degenerate: two marks, alternating
  request and success in the baseline, at distance 0. That is `best` = 0.000 in every row of both
  global tables. The distinct-marks column is what separates them from the figure — 2 against 8
  (legible) and 2 against 4 (sparse), exactly the values §8 predicted.
- **P2 held**, on both fixtures, and at *every* rung rather than only at `k = n`.
- **P3 was wrong, in the useful direction.** Recovery was predicted to fail at the `k = 3` short
  control and it does not: the query's top-1 at `k = 3` is still a planted occurrence at distance 0.
  The reason is visible in the data — the figure's first three observed events carry **three**
  distinct marks while a baseline fragment carries two, so even a three-event window of the figure is
  not confusable with a baseline fragment. What *does* degrade at `k = 3` is the null separation:
  `+0.145` legible and `+0.125` sparse, against `+0.464` and `+0.273` at the figure length, and the
  global order null reaches 0.000 at `k = 3` on both fixtures. So the short control is not
  discriminating; it is merely lucky, and the null is what says so.
- **P4 held.** `q-ev` is 0.000 in every row of both anchored tables. Event identity does all the
  discriminating on these fixtures; timing only orders the planted occurrences among themselves.
- **P5 held, and is the round's second most important number.** `sep-ord` is `+0.27` to `+0.49`
  across the ladders; `sep-tim` is `+0.00` to `+0.16` and is at or near zero at the short rungs. The
  order null destroys recovery. The timing null does not, because identity alone already identifies
  the figure. This was predicted in advance precisely so it would be read as "identity is doing the
  work here" rather than as a failure.
- **P6 held exactly.** The injected failing call is exactly one substitution, `event_norm` `1/8` and
  `1/4` respectively, and it is the worst-ranked of the planted occurrences on both fixtures.
- **P7 did not apply.** The real recording produced usable candidates. See §11.

### 9. Criterion S2 was mis-specified, and both readings are reported

S2 asked that "the best cross-region A↔B planted pair appears within the query window's top-5
neighbours". On the legible oracle it is rank 4 and S2 passes. On the sparse oracle the first
cross-region pair is **rank 30**, and S2 fails.

It fails for a reason that is a defect in the criterion rather than in the result. The sparse
fixture's region A contains thirty instances of the figure, exactly periodic, and twenty-nine of them
are at distance exactly 0 from the query. No cross-region pair *can* reach the top five: the top five
is full of correct answers. A criterion that a perfect detector cannot satisfy is not measuring what
it was written to measure.

The criterion is kept exactly as written. Beside it, the measurement it should have taken:

| fixture | rank of first A↔B pair | planted-occurrence prefix | distance at the prefix boundary |
|---|---|---|---|
| legible | 4 | ranks 1–7 | 0.119 → 0.174 |
| sparse | 30 | ranks 1–44 | 0.204 → 0.364 |

*Planted-occurrence prefix* is the number of leading neighbours that are occurrence-aligned: inside a
planted region, needing no insertion or deletion, differing by at most the one injected substitution.
On both fixtures it is exactly the number of other planted occurrences that exist. The test
`the_cross_region_recurrence_is_recovered_and_the_injected_failure_costs_one_substitution` pins that,
and deliberately asserts the alignment's own indel counts rather than region membership, because a
window straddling two instances lies inside a region too.

**This is the same shape of mistake as sprint:6 §4**, where a preregistered cross-region criterion
was too tight by up to a whole window and an overlap criterion had to be reported beside it. Twice is
a pattern: preregistering a *ranking* criterion against a fixture whose region contains many
occurrences requires thinking about how many correct answers precede the one being asked for, and
neither round did.

**The verdict, both ways.** By the letter of §11, S2 fails on the sparse fixture and the verdict is
**Mixed**. By the substance — did the representation recover the planted figure and separate from the
null — every other criterion passes on both fixtures at every rung, the unanchored masked global
minimum is the figure itself, and the answer is **Supported**. §13 gives the verdict as Supported and
says so with this paragraph attached, so a reader who prefers the letter has everything needed to
read it the other way.

### 10. Null and control

Two nulls, both fixed-seed Fisher–Yates, both run through the identical matcher, both anchored at the
same event index as the real query.

- **Order null** — marks permuted, every gap and offset left in place. Mark multiset preserved
  (asserted), timing profile preserved (asserted), receipts dropped because a permuted mark is not
  what that record said (asserted).
- **Timing null** — gaps permuted among positions after the first, marks left in place, offsets
  recomputed cumulatively so the result is still a timeline (asserted). Gap multiset preserved
  (asserted).

Both are deterministic across runs, asserted.

The order null separates: `+0.145` to `+0.485` on the legible oracle, `+0.125` to `+0.294` on the
sparse one. The timing null barely does, as P5 predicted.

**One place the null is uncomfortable, and it is reported rather than smoothed.** The *global*
order-null best on the sparse oracle at `k = 4` is `0.015` and at `k = 5` is `0.019` — close enough to
zero that an unanchored comparison against it would prove almost nothing. It is the *anchored*
comparison that carries the result there, because the anchored question is "what does this specific
window find in a stream whose order has been destroyed" rather than "does any pair anywhere match".
On a 318-event sequence with a mark vocabulary of four, some pair of four-event windows agreeing by
chance is unsurprising. The global null becomes informative at `k = 6` (`0.118`) and on the legible
oracle from `k = 6` upward.

### 11. Real recording, exploratory

The same untracked 234-record session sprint:4, sprint:5, sprint:6, and sprint:7 used. Not committed,
not copied, and nothing depends on its presence. 169 observed records retained, 65 filtered by
channel. No ground truth exists for it, so no region was supplied, no query window exists, and only
the global tables are meaningful.

```text
    k  windows      pairs     best  best-nd  null-ord  null-tim
    3      167      13530    0.001    0.002     0.004     0.002
    6      164      12561    0.015    0.064     0.108     0.056
    7      163      12246    0.020    0.077     0.157     0.060
    8      162      11935    0.043    0.099     0.218     0.095
    9      161      11628    0.044    0.095     0.232     0.098
   10      160      11325    0.074    0.086     0.266     0.122
```

Nothing reaches distance zero, which is the first thing worth saying: a real session contains no two
disjoint eight-event windows that are identical in identity *and* spacing. The order null sits at
roughly twice the non-degenerate best across the middle of the ladder.

**Manual inspection of the strongest non-degenerate candidate at `k = 8`**, distance 0.099, between
event indices 41 and 157:

```text
A, from +135.7 s, extent 42.4 s          B, from +986.7 s, extent 20.7 s
  tool_requested/Write                     tool_requested/Write
     --0.0s--> tool_succeeded/Write           --0.0s--> tool_succeeded/Write
    --11.0s--> tool_requested/Bash            --5.0s--> tool_requested/Bash
     --1.8s--> tool_succeeded/Bash            --2.3s--> tool_succeeded/Bash
     --6.3s--> tool_requested/Bash            --3.4s--> tool_requested/Bash
     --2.6s--> tool_succeeded/Bash            --2.8s--> tool_succeeded/Bash
    --19.0s--> tool_requested/Bash            --5.2s--> tool_requested/Bash
     --1.7s--> tool_succeeded/Bash            --2.1s--> tool_succeeded/Bash
```

The event sequences are identical, mark for mark: `ev 0.000`, no substitution, no insertion, no
deletion. The whole distance is timing — one span is twice as long as the other.

> **Experimental interpretation, not evidence.** A human reading these two regions **does** agree
> they are structurally similar: a `Write` request and its outcome, followed by three `Bash` requests
> each with its outcome, in the same order, at the same order of magnitude of spacing. What that
> similarity is *about* — whether the same kind of work was being done — is not established here, and
> no name is given to it. `Bash` is a delivered tool-name string and this round has read it as
> nothing else.

The second candidate, distance 0.100, between indices 117 and 153, is likewise identical in identity:
two `Bash` request/outcome pairs, a `Write` pair, and a third `Bash` pair, with one long stall inside
each (93.1 s in one, 66.7 s in the other).

**Compare sprint:6's strongest real-recording motif**: two 64-second windows each containing exactly
one non-empty bucket, ten minutes apart, in otherwise idle stretches, at distance 0.0, of which that
round wrote "a human reading the projection does **not** agree that these regions contain
meaningfully similar behaviour". This round's strongest real-recording candidate is eight events
carrying the same four marks in the same order. That is the difference the representation made, on
data with no ground truth at all.

No metric weight was changed after seeing any of this, as §5 of the preregistration requires.

### 12. Direct comparison with sampled Matrix Profile

| | sprint:6, sampled univariate Matrix Profile | sprint:8, event-native alignment |
|---|---|---|
| representation | 500 ms count raster, 78–94% empty | ordered marked events with relative gaps |
| what the top match usually was | two windows holding **one** non-empty bucket each | two occurrences of the planted figure |
| planted figure recovered? | **never**, at any window, on either fixture | at every rung of both ladders |
| best synthetic separation | `+0.548` (sparse, 128 s) — from a 1/1-occupancy pair | `+0.485` (legible) / `+0.294` (sparse) — from the figure |
| unanchored global minimum | two constant stretches, or two lone impulses | the planted figure, once degenerate windows are excluded |
| null reaching 0 | at every window up to 32 s, on everything | only at `k = 3`, and anchored comparisons never |
| real-recording top match | two lone spikes ten minutes apart; human disagreed | eight events, same four marks in order; human agreed |
| what dominated the ranking | emptiness, then lone-event alignment | exact repetition of *degenerate* figures |

The last row is the honest caveat and it is a real one: this representation has its own abundant
trivial match. It is not the same one. sprint:6's trivial matches were coincidences — two unrelated
events sharing a within-window offset, carrying no information about anything. This round's are
genuine multi-event exact repetitions of a two-mark figure, which is a real property of the recording
and merely an uninteresting one. The distinct-marks diagnostic separates them in one column, and the
masked global ranking then puts the planted figure first. Nothing analogous worked for sprint:6:
masking constant subsequences there left lone-event alignment untouched, because the lone-event
windows were not constant.

### 13. Verdict: **Supported**

Event-native matching recovers the planted figure substantially better than sampled Matrix Profile
and survives the order null. Specifically:

- **S1** — the planted query's top-1 non-overlapping neighbour is another planted occurrence, on both
  fixtures, at **every** rung of both ladders rather than only at `k = n`. Passed.
- **S2** — passed on the legible fixture (rank 4), failed on the sparse fixture (rank 30) for the
  reason §9 gives, which is a defect in the criterion. Reported both ways.
- **S3** — `separation_order ≥ 0.05` required; measured `+0.464` legible and `+0.273` sparse at the
  figure length, and positive at every rung. Passed.
- **S4** — the planted windows carry one distinct mark per event (8 and 4), no degenerate window
  reaches the query's top five, and the recovery is not an artefact of one mark repeating. Passed.

Falsification did not trigger on either clause: the planted query's nearest neighbour is never a
non-planted window, and the order null never comes within 0.05.

**Kept separate, as the brief requires.** That is a verdict on the *representation*. The particular
metric implemented here is a plain weighted edit distance with one timing term and five constants; it
is not elegant, it is not tuned, and nothing about the verdict depends on it being either. A
different simple sequence distance over the same representation would very likely have produced the
same three-way answer, and that is the point — the round tested whether events-with-gaps is a better
substrate than a sparse raster, not whether this particular table is a good one.

### 14. The strongest limitation of the event-native representation

**It cannot tell an interesting repetition from a boring one, and on real data most repetition is
boring.**

The global minimum of an unfiltered search is a pair of two-mark windows in both fixtures and,
implicitly, in the real recording. The distinct-marks diagnostic rescues the synthetic case because a
planted figure was deliberately built out of four distinct tool names, and there is no guarantee real
behaviour is like that: a genuinely interesting recurring figure made of `Bash` alone would be
indistinguishable, by this diagnostic, from an idle loop made of `Bash` alone. §8's diagnostic is a
heuristic that happens to fit these fixtures, and calling it more than that would be exactly the kind
of claim this project exists not to make.

Two further limits, smaller but real. The metric is `O(n²k²)` over all pairs, which is fine at 169 to
365 events and would not be at 10⁵. And the fixed event count means an insertion is always paired
with a deletion, so the indel machinery — which the microtests exercise properly on unequal lengths —
does almost no work in the fixture scans; variable-length figures are exactly what this round refused
to attempt.

### 15. Future similarity facets encountered and deliberately not used

Each is mechanically derivable from raw evidence already in the recording, and each is stated with
the provenance it actually has. None is used by anything in this round.

| candidate facet | where it already exists | the claim it is, precisely |
|---|---|---|
| `recorded_input_json_bytes` | `requested_input` / `effective_input` on the raw record | the serialized size of the input value **this recording holds**, after JSON normalization — not the size of what the integration delivered, and not the size of anything on disk |
| `recorded_response_json_bytes` | already a sprint:4 dimension | the serialized size of the response value this recording holds — **not** "bytes read from the filesystem", and not what a tool emitted |
| delivered file path | a key inside the uninterpreted input payload | the path string an integration delivered on a tool call — not evidence a file was read, written, or exists |
| path extension, repository-relative region | derivable from the above by string operations | properties of a delivered string. Two calls agreeing on `.rs` agree about a string |
| old/new edit payload sizes | inside `effective_input` on edit-shaped calls | sizes of two strings this recording holds. Not a diff, not a byte delta on disk |
| working-set union over a window | the set of delivered path strings in that window | the set of paths **named** in a window's payloads. First contact already measured the counterexample: files changed by a shell command with no path anywhere in any event |
| `duration_ms` | `PayloadFacets::duration_ms` | supplied on some completions and absent on others, and absent is not zero. Any similarity use needs a policy for the absences, which is decision:5's most-likely-broken condition |
| `interrupted` | `PayloadFacets::interrupted` | a delivered flag, absent meaning the integration said nothing either way |
| delivered error text | `PayloadFacets::error` | a string on a failure record |
| distinct correlation ids in a window | already a sprint:4 dimension | how many `tool_use_id` values appear, which says nothing about whether calls overlapped |

The two most tempting for a next round are the input and response byte counts, because they are
already extracted, already named honestly, and would add a magnitude axis the current mark deliberately
lacks — two windows with the same four marks in the same order but response sizes differing by three
orders of magnitude are currently at distance zero on the identity component. The trap is the naming,
and the table above exists so the naming is decided before the facet is used, not after.

**Not on this list, and still not derivable**: any semantic category of a tool name, any classification
of a command string, any account of what changed on disk, anything segmented by `prompt_id` while
dragon:3 is open.

### 16. Desire-path friction

**The preregistration still has no home in Scarp, for the third round running.** Same shape as
sprint:5 and sprint:6: the ladder rule, four ladders, five metric constants, the timing policy, both
nulls, seven predictions, and the success and falsification criteria all had to exist *before* the
matcher ran, and the only place to put them is a `###` subsection inside `Acceptance criteria`, where
they are neither criteria nor marked write-once. The evidence that they predate the run is `19fe52a`,
a commit made for that purpose alone and containing nothing else — which this round checked
deliberately, having read sprint:6's own addendum about `363ac20` being broader than its message.
Already **idea:5**; recorded as a third occurrence and nothing more.

**A criterion had to be corrected in prose, again.** §9 above is the second consecutive round where a
preregistered criterion turned out mis-specified and the repair had to be written as a paragraph
beside it. There is no way to mark a section as superseded-with-reason, so "the criterion was wrong
and here is the measurement it should have taken" is a convention this project has now invented twice
by hand. That is idea:4's and idea:5's shared territory rather than a new gap, and it is recorded as
the second occurrence.

**Appending a Result and an Outcome is still `cat >>`, and that is a version lag rather than a missing
design.** The `scarp` on this machine is 0.2.0 and `scarp close` offers only `--resolved-by`.
maintenance:1 records that upstream shipped result-on-close. Cited here as a version lag only, which
is the correction sprint:6's addendum made and this round is deliberately not repeating.

**One small new thing.** `scarp list` with no argument is an error — the collection is required — so
there is no single command that shows the state of the project across dragons, ideas, decisions,
sprints, and tasks. Five invocations answer it. Noted as first-occurrence friction; not promoted,
because one round is not a pattern and the workaround is five seconds.

**What went well, and is worth recording as such.** `scarp new sprint --body-file` followed by
`scarp new task --sprint sprint:8 --body-file` worked first time for both artifacts, with no section
rejection — the first round in four without one. The reason is that the section names were read off
an existing artifact first, which is idea:2's workaround rather than idea:2 being fixed, but the
workflow itself was frictionless.

### 17. Recommendation: exactly one next experiment

**Cross-recording figure matching, on two real recordings, with the same metric and the same
discipline — and nothing else.**

Not corpus accumulation, not motif families, not variable-length discovery, not any of the deferred
facets, and not a visualization. One question:

> Does a figure recovered inside one real recording match a figure inside a *different* real
> recording, above what a null over the two sequences' pooled marks would produce?

The reason it is next, rather than adding a facet or lengthening the metric, is that every result in
this round is within a single recording, and the interesting downstream hypotheses — that recurring
figures are a property of how an agent works rather than of one session — all require the
cross-recording step to be true, and none of them survives it being false. It is cheap: the matcher
already takes two sequences, `align` does not know or care which recording a window came from, and
the only new machinery is a null over two sequences instead of one. It is falsifiable in one round.
And it is the smallest possible next step that could invalidate the whole direction, which is the
property this line of work has been selecting for since sprint:4.

sprint:5's changepoint recommendation remains untaken and untouched, and this round adds nothing for
or against it.

**One follow-up recorded and not implemented, per the sprint's own non-goals.** If a visualization of
this result is ever wanted, the smallest useful one is a single strip of the recording's events as
marks along the `recorded_at` axis, with the query window and its top-k neighbours drawn as bands
beneath it and the event/timing decomposition printed per band. It would show, in one picture, that
the top forty-four neighbours of a sparse-oracle occurrence are the forty-four other occurrences.
That is a suggestion, it belongs to whatever sprint takes it, and the Behavioral Spectroscope was not
touched by this round.

### What this task did not do

No corpus accumulation, no motif families, no variable-length discovery, no MinHash, no Jaccard, no
path or extension or payload-magnitude or edit-delta facets, no semantic categorization, no learned
representation, no cantrip or proposal generation, no A/B testing, no historical replay, no workflow
compilation. No `MotifDetector` abstraction and no public motif schema. No change to the raw format,
the schema, the recorder, `inspection`, the viewer, the Spectroscope, or the product CLI. No
dependency added and no `Cargo.toml` change. No existing test altered and no check weakened. No real
recording committed or copied. Nothing pushed.

One observation made and not acted on: `cargo build --examples` without `--features
experiment-matrix-profile` fails, because `examples/spectroscope.rs` has no `required-features` entry
in `Cargo.toml` while `examples/matrix-profile.rs` does. `scripts/check.sh` uses `--all-features` so
the gate never sees it. It predates this round and fixing it here would have been an unrelated change
in an experiment commit.
