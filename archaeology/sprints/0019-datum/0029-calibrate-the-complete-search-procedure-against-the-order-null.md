---
id: tsk_01KZCACB1RZBPRTVVF9B00PX97
sequence: 29
kind: task
status: pending
sprint: spr_01KZCA9DMMGRAEBF9G31VKMWN5
created: 2026-08-06
---

# Calibrate the complete search procedure against the order null

## Objective

Measure whether the complete R1-based search procedure produces more exceptional maxima on observed
event sequences than the same procedure produces on order-null replicates, and determine whether the
order null can support that comparison at all.

`### Preregistration` was written and committed **before any calibration code existed and before any
observed or null maximum was computed**. What was computed first is §PHASE 0's premise verification
and the timing measurement that sizes `B`.

## Acceptance criteria

- Every premise verified from code; discrepancies recorded, not worked around.
- `T` derived from the implemented machinery; observed and null paths provably identical.
- Every data-dependent stage rerun inside each null replicate.
- `B`, seeds and every threshold fixed before execution, with reachability checked on **every** branch.
- Controls run before observational specimens.
- Specimen-level results; verdict never forced over a disagreement.
- `scripts/check.sh` passes unweakened; nothing pushed; no recording content committed.

### Preregistration

#### PHASE 0 — Premises, verified from code

| premise | verdict |
|---|---|
| R1 is the frozen sprint:18 pooled sum | **confirmed** — `repair.rs`, `p̂ = (ĉ_A+ĉ_B)/(N_A+N_B)`, unchanged |
| its honest reading is self-information of agreeing marks under pooled marginals | **confirmed** — task:28 §4 |
| it is not calibrated evidence of a shared motif | **confirmed** — task:28 §5 lists the four assumptions and names the selection effect as independently disqualifying |
| an order null already exists | **confirmed** — `order_null_seeded`, seed `0x4F52_4445_524E_554C`, `null_seed(realization, side)` |
| search selection creates a winner-selection effect | **confirmed structurally**, and §PHASE 3 demonstrates it on this machinery |

**Two discrepancies, both material, both recorded before any criterion was written.**

**D1 — the search does not rank by R1.** `cross_pairs` enumerates every window pair at span `k`,
computes `align()` — sprint:8's weighted global alignment with the timing term — and sorts by
`alignment.total` **ascending**, ties broken by `a.start` then `b.start`. R1 is never consulted during
selection. It is a **readout applied to an alignment-selected set**.

Consequently `T` is not "the maximum of R1 over a searched set" in the sense of a statistic driving its
own search. It is *the R1 value at the winner the alignment search chose*. The commissioning prompt's
suggested formula would have misstated the machinery, and §PHASE 1 derives `T` from the code instead.
This is the honest quantity, and it is arguably the more useful one: it calibrates the pipeline a user
would actually run.

**D2 — the existing null machinery is not search-aware.** `null_evidence` states in its own
documentation that it "reads the same spans out of each nulled pair, so the candidate's boundaries,
lengths, and observed gaps are held fixed and only identity is randomized." That is boundary-fixed
rescoring — exactly what this round forbids. **It is not reused.** A new search-aware path reruns
`cross_pairs`, `dedupe_overlapping` and R1 inside every replicate. sprint:11's results are unaffected
and unretracted; they simply answer a narrower question than a search-aware null answers.

#### PHASE 1 — The random variable

Derived from the implemented pipeline, stage by stage.

- **Candidate generation.** For span length `k`, every ordered pair `(a_start, b_start)` with
  `a_start < len_A − k + 1` and `b_start < len_B − k + 1`. No prefilter of any kind.
- **Self-matches.** Excluded at the recording level: `cross_pairs` returns `None` when both sequences
  carry the same session id. Within an admissible pair no exclusion is needed — windows from different
  recordings share no event.
- **Span lengths.** The frozen sprint:9 ladder `k ∈ {3, 4, 6, 8, 12}`. No other length is searched.
- **Ranking.** By `alignment.total` ascending, then `a.start`, then `b.start` — a **total** order, so
  ties are resolved deterministically and no tie-breaking randomness enters.
- **Truncation.** `top = usize::MAX`: the full ranking is retained before deduplication.
- **Deduplication.** `dedupe_overlapping(ranked, 5)`, greedy from the top; two windows clash when
  `|start_i − start_j| < max(k_i, k_j)` on **either** side. At most 5 survive.
- **Selection and readout.** R1 is computed on each survivor via `Observation::of`, over spans of equal
  length `k` by construction.

```text
T_k(A, B)  =  max over c in dedupe_overlapping(cross_pairs(A, B, k, usize::MAX), 5)  of  R1(c)
T_ladder(A, B)  =  max over k in {3, 4, 6, 8, 12} of T_k(A, B)
```

`T_k` is undefined when `cross_pairs` returns `None` or the kept set is empty; such specimens are
reported as undefined and excluded from the verdict rather than scored as zero.

**The primary unit is the specimen `(A, B, k)`** — the same unit sprint:16 and sprint:17 used, so the
results are comparable. `T_ladder` is reported alongside. Both are computed identically on observed and
null paths; a single function serves both, and a test asserts the null path calls it.

#### PHASE 2 — What the order null is

`order_null_seeded` permutes marks across the whole sequence by seeded Fisher–Yates, assigning marks to
positions. Gaps and offsets stay attached to **positions**, not to marks. Receipts are dropped.

| property | preserved | destroyed |
|---|---|---|
| total event count | ● | |
| mark vocabulary | ● | |
| mark marginal frequencies | ● *(exactly — it is a permutation of the multiset)* | |
| event offsets and the timing skeleton | ● *(every gap stays at its position)* | |
| candidate-boundary constraints (`window_count`) | ● *(a function of length only)* | |
| session identity, so `cross_pairs` still admits the pair | ● | |
| local adjacency | | ● |
| runs of a repeated mark | | ● |
| first-order transition frequencies | | ● *(driven to the product of marginals)* |
| higher-order sequential structure | | ● |
| the association between a mark and its own gap | | ● |
| record receipts | | ● *(set to `None`; a permuted mark is not that record's)* |

**The hypothesis it represents, stated exactly:** *the marks of this recording were assigned to its
time slots exchangeably — mark identity carries no information about position, about neighbouring
marks, or about the gap at that slot — while each mark's count and the entire timing skeleton are
exactly as observed.*

The commissioning prompt's wording is true of this null with one addition: **the mark-to-gap
association is destroyed as well**, and `align()`'s timing term reads gaps, so the null perturbs the
alignment through timing as well as identity.

**What that makes it too strong for.** The schema correlates a tool request with its own outcome by
`tool_use_id`, so a request mark is followed near-deterministically by an outcome mark. That adjacency
is a property of the instrument, not a reusable workflow motif, and this null destroys it. **A large
observed-versus-null separation is therefore consistent with nothing more than trivial instrument
adjacency.** §PHASE 7 measures this directly, and §PHASE 9 will not read separation as motif evidence.

#### PHASE 3 — Selection-effect demonstration

On a synthetic specimen generated **according to the null hypothesis itself** — i.i.d. marks, no
planted structure — compare two distributions computed with this project's own machinery:

- **arbitrary**: R1 at candidate pairs sampled uniformly from the admissible `(a_start, b_start)` grid
  at `k = 8`, 4 000 draws, seed `0xC0FFEE_5E1EC7`;
- **selected**: `T_8` from 400 complete searches on independently generated null specimens, seeds
  `null_seed(i, ·)` for `i ∈ 0..400`.

*Quantity:* the median and the 0.99 quantile of each distribution, and
`median(selected) − q99(arbitrary)`.
*Preregistered expectation:* `median(selected) > median(arbitrary)`, strictly. **No magnitude is
predicted and none is required** — the demonstration exists to show the effect is real on this
machinery, not to size it.
*Rule:* **confirmed** iff `median(selected) > median(arbitrary)`. Reachability: both are medians of
non-degenerate samples of the same statistic, so either ordering is attainable and the rule can fail.

#### PHASE 4 — Controlled fixtures

Both synthetic, both obviously so, neither derived from any recording.

**NEGATIVE control.** Two independent sequences of 160 and 90 events, marks drawn i.i.d. from a fixed
12-symbol categorical distribution, gaps drawn from a fixed distribution, seeds `0x0E6A_71VE` variants
fixed in code. This *is* the order-null hypothesis, so observed and null are draws from one law.
*Prediction:* `T_observed` ordinary — `p̂ > 0.01` at every `k`.
*Rule:* **PASS** iff no `k` yields `p̂ ≤ 0.01`. Reachability: `p̂ > 0.01` requires ≥ 10 exceedances of
999, which a true null attains with probability ≈ 0.99 per specimen.

**POSITIVE control.** The same generator, plus one fixed 12-mark figure planted at three disjoint
positions in each sequence, its marks **drawn from the same categorical distribution** so the marginals
move as little as the construction permits. Both marginal vectors are reported so the shift is visible.
*Prediction:* `T_observed` exceptional — `p̂ ≤ 0.01` at `k = 12`, the length the figure was planted at.
*Rule:* **PASS** iff `k = 12` yields `p̂ ≤ 0.01`. Reachability: `p̂ = 0.001` at 0 exceedances, so the
threshold is attainable at `B = 999`; and it is refutable, since a planted figure the search cannot
recover would leave exceedances high.

**`B = 999`, and the rationale.** One complete search over all six real pairs and the whole ladder
takes **0.097 s** in a release build, measured before this preregistration. `B = 999` costs ≈ 97 s of
search per corpus pass — affordable — and gives a finest resolvable tail of `1/1000`. Every threshold
below is `0.01`, ten times coarser than that floor, so no rule depends on the last resolvable digit.
Seeds are `null_seed(i, 0)` and `null_seed(i, 1)` for `i ∈ 0..999`, the existing deterministic scheme.

**The reported tail:** `p̂ = (1 + exceedances) / (B + 1)` where `exceedances = count(T_null ≥ T_observed)`.
This is a **Monte Carlo null tail estimate under the order null and this search**, and is described as
nothing else anywhere in this round.

#### PHASE 5 — Real corpus

Every specimen in decision:8's inventory: `8b68dece`, `57f18ff9`, `f5c18299`, `7d95c414`, all fifteen
ordered pairs reduced to six unordered ones, each at five `k`. For each: `T_observed`, null median,
null quantiles, maximum null `T`, exceedances, `p̂`, and the candidate counts on both paths.

Hygiene per decision:8: opaque session prefixes, counts, frequencies, scores, ranks, quantiles and
exceedances only. No prompts, commands, responses, paths or file contents. **No discovered span is
named, described, or inspected**, and recordings remain observational specimens.

#### PHASE 6 — Top-k descriptive check

`k_top = 5`, chosen before execution because it is the number `dedupe_overlapping` already keeps — no
new parameter is introduced. For each specimen, compare the observed sorted top-5 R1 order statistics
against the same order statistics from every null search, reporting each rank's exceedance count.

**Descriptive only.** Its purpose is to separate *one extraordinary candidate* from *a broad excess of
high-scoring candidates*; no verdict branch reads it, and if the correspondence is ambiguous the
ambiguity is reported rather than resolved.

#### PHASE 7 — Null adequacy

Three domain-neutral categorical-sequence summaries, observed versus null, on every specimen:

- **immediate repetition rate** — fraction of adjacent positions carrying the same mark;
- **mean run length** — mean length of maximal runs of one mark;
- **first-order transition entropy** — Shannon entropy in nats of the empirical bigram distribution,
  normalized by the entropy of the marginal-product distribution over the same vocabulary.

*Quantity:* observed value, null median, and the exceedance count for each.
*Rule:* if the observed value of **any** of the three lies outside the full range of its null
distribution, record the calibration limitation explicitly: **an exceptional `T` under this order null
may reject exchangeable ordering without establishing motif structure.**

No feature suite beyond these three. None reads recording contents.

#### PHASE 8 — Feasibility propagation pass

Recorded before execution. Every numerical rule, including every FAIL condition and verdict branch —
sprint:18's eleventh defect was a reachability check applied to PASS rules only.

| mechanism | rules touched | disposition |
|---|---|---|
| **D1** search ranks by alignment, not R1 | the definition of `T`; every claim about what is calibrated | **`T` rewritten before implementation.** The round calibrates the alignment search with an R1 readout, and says so. A `T` defined as "max R1 over the searched set" would have described machinery that does not exist. |
| **D2** existing null is boundary-fixed | reuse of `null_evidence`; the identity of observed and null paths | **Struck.** `null_evidence` is not called. One function computes `T` and both paths call it; a test asserts the null path reruns `cross_pairs` and `dedupe_overlapping`. |
| null preserves marginals exactly | R1's pooled `p̂` on null replicates | **Checked and kept.** `p̂` is a function of whole-recording counts, which the permutation preserves exactly, so `p̂` is *identical* on observed and null. **R1's per-mark weights do not move at all** — only which marks land in agreeing positions does. This makes the comparison unusually clean and is asserted by a test. |
| null preserves length and `window_count` | comparability of candidate counts | **Kept.** The two paths search identically sized spaces; the counts are reported to show it. |
| null destroys mark↔gap association | `align()`'s timing term | **Recorded.** The null perturbs alignment through timing as well as identity, so separation is not attributable to identity alone. §PHASE 9's claim is worded accordingly. |
| schema couples request to outcome | the meaning of any separation | **Propagated to the verdict before trials.** §PHASE 9's A branch is written to exclude motif structure explicitly, and §PHASE 7 measures the confound rather than assuming it away. |
| `B = 999` finite | every threshold | **Checked on both sides.** `p̂ ≤ 0.01` needs ≤ 9 exceedances — reachable, minimum `0.001`. `p̂ > 0.01` needs ≥ 10 — reachable, maximum `1.000`. The two are complementary, so no result falls between them. |
| PHASE 3's rule is an ordering | its reachability | **Checked.** Either ordering of two medians is attainable; the rule can fail. |
| negative control's rule is a **FAIL** condition | its reachability | **Checked explicitly, and this is the sprint:18 lesson applied.** "No `k` yields `p̂ ≤ 0.01`" is attainable — a true null exceeds 0.01 with probability ≈ 0.99 per `k` — and is refutable, since five `k` at 1% give ≈ 5% chance of a spurious flag. That residual is stated now, not discovered later. |
| specimens may disagree | the corpus verdict | **Partition fixed below** so disagreement is reported rather than averaged away. |

**Tiling.** `p̂ ≤ 0.01` and `p̂ > 0.01` are complementary and exhaust the specimen level. §PHASE 9's
three branches are ordered by precedence and cover every combination of control and corpus outcome.

#### PHASE 9 — Verdict partition

By precedence, so it tiles:

1. **C — NULL / CALIBRATION INADEQUATE.** Either control fails its rule, or the observed and null paths
   cannot be shown to compute the same `T`, or `T` is undefined for a majority of specimens. Checked
   first; nothing below applies if it fires.
2. **A — NULL-SEPARATING.** Controls pass **and** at least half of the specimens with defined `T` have
   `p̂ ≤ 0.01`.
3. **B — NOT SEPARATING.** Controls pass and fewer than half do.

**A earns exactly one claim:** *the complete search detects sequential structure not explained by this
order null.* **It does not earn** a calibrated probability of a motif, workflow identity, semantic
reuse, or any causal reading — and where §PHASE 7 records its limitation, A additionally does not
distinguish reusable motif structure from trivial instrument adjacency.

**Predicted: A, with §PHASE 7's limitation confirmed.** The schema's request→outcome coupling is enough
to separate observation from an exchangeable null on its own. Predicting this in advance is the point:
if the corpus does *not* separate, the search is weaker than the instrument's own regularity, which
would be the more surprising result.

Specimen-level results are reported in all three branches. R1 is not promoted to production.

#### PHASE 10 — What this task will not do

No change to R1, the null, the search, the alignment, the ladder, the representation or the incumbent
selector. No new score, no second detector, no aggregate statistic invented to summarize §PHASE 6. No
adoption. No threshold chosen after seeing data. No interpretation, naming or inspection of any
discovered span. No treatment of observational recordings as ground truth. No recording content in any
artifact. Nothing pushed.
