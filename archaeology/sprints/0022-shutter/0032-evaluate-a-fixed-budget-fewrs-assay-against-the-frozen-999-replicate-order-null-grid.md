---
id: tsk_01KZSCQY1V194BXKA99T107KHA
sequence: 32
kind: task
status: closed
sprint: spr_01KZSCKFB7AFVM9XA9DA5HV6ZE
created: 2026-08-11
closed: 2026-08-11
---

# Evaluate a fixed-budget FewRS assay against the frozen 999-replicate order-null grid

## Objective

Measure whether a fixed 459-search, search-aware maximum-null assay — Few Random Searches (FewRS) at
`alpha = 0.01`, certifying iff `observed > max(null)` — reproduces enough of sprint:19's frozen
999-replicate order-null conclusion to justify its narrower evidence surface, and recommend adopting it
narrowly, investigating further, or retiring the idea.

`### Preregistration` was written and committed **before any FewRS code existed and before any 459-
replicate maximum null was computed.** What was computed first is §PHASE 0's repository-truth pass,
§PHASE 1's arithmetic on `m`, and §PHASE 6's transcription of the frozen grid — all three read only the
committed archaeology and the committed source, and none of them runs a search.

## Acceptance criteria

- Repository truth reconstructed from code and archaeology before implementation; every discrepancy
  between the commission and the repository recorded, not worked around.
- `m` derived from the FewRS formula by code and pinned by a test at 459.
- Strict `observed > max(null)`; ties do not certify; undefined `T` does not certify; an empty null set
  does not certify.
- The unchanged `calibration::complete_search` rerun inside every replicate through the existing
  `calibrate` path. No second implementation of `T_k`. No change to the null generator or seed schedule.
- Controls executed before observational cells; each control rule's reachability checked, including the
  rules that cannot fail.
- All 30 cells, none selected post hoc; both comparison grids reported.
- The classification thresholds applied exactly as fixed below.
- `scripts/check.sh` passes unweakened; nothing pushed; no recording content in any artifact.

### Preregistration

#### PHASE 0 — Repository truth, verified from code and archaeology

| premise | verdict |
|---|---|
| `T_k(A,B) = max over dedupe_overlapping(cross_pairs(A,B,k,MAX), 5) of R1(c)` | **confirmed** — `calibration::complete_search`, unchanged since sprint:19 |
| both paths call one `complete_search`; every data-dependent stage reruns inside each replicate | **confirmed** — `calibration::calibrate_with` rebuilds both sides then calls it; `null_evidence` is not called |
| the order null is `order_null_seeded`; seeds are `null_seed(i, side)`, `i` ascending from 0 | **confirmed** — `event_sequence::null_seed`, `ORDER_NULL_SEED = 0x4F52_4445_524E_554C` |
| the replicate count is already a parameter | **confirmed** — `calibrate(.., replicates)`; `REPLICATES = 999` is the sprint:19 default, not a hard-wired loop bound |
| sprint:19 published a 30-cell order-null grid over decision:8's four admitted specimens at `k in {3,4,6,8,12}` | **confirmed** — task:29 §9; reproduced cell for cell by sprint:20 §PHASE 0 |
| the controls are `calibration::negative_control()` and `positive_control()`, the planted figure at `k = 12` | **confirmed** — `PLANTED_FIGURE`, `PLANT_SITES_A/B` |
| the FewRS upstream repository provides no reusable implementation | **taken from the commission and not independently verified**; the procedure implemented here is local and tiny either way |

**Five discrepancies, all recorded before any criterion below was written.**

**D8 — the frozen sprint:19 grid is not a maximum-null grid, and "agreement" therefore conflates two
changes.** sprint:19's verdict rule is `p_hat = (1 + exceedances)/1000 <= 0.01`, which admits up to
**nine** null replicates reaching or beating the observation. FewRS certification is
`exceedances = 0`. Comparing a 459-cell FewRS grid against the frozen `p_hat <= 0.01` grid therefore
mixes a **budget change** (999 -> 459) with a **decision-rule change** (tail threshold -> strict
maximum). Both comparisons are reported, and neither is presented as the other:

- **Primary, as the commission names it:** FewRS(459) certification against sprint:19's
  `exceptional = p_hat <= 0.01` grid — **23 of 30**.
- **Secondary, rule-matched:** FewRS(459) certification against the strict-maximum rule applied to the
  same frozen numbers. `exceedances = 1000 * p_hat - 1`, so `exceedances = 0` is recoverable exactly
  from the published grid without rerunning anything — **13 of 30**.

**D9 — the first 459 seeds are a prefix of the 999 already spent, so most of this round's outcome is
already determined by the frozen archaeology.** `calibrate` consumes realizations `0..B` in index
order and `null_seed(i, side)` does not depend on `B`, so the 459-replicate null sample is a **subset**
of the 999-replicate one and `max_{i<459} T_null <= max_{i<999} T_null`. Three consequences, all fixed
before execution:

1. Every cell with `exceedances = 0` at 999 **certifies at 459 with certainty** — 13 cells.
2. No cell certifies at 459 unless **every one** of its 999-exceedances falls at index `>= 459`.
3. The only live quantity in the whole observational assay is how many of the remaining 17 cells'
   exceedance sets happen to sit entirely in the tail of the seed schedule.

This is disclosed here rather than discovered in the Result. It also means the round is a **retrospective
audit of a budget**, not an independent replication: it re-runs 459 of the same 999 searches and asks
what a different rule would have said about them.

**D10 — the positive control's rule cannot fail, given the frozen archaeology.** sprint:19 published
`p_hat = 0.001` for the positive control at `k = 12`, i.e. zero of 999 null searches reached it. By D9
the 459-prefix maximum cannot exceed the 999 maximum, so certification at `k = 12` is **certain**. The
rule is retained as an **implementation check** — a failure would mean the assay is not calling the
frozen machinery, which is worth catching — and is explicitly **not** an empirical test of FewRS's
sensitivity. This is sprint:20's D4 disclosure applied to this round, and sprint:18's eleventh defect
applied to a rule that passes rather than to one that fails.

**D11 — FewRS's guarantee is family-wise over a pooled maximum, and this round does not implement
that.** The paper's rule compares each analysis against `delta-hat(S)`, the maximum statistic over
**all analyses and all m resamples**, and its Corollary 3 derives `FWER <= alpha` from that pooled
maximum together with subset pivotality and i.i.d. resamples. This round compares each cell against
**its own** null maximum, per the commission. That per-cell rule is a valid exact conditional test at
level `1/(m+1) = 1/460` by exchangeability of the observed statistic with its own 459 null statistics,
and it is **not** FewRS's family-wise procedure.

The pooled variant is additionally **unavailable on this grid**: `T_k` at different `k` are R1 sums over
different span lengths and are not on a common scale, and §PHASE 11 forbids normalizing or combining
them. So the only thing FewRS can buy here is the **budget**, not the multiplicity guarantee. Recorded
before execution because it decides what §PHASE 9's recommendation may say: a positive result is a
statement about cost, never about coverage.

**D12 — sprint:20 already established that the sprint:19 order-null separation is fully explained by
first-order transition structure.** Zero of 30 cells separate under the exact doublet null; retention
`0/23`. This round's cells are **order-null** cells, and per the commission **no observational
conclusion here reads the first-order null's distributional behaviour**. But a recommendation to adopt a
cheaper order-null assay would be recommending a cheaper route to a quantity sprint:20 showed to be
uninformative about motif structure. That is carried into §PHASE 9's recommendation branches now, not
discovered later.

#### PHASE 1 — The budget, derived rather than asserted

```text
m = ceil( ln(1/alpha) / ln(1/(1-alpha)) )
```

At `alpha = 0.01`: `ln(1/0.01) = 4.6051701860`, `ln(1/0.99) = 0.0100503359`, ratio `458.210577`,
`ceil = 459`.

`fewrs_budget(alpha)` computes this. It **rejects** any `alpha` that is not finite or not strictly
inside `(0, 1)`; `0` and `1` are rejected as well, since the formula is undefined at both. A test pins
`fewrs_budget(0.01) == 459` and a second test pins the module constant to the function's own output, so
the constant cannot drift from the derivation.

`alpha = 0.01` is sprint:19's frozen `TAIL_THRESHOLD` and is not re-chosen here.

#### PHASE 2 — The decision rule

```text
certified(observed, nulls)  iff  observed is defined
                            and  nulls is non-empty
                            and  every n in nulls satisfies n < observed
```

Equivalently `observed > max(nulls)`. **Ties do not certify.** An undefined `T` does not certify and is
reported as undefined. An empty null set does not certify — no evidence is not evidence — and this is
the branch that would otherwise turn a search that admitted no candidate into a certification.

The rule reads only the null values, so it is order-invariant by construction; a test asserts that a
shuffled null vector gives the same answer.

#### PHASE 3 — The analysis inside every replicate

Unchanged, and rerun in full inside each of the 459 replicates:

```text
all window pairs at k  ->  align()  ->  rank by alignment.total asc, then a.start, then b.start
                       ->  dedupe_overlapping(.., 5)  ->  R1 readout  ->  max = T_k
```

The FewRS path calls `calibration::calibrate(specimen, A, B, k, 459)` and reads `observed` and the
`samples` vector it already computes. **No new search, no new null, no new seed schedule, no
preselection of motifs or candidates.** A test asserts that the FewRS cell's null maximum equals the
maximum of the calibration's own `samples`, and that its replicate count is 459.

#### PHASE 4 — Execution order and the control rules

**Order is fixed: negative control, then positive control, then — only if both pass — the 30
observational cells.**

**NEGATIVE control.** `calibration::negative_control()`, unchanged: two i.i.d. sequences of 160 and 90
marks over the 12-symbol synthetic alphabet. This *is* the order-null hypothesis.
*Quantity:* `certified` at each `k` in `{3,4,6,8,12}` at `m = 459`.
*Rule:* **PASS** iff `certified` is false at every `k`.
*Reachability:* attainable — under the null the per-`k` certification probability is `1/460`, so all
five staying false has probability about `0.989`. Refutable — five cells at `1/460` leave a residual
false-flag risk of about **1.1%**, stated now rather than discovered later. It is also refutable in a
second and more useful way: sprint:19 measured 34 exceedances of 999 at `k = 8` for this control, and
certification at 459 would require all 34 to fall at index `>= 459`, which is possible and would be a
real signal that the seed prefix is unrepresentative.

**POSITIVE control.** `calibration::positive_control()`, unchanged: the same generator with the fixed
12-mark figure planted at three sites in each sequence.
*Quantity:* `certified` at `k = 12` at `m = 459`.
*Rule:* **PASS** iff `certified` is true at `k = 12`.
*Reachability:* **This rule cannot fail — see D10.** It is an implementation check, not an empirical
test, and the Result says so in those words.

*If either control fails, the observational assay is not run* and the round reports falsification with
the control's own numbers.

#### PHASE 5 — The observational assay

All 30 cells: decision:8's four admitted specimens `8b68dece`, `57f18ff9`, `f5c18299`, `7d95c414`,
reduced to six unordered pairs, at every `k` in the frozen ladder `{3,4,6,8,12}`. **No cell is selected
post hoc and none is excluded.** Null replicates `i in 0..459`, seeds `null_seed(i, 0)` and
`null_seed(i, 1)` — the existing deterministic schedule, its first 459 realizations, unchanged.

Per cell, retained for audit: `observed` `T_k`; `null_max`; `certified`; `null_searches` actually
performed; the historical 999-replicate `p_hat`; the historical `exceedances`; the historical
`exceptional` verdict; the historical strict-maximum verdict; agreement under each; and the seed range
identity `null_seed(0..459, {0,1})`.

Hygiene per decision:8: opaque eight-character session prefixes, counts, scores and verdicts only. No
prompt, command, response, path, host identity or payload excerpt, and **no discovered span is named,
described or inspected.**

#### PHASE 6 — The frozen reference grid

Transcribed from task:29 §9 and reproduced cell for cell by sprint:20 §PHASE 0. `exceedances` is derived
exactly as `1000 * p_hat - 1`; no fresh 999-replicate campaign is run, per the commission and because
the historical benchmark reconstructs honestly.

| pair | k=3 | k=4 | k=6 | k=8 | k=12 |
|---|---|---|---|---|---|
| `8b68dece x 57f18ff9` | 0.157 (156) | 0.004 (3) | 0.005 (4) | 0.004 (3) | 0.007 (6) |
| `8b68dece x f5c18299` | 0.004 (3) | 0.003 (2) | 0.015 (14) | 0.002 (1) | 0.075 (74) |
| `8b68dece x 7d95c414` | 0.133 (132) | 0.031 (30) | 0.027 (26) | 0.003 (2) | 0.002 (1) |
| `57f18ff9 x f5c18299` | 0.035 (34) | 0.001 (0) | 0.001 (0) | 0.001 (0) | 0.001 (0) |
| `57f18ff9 x 7d95c414` | 0.002 (1) | 0.001 (0) | 0.001 (0) | 0.001 (0) | 0.001 (0) |
| `f5c18299 x 7d95c414` | 0.001 (0) | 0.001 (0) | 0.001 (0) | 0.001 (0) | 0.001 (0) |

**23 of 30** at `p_hat <= 0.01`. **13 of 30** at `exceedances = 0`. A test asserts both counts against
the transcribed table, so a typo in the transcription is caught by the suite rather than by a reader.

#### PHASE 7 — Cost accounting, in the quantities the code will report

- `null_searches` — complete searches actually performed, summed over controls and cells. Planned:
  `459 * (5 + 5 + 30) = 18360`. The sprint:19 reference at the same coverage is `999 * 40 = 39960`.
- `null_datasets` — order-null sequence realizations generated: two per replicate, so `2 * 459` per
  cell, `36720` planned.
- `searches_avoided` — `39960 - 18360 = 21600`, ratio `999/459 = 2.176`.
- `candidate_evaluations` — reported **only** from existing instrumentation: `Calibration.
  observed_considered` and `null_considered_mean` already count enumerated window pairs per search, and
  the product with `null_searches` is a trustworthy count of enumerated pairs. No new counter is added,
  and no per-candidate R1 count is claimed, because none is instrumented.
- `wall_clock_seconds` — measured once, on one machine, and labelled machine-specific and secondary. It
  decides nothing.
- `earliest_refutation` — for each non-certifying cell, the smallest replicate index at which a null
  reached the observation. Reported as the **counterfactual** cost of an early-stopping variant.
  **No early-stopping path is executed**: the fixed 459-replicate run is what produces the maximum-null
  comparisons and the audit output, and reporting a counterfactual index is honest where reporting a
  saving that was never realised would not be.

#### PHASE 8 — Feasibility propagation pass

Every numerical rule, on **every** branch — PASS, FAIL and classification alike.

| mechanism | rules touched | disposition |
|---|---|---|
| **D8** the frozen grid is a tail grid, not a maximum grid | the agreement rate; the classification's "preserving sprint:19's majority verdict" | **Propagated before execution.** Two agreement rates are defined, computed and reported; the primary one is named in §PHASE 9 so the classification cannot be re-pointed at whichever comparison flatters the result. |
| **D9** the 459 seeds are a prefix of the 999 | every prediction; the meaning of the whole round | **Propagated.** The 13 guaranteed cells are named in advance; §PHASE 10 records the predicted count and the arithmetic behind it; the Result must describe this as a retrospective budget audit and not as an independent replication. |
| **D9**, again, on the classification threshold | the 15-of-30 boundary | **Checked, and the result is uncomfortable, which is why it is written down.** Under a heuristic treating each cell's 999-exceedances as exchangeable across the seed schedule, the expected certified count is **15.79** against a threshold of **15**. The round is close to its own boundary by construction. The threshold is **not** moved. |
| **D10** the positive control cannot fail | the positive control rule; the falsification branch | **Disclosed and demoted** to an implementation check. The falsification branch still fires on it, because an implementation check that fails means the assay is not running the frozen machinery. |
| **D11** FewRS's guarantee is family-wise; this is not | every claim about what certification buys; the recommendation | **Propagated.** The Result claims only a per-cell exact conditional test at level `1/460` and explicitly disclaims FWER across the 30 cells. The recommendation may cite cost, never coverage. |
| **D11**, again, on the budget's justification | whether `m = 459` is the right number for a per-cell test | **Recorded.** For a single scalar assay the exchangeability bound already gives level `1/(m+1)`, so `m = 99` would suffice for `alpha = 0.01` and `m = 459` is **conservative by a factor of about 4.6**. The commission freezes 459 and this round runs 459; the observation that a per-cell reading of FewRS over-buys is a finding, not a licence to change the budget mid-round. |
| **D12** sprint:20 collapsed the order-null separation | the recommendation branches | **Propagated.** No observational conclusion here reads sprint:20's distributions, and the recommendation states that a cheaper route to an order-null verdict is a cheaper route to a quantity sprint:20 bounded. |
| strict `>` versus `>=` | certification of every cell | **Checked.** `observed > max(null)` and `exceedances = count(T_null >= T_observed) = 0` are the same event, so the frozen grid's exceedance counts and this round's rule agree in definition and the secondary comparison is exact rather than approximate. |
| `T_k` undefined | certification; the cell count | **Checked.** `calibrate` returns `observed = None` when `cross_pairs` admits nothing. §PHASE 2 makes that non-certifying, and sprint:19 reported `T` defined for all 30 cells, so the branch is expected to be unexercised and is reported as such if so. |
| floating point equality at the tie boundary | the tie rule | **Checked and left exact.** `T` values on the two paths come from the same `f64` arithmetic in the same order, so an exact tie is representable and the strict comparison resolves it against certification. No epsilon is introduced; an epsilon would be a threshold chosen after seeing data. |
| the negative control's rule is a FAIL condition | its reachability | **Checked explicitly**, on both the per-cell level (`1/460` each, ~1.1% across five) and the historical prefix (34 exceedances at `k = 8` would all have to land past index 459). |
| the three classification branches | tiling | **Checked.** Falsification is checked first and fires on: positive control fails, negative control certifies at any `k`, the seed or null contract cannot be reconstructed, or the assay yields no conclusion beyond a cheaper non-rejection. Otherwise `certified >= 15` is Strong and `certified < 15` is Weak/mixed. The two are complementary and exhaust the remainder, so no outcome falls between them. |
| "some of the strongest cells certify" in the Weak branch | whether Weak is a criterion or a description | **Amended before execution.** The commission's Weak branch reads "both controls pass and some of the strongest cells certify, but fewer than 15 of 30 certify." "Some of the strongest cells" is not a computed quantity and would not tile against Strong. The branch is therefore decided **only** by `controls pass and certified < 15`, and the strongest-cell observation is reported inside it as a description. decision:7 Rule 1. |

#### PHASE 9 — Classification partition, by precedence

1. **FALSIFICATION.** Any of: the positive control fails its §PHASE 4 rule; the negative control
   certifies at any `k`; the seed or null contract cannot be reconstructed honestly; or the method
   yields no useful conclusion beyond a cheaper non-rejection. Checked first.
2. **STRONG.** Both controls pass **and** `certified >= 15` of the 30 cells.
3. **WEAK / MIXED.** Both controls pass **and** `certified < 15`.

`certified` is the count of `(pair, k)` cells whose `certified` field is true at `m = 459`. The
**primary agreement rate** is `|{cells where certified == exceptional_at_999}| / 30` against the frozen
`p_hat <= 0.01` grid. The **secondary agreement rate** is the same count against
`exceedances_at_999 == 0`. Both are reported as exact fractions and neither substitutes for the other.

**The recommendation is a separate output from the classification** and takes one of three values,
decided on the classification together with D11 and D12:

- **adopt narrowly** — only if Strong *and* the round can state a use for a binary order-null
  certification that sprint:20 has not already bounded;
- **investigate further** — Strong or Weak with a specific named next measurement;
- **retire the idea** — Falsification, or a result whose only content is that 2.18x fewer searches buy
  a strictly weaker readout of a quantity this project has already stopped trusting.

#### PHASE 10 — Predicted outcome, recorded before execution

**Predicted: `certified` between 13 and 19, most likely 15 to 17; classification most likely STRONG but
genuinely capable of landing WEAK/MIXED.** The arithmetic, from the frozen grid alone: 13 cells have
zero exceedances at 999 and certify with certainty; treating each remaining cell's exceedances as
exchangeable across the seed schedule gives an expected additional 2.79 cells and an expected total of
**15.79**, against a threshold of 15. The cells with a real chance of joining are exactly those with
`exceedances <= 6`: `8b68dece x f5c18299` at `k = 8` (1) and `k = 4` (2), `8b68dece x 7d95c414` at
`k = 12` (1) and `k = 8` (2), `57f18ff9 x 7d95c414` at `k = 3` (1), `8b68dece x 57f18ff9` at `k = 4`
(3), `k = 8` (3), `k = 6` (4) and `k = 12` (6), and `8b68dece x f5c18299` at `k = 3` (3).

**Predicted primary agreement rate: about 0.73 to 0.83** — FewRS can only lose cells relative to the 23,
essentially never gain them, so agreement is about `(30 - (23 - certified))/30`.

**Predicted recommendation: retire the idea or investigate further, whatever the classification.** The
prediction is recorded here so it cannot be adjusted afterwards: even a Strong classification buys a
2.18x cost reduction on a per-cell test that D11 shows is already over-budgeted by 4.6x for its actual
guarantee, on a quantity sprint:20 showed is explained by first-order structure. If that reasoning is
wrong the Result says so.

#### PHASE 11 — What this task will not do

No general FewRS subsystem and no multi-analysis infrastructure. No change to `T_k`, `complete_search`,
`cross_pairs`, `align`, `dedupe_overlapping`, R1, the ladder, the null generator, the seed schedule, the
`KEEP`/`TOP_K` constants, or decision:8's inventory. No normalizing or combining of statistics across
`k`. No claim of family-wise error control across the 30 cells. No claim that FewRS validates the LCG,
the modulo reduction, or the capped-rejection doublet sampler — those are recorded as caveats and not
worked on. No presentation of a non-certification as evidence that observed structure "collapsed" into
the null. No fresh 999-replicate campaign. No adoption, no promotion to production, no threshold moved
after seeing data. No recording content in any artifact. Nothing pushed.

## Result

**Classification: STRONG**, by the preregistered partition and without softening it. Both controls
passed and **17 of 30** cells certified at `m = 459`, against a threshold of 15. Primary agreement with
sprint:19's frozen 999-replicate grid **24/30 = 0.8000**; rule-matched agreement **26/30 = 0.8667**.

**Recommendation: RETIRE the idea for this workflow.** The classification is STRONG and the
recommendation is still to retire, and that is not a contradiction — it is the round doing what
sprint:22 was built to do. §PHASE 9 makes the recommendation a separate output from the classification,
and §10 below is why: a descriptive measurement no criterion reads shows the FewRS budget is
**dominated** here. At `m = 99` — the smallest budget at which a strict-maximum rule is a valid
level-0.01 test, and *identical* to sprint:19's own `p_hat <= 0.01` rule at `B = 99` — **22 of 30**
cells certify at **4.6x lower cost than 459** and **10.1x lower than 999**, with better agreement
against the frozen grid (27/30). FewRS's `m = 459` is calibrated for a family-wise pooled maximum this
grid cannot form, and per cell it buys nothing that 99 replicates do not buy more cheaply.

`T_k`, the complete search, the null generator, the seed schedule and decision:8's inventory were not
changed. Nothing was adopted.

### 1. Repository truth, and the five discrepancies

All seven §PHASE 0 premises confirmed from the committed source. The five discrepancies recorded before
any criterion was written all held up, and three of them decided the round:

- **D8 confirmed.** sprint:19's grid is a tail grid; 23 of 30 cells clear `p_hat <= 0.01`, but only
  **13** have zero exceedances. Both agreement rates are reported below and neither substitutes for the
  other.
- **D9 confirmed empirically, not just argued.** See §6: all 40 cells' refuting counts at 459 came in at
  or below their frozen 999 exceedance counts, which is the prefix property measured rather than
  assumed.
- **D10 confirmed.** The positive control certified at `k = 12`, as it was guaranteed to. It is reported
  as an implementation check and is **not** evidence of sensitivity.
- **D11 confirmed and load-bearing.** See §10 and §11.
- **D12 confirmed and load-bearing.** See §12.

### 2. The budget, derived

`m = ceil(ln(1/0.01) / ln(1/0.99)) = ceil(4.6051701860 / 0.0100503359) = ceil(458.2106) = 459`.

Computed by `fewrs_budget`, which refuses any `alpha` outside the open unit interval; the module
constant is pinned to the function's own output by a test, so a transcription cannot drift from the
derivation. `alpha` is sprint:19's frozen `TAIL_THRESHOLD`, re-used and not re-chosen.

### 3. Controls, run before any recording was replayed

| control | k | `T` observed | null max (459) | refuting/459 | certified |
|---|---|---|---|---|---|
| negative | 3 | 6.4803 | 9.3790 | 295 | no |
| negative | 4 | 7.4233 | 11.4657 | 383 | no |
| negative | 6 | 9.9306 | 13.7242 | 178 | no |
| negative | 8 | 13.5958 | 15.4712 | 14 | no |
| negative | 12 | 13.3727 | 18.5008 | 163 | no |
| positive | 3 | 7.2099 | 9.2920 | 181 | no |
| positive | 4 | 9.4663 | 12.1924 | 50 | no |
| positive | 6 | 13.8184 | 14.0917 | 1 | no |
| positive | 8 | 18.2795 | 15.2047 | 0 | **YES** |
| positive | 12 | 27.5715 | 19.6596 | 0 | **YES** |

**Negative: PASS** — no `k` certifies. **Positive: PASS** at `k = 12` — and per D10 this rule could not
have failed, so it establishes that the assay is calling the frozen machinery and nothing more.

The observed `T` column reproduces sprint:19 §8 exactly: the positive control's climb of
`13.8184 -> 18.2795 -> 27.5715` across `k = 6, 8, 12` is the `13.82 -> 18.28 -> 27.57` sprint:19
published.

### 4. The 30-cell FewRS grid

`m = 459`, seeds `null_seed(0..459, {0,1})`, the complete search rerun inside every replicate. No cell
was excluded and `T` was defined for all 30.

| pair | k | `T` observed | null max (459) | certified | refuting/459 | 999 `p_hat` | 999 exceedances | primary agree |
|---|---|---|---|---|---|---|---|---|
| `8b68dece x 57f18ff9` | 3 | 7.7696 | 10.0722 | no | 69 | 0.157 | 156 | yes |
| `8b68dece x 57f18ff9` | 4 | 11.3589 | 11.0736 | **YES** | 0 | 0.004 | 3 | yes |
| `8b68dece x 57f18ff9` | 6 | 12.2136 | 13.8764 | no | 2 | 0.005 | 4 | **NO** |
| `8b68dece x 57f18ff9` | 8 | 13.2656 | 14.4650 | no | 2 | 0.004 | 3 | **NO** |
| `8b68dece x 57f18ff9` | 12 | 14.5523 | 16.8185 | no | 3 | 0.007 | 6 | **NO** |
| `8b68dece x f5c18299` | 3 | 9.7022 | 9.3737 | **YES** | 0 | 0.004 | 3 | yes |
| `8b68dece x f5c18299` | 4 | 11.3642 | 11.3398 | **YES** | 0 | 0.003 | 2 | yes |
| `8b68dece x f5c18299` | 6 | 11.3642 | 15.5107 | no | 5 | 0.015 | 14 | yes |
| `8b68dece x f5c18299` | 8 | 15.1985 | 14.5092 | **YES** | 0 | 0.002 | 1 | yes |
| `8b68dece x f5c18299` | 12 | 11.6675 | 18.8730 | no | 35 | 0.075 | 74 | yes |
| `8b68dece x 7d95c414` | 3 | 5.5266 | 7.9798 | no | 67 | 0.133 | 132 | yes |
| `8b68dece x 7d95c414` | 4 | 8.8190 | 9.1078 | no | 14 | 0.031 | 30 | yes |
| `8b68dece x 7d95c414` | 6 | 10.7862 | 13.3511 | no | 12 | 0.027 | 26 | yes |
| `8b68dece x 7d95c414` | 8 | 13.0205 | 14.3239 | no | 2 | 0.003 | 2 | **NO** |
| `8b68dece x 7d95c414` | 12 | 16.6878 | 17.7498 | no | 1 | 0.002 | 1 | **NO** |
| `57f18ff9 x f5c18299` | 3 | 8.1411 | 9.7506 | no | 17 | 0.035 | 34 | yes |
| `57f18ff9 x f5c18299` | 4 | 11.2169 | 11.0346 | **YES** | 0 | 0.001 | 0 | yes |
| `57f18ff9 x f5c18299` | 6 | 16.6753 | 12.7241 | **YES** | 0 | 0.001 | 0 | yes |
| `57f18ff9 x f5c18299` | 8 | 22.7215 | 14.4135 | **YES** | 0 | 0.001 | 0 | yes |
| `57f18ff9 x f5c18299` | 12 | 31.6766 | 17.1639 | **YES** | 0 | 0.001 | 0 | yes |
| `57f18ff9 x 7d95c414` | 3 | 10.2028 | 10.6728 | no | 1 | 0.002 | 1 | **NO** |
| `57f18ff9 x 7d95c414` | 4 | 11.3388 | 10.9494 | **YES** | 0 | 0.001 | 0 | yes |
| `57f18ff9 x 7d95c414` | 6 | 13.7322 | 12.7739 | **YES** | 0 | 0.001 | 0 | yes |
| `57f18ff9 x 7d95c414` | 8 | 17.0373 | 14.8262 | **YES** | 0 | 0.001 | 0 | yes |
| `57f18ff9 x 7d95c414` | 12 | 18.2946 | 15.5240 | **YES** | 0 | 0.001 | 0 | yes |
| `f5c18299 x 7d95c414` | 3 | 10.9234 | 9.4963 | **YES** | 0 | 0.001 | 0 | yes |
| `f5c18299 x 7d95c414` | 4 | 14.2376 | 10.4591 | **YES** | 0 | 0.001 | 0 | yes |
| `f5c18299 x 7d95c414` | 6 | 16.6175 | 11.6042 | **YES** | 0 | 0.001 | 0 | yes |
| `f5c18299 x 7d95c414` | 8 | 17.7626 | 12.7493 | **YES** | 0 | 0.001 | 0 | yes |
| `f5c18299 x 7d95c414` | 12 | 20.1425 | 15.8224 | **YES** | 0 | 0.001 | 0 | yes |

`57f18ff9 x f5c18299` at `k = 12` returns `T = 31.6766`, which is sprint:19's published `31.68`, against
a 459-null maximum of `17.1639` — below the `18.18` sprint:19 measured over 999, exactly as a prefix
must be.

### 5. Comparison with the frozen 999-replicate grid, and the disagreements

| comparison | rule compared against | agreement |
|---|---|---|
| **primary** | sprint:19's own `p_hat <= 0.01` (23 of 30) | **24/30 = 0.8000** |
| **rule-matched** | strict maximum at 999, `exceedances == 0` (13 of 30) | **26/30 = 0.8667** |

**Six cells sprint:19 called exceptional do not certify**, and the cause is the same in every one: the
999-replicate tail rule tolerates up to nine exceedances and the strict-maximum rule tolerates none.

| cell | 999 exceedances | refuting/459 |
|---|---|---|
| `8b68dece x 57f18ff9` k=6 | 4 | 2 |
| `8b68dece x 57f18ff9` k=8 | 3 | 2 |
| `8b68dece x 57f18ff9` k=12 | 6 | 3 |
| `8b68dece x 7d95c414` k=8 | 2 | 2 |
| `8b68dece x 7d95c414` k=12 | 1 | 1 |
| `57f18ff9 x 7d95c414` k=3 | 1 | 1 |

Every one had at least one refuting null inside the first 459. **This is a rule difference, not a budget
difference**, and the rule-matched column separates the two: against the strict rule at 999 the same six
cells are non-certifications there as well.

**Four cells certify at 459 that the strict rule at 999 refuses**, which is the other direction and the
more interesting one:

| cell | 999 exceedances | refuting/459 |
|---|---|---|
| `8b68dece x 57f18ff9` k=4 | 3 | 0 |
| `8b68dece x f5c18299` k=3 | 3 | 0 |
| `8b68dece x f5c18299` k=4 | 2 | 0 |
| `8b68dece x f5c18299` k=8 | 1 | 0 |

All four had every one of their 999-exceedances land at replicate index 459 or later. **A FewRS
certification at `m` can be overturned by a larger `m`** — the null maximum is monotone increasing in
the budget, so certification is monotone *decreasing*. That is not a defect: the level guarantee holds
at each `m`. But it means a certification is a statement made at a budget, and the budget is part of the
claim. Nothing in the FewRS framing says otherwise; it is worth writing down because a binary "certified"
column reads as if it were not.

**No cell certified that sprint:19's tail rule called ordinary**, so the primary disagreement is
one-directional at this budget: 24 = 30 − 6.

### 6. The seed-prefix property, measured

For all **40** cells — ten control, thirty observational — the refuting-null count at 459 was less than
or equal to the frozen 999 exceedance count, without exception. Examples at both ends: the negative
control at `k = 4` returned 383 of 459 against sprint:19's 824 of 999; the positive control at `k = 12`
returned 0 against 0.

This is D9's prefix claim measured rather than argued, and it is also the round's independent check that
the machinery still reproduces sprint:19: forty separate counts, forty consistent, on a grid nothing in
this round could tune.

### 7. Classification, applied as fixed

`certified = 17`, `undefined = 0`, both control rules PASS. §PHASE 9's precedence: falsification does
not fire; `17 >= 15` gives **STRONG**. §PHASE 10 predicted 13 to 19, most likely 15 to 17, and predicted
a primary agreement rate of 0.73 to 0.83. Both landed inside: 17 and 0.8000. The prediction is recorded
in the preregistration commit `1c1330e` and was not adjusted.

**The threshold was not moved and the classification is not softened.** It is also not the whole
finding, and §PHASE 9 anticipated that by making the recommendation a separate output.

### 8. Cost accounting

| quantity | this round | sprint:19 reference | change |
|---|---|---|---|
| complete null searches | **18 360** | 39 960 | −21 600, **2.176x fewer** |
| null sequence realizations generated | **36 720** | 79 920 | −43 200 |
| window pairs enumerated inside null searches | **117 428 265** | — | existing instrumentation only |
| wall clock, one release build, one machine | **72.9 s** | not comparable — see below | machine-specific and secondary |

Coverage is ten control cells plus thirty observational cells at both budgets. **The wall clock has no
honest reference figure**: sprint:20 measured a full `--calibrate` pass at 196 s, but that pass also runs
the 400-search selection-effect demonstration and the 199-replicate adequacy summaries, so it is not
like-for-like and is not used as a ratio here. The search counts are the comparison that means
something. `null_candidate_
evaluations` comes from `Calibration::null_considered_mean`, which already counted enumerated window
pairs before this round; **no new counter was added and no per-candidate R1 count is claimed**, because
none is instrumented.

The theoretical ratio is `999/459 = 2.176`, and the measured search count reproduces it exactly because
no cell stopped early. **This is not the 8-to-64-search figure FewRS's examples suggest** — those are at
looser `alpha`, where the budget formula collapses quickly. At `alpha = 0.01` the formula returns 459.

**Early stopping was not implemented and not run.** The Result reports, per non-certifying cell, the
expected stop index under exchangeable replicate ordering, `(m+1)/(r+1)`. It is an expectation and is
labelled as one: `calibrate` returns its samples sorted, so no measured first-refutation index exists in
this run, and reading a position out of a sorted vector and calling it a replicate index would have been
a fabricated saving. Where a cell has many refuting nulls the counterfactual saving is large — the
negative control at `k = 4` would have stopped around replicate 1 — and where it has one, it is not
(around 230). Since 13 of 30 cells certify and therefore spend the full budget regardless, early
stopping would have reduced the observational pass by well under half.

### 9. Binary certification is not distributional calibration

This is the boundary the round exists to draw, and it is not a caveat but a structural fact about the
instrument.

A FewRS cell reports two numbers and one bit: `observed`, `null_max`, `certified`. From those, the
following quantities **cannot** be recovered, at any budget:

- the null **median** and every other quantile — sprint:19's `null_quantiles` at 0.05/0.25/0.50/0.75/
  0.95/0.99;
- the observed value's **percentile** within the null, `count(T_null < T_observed)/B`;
- **percentile movement** between two nulls, which is sprint:20 §5's `Delta percentile` of `−0.6106`
  median over cells;
- the **median shift** between two null distributions, sprint:20's `+6.3399` nats;
- the statement that **19 of 30 observed values sit at or below the first-order null's median** — a
  statement about the body of a distribution, which a maximum cannot see;
- the paired **two-nulls-on-one-axis** plot sprint:20 rendered, whose whole content is where the second
  distribution sits relative to the observation.

sprint:20's verdict — that the sprint:19 separation is fully explained by first-order transition
structure — rests on exactly these. **A FewRS assay could not have produced sprint:20.** Under a binary
maximum rule sprint:20 would have reported "0 of 30 certified" and stopped, with no way to say whether
the observations had become *slightly* ordinary or *thoroughly* ordinary, and no way to show the
monotone ordering across three nulls that was the round's result stated a second way.

That is the trade in one sentence: **FewRS buys 2.18x on the decision and gives up the description.**

### 10. The measurement no criterion reads, and why it decides the recommendation

Descriptive, added after the preregistered assay completed, read by **no verdict branch**, and reported
because §PHASE 9 makes the recommendation a separate output. The precedent is sprint:20 §6, which ran
and reported a null its own criteria disqualified.

The Monte Carlo tail `p_hat = (1 + exceedances)/(B + 1)` is a valid p-value under the null, so
`p_hat <= alpha` is a level-`alpha` test at **any** `B`. At `B = 99` that rule reduces to *exactly* the
FewRS rule — `p_hat <= 0.01` iff zero exceedances iff `observed > max(null)`. So the smallest budget at
which a strict-maximum rule is a valid level-0.01 test is **99**, not 459.

Rerunning the identical assay at `m = 99`:

| budget | rule | certified | primary agreement with the frozen grid | null searches |
|---|---|---|---|---|
| 99 | strict max | **22 of 30** | **27/30 = 0.9000** | 3 960 |
| 459 (FewRS) | strict max | 17 of 30 | 24/30 = 0.8000 | 18 360 |
| 999 (sprint:19) | `p_hat <= 0.01` | 23 of 30 | — (the reference) | 39 960 |

The `m = 99` certifications are a strict superset of the `m = 459` ones — all 17, plus
`8b68dece x 57f18ff9` at `k = 6` and `k = 8`, `8b68dece x 7d95c414` at `k = 12`,
`8b68dece x f5c18299` at `k = 6`, and `57f18ff9 x 7d95c414` at `k = 3` — which is the monotonicity of
§5 running the other way.

**`m = 459` is dominated per cell.** At the same nominal `alpha`, `m = 99` is 4.6x cheaper, certifies
more cells, and agrees with the frozen grid more closely. FewRS's larger budget is not buying per-cell
power; it is buying the pooled maximum that its family-wise guarantee requires, and §11 says why this
grid cannot form one.

**One honest complication in the other direction.** At `m = 99`, `8b68dece x f5c18299` at `k = 6`
certifies although sprint:19's 999-replicate tail rule called it ordinary at `p_hat = 0.015`. A
strict-maximum rule at a small budget is noisier in both directions, which is the same instability §5
recorded and is the price of resolving a tail with fewer samples. It is stated here rather than left in
the 0.9000 agreement figure where a reader would not see it.

### 11. Caveats, recorded rather than worked on

**The multiple-analysis caveat, and it is the central one.** FewRS's published guarantee is
`FWER <= alpha` over a family of analyses, and it follows from comparing every analysis against the
maximum statistic over **all analyses and all `m` resamples**. This round compares each cell against its
own maximum, per the commission. Each certification is therefore a valid **exact conditional test at
level `1/(m+1) = 1/460`** by exchangeability, and **no family-wise guarantee across the 30 cells is
claimed or earned.** With 30 cells at that per-cell level the expected number of false certifications
under a global null is about 0.065, which is a calculation and not a guarantee.

The pooled variant is not merely unimplemented, it is **unavailable on this grid**: `T_k` at different
`k` are R1 sums over different window lengths and are not on a common scale, and §PHASE 11 forbids
normalizing them into one. That is why the budget is over-bought here — 459 is the price of a
multiplicity guarantee that this grid's geometry cannot accept.

**RNG and sampler caveats, unchanged and unexamined.** The order null draws from the existing LCG
(`state * 6364136223846793005 + 1442695040888963407`, top 31 bits) with **modulo reduction** in
`next_below`, which is biased for bounds that do not divide the range; and sprint:20's doublet sampler
uses capped rejection on its arborescence nomination. Both were known before this round and both are
untouched by it. **FewRS validates neither.** A maximum-null rule is, if anything, *more* sensitive to
generator quality than a tail rule, since one atypical replicate decides the whole cell — but that is an
observation about exposure, not a measurement, and this round made none.

**The specimen envelope is unchanged.** Four observational recordings from two projects, 32 to 169
events. decision:8's limit applies here as everywhere: an envelope, not a distribution.

### 12. Recommendation: retire the idea for this workflow

Three reasons, in order of weight:

1. **Dominated per cell.** §10. At the same `alpha`, 99 replicates certify more cells for 4.6x less, and
   the rule at 99 is sprint:19's own rule. The FewRS budget is right for a procedure this workflow does
   not run.
2. **The guarantee that justifies 459 cannot be taken here.** §11. The pooled maximum requires
   commensurable statistics; `T_k` across the ladder are not commensurable and §PHASE 11 forbids making
   them so.
3. **The target quantity is already bounded.** D12: sprint:20 established that the order-null separation
   this assay certifies is fully explained by first-order transition structure — 0 of 30 under the exact
   doublet null. A 2.18x cheaper route to an order-null verdict is a cheaper route to a quantity this
   project has already stopped treating as evidence of motif structure. No observational conclusion above
   reads sprint:20's distributions; this is a statement about what the *recommendation* may be worth.

**The one place the idea might still belong, named rather than built.** FewRS's pooled maximum wants a
family of *commensurable* analyses. sprint:21's corpus-report calibration already has that shape — task:31
specifies that it compares an observed family's session count against the distribution of the null
corpus's **best** family, which is a pooled maximum over a family, family-wise by construction, on one
scale. Whether the FewRS budget helps *there* is a different question with a different grid, and it needs
its own round. It is recorded as a pointer and no measurement here supports it.

### 13. What this round does not establish

Not earned and not asserted: that FewRS is wrong, or that its guarantee fails — this round implements a
per-cell reduction of it, not the procedure, and measures nothing about the published version. That the
existing LCG, its modulo reduction, or the doublet sampler are sound; FewRS says nothing about any of
them. Family-wise error control across the 30 cells. That any certified span is a motif, that the search
is useful, or that a non-certification means observed structure "collapsed" into the null — a
non-certification here means one of 459 null searches reached the observation, and nothing more. No
discovered span was named, described or inspected, and no recording content appears in any artifact.

R1 remains a proposal. Nothing was adopted and nothing was promoted to production.

### 14. Gates, and one criterion defect caught by the suite

`scripts/check.sh` green and unweakened. **456 tests**, up from 434; 22 new in `tests/fewrs.rs`.
`scarp doctor` clean. Nothing pushed. Counts, scores and verdicts only, per decision:8.

**One criterion defect, this round's own, caught by the suite before it reached a report.**
`tests/fewrs.rs` first asserted the negative control does not certify at a 15-replicate budget, and it
failed at `k = 8`. The machinery was correct: 15 replicates cannot resolve a cell whose sprint:19
exceedance rate is `34/999 ~ 0.034`, where the chance no replicate reaches the observation is about 0.6.
The assertion was a coin toss dressed as a rule — the same unreachable-criterion class sprint:18 recorded
as its eleventh and `tests/calibration.rs` already carries a comment about. It was fixed by giving the
test a budget that resolves the rule (60, where the measured refuting count at `k = 8` is 1), **never**
by weakening the rule. The experiment's own `m = 459` was unaffected; the defect lived only in the test's
replicate count.

Nothing renders. No card was added to the evidence page: the round's output is a binary grid whose whole
finding is that a binary grid is the wrong instrument for this workflow, and drawing it beside sprint:19
and sprint:20's distribution plots would make the narrower instrument look like a peer of the wider ones.

Commits: preregistration `1c1330e`, alone and before any FewRS code existed; the experiment below.

### 15. Scarp desire paths

**idea:1 recurred, as it has every round.** This Result was written into the task file with a shell
append before `scarp close task:32`, because closing a task and recording its result are two writes and
only one of them is a Scarp command.

**One new piece of friction, and it is small.** `scarp new task --body-file` rejects a body whose
headings include `## Result`, naming the sections a task does have. The rejection is correct — Scarp owns
the template — and the error message is good enough that the fix took one edit. But the body file for a
preregistered round is written knowing a Result will follow, and the natural draft carries an empty
`## Result` heading as a placeholder. The affordance that would remove it is idea:2's: expose the
collection's section template before the first artifact exists, so a body file can be drafted against the
sections rather than against a rejection. Nothing new to add to the idea beyond that it now has a second
shape.

**idea:5 recurred, and this round is an unusually clean case for it.** §PHASE 10's prediction — 13 to 19
cells, most likely 15 to 17, agreement 0.73 to 0.83 — landed on 17 and 0.8000, and the only evidence that
it predated the run is that commit `1c1330e` contains the preregistration and no FewRS code. That works
inside the repository and is worth nothing outside it.

### 16. Correction, appended by maintenance:3 — 2026-08-11

**Nothing above is rewritten.** §7 of `CLAUDE.md` keeps previous conclusions where they were written, so
this section supersedes specific sentences rather than editing them, and maintenance:3 carries the full
account. **The result itself is unchanged and was re-verified, not recomputed:** controls PASS, 17 of 30
cells certified at `m = 459`, classification `STRONG`, agreement 24/30 and 26/30, 18 360 null searches.

**The statistical explanation in §10, §11, §12 and §PHASE 0 D11 was wrong in its reasoning, and the
conclusion it supported was right.** Those sections explain `m = 459` as the price of FewRS's family-wise
pooled maximum, and conclude the budget is over-bought *because* this grid cannot form one. Withdrawn,
and replaced by:

> `m = ceil(ln(1/alpha)/ln(1/(1-alpha)))` is the cost of FewRS's **particular high-probability
> upper-bound construction**. The formula reads `alpha` and nothing else, and **applies to a single
> analysis exactly as it does to a family** — so the budget is not caused by pooling and does not shrink
> if you stop pooling. What this round implemented is not FewRS's procedure but an **ordinary
> strict-maximum randomization test** per cell, whose guarantee comes from exchangeability alone: the
> probability the observation is the strict maximum of itself and `m` null statistics is at most
> `1/(m+1)`. For one exchangeable scalar statistic at `alpha = 0.01`, `m = 99` already gives at most
> `1/100 = 0.01`, and §10 measured it at 22 of 30 cells and 27 of 30 agreement against 459's 17 and 24.
> FewRS is operationally dominated for this narrow binary per-cell question.

Three boundaries the corrected reading does **not** cross. The 99-draw test is a per-cell test and
confers **no** family-wise control over this heterogeneous 30-cell grid; a pooled max-statistic test
would need a coherent null dataset, a family statistic on a commensurable or defensibly normalized
scale, and its own error-control contract, none of which this round built. And the paper's stronger
threshold guarantee should not be relied on operationally here without independent statistical review —
its assumptions were never checked against this pipeline.

**§12's nomination of sprint:21 is withdrawn.** It rested on the mistaken reasoning above — if 459 is not
the price of pooling, "find something that pools" is not a reason to keep FewRS — and it pointed at a
structure this round measured nothing about, read from task:31's acceptance criteria rather than
exercised. What survives is narrower and is a different investigation: sprint:21's calibration may be a
candidate for an *ordinary pooled max-statistic randomization test*, which is not FewRS. No sprint is
opened for it.

**Why `STRONG` and "retire" are compatible, stated plainly because §7 above left it implicit:**

> The preregistered experiment produced a STRONG result under its frozen success criterion, but FewRS
> was still retired because a simpler 99-draw scalar randomization test delivered at least the same
> certification count — 22 of 30 against 459's 17 — with substantially less computation. The
> classification and the adoption decision answer different questions.

**Two engineering defects in this round's own deliverables, both repaired by maintenance:3 and neither
touching a number above.** `--json` printed the human report to stdout and appended the document after
it, so the runbook's documented redirection produced a file no parser accepts; nothing committed was
malformed, because `.witnessglass/` is gitignored. And `classify` read only a control flag and a
certified count, so **any** invocation was scored against the frozen 15-of-30 threshold —
`--fewrs --replicates 99`, the diagnostic §10 rests on, printed `STRONG`. Both are now gated and tested;
§10's numbers were computed by hand from the run's own output at the time and are unaffected.
