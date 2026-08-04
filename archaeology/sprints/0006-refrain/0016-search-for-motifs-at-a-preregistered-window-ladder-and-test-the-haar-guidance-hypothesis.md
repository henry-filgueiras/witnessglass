---
id: tsk_01KZ7ABSQ85GQ6X8RVJY0C24P0
sequence: 16
kind: task
status: pending
sprint: spr_01KZ7A9ZHPYSQH7WATWCZXA133
created: 2026-08-04
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
