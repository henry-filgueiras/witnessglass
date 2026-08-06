---
id: tsk_01KZCACB1RZBPRTVVF9B00PX97
sequence: 29
kind: task
status: closed
sprint: spr_01KZCA9DMMGRAEBF9G31VKMWN5
created: 2026-08-06
closed: 2026-08-06
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

## Result

**Verdict: A — NULL-SEPARATING**, by the preregistered partition: both controls passed and **23 of 30**
specimens with defined `T` have `p̂ ≤ 0.01`. **§PHASE 7's limitation confirmed in the strongest form
available**, which was also predicted in advance. R1 was not changed, not adopted, and not promoted.

### 1. Verified premises

All five confirmed from code (§PHASE 0). **Two discrepancies, both material, both recorded before any
criterion was written:**

- **D1 — the search does not rank by R1.** `cross_pairs` sorts by `alignment.total` ascending. R1 is a
  readout applied to an alignment-selected set. `T` was derived from the machinery instead of from the
  commissioning prompt's formula, which would have described a pipeline that does not exist.
- **D2 — the existing null machinery is not search-aware.** `null_evidence` states in its own
  documentation that it holds candidate boundaries fixed and randomizes only identity. That is the
  comparison this round forbids. It was **not called**; a new path reruns the complete search inside
  every replicate, and a test proves the difference is real rather than nominal.

### 2. The complete search statistic

```text
T_k(A, B) = max over c in dedupe_overlapping(cross_pairs(A, B, k, usize::MAX), 5) of R1(c)
```

Candidate generation is every ordered window pair at `k`; no prefilter; self-matches excluded at the
recording level; ladder `k ∈ {3,4,6,8,12}`; ranking by alignment total then `a.start` then `b.start`, a
total order; no truncation before deduplication; greedy dedupe to 5 with clash at
`|start_i − start_j| < max(k_i, k_j)` on either side. `complete_search` is the only implementation and
**both paths call it**.

### 3–4. The order null, and what it does

Preserved: event count, vocabulary, **mark marginal frequencies exactly**, offsets and the whole timing
skeleton, `window_count`, session identity. Destroyed: local adjacency, runs, first-order transitions,
higher-order structure, **the mark-to-gap association**, receipts.

Because marginals survive exactly, **R1's per-mark weights are identical on both paths** — only which
marks land in agreeing positions moves. Asserted by a test.

### 5–6. Preregistration and feasibility audit

Committed alone at `89f53d2`, before any calibration code existed. `B = 999` from a measured 0.097 s
complete search; thresholds checked for reachability **on both sides**, the negative control's FAIL
condition included, which was sprint:18's lesson applied.

### 7. Selection effect — the reason this round exists

On a specimen generated from the null hypothesis itself:

| | median | q99 |
|---|---|---|
| arbitrary candidates (n = 4 000) | **1.6094** | 7.3843 |
| search-selected maxima (n = 400) | **11.0187** | — |

`median(selected) − q99(arbitrary) = +3.6344` nats. **The median of what the search selects sits well
above the 99th percentile of what an arbitrary candidate scores** — on data with no structure at all.
Any comparison of an observed search-selected score against a distribution of arbitrary candidate
scores would have been meaningless, and this is that stated as a measurement.

### 8. Controlled fixtures

| control | k=3 | k=4 | k=6 | k=8 | k=12 | rule |
|---|---|---|---|---|---|---|
| **negative** `p̂` | 0.657 | 0.825 | 0.425 | 0.035 | 0.358 | **PASS** — no `k` exceptional |
| **positive** `p̂` | 0.396 | 0.109 | **0.005** | **0.001** | **0.001** | **PASS** — exceptional at the planted length |

The positive control's `T` climbs 13.82 → 18.28 → 27.57 across `k = 6, 8, 12` against null medians near
10–13, and at `k ∈ {8, 12}` **no null search in 999 reached it**. The machinery detects a planted
figure and does not flag a specimen drawn from the null.

### 9. Real corpus — search-aware null

30 specimens, `B = 999`. **23 separate at `p̂ ≤ 0.01`; 7 do not.** Specimen-level, as required:

| pair | k=3 | k=4 | k=6 | k=8 | k=12 |
|---|---|---|---|---|---|
| `8b68dece × 57f18ff9` | 0.157 | **0.004** | **0.005** | **0.004** | **0.007** |
| `8b68dece × f5c18299` | **0.004** | **0.003** | 0.015 | **0.002** | 0.075 |
| `8b68dece × 7d95c414` | 0.133 | 0.031 | 0.027 | **0.003** | **0.002** |
| `57f18ff9 × f5c18299` | 0.035 | **0.001** | **0.001** | **0.001** | **0.001** |
| `57f18ff9 × 7d95c414` | **0.002** | **0.001** | **0.001** | **0.001** | **0.001** |
| `f5c18299 × 7d95c414` | **0.001** | **0.001** | **0.001** | **0.001** | **0.001** |

The seven non-separating specimens cluster at `k = 3`, the shortest span, where a three-mark agreement
is cheap for a shuffle to reproduce. The strongest separation is `57f18ff9 × f5c18299` at `k = 12`:
`T = 31.68` against a null maximum of `18.18`. sprint:9 established those two are executions of one
runbook and decision:8 carries that dependence; it is **not** treated as ground truth here, and the
observation is reported as a mechanically derived quantity only.

### 10. Top-k descriptive check — ambiguous, and reported as such

Per-rank exceedance counts over the top 5 split into two shapes:

- **Broad excess.** `57f18ff9 × f5c18299` returns `[0,0,0,0,0]` at every `k ≥ 4`: the entire kept set
  clears every null search at every rank. `f5c18299 × 7d95c414` at `k = 12` likewise.
- **Rank-1 only, and non-monotonic.** `57f18ff9 × 7d95c414` at `k = 12` gives `[0, 712, 0, 0, 0]` —
  rank 1 exceptional, rank 2 thoroughly ordinary, ranks 3–5 exceptional again. `8b68dece × 7d95c414` at
  `k = 8` gives `[2, 0, 49, 3, 0]`.

**The non-monotonic pattern is ambiguous and is not resolved here.** Each observed rank is compared
against the null's distribution *for that rank*, and those distributions differ, so a dip at one rank
does not order against its neighbours. §PHASE 6 preregistered that ambiguity would be reported rather
than resolved, and no verdict branch reads this section.

### 11. Null adequacy — the finding that bounds everything above

| specimen | immediate repetition rate | mean run length | transition entropy ratio |
|---|---|---|---|
| `8b68dece` | **0.0000** vs null 0.2857 *(0.179–0.375)* | **1.0000** vs 1.3967 | **0.6024** vs 0.9265 |
| `57f18ff9` | 0.0000 vs 0.0645 *(0.000–0.194)* | 1.0000 vs 1.0667 | **0.5931** vs 0.6448 |
| `f5c18299` | 0.0000 vs 0.0625 *(0.000–0.188)* | 1.0000 vs 1.0645 | **0.5727** vs 0.6456 |
| `7d95c414` | **0.0000** vs 0.2632 *(0.145–0.382)* | **1.0000** vs 1.3509 | **0.6348** vs 0.8795 |

Bold entries lie **outside the entire null range** over 199 replicates.

**Every observed recording has an immediate repetition rate of exactly zero and a mean run length of
exactly one: no mark ever follows itself.** The null, which shuffles marks, produces repeats
constantly — a median of 26–29% on the two larger recordings. Transition entropy is far below null on
all four.

This is the confound §PHASE 2 predicted from the schema before any measurement: tool events are
correlated request→outcome pairs, so consecutive events carry different kinds and therefore different
marks. **The alternation is a property of the instrument, not a reusable workflow motif.**

**Therefore the calibration limitation is confirmed at full strength: an exceptional `T` under this
order null rejects exchangeable ordering, and does not establish motif structure.** The recordings
depart from exchangeability on the most trivial local statistic there is, and the search cannot help
but see it.

### 12. Specimen-level verdicts

`57f18ff9 × 7d95c414` and `f5c18299 × 7d95c414` separate at every `k`. `57f18ff9 × f5c18299` separates
at every `k ≥ 4`. `8b68dece × 57f18ff9` separates at every `k ≥ 4`. `8b68dece × f5c18299` and
`8b68dece × 7d95c414` separate at three of five `k` each. No specimen fails to separate anywhere, and
no specimen separates everywhere except the three named. The corpus is not forced to one verdict beyond
the preregistered majority rule.

### 13. The narrowest supported claim

> **The complete search procedure — alignment-ranked candidate generation, greedy deduplication, and an
> R1 readout — produces maxima on 23 of 30 real cross-recording specimens that no more than 1% of
> order-null replicates of the same search reach.** The controls establish that the procedure does not
> flag a specimen drawn from the null and does recover a planted figure.

### 14. Explicitly unsupported

Not earned, and not asserted: a calibrated probability that any span is a motif; workflow identity for
any discovered span; semantic reuse; any causal reading; and — because of §PHASE 11 — **any claim that
the separation reflects reusable motif structure rather than the instrument's own request→outcome
alternation.** Observational recordings remain specimens, not ground truth. No discovered span was
named, described, or inspected, and no recording content appears in any artifact.

R1 remains a proposal. Nothing was adopted, nothing was promoted to production, and no second score
was invented.

### 15. One next experiment

The order null is too weak, and §PHASE 11 says exactly how: it destroys an instrument regularity that
has nothing to do with motifs. The narrowest better null that fixes this is a **transition-preserving
null** — resample each sequence from its own first-order Markov chain, so vocabulary, marginals, length
and the request→outcome alternation all survive while longer-range reuse does not.

The next round should rerun **this exact machinery** against that null, changing nothing else: same
`T`, same search, same controls, same `B`, same thresholds. If separation survives a null that already
reproduces the alternation, the claim strengthens materially. If it collapses, this round's separation
was the alternation all along — and that is the more valuable outcome to know.

### 16. Gates

`scripts/check.sh` green and unweakened. **393 tests**, up from 375; 18 new in `tests/calibration.rs`.
`scarp doctor` clean. Nothing pushed. Counts, scores, ranks, quantiles and exceedances only, per
decision:8.

**One criterion defect, this round's own, caught by the suite.** The first draft of
`tests/calibration.rs` asserted the positive control was `exceptional` at `B = 49`, where the best
attainable tail is `1/50 = 0.02` and the preregistered `0.01` threshold can never be reached. The
assertion could not pass at that `B`. It was fixed by giving the test a `B` that resolves the
threshold — never by loosening the threshold — and a new test now asserts the general relation
`1/(B+1) ≤ threshold`, so the class is caught by construction rather than by luck. **The experiment's
own `B = 999` was unaffected**; the defect lived only in the test's replicate count.

The calibration renders as one card on the existing evidence page. It draws both distributions of
§PHASE 7 on **one shared axis**, because the selection effect is a statement about where one sits
relative to the other and two separate plots would let a reader miss it. The null-adequacy table sits
beside the separation table rather than in a section of its own: an exceptional `T` that the
instrument's own alternation explains must not be legible as a motif finding, and separating the two
would allow exactly that.

Commits: preregistration `89f53d2`, alone and before implementation; experiment and rendering below.
