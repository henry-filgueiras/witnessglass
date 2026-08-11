---
id: tsk_01KZSCQY1V194BXKA99T107KHA
sequence: 32
kind: task
status: pending
sprint: spr_01KZSCKFB7AFVM9XA9DA5HV6ZE
created: 2026-08-11
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
