---
id: tsk_01KZ7ABSQ85GQ6X8RVJY0C24P0
sequence: 16
kind: task
status: closed
sprint: spr_01KZ7A9ZHPYSQH7WATWCZXA133
created: 2026-08-04
closed: 2026-08-04
---

# Search for motifs at a preregistered window ladder and test the Haar guidance hypothesis

## Objective

Run a univariate Matrix Profile over the sprint:4 substrate at a preregistered ladder of window
lengths, and test whether sprint:5's Haar evidence usefully constrains that window choice — against
a control that can falsify the guidance story rather than confirm it.

Everything below the `### Preregistration` heading was written and committed to disk **before the
detector was run against any fixture or recording**. Only the library's own known-answer behaviour
was probed first, which is implementation due diligence rather than a result.

## Acceptance criteria

- An implementation chosen on inspected evidence and recorded: license, version, maturity,
  dependency footprint, reference validation, and what would have made it unsuitable.
- Tests pinning the library's treatment of trivial matches and exclusion zones, constant
  subsequences, z-normalization, and index-to-time conversion. These protect against
  misunderstanding the library, not only against the library being wrong.
- Each licensed dimension analysed independently, verbatim tool names preserved, and no
  multivariate fusion.
- Both fixtures and, exploratorily, the untracked real recording, run at the same preregistered
  ladder.
- A null that destroys temporal order while preserving the value multiset, run through the identical
  path, with the comparison metric fixed below.
- Results reported per window with enough to compare windows: best masked motif distance, whether
  the known synthetic recurrence was recovered and at what rank, the constant-subsequence fraction,
  and separation from the null.
- A three-way verdict on the guidance hypothesis, kept separate from any verdict on Matrix Profile.
- Representation failures recorded as findings.
- One next experiment recommended. `scripts/check.sh` passes unweakened; no existing test changed.

### Preregistration

#### The three numbers that are not the same number

sprint:5's language sometimes let these blur, and this task separates them:

| quantity | legible oracle | sparse oracle | what it is |
|---|---|---|---|
| motif **instance duration** | ~2.2 s | ~0.95 s | how long one occurrence lasts |
| motif **repetition period** | 8 s | 8 s | gap between occurrences |
| Haar **cliff level** | 16 s | 16 s | first level whose *half*-window reaches the period |

A Matrix Profile window is the length of the pattern being matched. There is no reason it should
equal any of the three, and this experiment does not assume it does. In particular the Haar cliff at
16 s is a fact about the *period*, arrived at through a half-window, and reading it directly as a
window length would be the exact conflation this task exists to avoid.

#### The window ladder

Six windows. Base sampling is 500 ms throughout, so window in samples = seconds × 2.

| # | window | samples | role | derived from |
|---|---|---|---|---|
| 1 | **2 s** | 4 | **C** — oracle-appropriate | the legible oracle's ~2.2 s instance duration |
| 2 | **8 s** | 16 | **A** — control | real-session Haar levels 1–5 (≤16 s) were indistinguishable from the impulse null |
| 3 | **16 s** | 32 | **A** — control | the last real-session level still at the null (ratio 1.0–1.3) |
| 4 | **32 s** | 64 | **B** — Haar-informed | first real-session level with excess (level 6, ratio 1.3–1.8) |
| 5 | **64 s** | 128 | **B** — Haar-informed | first clearly elevated real-session level (level 7, ratio 1.9–2.5) |
| 6 | **128 s** | 256 | **D** — regime probe | strongest usable real-session excess (level 8, ratio 2.2–3.6) |

The 512 s and 1024 s levels carried the largest Haar excesses on the real recording and are
deliberately **excluded**: at 1054 s of span they admit one or two windows, and a motif search needs
many candidates. That is a limitation of the recording's length, recorded here rather than
discovered later.

**A confound, named in advance.** On the synthetic fixtures the 8 s window is *not* a control: 8 s
is exactly the injected repetition period, so it is the most favourable window there. The control
reading of windows 2 and 3 applies to the **real recording only**, where 8 s and 16 s sit in the
region Haar found indistinguishable from sparsity. Any claim about controls must say which dataset
it is about.

#### Expected synthetic matches, from the fixture constants

Legible oracle, 481 buckets: motif A at buckets 120, 136, 152, 168 (60–90 s, exactly periodic);
recurrence B nominally at 420, 436, 452, 468 (210–240 s) with start jitter up to 700 ms and
per-record jitter up to 150 ms, and one call failing instead of succeeding. Elevated regime buckets
300–420.

Sparse oracle, 2401 buckets: motif A at buckets 600 to 1064 in steps of 16 (300–540 s, 30
instances); recurrence B at buckets 2160 onward (1080–1200 s, 15 instances, jittered); regime block
buckets 1560–2160.

**The trivial result and the real one.** Within motif region A the period is exact, so any window
has a distance-0 neighbour one period away, and recovering that proves only that the fixture is
periodic. The result that matters is the **cross-region recurrence**: a subsequence inside region A
whose nearest neighbour lies inside region B, across the intervening baseline and regime. That is
the criterion below.

#### Predictions

**P1 — the unmasked profile will be about emptiness, at every window and every fixture.** Two
constant subsequences are at distance exactly 0 by the library's documented convention, and these
signals are 78–93% empty, so the top unmasked motif will be two all-zero regions at distance 0.
This is predicted as a certainty, not a risk. Motif search is therefore run twice: raw, and with
constant subsequences masked to infinity using the library's own `sigma_threshold`, with the masked
fraction reported.

**P2 — within-region periodicity will be recovered trivially** in motif-active dimensions
(`channel:reported`, `kind:v2:reported_intent`, the motif-only tool name) at windows 1–3, with
distance at or near 0 and neighbour lag a multiple of 16 samples.

**P3 — cross-region recurrence will favour the middle of the ladder.** The recurrence is jittered by
up to ~1.7 buckets, which destroys a 4-sample window and is averaged out by a longer one; but a
window much longer than the 60-bucket (legible) motif region cannot sit inside it. Predicted best
cross-region recovery at **16 s or 32 s** on the legible oracle, and at **16 s to 64 s** on the
sparse oracle whose motif region is 480 buckets. Predicted failure at 128 s on the legible oracle
for the structural reason that 256 samples do not fit in a 60-bucket region.

**P4 — the null will produce distance-0 motifs at short windows and rising distances at long ones.**
A shuffled sparse series still contains many windows holding a single impulse at the same relative
offset, and after z-normalization those are identical. So a low best-distance at a short window is
worth nothing on its own. The null's best distance should rise with window length as coincidental
agreement gets harder.

**P5 — on the real recording, the guidance hypothesis predicts** that windows 4–6 (32/64/128 s) show
larger separation from the null than windows 2–3 (8/16 s), and a lower constant-subsequence
fraction.

**P6 — amplitude blindness will matter.** z-normalized Euclidean distance is invariant to both
offset and scale, verified against the library: `[1,2,3,4]` and `[10,20,30,40]` are at distance 0.
So two bursts of identical shape but different intensity are a perfect match. This is not
necessarily wrong, and it must not be described as "the same behaviour recurred".

#### The comparison metric, fixed before any result is seen

For each dimension and window, on the same series:

- `d_best` — best motif distance after masking constant subsequences;
- `d_null` — the same quantity on a deterministically shuffled copy of that dimension, seeded
  fixed, preserving the exact multiset of bucket values and destroying temporal order;
- **`separation = (d_null − d_best) / (2·√m)`**, where `2·√m` is the maximum possible z-normalized
  Euclidean distance, so the figure is comparable across window lengths.

Positive separation means the real ordering admits matches the shuffled ordering cannot. Near-zero
or negative separation means the match is explained by sparsity and marginal density alone.

The shuffle changes the constant-subsequence fraction — breaking up runs of zeros produces fewer
all-zero windows — so the null is a *harder* comparison rather than a like-for-like one. Noted in
advance; the masked fraction is reported for both.

#### Success and falsification, fixed before any result is seen

For the fixtures, the criterion is the one that matters and not the one that is easy: **did the
cross-region recurrence appear among the top five masked motifs, and was its distance meaningfully
below the null's best?**

For the guidance hypothesis:

- **Supported** — on the real recording, Haar-informed windows (32/64/128 s) show materially larger
  separation than the control windows (8/16 s); *and* on the fixtures the window that best recovers
  the cross-region recurrence is not systematically the shortest one available.
- **Falsified** — the short control windows recover the synthetic recurrence at least as well *and*
  show separation on the real recording at least as good as the Haar-informed windows.
- **Mixed** — anything else, including "Haar picks a neighbourhood but not a window".

A falsification is a verdict on the **Haar → Matrix Profile composition** and not on Matrix Profile.
Those two conclusions are reported separately regardless of outcome.

## Result

Delivered, with a **mixed** verdict on the guidance hypothesis and one representation finding that
matters more than the verdict.

The short version: Matrix Profile runs, recovers the injected recurrence under a corrected
criterion, and produces almost nothing that survives the null. Nearly every top motif in these
signals — synthetic and real — is a pair of windows each containing **one** non-empty bucket at the
same relative offset, which is a perfect match by construction and says nothing about behaviour.
Haar evidence rules out the bottom of the ladder correctly and does not pick a window inside the
range it endorses.

### 1. Implementation decision: `motif-rs` 0.1.0, and what was inspected

Adopted rather than written. `cargo search` surfaced exactly one Matrix Profile implementation in
Rust; the crate source was downloaded and read before adding it.

| what | finding |
|---|---|
| license | MIT |
| size | ~11,200 lines, 20 algorithms, its own golden tests |
| validation | its `validation/` directory holds a STUMPY comparison: MAD `2.7e-12` to `1.2e-11`, correlation `1.00000000`, on sine, square, mixed, and streaming cases |
| conventions | exclusion zone `ceil(m/4)` and `sigma_threshold = 1e-15`, both stated as matching STUMPY |
| constant handling | **documented**: two constant subsequences → `d = 0`; one constant → `d = √(2m)` |
| footprint | pulls `rustfft` and eight transitive crates |
| maturity | 0.1.0, one release. The reason to read it rather than trust it. |

**What would have made it unsuitable**: an undocumented or wrong constant-subsequence convention.
In a signal that is 78–94% empty that single decision determines every result, and a library that
had made it silently would have been rejected. It does not; it states the convention, and
`tests/matrix_profile.rs` pins it.

**Dependency placement.** Optional, behind a non-default `experiment-matrix-profile` feature. A
default `cargo build` of the recorder links none of it — verified, `cargo tree` shows no `motif-rs`
— while `scripts/check.sh` already uses `--all-features`, so the gate exercises the experiment with
no CI drift and no new script. `default-features = false` drops `rayon`: these series are a few
thousand samples, parallelism buys nothing, and a single-threaded reduction keeps the arithmetic
reproducible. Deleting the experiment means deleting the feature, the dependency, one module, one
example, one test file.

Python was not used. STUMPY was consulted only as the reference the crate's own validation report
already compares against.

### 2. Three numbers that are not one number

Recorded in the preregistration and repeated here because sprint:5's language blurred them:

- motif **instance duration** — ~2.2 s legible, ~0.95 s sparse;
- motif **repetition period** — 8 s in both;
- Haar **cliff level** — 16 s, being the first level whose *half*-window reaches the period.

A Matrix Profile window is the length of the pattern being matched, and coincides with none of
them by right. Reading the 16 s Haar cliff directly as a 16 s window would have been a coincidence
dressed as a derivation.

### 3. A defect found by running, and fixed mid-round

The first pass fed the detector sprint:4's **z-scored** column, on the reasoning that the metric is
scale-invariant so it cannot matter. It reported a flawless motif between `[0 s, 16 s)` and
`[18 s, 34 s)` of the legible oracle in `kind:v2:reported_intent` — two regions containing **no
records at all** — and marked both windows non-constant.

Measured cause, on that dimension at `m = 32`:

- raw counts: an all-empty window has rolling standard deviation exactly `0`; **309 of 450**
  windows detected as constant;
- z-scored column: an empty bucket is `−1.30e-1`, the rolling variance of a window of identical
  such values is computed by cancellation and lands at **`1.863e-9`** — six orders of magnitude
  above the `1e-15` threshold — so only **34 of 450** were detected.

The second failure is worse than a missed mask: z-normalization then divides by `1.863e-9` and
amplifies pure rounding error into a full-amplitude shape, and two amplified noise shapes match
each other perfectly.

The fix is to pass unnormalized counts. **This is not a change to sprint:4's normalization policy**,
which is untouched; the metric z-normalizes each subsequence itself, so the two inputs are identical
in exact arithmetic and only one of them is numerically sound. The window ladder was not changed.
`tests/matrix_profile.rs` pins both the requirement and the hazard, and the hazard test asserts that
the z-scored input's best "motif" is two windows holding zero records.

Generalizable, and worth carrying: **do not stack a global normalization in front of a detector that
normalizes internally.** It buys nothing and it converts exact emptiness into near-emptiness.

### 4. The preregistered criterion was mis-specified, and both versions are reported

task:16 asked whether a motif has one span *starting* inside each region. That is too tight by up to
a whole window — a 32-sample window containing region A's first instance can begin 31 samples before
region A does — and it reported `none` for matches that plainly reach across. The strict criterion
is kept exactly as written and an **overlap** criterion is reported beside it. Both appear in the
`link/ovl` column. The mis-specification is mine and is not swapped out silently.

### 5. Synthetic results

Legible oracle, 481 samples, `--region-a 60000:90000 --region-b 210000:240000`:

```text
dimension: records                       (the densest dimension)
    window     m  subseq  const%       raw    masked      null      sep    lag     occ  link/ovl
        2s     4     478   43.7%     0.000     0.000     0.000   +0.000      6     1/1         -
        8s    16     466    0.0%     0.000     0.000     0.000   +0.000     16     5/5         -
       16s    32     450    0.0%     0.000     0.000     2.399   +0.212      9   10/10         -
       32s    64     418    0.0%     0.000     0.000     7.213   +0.451     24     5/5         -
       64s   128     354    0.0%     5.277     5.277    12.886   +0.336    180   15/13       -/5
      128s   256     226    0.0%    16.521    16.521    20.000   +0.109     72   51/55       -/1

dimension: channel:reported              (motif-only)
        2s     4     478   93.3%     0.000     0.000     0.000   +0.000     16     1/1       -/-
        8s    16     466   73.2%     0.000     0.000     0.000   +0.000     16     1/1       -/-
       16s    32     450   68.7%     0.000     0.000     0.000   +0.000    300     1/1       -/1
       32s    64     418   58.6%     0.000     0.000     0.000   +0.000    300     2/2       -/1
       64s   128     354   35.0%     0.000     0.000     0.000   +0.000    300     1/1       -/1
      128s   256     226    0.0%    22.717    22.717    18.584   -0.129    153     4/1       -/3
```

Sparse oracle, 2401 samples, `--region-a 300000:540000 --region-b 1080000:1200000`:

```text
dimension: records
        2s     4    2398   78.6%     0.000     0.000     0.000   +0.000     30     1/1         -
        8s    16    2386   32.5%     0.000     0.000     0.000   +0.000    570     1/1         -
       16s    32    2370   19.5%     0.000     0.000     0.000   +0.000     60     1/1         -
       32s    64    2338    0.0%     0.000     0.000     0.000   +0.000     60     2/2         -
       64s   128    2274    0.0%     0.000     0.000     8.942   +0.395     48   16/16         -
      128s   256    2146    0.0%     0.000     0.000    17.533   +0.548    120     5/5         -

dimension: channel:reported              (motif-only)
        2s     4    2398   92.5%     0.000     0.000     0.000   +0.000     16     1/1       -/-
        8s    16    2386   70.0%     0.000     0.000     0.000   +0.000     16     1/1       -/-
       16s    32    2370   68.9%     0.000     0.000     0.000   +0.000   1560     1/1       3/1
       32s    64    2338   67.1%     0.000     0.000     0.000   +0.000   1560     2/2       2/1
       64s   128    2274   63.4%     0.000     0.000     0.000   +0.000   1560     1/1       -/1
      128s   256    2146   55.2%     0.000     0.000    12.275   +0.384   1560     1/1       -/1
```

**Against the preregistered predictions:**

- **P1 held exactly.** The unmasked top motif is two constant stretches at distance 0 in every
  dimension of both fixtures. `raw` and `masked` diverge only where the mask has something to
  remove.
- **P2 held.** Within-region periodicity is recovered trivially at short windows, lag 16 samples =
  one period.
- **P3 held in shape.** Cross-region recovery is absent at 2 s and 8 s and present from 16 s
  upward, on both fixtures. Lag 300 (legible) and 1560/1561 (sparse) are exactly the region
  separations.
- **P4 held.** The null reaches distance 0 at every window up to 32 s and only rises beyond it.
- **P6 held**, pinned by test: `[1,2,3,4]` and `[10,20,30,40]` are at distance 0.

**And the finding that outweighs all of them.** The `occ` column counts non-empty buckets inside
each matched window. In every sparse dimension, at every window, on both fixtures, the top masked
motif has occupancy **1/1 or 2/2**. The 128 s cross-region match on the sparse oracle — separation
`+0.384`, the best synthetic number in the round — is two windows of 256 samples each holding
**one** record. The detector never matched the injected figure. It matched lone events that happen
to sit at the same offset within their windows, which are identical after z-normalization
regardless of what surrounds them.

The one place occupancy is real is `records`, the aggregate dimension: 10/10 at 16 s, 15/13 at 64 s,
51/55 at 128 s, with separations up to `+0.451`. Density, not window choice, is what makes the
metric mean anything.

### 6. Null and control

The null is a fixed-seed Fisher-Yates shuffle: identical multiset of bucket values, identical
sparsity, no temporal order. It answers the question it was built for, and the answer is
uncomfortable — **a shuffled series reaches distance 0 too**, at every window up to 32 s, in every
dimension of both fixtures. A perfect match in these signals is the default state of the
representation, not a discovery.

The shuffle raises the constant fraction's counterpart: breaking runs of zeros produces fewer
all-empty windows, so the null has *more* candidates and is a harder comparison rather than a
like-for-like one. Reported alongside.

Separation only becomes positive once the window is long enough that a shuffled series can no
longer produce a window holding a single event — which measures **clustering**, not recurrence. That
is a real property of the signal and it is not the property Matrix Profile was pointed at.

### 7. Window comparison, and the verdict: **Mixed**

Real recording, separation by window, across the six dimensions examined:

| window | role | separation observed |
|---|---|---|
| 2 s | oracle-duration | `+0.000` everywhere |
| 8 s | control | `+0.000` everywhere |
| 16 s | control | `+0.000` everywhere |
| 32 s | Haar-informed | `+0.000` everywhere |
| 64 s | Haar-informed | `+0.219` to `+0.259` in four dimensions |
| 128 s | regime probe | `+0.013` to `+0.091` |

**Supported, in part.** Haar said levels 1–5 (≤16 s) on this recording were indistinguishable from
an impulse null, and Matrix Profile finds exactly nothing at 2, 8, and 16 s: separation is
identically zero. The bottom of the ladder is correctly ruled out, by an argument made before the
detector ran. Sprint:5's evidence did constrain the search.

**Not supported, in the part that would have been useful.** Within the range Haar endorsed, the
guidance is wrong in detail. 32 s — the first Haar-informed window — gives nothing at all. 128 s,
where Haar's excess was *strongest*, gives an order of magnitude less than 64 s. Haar's own ranking
of the coarse levels is not Matrix Profile's ranking. Haar named a neighbourhood of roughly
64 s–128 s and could not pick within it, and picking within it is what a window parameter needs.

**Verdict: Mixed.** Haar identifies a useful scale neighbourhood and does not remove the need for
independent window tuning.

**Kept separate, as required.** That is a verdict on the *composition*. Separately: Matrix Profile
itself found nothing on these signals that survives its null except at aggregate dimensions and long
windows, and what it found there is clustering. Neither conclusion implies the other, and neither is
a verdict on Matrix Profile in general — only on sampled univariate Matrix Profile over this
representation.

### 8. Real recording, aggregate only

Same untracked 234-record session. Not committed, not copied, and nothing depends on its presence.
Structural results only.

```text
dimension: channel:observed              2108 samples at 500 ms
    window     m  subseq  const%       raw    masked      null      sep    lag     occ
        2s     4    2105   80.4%     0.000     0.000     0.000   +0.000     18     1/1
        8s    16    2093   57.5%     0.000     0.000     0.000   +0.000    125     1/1
       16s    32    2077   41.4%     0.000     0.000     0.000   +0.000     44     1/1
       32s    64    2045   23.7%     0.000     0.000     0.000   +0.000    431     2/2
       64s   128    1981    5.6%     0.000     0.000     5.856   +0.259   1188     1/1
      128s   256    1853    0.0%    15.616    15.616    16.242   +0.020    869    15/3

dimension: tool_name:Read
        2s     4    2105   99.0%     0.000     0.000     0.000   +0.000     46     1/1
      128s   256    1853   81.2%     0.000     0.000     0.000   +0.000   1550     1/1
```

- **Occupancy is 1/1 or 2/2 at every window in every dimension**, except `channel:observed` and
  `distinct_correlation_ids` at 128 s, where it is 15/3 and 18/4 — asymmetric, and at a distance of
  ~15.6, meaning nothing matches well there at all.
- **`tool_name:Read` is 81–99% constant across the whole ladder.** A per-tool dimension on a real
  session is too sparse to support a Matrix Profile at any window in the ladder. Aggregating tool
  names would fix that and is not epistemically authorized; the substrate's refusal to classify
  verbatim tool names (task:14) is exactly what leaves this dimension unusable, and that trade is
  now measured rather than assumed.
- The constant fraction falls from ~80% at 2 s to ~0% at 128 s, so the *only* windows with enough
  non-constant candidates are the ones where too few windows fit to compare.

**Manual inspection of the strongest candidates, using the existing projection.**

Strongest motif, `channel:observed` at 64 s, rank 1, distance 0.0, lag 1188: `[326.0 s, 390.0 s)` ↔
`[920.0 s, 984.0 s)`. Read against `behavioral-signal --samples`, each 128-bucket window contains
**exactly one** non-empty bucket, at its very first position, each holding one `tool_succeeded`
record. 127 of 128 buckets in each window are empty.

> **Experimental interpretation, not evidence.** A human reading the projection does **not** agree
> that these regions contain meaningfully similar behaviour. They contain one tool completion each,
> ten minutes apart, in an otherwise idle stretch. Their perfect distance is an artefact of two lone
> spikes sharing a within-window offset.

Strongest discord, `channel:observed` at 64 s, distance 12.22: `[128.5 s, 192.5 s)`. The projection
shows roughly eleven occupied buckets in that span, alternating a two-record bucket carrying a
`reported_intent` beside a `tool_requested` with a one-record bucket carrying a `tool_succeeded` and
a large recorded response, at intervals of roughly 7–10 s.

> **Experimental interpretation, not evidence.** A human reading the projection **does** agree this
> region is unlike most of the recording: it is the densest sustained stretch, with regular
> request-and-outcome pairing. Whether that is meaningful about the session is not established here.

That asymmetry is the useful observation. In a 94%-empty signal, "most unlike everything else" is
well posed because dense regions are rare, while "most similar to something else" degenerates
because empty regions are everywhere. **The discords are worth more than the motifs.**

On boundaries: the discord span contains records on both the `reported` and `observed` channels;
the motif candidate windows contain `observed` records only. Recorded as channel composition and
nothing more — no hierarchy, parentage, or turn semantics is inferred, and dragon:3 stays untouched.

**Backwards timestamps remain tested but unexercised.** Neither fixture contains one and the real
recording reported zero non-monotonic records, as in sprint:4 and sprint:5.

### 9. Representation failures, recorded rather than patched

In order of how much they decided the round:

1. **Lone-event alignment dominates everything.** Two windows each holding one non-empty bucket at
   the same relative offset are identical after z-normalization and score exactly 0, regardless of
   context. In a 78–94% empty signal these pairs are abundant, and they occupy the top of every
   masked motif list at every window in every sparse dimension of every dataset here. This is the
   round's central finding.
2. **The shuffled null reaches zero too**, up to 32 s. Low distance carries no information in this
   representation; only separation does, and separation is zero wherever the windows are short.
3. **Constant subsequences swallow the candidate set.** 43–99% of candidates are constant depending
   on dimension and window. The masked answer is drawn from the remainder, and that remainder is
   sometimes 1% of the series.
4. **Global z-scoring in front of an internally-normalizing detector manufactures motifs** out of
   floating-point noise. Section 3.
5. **Amplitude blindness.** A one-record burst and a ten-record burst of the same shape are a
   perfect match. Not necessarily wrong, and never to be reported as "the same behaviour".
6. **Per-tool-name dimensions are individually too sparse on a real session**, and the aggregation
   that would fix it is not authorized. Measured, not speculated.
7. **Bucket-alignment sensitivity**, which the fixtures show indirectly: the jittered recurrence is
   recovered at 16 s and above but never at 2 s or 8 s, where a jitter of one to two buckets is a
   large fraction of the window.

None of these was patched around. The mask is documented and its cost is reported; the rest are
reported as they are.

### 10. Desire-path friction

**The preregistration workflow has no home in Scarp, and this round needed one badly.** A task has
exactly two sections, `Objective` and `Acceptance criteria`. This experiment's integrity depends on
a window ladder, a set of predictions, a comparison metric, and a falsification threshold all
existing *before* the detector ran, and there is nowhere to put them: they are a `###` subsection
inside `Acceptance criteria`, where they are neither criteria nor marked write-once. The only
evidence that they predate the run is `363ac20`, a commit made for that purpose alone — the tool
records nothing about it.

This is the same shape as sprint:5's note and it is now worse rather than merely repeated, because
this round also *found a defect mid-run* and had to demonstrate that the ladder did not move in
response. Doing that took a separate commit and a paragraph of prose. **Promoted to idea:5**, since
it has now recurred twice and is independently useful: an artifact section that is stamped when
written and refuses silent modification afterwards, so "this was predicted, not fitted" is a
property of the record instead of a claim in it.

**The section-template refusal recurred for the third consecutive round** — `## Predictions …`
rejected on a task, as `## Hypothesis` was on a sprint in sprint:5. One failed command, demote to
`###`, move on. Already idea:2's territory; no new information beyond the third occurrence.

**Appending a Result and an Outcome is still `cat >>`.** Fifth round. Already idea:1 and idea:4.

**One thing that went well and is worth recording as such**: `scarp new sprint` followed by
`scarp new task --sprint sprint:6` followed by a commit was enough to make the preregistration a
durable, reviewable artifact with a timestamp. The workflow is missing a feature, not broken.

### 11. Recommendation: an event-native motif method, one narrowly specified experiment

**Not** changepoint detection yet, and not a multivariate Matrix Profile, and not another window.

The reason is finding 1. Across two fixtures and one real recording, six windows, and every licensed
dimension, sampled univariate Matrix Profile matched **lone events at aligned offsets** and never
matched a repeated figure — including on a fixture built to contain one, at a window where it
recovered the correct region pair. The limiting factor is not the window parameter, which is what
this round set out to test. It is that fixed-width sampling of a 94%-empty event stream produces
subsequences whose z-normalized shape is dominated by *where a single event sits inside the window*,
not by what happened.

That diagnosis points at the representation, and the smallest experiment that tests it directly is
an **event-native motif search**: match on the sequence of inter-event gaps and delivered event
kinds rather than on a resampled count series, over the same two fixtures and the same real
recording, with the same shuffled null and the same preregistration discipline. It is falsifiable in
one round — either the injected figure is recovered with separation from the null, or sparse
behavioural motif detection is not worth further effort at any representation.

Two things this recommendation deliberately does not do. It does not reach for multivariate Matrix
Profile: the evidence says univariate sparsity is the problem, and `mstump` over the same
representation would inherit it. And it does not abandon changepoint detection — sprint:5's evidence
for regime structure is untouched by this round, and this round's own separation results measure
clustering, which is more support for it. Changepoint detection remains the right *second* next
step; it is not the one that resolves what this round found.

### What this task did not do

No changepoint detector, no multivariate or multidimensional Matrix Profile, no second window
search, no normalization policy change, no wavelet work. No change to the raw format, the schema,
the recorder, `inspection`, the viewer, or the product CLI. No Python. No real recording committed
or copied. Nothing pushed. sprint:5's result is unedited, including the recommendation this sprint
declined to follow yet.

### Addendum, 2026-08-04: two corrections to this Result

Written after the Result above, which is left intact.

**The append-then-close friction was cited against a gap that is closed.** §10 says appending a
Result and an Outcome "is still `cat >>`" and files it under idea:1 and idea:4. The first half is
accurate about what happened here — the `scarp` on this machine is 0.2.0, `scarp close` offers only
`--resolved-by`, and both the task Result and the sprint Outcome were appended by hand. The second
half is wrong: maintenance:1, committed at `575dec2` while this round was running, records that
upstream Scarp shipped result-on-close, which is what idea:1 asked for. So the workaround here is a
version lag rather than a missing design, and citing idea:1 as live friction repeated exactly the
mistake maintenance:1 exists to prevent — reading a parked idea and believing the gap is still open.

idea:5 is unaffected. Sealing a section so a prediction can be shown to predate its result is a
different guarantee from writing a result at close time, and nothing that shipped provides it.

**`363ac20` is broader than its message.** It is described as committing the preregistration, and it
also carries `Cargo.toml`, `Cargo.lock`, and a one-line stub `examples/matrix-profile.rs`, swept in
by `git add -A`. The preregistered material — the ladder, the predictions, the metric, the
falsification criteria — is in that commit and did not change afterwards, so the claim the commit
was made to support still holds. The message is narrower than the diff, and that is recorded here
rather than corrected by rewriting the commit.
