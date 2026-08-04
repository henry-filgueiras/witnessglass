---
id: tsk_01KZ780S6Z3E51YETFHH8GZ91C
sequence: 15
kind: task
status: closed
sprint: spr_01KZ77XQGFN2C3CD0WWESMAT98
created: 2026-08-04
closed: 2026-08-04
---

# Decompose behavioral signals across dyadic scales with a Haar transform

## Objective

Run one detector against the sprint:4 substrate: a Haar multiresolution decomposition, per
dimension, over both a legible and a realistically sparse synthetic oracle, and then — exploratory
only — over a real untracked recording. Earn or reject the next detector.

**What a Haar transform is being asked to do here, stated narrowly.** It operates on an
already-sampled signal. It does not choose, derive, or justify the 500 ms sampling interval, and it
cannot see anything faster than that interval — structure below the sampling rate is absent from
the input, which is not the same as absent from the session. What it offers is a decomposition
across dyadic scales, from which the distribution of energy is *evidence about which scales carry
behaviour*. That evidence can inform a later Matrix Profile subsequence length or a coarser
aggregation. sprint:4's §8 claim that Haar "answers the width question" is corrected here and not
repeated.

**The null that makes the results readable.** A 94%-empty signal is a train of isolated impulses,
and an isolated unit impulse has an exactly known orthonormal-Haar signature: detail energy `2^-L`
at level `L`, halving at every coarser scale, regardless of what produced it. Any result that
merely reproduces that decay has measured the sparsity and not the session. Every reading in this
task is therefore reported against that null, and the experiment can fail.

## Acceptance criteria

- A Haar decomposition in `witnessglass::experiment`, small enough to read in one sitting, with:
  the orthonormal convention stated explicitly; a deliberate, documented policy for input lengths
  that are not powers of two, which neither pads with invented values nor silently discards
  evidence; and an exact energy identity — input energy equals total detail energy plus final
  approximation energy plus set-aside remainder energy — held by a test rather than asserted.
- Tests against tiny hand-checkable vectors whose coefficients can be verified by hand, including
  a constant vector, a single impulse, an odd-length vector, and a vector whose full
  level-by-level decomposition is written out.
- A second committed synthetic fixture, generated deterministically like the first and regenerable
  byte for byte, whose emptiness at 500 ms falls in the 90–95% band the real recording motivates.
  It carries a repeated motif at a known period, a substantially longer regime block, and a
  recurrence with deterministic jitter. The legible oracle's committed bytes are unchanged, and a
  test says so.
- Both fixtures' measured empty-bucket percentages reported. The two are labelled for what they
  are: the existing one intentionally legible and best-case, the new one a stress case informed by
  observed density.
- Each licensed dimension decomposed independently. No multivariate fusion, and no path by which
  one dimension's magnitude can reach another dimension's result.
- Per dimension and level: the detail energy, its share of that dimension's total detail energy,
  the isolated-impulse null share, and the ratio between them.
- The predictions below tested as written, without revision after the fact.
- The heavy-tailed dimension handled by measurement: the analysis run with it and without it, with
  the equality or inequality of every other dimension's spectrum between the two runs *computed*
  rather than assumed. A `ln(1+x)` variant of that one dimension may be reported, clearly labelled
  exploratory, and is not adopted as policy.
- One invocation surface, an example binary, not the product CLI. No new dependency.
  `scripts/check.sh` passes unweakened and no existing test is changed.
- Exactly one next experiment recommended, with its empirical reason, "stop" included as a live
  option.

### Predictions, recorded before the transform was run

Convention assumed by all of these: orthonormal Haar, `a=(x₀+x₁)/√2` and `d=(x₀−x₁)/√2`, applied
repeatedly to the approximation. A level-`L` detail coefficient is computed from `2^L` consecutive
base samples and measures the contrast between two adjacent means each `2^(L−1)` samples wide. The
scale reported for level `L` is the window, `2^L × 500 ms`: level 1 = 1 s, 2 = 2 s, 3 = 4 s,
4 = 8 s, 5 = 16 s, 6 = 32 s, 7 = 64 s, 8 = 128 s, and upward.

**The null.** An isolated unit impulse has detail energy exactly `2^-L` at level `L`. Normalized
across the available levels that is approximately 50%, 25%, 12.5%, 6.3%, 3.1%, 1.6%, 0.8%, 0.4%.
A dimension made of well-separated single records should track it. Reading the level-1 peak that
results as a finding would be reading the sparsity.

**1 — periodic structure should produce a cutoff, not a peak.** For an impulse train of period `P`
samples aligned to the dyadic grid, levels whose half-window `2^(L−1)` is below `P` see at most one
impulse per half and produce large details; levels whose half-window reaches `P` see equal numbers
in both halves and cancel. So energy should be present up to `log₂(P)` and drop above it. The 8 s
motif is 16 base samples, so the prediction is: energy through **level 4 (8 s)** and a ratio-to-null
materially below 1 at **level 5 (16 s)** and above, in motif-dominated dimensions —
`channel:reported`, `kind:v2:reported_intent`, and `tool_name:SyntheticSearcher`.

Two dilutions are expected in advance: the motif occupies only part of each recording, so
cancellation competes with whatever else the dimension does; and Haar is not shift-invariant, so
the jittered recurrence should cancel less cleanly than the exact original.

**2 — block structure should produce a coarse-scale excess.** A dimension on for one contiguous
block and off otherwise is a rectangular pulse: at fine scales only its two edges carry energy, and
at scales approaching its width it behaves like one large impulse. Ratio-to-null should exceed 1
near the block width. In the legible oracle the elevated regime is 60 s ≈ 2^7 base samples, so the
prediction is excess at **levels 6–8 (32–128 s)** in `recorded_response_json_bytes`,
`tool_name:SyntheticShell`, and `tool_name:SyntheticReader`. In the sparse oracle the block is
300 s, which is deliberately *not* a power of two — between 2^9 (256 s) and 2^10 (512 s) — so the
prediction is excess at **levels 9–10**, smeared across both rather than concentrated. One
structure in these fixtures flatters Haar and one does not, on purpose.

**3 — constant dimensions should have exactly zero energy at every level.** `kind:v2:tool_denied`
in both fixtures.

**4 — lone impulses should track the null.** `kind:v2:session_started` and `kind:v2:session_ended`
are one record each. Ratio-to-null ≈ 1 at every level. This is the control that makes the null
column readable rather than merely assumed.

**5 — sprint:4's normalization policy cannot affect any of this.** Haar detail coefficients are
exactly invariant to a constant offset and scale linearly with a constant factor, so energy
*shares* are invariant to both. A z-score is exactly such an offset and factor. The prediction is
that the share spectrum computed from raw counts and from z-scored values is identical to
floating-point tolerance for every dimension — and therefore that the heavy-tailed dimension cannot
contaminate any other dimension's spectrum, and that this round's findings are independent of the
Round 1 normalization question.

**6 — the sparse oracle should look worse, and the question is how much.** At ~93% empty, more
windows contain nothing at all and the impulse null should dominate more strongly. The prediction
is the same qualitative signatures as 1 and 2, with smaller departures from the null.

**Falsification condition, fixed in advance.** If every dimension of both fixtures produces a
ratio-to-null within roughly ±25% of 1 at every level — that is, if nothing departs from the
isolated-impulse decay — then Haar has found the sparsity and not the structure, and the
recommendation is to stop rather than to proceed to a second detector.

## Result

Delivered. **Haar exposes the injected structure, decisively, on both fixtures**, and it does so in
a way that distinguishes the two kinds of injected structure from each other and both from the
sparsity that dominates a real recording. Five of the six predictions recorded above held; the
sixth was wrong in an informative direction and is recorded as wrong rather than dropped.

The falsification condition fixed in advance — everything within ±25% of the isolated-impulse null
— was not met, and not narrowly: observed ratios span **0.02 to 34.4**.

No Matrix Profile and no changepoint detector was implemented.

### 1. The transform and its conventions

`witnessglass::experiment::haar`, one file, no dependency. Orthonormal Haar:
`a[k] = (x[2k] + x[2k+1])/√2`, `d[k] = (x[2k] − x[2k+1])/√2`, applied repeatedly to the
approximation until fewer than two samples remain. There is no inverse, no second wavelet, no
filter bank, and no boundary-extension mode.

**Scale convention, fixed and stated.** A level-`L` coefficient is computed from `2^L` consecutive
base samples and contrasts two adjacent means each `2^(L−1)` wide. Both readings are carried:
`Level::scale_ms` is the window (`2^L × base_ms`, so at 500 ms level 1 = 1 s, 4 = 8 s, 10 = 512 s)
and `Level::contrast_ms` is the half-window. Every table in this result reports the window.

**Non-power-of-two lengths.** At each level, if the current approximation has odd length its final
unpaired sample is set aside as a `Remainder` — level, index, value, energy — and does not
propagate. Zero-padding was rejected because in *this* signal a zero is a meaningful observation
("no record in that interval"), so padding would fabricate evidence of quiet; periodic and
symmetric extension invent values that look like data; truncation discards up to half the
recording; carrying an unpaired sample forward unscaled breaks orthonormality and with it the only
exact check the module has.

**The exact check.** Orthonormality gives
`input_energy = detail_energy + approximation_energy + remainder_energy`, held by a test across
every length from 0 to 64 and across every dimension of both fixtures. Worst observed residual on
a real projection: `4.8e-10`. This is the reconstruction-class sanity property, obtained without
implementing an inverse.

**What the policy costs, discovered by running rather than by predicting.** A remainder is excluded
from the level that set it aside *and from every coarser level*, because it never reaches the next
approximation. A level-1 remainder is therefore represented at **no level at all**. Both fixtures
have odd sample counts, so each puts its final base sample in the level-1 remainder — and in each,
that sample is the only one the `kind:v2:session_ended` dimension has. It decomposes to zero detail
energy everywhere. The transform now reports that as `Silence::OnlyInRemainders`, distinct from a
genuinely flat dimension, because a reader who cannot tell them apart reads an artefact of the
transform as a property of the recording.

Coverage falls with depth and is now printed per level. The legible oracle's 481 samples are 100%
covered through level 5 and **53% covered at level 8**; the sparse oracle's 2401 hold 85% at its
coarsest; the real recording's 2108 hold 97%. A coarse-scale ratio is a statement about that much
of the recording and no more.

### 2. The sparse companion fixture, and measured sparsity

`fixtures/synthetic-behavioral-oracle-sparse.ndjson`: 365 records over 1200 s, 2401 buckets at
500 ms, generated deterministically and regenerable byte for byte with
`cargo run --example behavioral-signal -- --emit-sparse-oracle`. Its session id, adapter, origin,
prompt id, correlation-id prefix, and entire tool vocabulary are disjoint from the legible oracle's,
and a test asserts neither fixture contains the other's identifiers.

```text
       0 ..  300000   sparse baseline   one two-record call every 30 s
  300000 ..  540000   motif             a five-record figure every 8 s, exactly
  540000 ..  780000   sparse baseline   as before
  780000 .. 1080000   regime block      one two-record call every 6 s, a tool name
                                        used nowhere else, one subagent pair
 1080000 .. 1200000   recurrence        the same figure with deterministic jitter,
                                        and one call that fails
 1200000              session ends
```

**Measured empty-bucket percentage at 500 ms:**

| fixture | buckets | empty | what it is |
|---|---|---|---|
| legible oracle | 481 | **78.2%** | intentionally legible, best case |
| **sparse oracle** | 2401 | **92.7%** | stress case, in the band observation motivates |
| real untracked session | 2108 | **94.4%** | the observation |

A test asserts the sparse fixture lands in 90–95%, that the legible one stays below 85%, and that
the two differ by more than ten points, so the contrast cannot silently erode.

**One structure is dyadic and one deliberately is not.** The motif period is 8 s = 16 base samples,
a power of two, and is the positive control. The regime block is 300 s = 600 base samples, sitting
between 2^9 and 2^10, and cannot land on a single level. A fixture in which every structure
happened to be dyadic would be a fixture built to flatter one transform.

The fixture takes exactly one number from observation — roughly how empty a real session's buckets
are — and nothing else. No content, timing, tool name, or payload is derived from any recording.

### 3 and 4. Predictions against outcomes

The predictions are above, written before the transform ran. Verdicts:

| # | prediction | verdict |
|---|---|---|
| 1 | an 8 s motif shows as a **cutoff above its period**, not a peak at it | **held**, both fixtures |
| 2 | a block shows as a **coarse-scale excess** | **held in direction**, and the data sharpened *where* |
| 3 | a constant dimension has zero energy at every level | **held**, both fixtures |
| 4 | a lone impulse tracks the null | **held**, to within 0.02 at every level |
| 5 | sprint:4's normalization cannot move a share | **held**, to `7.2e-16` |
| 6 | the sparse fixture shows *smaller* departures than the legible one | **falsified** — it shows larger ones |

**Prediction 1, held.** The two signatures are not subtle. On the sparse oracle, the motif-only
tool:

```text
dimension: tool_name:SparseSyntheticSearcher
    L     window        energy    share     null   ratio    covers
    1       1.0s     1166.3481   48.92%   50.02%    0.98    100.0%
    2       2.0s      620.0838   26.01%   25.01%    1.04    100.0%
    3       4.0s      332.1878   13.93%   12.51%    1.11    100.0%
    4       8.0s      166.0939    6.97%    6.25%    1.11    100.0%   <- through the period
    5      16.0s        5.5365    0.23%    3.13%    0.07    100.0%   <- cliff above it
    6      32.0s        2.7682    0.12%    1.56%    0.07     98.6%
```

Ratio 1.11 at the 8 s period and **0.07** at 16 s. The mechanism is the one predicted: once a
half-window reaches the period, both halves hold equal numbers of instances and the difference
cancels. The same shape appears on `channel:reported` and `kind:v2:reported_intent` in both
fixtures — 1.0, 1.0, 1.0, 1.0, then 0.3 (legible) or 0.1 (sparse).

**A period therefore shows as the last level before a cliff, not as a peak.** Anyone reading these
spectra looking for a bump at the period would have concluded there was no periodicity.

**Prediction 2, held in direction; the run sharpened where.** Predicted: excess near the block
width. Observed: the excess sits where one half of the window falls inside the block and the other
outside — around *twice* the block width — while a window fitting entirely inside the block cancels
like any other constant stretch.

```text
dimension: tool_name:SparseSyntheticShell        (the 300 s block-only tool)
    8     128.0s       16.4748    0.69%    0.39%    1.76     96.0%
    9     256.0s        0.0958    0.00%    0.20%    0.02     85.3%   <- fits inside: cancels
   10     512.0s       80.5062    3.36%    0.10%   34.39     85.3%   <- spans its edge: peaks
   11    1024.0s       40.2531    1.68%    0.05%   34.39     85.3%
```

The prediction named levels 9–10 "smeared across both". It is not smeared: 256 s is a deficit of
0.02 and 512 s an excess of 34.4. The prediction was directionally right and mechanically
incomplete, and the correction is the data's.

On the legible oracle the 60 s regime produces the same shape at 32–64 s:
`recorded_response_json_bytes` reaches 4.1 and 5.8 there, `tool_name:SyntheticShell` 1.6 and 2.1,
and `records`, `channel:observed`, and `distinct_correlation_ids` 2.9, 2.8, and 3.4 at 64 s.

One named dimension did not behave: `tool_name:SyntheticReader` was predicted to show block excess
and shows none (max 1.1). It is not a rectangular pulse — it is on for 150 s, off for 60 s, on for
30 s — so the prediction misclassified it. Recorded as a wrong call about which dimension, not
about the mechanism.

**Prediction 4, held, and it is what makes everything else readable.** `kind:v2:session_started` —
one record — produces shares of 50.20%, 25.10%, 12.55%, 6.27%, 3.14%, 1.57%, 0.78%, 0.39% against
a null of exactly the same values. Ratio 1.00 at every level of both fixtures. The null column is
therefore a measured control, not an assumption.

**Prediction 6, falsified.** The sparse fixture shows *sharper* departures than the legible one
(motif cutoff 0.07 vs 0.3; block excess 34.4 vs 5.8). Density was not the limiting factor. What
mattered was how isolated a structure is from everything else in the same dimension, and how many
dyadic levels the recording's length supports — the sparse fixture is five times longer and reaches
level 11. **A sparser recording is not automatically a harder one for this transform**, which is
the opposite of what sprint:4's framing implied and is worth carrying forward.

### 5. Per-scale results, both fixtures

Ratio to the isolated-impulse null, by window scale, at the 500 ms base. `.` is a dimension that is
zero everywhere; `REM` is one whose only observation fell into the level-1 remainder.

```text
LEGIBLE ORACLE (481 samples, 78.2% empty)
  dimension                          1.0s 2.0s 4.0s 8.0s 16.0s 32.0s 64.0s 128.0s
  (base samples still covered)       100% 100% 100% 100% 100%  93%  80%  53%
  records                             0.9  0.7  1.7  1.6  0.6  0.8  2.9  0.3
  channel:reported                    1.0  1.0  1.0  1.0  0.3  0.8  1.3  0.5
  channel:observed                    1.0  0.7  1.5  1.4  0.5  0.8  2.8  0.2
  channel:recorder                    1.0  1.0  1.0  1.0  1.0  1.0  1.0  1.0
  kind:v2:session_started             1.0  1.0  1.0  1.0  1.0  1.0  1.0  1.0
  kind:v2:session_ended               REM  REM  REM  REM  REM  REM  REM  REM
  kind:v2:reported_intent             1.0  1.0  1.0  1.0  0.3  0.8  1.3  0.5
  kind:v2:tool_requested              0.9  0.7  1.8  1.3  0.5  0.7  2.5  0.2
  kind:v2:tool_succeeded              1.0  0.8  1.2  1.4  0.5  0.7  2.5  0.2
  kind:v2:tool_failed                 1.0  1.0  1.0  1.0  1.0  0.0  0.0  0.0
  kind:v2:tool_denied                   .    .    .    .    .    .    .    .
  kind:v2:subagent_started            1.0  1.0  1.0  1.0  1.0  1.0  1.0  0.0
  kind:v2:subagent_stopped            1.0  1.0  1.0  1.0  1.0  1.0  0.0  0.0
  tool_name:SyntheticReader           1.0  1.1  1.1  0.7  0.2  0.4  0.7  0.2
  tool_name:SyntheticSearcher         1.0  0.7  1.4  1.4  0.3  1.0  1.7  0.7
  tool_name:SyntheticEditor           1.0  1.2  0.8  0.7  0.3  0.7  3.2  0.2
  tool_name:SyntheticShell            1.1  1.2  0.6  0.6  0.4  1.6  2.1  0.2
  distinct_correlation_ids            0.8  0.7  2.1  2.1  0.7  0.9  3.4  0.7
  recorded_response_json_bytes        1.3  0.6  0.4  0.3  0.4  4.1  5.8  0.0

SPARSE ORACLE (2401 samples, 92.7% empty)
  dimension                          1.0s 2.0s 4.0s 8.0s 16.0s 32.0s 64.0s 128.0s 256.0s 512.0s 1024.0s
  (base samples still covered)       100% 100% 100% 100% 100%  99%  96%  96%  85%  85%  85%
  records                             0.5  1.4  1.9  1.7  0.2  0.2  0.5  1.4  0.9 22.3  2.2
  channel:reported                    1.0  1.0  1.0  1.0  0.1  0.1  0.4  1.9  0.8 16.8 13.1
  channel:observed                    0.6  1.4  1.7  1.6  0.3  0.2  0.4  1.0  0.7 20.1  0.4
  channel:recorder                    1.0  1.0  1.0  1.0  1.0  1.0  1.0  1.0  1.0  1.0  1.0
  kind:v2:session_started             1.0  1.0  1.0  1.0  1.0  1.0  1.0  1.0  1.0  1.0  1.0
  kind:v2:session_ended               REM  REM  REM  REM  REM  REM  REM  REM  REM  REM  REM
  kind:v2:reported_intent             1.0  1.0  1.0  1.0  0.1  0.1  0.4  1.9  0.8 16.8 13.1
  kind:v2:tool_requested              0.6  1.4  1.7  1.5  0.3  0.2  0.4  0.9  0.7 19.4  0.6
  kind:v2:tool_succeeded              0.6  1.4  1.7  1.5  0.3  0.2  0.4  0.9  0.7 19.5  0.6
  kind:v2:tool_failed                 1.0  1.0  1.0  1.0  1.0  1.0  1.0  1.0  0.0  0.0  0.0
  kind:v2:tool_denied                   .    .    .    .    .    .    .    .    .    .    .
  kind:v2:subagent_started            1.0  1.0  1.0  1.0  1.0  1.0  1.0  1.0  1.0  1.0  1.0
  kind:v2:subagent_stopped            1.0  1.0  1.0  1.0  1.0  1.0  1.0  1.0  1.0  1.0  1.0
  tool_name:SparseSyntheticReader     1.0  1.0  1.1  1.1  0.4  0.3  0.2  1.3  0.3  8.5 11.9
  tool_name:SparseSyntheticSearcher   1.0  1.0  1.1  1.1  0.1  0.1  0.5  2.1  0.8 18.3 14.3
  tool_name:SparseSyntheticShell      1.0  1.0  1.0  0.5  0.3  0.3  0.1  1.8  0.0 34.4 34.4
  distinct_correlation_ids            0.5  1.3  2.1  2.0  0.3  0.2  0.3  1.9  0.6 16.4  0.5
  recorded_response_json_bytes        1.0  1.1  1.2  0.7  0.3  0.3  0.1  0.8  0.1 28.7 14.8
```

**One alignment control, run because the motif period was chosen dyadic.** Re-sampling the sparse
oracle at a 700 ms base makes the 8 s period 11.43 samples — not a power of two. The cutoff
survives and blurs: `tool_name:SparseSyntheticSearcher` reads 1.0, 1.1, 1.2, **0.5**, 0.2, 0.1
across 1.4 s, 2.8 s, 5.6 s, 11.2 s, 22.4 s, 44.8 s. The transition moves to the level straddling
the period and softens from a factor of 15 to a factor of 2.4. Haar is not shift- or
scale-invariant and a non-dyadic period costs contrast; it does not hide it.

### 6. The heavy-tailed dimension did not cause trouble, and this was measured twice

**It cannot contaminate another dimension, for a reason stronger than "we ran it separately".**
Haar detail coefficients are exactly invariant to a constant offset — the difference of two values
does not move when both move — and scale linearly with a constant factor, so energy *shares* are
invariant to both. Two consequences, both tested:

- **Condition A versus condition B.** Running with `recorded_response_json_bytes` and running with
  `--exclude recorded_response_json_bytes` produce **bit-identical** spectra for every other
  dimension of both fixtures. Verified by computing both and comparing, not by asserting that
  per-column analysis must give that. Also checked at the command line: the two summary tables
  diff clean.
- **The Round 1 normalization policy cannot move a single share.** A z-score is exactly an offset
  and a factor. Worst per-level share difference between raw counts and z-scored values, across
  every dimension of both fixtures and of the real recording: **`7.2e-16`, `3.9e-16`, `3.9e-16`**.
  This round's findings are therefore *independent of the Round 1 normalization question
  entirely*, and that question can be adjudicated later on its own evidence without invalidating
  anything here.

**Its own spectrum is unusual, and the block explains it rather than the tail.** On the sparse
oracle it reads 1.0, 1.1, 1.2, 0.7, 0.3, 0.3, 0.1, 0.8, 0.1, **28.7**, 14.8 — the block signature,
because response bytes are concentrated in the regime block by construction.

**Exploratory `ln(1+x)`, clearly labelled and not adopted.** Under the transform the same
dimension reads 0.7, 1.3, 1.5, 1.2, 0.3, 0.2, 0.3, 0.6, 0.5, **21.6**, 0.6. The coarse peak
survives at 21.6. So the heavy tail moves the fine-scale detail somewhat and does not change what
the dimension says. **No normalization policy change is proposed by this task.** The relevant
finding for a future adjudication is that for *scale-spectrum* purposes the question is moot, and
any argument for changing the policy has to come from a use that is not scale-invariant.

Verdict against the three options the task posed: the heavy-tailed dimension **merely has unusual
energy itself**. It does not affect interpretation of any other dimension, and it does not poison
the experiment.

### 7. The real recording, exploratory only

Run locally against the same untracked 234-record session sprint:4 used. Not committed, not copied,
and nothing here depends on its presence. Aggregate shape only.

```text
REAL SESSION (2108 samples at 500 ms, 94.4% empty, 20 dimensions, 3 constant)
  dimension                          1.0s 2.0s 4.0s 8.0s 16.0s 32.0s 64.0s 128.0s 256.0s 512.0s 1024.0s
  (base samples still covered)       100% 100% 100%  99%  99%  97%  97%  97%  97%  97%  97%
  records                             0.9  1.0  0.9  0.8  1.3  1.3  2.1  3.3  1.7  7.3 10.9
  channel:reported                    1.0  1.0  0.8  1.1  1.0  1.1  2.0  2.2  0.4  3.4 12.9
  channel:observed                    0.9  1.0  1.0  0.8  1.3  1.4  1.9  3.3  2.4  8.2  8.1
  channel:recorder                      .    .    .    .    .    .    .    .    .    .    .
  kind:v2:session_started             1.0  1.0  1.0  1.0  1.0  1.0  1.0  1.0  1.0  1.0  1.0
  kind:v2:reported_intent             1.0  1.0  0.8  1.1  1.0  1.1  2.0  2.2  0.4  3.4 12.9
  kind:v2:tool_requested              1.0  1.0  0.9  0.8  1.2  1.1  1.6  2.6  2.0  7.0  7.0
  kind:v2:tool_succeeded              0.9  1.0  1.1  0.8  1.0  1.3  1.2  2.3  1.9  5.6  5.3
  tool_name:Bash                      0.9  1.0  0.9  1.1  1.1  1.6  2.3  2.8  0.6  4.1 16.0
  tool_name:Read                      0.7  1.0  1.0  2.1  1.6  2.7  1.9  3.6  3.6  3.6  0.1
  tool_name:Write                     1.0  1.0  1.0  1.0  1.0  1.0  1.0  1.0  0.2  1.0  0.2
  tool_name:Agent                     1.0  1.0  1.0  1.0  1.0  1.0  1.0  1.0  1.0  1.0  0.0
  tool_name:Edit                      1.0  1.0  1.0  1.0  0.6  1.4  1.0  0.2  3.4  1.8  5.0
  distinct_correlation_ids            1.0  1.0  0.9  0.7  1.3  1.8  2.5  2.9  1.8  8.1 11.2
  recorded_response_json_bytes        0.9  1.1  1.0  1.0  0.6  1.2  1.2  3.0  2.3  5.0  1.0
```

Answering the questions posed, and no more than they license:

- **Which dimensions have non-trivial energy, and at which scales.** `records`,
  `channel:observed`, `channel:reported`, `distinct_correlation_ids`, `tool_name:Bash`,
  `tool_name:Read`, `recorded_response_json_bytes`, and both tool-lifecycle kinds all depart from
  the null, and **every one of them departs upward and only at coarse scales**. `tool_name:Read`
  is the single dimension with any mid-scale excess (2.1 at 8 s, 2.7 at 32 s).
- **Is the spectrum dominated by sparsity?** At fine scales, yes, completely. Across levels 1–5
  (1 s to 16 s) almost every dimension sits between 0.7 and 1.6, which is what isolated impulses
  alone produce. Below about 32 s this recording is, to this transform, indistinguishable from its
  own emptiness. That is a statement about what Haar found, not a claim that nothing is there.
- **Which signature does it match?** This is the useful comparison, and it is only available
  because the oracles established both signatures first. The recording shows **the block
  signature** — several dimensions dip near 256 s and then spike at 512 s, which is exactly the
  "window inside the regime cancels, window spanning its edge peaks" shape the sparse oracle
  produced for a contiguous block. **No dimension shows the periodic signature**: nothing has the
  flat-then-cliff shape a repeating motif produced in both fixtures.
- **Is 500 ms too coarse or unnecessarily fine?** Unnecessarily fine, for this recording and this
  question. Levels 1–5 add nothing above the null, so a resampling at 4–8 s would lose nothing
  this transform can measure. Confirmed directly by re-running at a 4000 ms base, where the same
  crossover appears one level lower and the coarse excesses survive (`tool_name:Bash` 11.7,
  `distinct_correlation_ids` 7.7). **Absence of measurable excess is not absence of structure**,
  and this does not license a claim that nothing happens below 32 s.

Nothing above assigns meaning to a peak. "`tool_name:Read` carries excess energy near a 32 s
scale" is what the evidence says. What the agent was doing is not in evidence and is not stated.

**Backwards timestamps remain tested but unexercised.** Neither fixture contains one, and the real
recording reported zero non-monotonic records. sprint:4's statement stands unchanged.

### 8. Desire-path friction

**The section-template friction from task:14 recurred, on the other collection.** `scarp new task`
refused a body containing a `## Predictions, recorded before the transform was run` heading,
naming the two sections a task has. Same shape as last round's `## Hypothesis` refusal on a sprint,
same excellent error message, same workaround — demote to `###` and re-run. It is now two
collections in two consecutive rounds.

The sharper version of the complaint, which last round only half-stated: **this experiment needed
to record predictions before running, as a first-class thing, and Scarp has nowhere to put them.**
They live inside `## Acceptance criteria` as a subsection, which is the wrong place semantically —
a prediction is not a criterion — and nothing in the artifact's structure marks them as
write-once. The integrity of the whole round rests on those predictions predating the run, and the
only evidence of that is the git history, not the tool. idea:2's affordance (expose a collection's
sections before the first artifact) would have saved the failed command; it would not have solved
this. Not promoted to a new idea: one round is not a pattern, and if a third experiment wants the
same thing it is worth writing up properly then.

**Appending a Result and an Outcome is still `cat >>`**, for the fifth round running. Already
idea:1, extended by idea:4. No new information, recorded only because the count is the point.

**Everything else was frictionless**, including one thing worth naming as a positive: `scarp new
sprint` before `scarp new task --sprint sprint:5` was two commands and no ceremony, and having
sprint:4 closed meant there was no ambiguity about where this work belonged.

### 9. Recommendation: changepoint detection

Exactly one next experiment, and it is **not** the one sprint:4 assumed would follow.

**The empirical reason.** This round did not just find structure; it established two *distinguishable*
signatures and then found only one of them in real data.

- A repeating motif produces a flat-to-null spectrum up to its period and a cliff above it. Both
  fixtures show it emphatically (ratio 1.1 → 0.07).
- A contiguous regime produces a deficit at scales inside it and a large excess at the scale
  spanning its edge. Both fixtures show it (ratio 0.02 → 34.4).
- **The real recording shows the second and not the first.** Its departures are all coarse-scale
  excesses with a dip below them; no dimension has the periodic cliff.

Matrix Profile searches for repeated subsequences. The evidence available says the structure this
recording carries is not repetition, and a Matrix Profile run against it would be looking for the
one thing the spectrum gives no sign of. There is a second, independent problem: the scales where
the excess actually lives are 512 s and 1024 s on a 1054 s recording, which is one or two windows.
A motif search needs many candidate windows; at those scales there are none to compare.

Changepoint detection is what the block signature is evidence for, and it is cheap to falsify: the
sparse oracle has a known regime boundary at 780 s and a known return at 1080 s, the legible oracle
has one at 150 s and 210 s, and a detector either recovers them or does not.

**Two parameters that experiment should take from this round rather than guess:**

- **Resample at 4–8 s rather than 500 ms.** Levels 1–5 of the real recording carry nothing above
  the impulse null, so the fine sampling is buying resolution that this evidence cannot use, and
  the coarser series is denser, shorter, and cheaper. 500 ms should stay the substrate default —
  it is right for resolving a single call's request/outcome gap — and the *analysis* should
  resample.
- **Expect the boundary, not the period.** The candidate scale for a regime is hundreds of seconds,
  not tens.

**What would change this recommendation:** a recording that does show the periodic cliff. One
session is one session, and the block-only reading is a finding about this recording and not about
coding agents. If a second real recording shows a motif signature, Matrix Profile earns its turn
then.

### What this task did not do

No Matrix Profile, no changepoint detector, no second wavelet, no inverse transform, no filter
bank. No change to the raw format, the schema, the recorder, `inspection`, the viewer, or the
product CLI. No new dependency. No normalization policy change. No real recording committed,
copied, or depended on. Nothing pushed.

The legible oracle is unchanged: its generator was refactored to share a parameterized record
builder with the sparse one, and the byte-for-byte regeneration test from task:14 is what
establishes that its output did not move.
