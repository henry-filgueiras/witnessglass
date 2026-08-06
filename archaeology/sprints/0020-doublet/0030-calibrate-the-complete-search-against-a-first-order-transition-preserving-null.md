---
id: tsk_01KZCDZ09SBN52C3MY0GG1ZAV0
sequence: 30
kind: task
status: closed
sprint: spr_01KZCCTGTQM5J9CXX46A4R53C1
created: 2026-08-06
closed: 2026-08-06
---

# Calibrate the complete search against a first-order transition-preserving null

## Objective

Rerun sprint:19's complete search-aware calibration, changing only the null, against a null that
preserves each recording's first-order categorical transition structure — and determine whether the
separation sprint:19 measured survives, collapses, or cannot be interpreted.

`### Preregistration` §PHASE 4 onward was written and committed **before any calibration code existed
and before any observed or null `T` was computed under any first-order null**. What was computed first
is §PHASE 0's premise reproduction and §PHASE 1–3's null-construction and adequacy measurements, at
commit `58cbf07`, which is what decides which construction may be called transition-preserving. That
ordering is stated here rather than assumed: the adequacy rule in §PHASE 2 **describes a decision the
measurements already determined**, and only §PHASE 4 onward is predictive.

## Acceptance criteria

- sprint:19's numbers reproduced from the repository before anything new is built; every discrepancy
  recorded, not worked around.
- The null specified exactly, with exact versus in-expectation preservation asserted by tests.
- Adequacy decided before any calibration, on sprint:19's own summaries plus transition fidelity.
- Controlled fixtures whose background is first-order and whose plant is not available from
  first-order counts; contamination measured.
- Identical `complete_search` on both paths; every data-dependent stage inside every replicate.
- A paired sprint:19-versus-sprint:20 table over the same specimen and `k` grid.
- Reachability checked on **every** branch, including the ones that cannot fail.
- Specimen-level results; verdict never forced over a disagreement.
- `scripts/check.sh` passes unweakened; nothing pushed; no recording content committed.

### Preregistration

#### PHASE 0 — sprint:19's premises, re-verified

Every premise re-verified from the repository, and sprint:19's published numbers **reproduced exactly**
by rerunning `--calibrate` over decision:8's four admitted specimens at `B = 999` in 3 min 16 s.

| premise | verdict |
|---|---|
| `T_k(A,B) = max over dedupe_overlapping(cross_pairs(A,B,k,MAX), 5) of R1(c)` | **confirmed** — `calibration::complete_search`, unchanged |
| every data-dependent stage reruns inside each replicate | **confirmed** — `calibrate` rebuilds both sides then calls `complete_search`; `null_evidence` is not called |
| the order null is `order_null_seeded`, seeds `null_seed(i, side)` | **confirmed** |
| `B = 999`, threshold `0.01`, ladder `{3,4,6,8,12}`, keep 5, top-k 5 | **confirmed** — constants, pinned by a test |
| controls: i.i.d. 12-symbol negative, same generator plus a planted 12-mark figure at three sites | **confirmed** |
| corpus: `8b68dece`, `57f18ff9`, `f5c18299`, `7d95c414`, six unordered pairs, five `k` | **confirmed** |
| selection effect: arbitrary median 1.6094, q99 7.3843; selected median 11.0187; margin +3.6344 | **reproduced to four decimals** |
| corpus separation: 23 of 30 at `p̂ ≤ 0.01` | **reproduced, cell for cell** |
| the three first-order summaries condemning the order null | **reproduced**; every observational specimen has immediate repetition rate 0.0000 and mean run length 1.0000, outside the null range wherever the null produces repeats at all |

**Four discrepancies, all recorded before any criterion below was written.**

**D3 — ordinary Markov resampling is not transition-preserving at these lengths, and the round does
not call it that.** The commissioning model was "fit a first-order chain, generate a replacement
sequence." Measured at commit `58cbf07` over 199 replicates: the fitted chain moves the
transition-frequency matrix by a total-variation distance of **0.0952 to 0.2903** on the four
observational specimens, **misses 4 to 7 observed transitions outright** in the median replicate,
moves the mark marginals by **0.0710 to 0.1875**, and drives at least one state's outgoing
distribution to a total-variation distance of 1.0 on three of the four. On the controls it lands
**outside its own null range on transition entropy ratio** for `fo-neg-a` (observed 0.7863 against a
null range of 0.7342–0.7844) — the very failure mode this round exists to repair. It is therefore
characterised as a **first-order-model null whose samples preserve transitions only in expectation**,
and it is not the primary null.

**D4 — the primary null's adequacy rule cannot fail, and that is disclosed rather than discovered
later.** The exact construction preserves immediate repetition rate, mean run length and transition
entropy ratio **by theorem**, so sprint:19's adequacy test returns "inside the null range" for it
whatever the data is — every replicate carries the observed value exactly. sprint:18's eleventh defect
was a reachability check applied to PASS rules only; this is the same class, caught in advance. The
rule is retained as an **implementation check** — a construction that failed it would be a broken
implementation, not an inadequate null — and the empirical adequacy question is live only for the
fitted chain, where it is the rule that disqualifies it.

**D5 — the exact null is partially degenerate on the largest specimen.** Over 199 replicates,
**74 of 199 (0.3719)** doublet replicates of `8b68dece` are the observed mark sequence itself, and only
126 distinct sequences are reached. `7d95c414` returns 2 of 199 (0.0101); `57f18ff9` and `f5c18299`
return none, at 187 and 169 distinct. Conditioning on every first-order transition count of a
169-event sequence over a 14-mark vocabulary leaves little freedom, and where it leaves none the null
distribution contains the observed pair. §PHASE 9 propagates this into an attainability rule and
§PHASE 10 into the verdict.

**D6 — the repository holds recordings decision:8 does not admit.** `cuecraft` now carries six
recordings where decision:8's inventory reflects three. **No specimen is added by this round**: the six
pairs are exactly decision:8's four admitted specimens, and admitting another would amend that decision
rather than appear in a Result.

#### PHASE 1 — the first-order null, specified exactly

Two constructions, both consuming **only categorical marks and sequence order**. Neither reads schema
semantics, tool categories, timing, or paths. Both leave gaps and offsets attached to positions exactly
as `order_null_seeded` does, and both drop receipts.

**The primary null — `doublet_null_seeded`, exact.**

| element | specification |
|---|---|
| state space | the distinct marks of *that* sequence, compared by `Mark` equality; no grouping, no category |
| transition-count estimator | **none.** The construction conditions on the observed adjacent-pair counts; nothing is estimated |
| initial-state distribution | degenerate at the observed first mark |
| transitions never observed | never generated — the replicate's support is the observed support, exactly |
| states with zero outgoing transitions | impossible except the terminal mark; the arborescence condition guarantees the trail completes |
| end of sequence | not modelled; the trail ends when the edges are used, which is at the observed length |
| generated length | exactly the observed length |
| RNG seeds | `null_seed(i, 0)` and `null_seed(i, 1)`, sprint:19's own scheme, unchanged |
| marginal frequencies | **preserved exactly** — the replicate is a permutation of the observed mark multiset |
| first-order transition counts | **preserved exactly**, every cell |

Construction: read the sequence as a walk on a multigraph whose vertices are marks and whose edges are
the observed adjacent pairs; draw an Eulerian trail of that multigraph starting at the observed first
mark. Every vertex but the terminal one nominates one outgoing edge to be used last, uniformly; the
nomination is accepted only when the nominations form a spanning arborescence rooted at the terminal
vertex, and redrawn otherwise; each vertex's remaining edges are shuffled and the nominated edge
appended; the trail is walked. This is the doublet-preserving shuffle of Altschul and Erickson as
corrected by Kandel and colleagues.

**Why this construction is primary, on grounds fixed before any `T` was computed.**

1. *It is an exact conditional test of the first-order hypothesis.* For a first-order Markov source the
   likelihood of a path is `prod p_ij^N_ij`, so the transition-count matrix together with the first
   mark is a sufficient statistic, and **conditional on it every path with those counts is equally
   likely**. A uniform draw over Eulerian trails is therefore a draw from the exact conditional
   distribution, with no fitted parameter and no appeal to asymptotics at a 32-event recording.
2. *It keeps sprint:19's cleanest property.* Because it is a permutation of the mark multiset, R1's
   pooled `p̂` is **identical** on the observed and null paths, exactly as under the order null. Only
   which marks land in agreeing positions moves. The fitted chain moves the marginals and therefore
   moves R1's own weights, confounding the comparison.
3. *It is measured, not asserted.* Every fidelity quantity is exactly zero, on every specimen, and a
   test asserts transition counts, marginals, length and both endpoints replicate for replicate.

**The secondary null — `markov_null_seeded`, in expectation.** Maximum-likelihood adjacent-pair counts,
no smoothing; initial state held at the observed first mark; unseen transitions never generated; a
dead end escaped by drawing from the empirical marginal, with each escape counted; exactly the observed
length; end of sequence not modelled. **Marginals and transition counts are preserved only in
expectation, and D3 measures how far from preserved that is.** It is run and reported as a reference,
and **no criterion and no verdict branch reads it.**

**The hypothesis the primary null represents, stated exactly:**

> This recording's marks are a path drawn uniformly from all paths having exactly its observed
> first-order transition counts and its observed first mark. Every first-order tendency — vocabulary,
> mark frequencies, immediate repetition, run structure, request→outcome-style alternation insofar as
> it is encoded in adjacent marks — is exactly as observed; no additional longer-range recurring
> structure is present.

#### PHASE 2 — the adequacy rule, and what it already decided

*Quantities:* for each of the eight sequences (four observational specimens, four control sequences),
each construction, and each of sprint:19's three summaries, over `R = 199` replicates: `observed`,
`null_min`, `null_max`, and `outside_null_range = observed < null_min || observed > null_max`. Plus
per-replicate `transition_tv`, `max_state_tv`, `absent_transitions`, `marginal_tv`, reported as medians.

*Rule:* a construction is **ADEQUATE** iff `outside_null_range` is false for every sequence and all
three summaries. *Primary-null rule, fixed before the calibration:* if both are adequate the exact
construction is primary on §PHASE 1's three grounds; if exactly one is adequate it is primary; **if
neither is adequate the round stops at verdict C** with a null-inadequacy result and no calibration is
interpreted.

*Already decided, at commit `58cbf07`:* the exact construction is adequate on all eight sequences with
every fidelity quantity exactly zero; the fitted chain is **not** adequate, failing on `fo-neg-a`'s
transition entropy ratio. **The exact construction is the primary null.** Per D4, the first half of
that is a theorem and the rule could not have rejected it.

#### PHASE 3 — what the primary null preserves and destroys

| property | order null (sprint:19) | first-order null (this round) |
|---|---|---|
| sequence length | preserved exactly | preserved exactly |
| mark vocabulary | preserved exactly | preserved exactly |
| marginal frequencies | preserved exactly | preserved exactly |
| candidate-boundary constraints (`window_count`) | preserved exactly | preserved exactly |
| session identity | preserved | preserved |
| timing skeleton — every gap at its position | preserved exactly | preserved exactly |
| first mark, last mark | destroyed | **preserved exactly** |
| immediate repetition tendencies | **destroyed** | **preserved exactly** |
| first-order transition counts, every cell | **destroyed** (driven to the marginal product) | **preserved exactly** |
| per-state outgoing distributions | **destroyed** | **preserved exactly** |
| run-length structure | **destroyed** | **preserved exactly** |
| request→outcome-style alternation, insofar as it is an adjacent-mark property | **destroyed** | **preserved exactly** |
| second-order transitions | destroyed | **destroyed** (preserved only as first order forces) |
| longer n-grams | destroyed | **destroyed** (preserved only as first order forces) |
| long-range recurrence | destroyed | **destroyed** |
| exact motif placement | destroyed | **destroyed** |
| the association between a mark and its own gap | destroyed | **destroyed** |
| record receipts | dropped | dropped |
| timing values themselves | preserved at positions | preserved at positions |

Nothing in the middle block is "preserved in expectation" for the primary null: each is a function of
the transition-count matrix and the mark multiset, both of which the construction holds fixed.

**The interpretation boundary, written before results.** If observed maxima separate from this null,
the narrow claim is: **first-order categorical transition structure is insufficient to explain what the
search finds.** It does **not** imply semantic workflow recurrence, causality, independence of the
repeated evidence, or a calibrated probability that any span is a motif. If they do not separate, the
claim is that first-order transition structure **suffices** to explain what the search finds on this
corpus — which does not make R1 wrong, only uninformative beyond first order here.

#### PHASE 4 — controlled fixtures

Both synthetic and obviously so; neither derives from any recording.

**The background chain.** Twelve synthetic marks; from state `i` the six successors `i+1, i+2, i+4,
i+5, i+7, i+9` mod 12, chosen with probability proportional to sprint:19's uneven weight vector at the
target. No self-loop, so the background's own immediate repetition rate is exactly zero. Initial state
0. Gaps `500 + U{0..4499}` ms, as sprint:19's controls. Lengths **160** and **90** and seeds
`0x000D0B1E00000001` and `0x000D0B1E00000002`, so the search space and its cost are sprint:19's.

**NEGATIVE control.** The two background walks, unplanted. This *is* the first-order null hypothesis,
so observed and null are draws from one law.
*Prediction:* `T_observed` ordinary — `p̂ > 0.01` at every `k`.
*Rule:* **PASS** iff no `k` yields `p̂ ≤ 0.01`. *Reachability:* attainable — an exact conditional null
exceeds 0.01 with probability ≈ 0.99 per `k`; refutable — five `k` at 1% leave ≈ 5% residual risk of a
spurious flag, stated now rather than discovered later.

**POSITIVE control.** The same two background walks with the 12-mark figure
`[3,4,8,9,1,5,6,10,11,0,7,2]` overwritten at **two** sites in each, sites chosen as the first position
at or after 20 and 100 (respectively 10 and 55) where the walk is already in the figure's first state
*and* returns to a legal successor of its last. **Every transition the figure contains, and both
boundary transitions, are therefore in the background's support**, asserted by a test: planting changes
transition *frequencies* and adds no transition the chain could not have produced. What it adds is a
specific 12-long path twice per sequence, which the transition matrix does not determine — reproducing
it needs eleven particular choices out of six.
*Prediction:* `T_observed` exceptional — `p̂ ≤ 0.01` at `k = 12`.
*Rule:* **PASS** iff `k = 12` yields `p̂ ≤ 0.01`. *Reachability:* `p̂ = 0.001` at 0 exceedances is
attainable at `B = 999`; refutable, since a plant the search cannot recover leaves exceedances high.

*Contamination, measured and reported rather than assumed small:* the transition-frequency and marginal
total-variation distance between the planted and unplanted walks, and the count of transitions outside
the background support (**zero by construction**).

*Fixture feasibility, checked on a quantity that is not `T`.* The longest run of marks the pair shares
is **14** for the planted pair against a doublet-null median of 6 and maximum of 9 over 199 replicates,
and **5** for the unplanted pair against a median of 6. The fixture plants cross-sequence structure the
exact null does not reproduce, and the unplanted fixture is ordinary. The chain's branching width was
chosen from this diagnostic before this preregistration and with no `T` computed at either setting —
recorded in `transition_null::SUCCESSORS` and in commit `58cbf07`.

**Bridge, descriptive only.** sprint:19's two i.i.d. controls are rerun unchanged under the primary
null. No rule reads them; they exist so the two rounds' control behaviour is comparable.

#### PHASE 5 — the calibration, unchanged in everything but the null

For every controlled and real specimen, at every `k` in `{3,4,6,8,12}`: run the unchanged observed
complete search; compute the same `T`; generate `B = 999` first-order-null replicates of **both**
sequences at seeds `null_seed(i,0)` and `null_seed(i,1)`; **rerun the complete search independently in
every replicate**; record `T_null`. Report `T_observed`, null median, quantiles at 0.05/0.25/0.50/0.75/
0.95/0.99, null max, `exceedances = count(T_null ≥ T_observed)`, and
`p̂ = (1 + exceedances)/(B + 1)` — a Monte Carlo null tail estimate under this null and this search, and
nothing else. `B = 999` is unchanged from sprint:19 and resolves `1/1000`, ten times finer than the
`0.01` threshold; the pass cost 3 min 16 s per null, measured.

sprint:19's order-null distribution is recomputed in the same run so the comparison is paired, and is
**never mixed into** the first-order null's distribution. `T_observed` is identical under both by
construction, and a test asserts one `complete_search` serves both paths.

#### PHASE 6 — top-k descriptive comparison

`k_top = 5`, unchanged. For each specimen, the observed sorted top-5 R1 order statistics against the
same order statistics from every first-order-null search, reported as one exceedance count per rank.
Reported as one of: **one isolated exceptional winner**, **several high-scoring candidates**, **broad
elevation**, or **absent/ambiguous**. **Descriptive only**; no verdict branch reads it, and ambiguity is
reported rather than resolved.

#### PHASE 7 — the direct test of sprint:19's own explanation

*The falsifiable prediction:* if sprint:19's separation was largely first-order grammar, replacing the
exchangeable null with the first-order null substantially reduces observed-versus-null separation.

*"Reduce" in exact computed quantities*, per specimen and `k`, over the identical 30-cell grid:

- `Δexceedances = exceedances_first_order − exceedances_order`;
- `p̂_order` and `p̂_first_order` side by side;
- `null_median_first_order − null_median_order` — the null's movement toward the observed value, which
  is what the paired plot draws;
- `percentile = count(T_null < T_observed) / B` under each null, and `Δpercentile`;
- `retention = |{cells with p̂ ≤ 0.01 under both}| / |{cells with p̂ ≤ 0.01 under the order null}|`,
  whose denominator is 23.

**No threshold is placed on any of these**, and none is invented after the fact: the A/B branch is
decided by the *same majority rule at the same threshold* sprint:19 used, so the two rounds are paired
by construction. These quantities describe the size and direction of the change.

#### PHASE 8 — known failure modes of a first-order null

Enumerated before execution; none is repaired in this round.

| failure mode | fitted chain | exact conditional null |
|---|---|---|
| finite-sample transition estimation | **exposed** — D3 measures it | **escaped**: it conditions on the counts and estimates nothing |
| zero / unseen transitions | **exposed** — 4 to 7 observed transitions missing per replicate | **escaped**: support is preserved exactly |
| nonstationarity, phase or regime change | exposed | **exposed** — pooled counts describe no regime, so a source whose transition structure changes mid-sequence would separate without any motif |
| deterministic adapter emissions (a request's own outcome) | largely handled | **handled exactly** — those doublets are held fixed, which is the whole repair |
| higher-order grammar that is not a motif | **exposed** — an order-2 regularity separates and is not a reusable figure | **exposed**, identically |
| dependence exceeding order 1 within repeated marks | exposed | exposed |
| corpus and session length effects | exposed | **exposed and measured** — D5: at 169 events over a 14-mark vocabulary the conditional distribution is 37% concentrated on the observed sequence |
| the mark-to-gap association, destroyed by both nulls | exposed | **exposed** — `align()` reads gaps, so separation is not attributable to mark identity alone |

*If results separate strongly*, the question "could a known first-order inadequacy explain this?" is
answered with domain-neutral diagnostics only: each separating specimen's degeneracy numbers, its
repeated 4-gram counts, and its longest shared run, observed against the same null. **No recording
content is inspected to rationalise any winner**, and no discovered span is named or described.

#### PHASE 9 — feasibility propagation pass

Every numerical rule, on **every** branch — PASS, FAIL, and verdict alike.

| mechanism | rules touched | disposition |
|---|---|---|
| **D3** the fitted chain is not transition-preserving | which null is primary; every claim about what is preserved | **Propagated before anything was preregistered.** The exact construction is primary; the fitted chain is reported and read by no rule; the round never calls it transition-preserving. |
| **D4** the adequacy rule cannot fail for the exact construction | §PHASE 2's rule | **Disclosed, and demoted.** It is an implementation check, not an empirical test. The live adequacy question is the fitted chain's, and it is the rule that disqualifies it. sprint:18's lesson applied to a rule that passes rather than to one that fails. |
| **D5** partial degeneracy on `8b68dece` | attainability of `p̂ ≤ 0.01`; the verdict count | **New rule, stated here.** A specimen is **null-degenerate** iff `identical_fraction(A) × identical_fraction(B) > 0.01`, since more than 1% of replicates would then be the observed pair itself and the threshold could not be reached whatever the data does. Degenerate specimens are reported and **excluded from the verdict count**. At `R = 199` the largest product is `0.3719 × 0.0101 = 0.0038`, so none is currently flagged; recomputed at `B = 999`. |
| D5, again, on power | the meaning of a non-separating cell | **Recorded, with its sign left open.** Freezing one side of the pair in a third of replicates changes the null distribution in a direction this round does not establish. A non-separating cell involving `8b68dece` is therefore weaker evidence of collapse than a non-separating cell elsewhere, and §PHASE 10's B branch says so. |
| the null preserves marginals exactly | R1's pooled `p̂` on null replicates | **Checked and kept, as in sprint:19.** `p̂` is a function of whole-recording counts, which a permutation preserves, so R1's per-mark weights are identical on both paths. Asserted by a test. |
| the null preserves length and `window_count` | comparability of candidate counts | **Kept.** Both paths search identically sized spaces; the counts are reported. |
| the null destroys mark↔gap association | `align()`'s timing term | **Recorded.** Unchanged from sprint:19, so the *paired* comparison isolates the mark process; the absolute claim under either null carries the timing perturbation. |
| the null preserves the first and last mark | `cross_pairs` admissibility; comparability with sprint:19 | **Checked.** Two extra positions are held fixed relative to the order null. At `k = 3` this is 1/32 of the shorter specimen and cannot be dismissed as negligible; it is a property of an exact conditional test and is reported in §PHASE 3 rather than corrected. |
| the plant shifts transition frequencies | the positive control's honesty | **Measured, not assumed.** Support contamination is zero by construction and asserted by a test; frequency contamination is reported. |
| the fixture's branching width was chosen from a diagnostic | the independence of the positive control | **Disclosed.** The diagnostic is the longest shared run, not `T`; the choice was made before this preregistration; both settings are in the commit history. |
| `B = 999` finite | every threshold | **Checked on both sides**, unchanged from sprint:19: `p̂ ≤ 0.01` needs ≤ 9 exceedances, minimum 0.001; `p̂ > 0.01` needs ≥ 10, maximum 1.000. Complementary, so no result falls between them. |
| negative control's rule is a **FAIL** condition | its reachability | **Checked.** "No `k` yields `p̂ ≤ 0.01`" is attainable under an exact conditional null and refutable at ≈ 5% across five `k`. |
| positive control's rule is a **PASS** condition | its reachability | **Checked**, and additionally checked for *fixture* feasibility on a non-`T` diagnostic, because a rule whose fixture cannot discriminate is unreachable in practice however reachable it is in arithmetic. |
| specimens may disagree | the corpus verdict | **Partition fixed below**, so disagreement is reported rather than averaged. |
| the round could produce every-cell separation, every-cell collapse, or a split | the partition's tiling | **Checked.** `p̂ ≤ 0.01` and `p̂ > 0.01` are complementary and exhaust each cell; §PHASE 10's three branches are ordered by precedence and cover every combination of control outcome, adequacy outcome and corpus outcome. |

#### PHASE 10 — verdict partition

By precedence, so it tiles:

1. **C — FIRST-ORDER NULL INADEQUATE.** Any of: the primary construction fails §PHASE 2's adequacy
   rule; either control fails its §PHASE 4 rule; the observed and null paths cannot be shown to compute
   the same `T`; `T` is undefined for a majority of the 30 cells; or a majority of cells are
   null-degenerate by §PHASE 9's rule. Checked first; nothing below applies if it fires. Recommend the
   narrowest improved-null experiment.
2. **A — SURVIVES THE FIRST-ORDER NULL.** Controls pass **and** at least half of the cells with defined,
   non-degenerate `T` have `p̂ ≤ 0.01` under the primary null. Earns exactly one claim: *the complete
   search detects sequence structure not explained by the exact first-order categorical null.* It does
   **not** establish motif semantics, workflow identity, causality, independence of repeated evidence,
   or a calibrated probability of motif identity.
3. **B — COLLAPSES UNDER THE FIRST-ORDER NULL.** Controls pass and fewer than half do. Interpretation:
   *sprint:19 was predominantly detecting first-order local grammar rather than longer-range recurring
   structure.* R1 may remain descriptive; this calibration then supplies little evidence for
   motif-like structure beyond first-order behaviour. Cells involving a partially degenerate specimen
   are the weakest evidence in this branch and are named as such.

**Predicted: B, with `57f18ff9 × f5c18299` the exception at `k ≥ 4`.** The prediction is made on the
longest-shared-run diagnostic, which is not `T`: of the six real pairs, only that one places its
observed shared run outside the doublet null's entire range (26 against a null range of 5–18 at
`R = 199`), while the other five sit at or below their null medians. If that prediction is wrong in
either direction the round says so; it is recorded here so it cannot be adjusted afterwards.

Specimen-level results are reported in all three branches. **R1 is not adopted in this round whatever
the outcome.**

#### PHASE 11 — what this task will not do

No change to R1, the complete search, candidate generation, ranking, deduplication, boundary
constraints, top-k reporting, the ladder, the representation, or real-corpus hygiene. No second-order
model, richer marks, semantic categories, timing features or paths. No new statistic, no second
detector, no aggregate invented to tidy §PHASE 6. No adoption and no promotion to production. No
threshold chosen after seeing data. No interpretation, naming or inspection of any discovered span. No
specimen added to decision:8's inventory. No treatment of an observational recording as ground truth.
No recording content in any artifact. Nothing pushed.

## Result

**Verdict: B — COLLAPSES UNDER THE FIRST-ORDER NULL**, by the preregistered partition. Both controls
passed and **0 of 30** eligible cells have `p̂ ≤ 0.01` under the exact first-order null, against **23 of
30** under sprint:19's order null. **Retention 0/23 = 0.0000.** The prediction recorded in §PHASE 10 was
B; the one exception it named did not survive either. R1 was not changed, not adopted, not promoted.

### 1. Verified premises

sprint:19's published numbers reproduced cell for cell in 3 min 16 s — the selection effect to four
decimals, the 23-of-30 separation table, and all twelve null-adequacy rows. Four discrepancies were
recorded before any criterion was written (§PHASE 0): **D3** the fitted chain is not
transition-preserving at these lengths; **D4** the primary null's adequacy rule cannot fail and is
demoted to an implementation check; **D5** the exact null is 37% concentrated on the observed sequence
for the largest specimen; **D6** the repository holds recordings decision:8 does not admit, and none
was used.

**A fifth, D7, found during this round and recorded here rather than in a later one.** §PHASE 6's
per-rank exceedance counter — sprint:19's, unchanged — returns zero for a rank the observed or null
kept set does not have. `dedupe_overlapping` requires survivors to be at least `k` apart on **both**
sides, so the survivor count is capped at `⌊(n−k)/k⌋+1` on the shorter sequence: **2 at `k = 12` for
the 32- and 33-event specimens, 4 at `k = 8`.** Ranks 3 to 5 are therefore *geometrically unavailable*
there, and the zeros §PHASE 6 reports at those ranks are absences rather than exceptional agreements.
§PHASE 6 is interpretable at rank 1 — which is `T` — and, on much of the grid, nowhere else. No verdict
branch in either round reads it, and neither round's conclusions move.

### 2. The first-order null, as built

`doublet_null_seeded`: an Eulerian trail of the sequence's own adjacency multigraph, drawn from the
observed first mark. **Exactly preserved, asserted by tests replicate for replicate:** every
first-order transition count, every per-state outgoing distribution, the mark multiset, the length,
the first mark and the last. Transition TV, per-state TV, absent transitions and marginal TV are
**0.0000 on every specimen at B = 999**. Immediate repetition rate, mean run length and transition
entropy ratio are preserved identically, because each is a function of the counts it holds fixed.

It is also an **exact conditional test**: transition counts and the first mark are sufficient
statistics of a first-order chain, so conditional on them every path with those counts is equally
likely, and no parameter is estimated at a 32-event recording. And it is a permutation, so **R1's
pooled weights are identical on observed and null paths**, exactly as in sprint:19.

### 3. Exact versus in-expectation, measured

| construction | transition TV | absent transitions | marginal TV | adequate |
|---|---|---|---|---|
| order (sprint:19) | 0.6053 – 0.8750 | 11 – 19 | 0.0000 | **no** — outside range on all three summaries |
| **doublet (primary)** | **0.0000** | **0** | **0.0000** | **yes**, by theorem |
| markov (fitted) | 0.0952 – 0.2903 | 4 – 7 | 0.0710 – 0.1875 | **no** — outside its own null range on `fo-neg-a`'s transition entropy |

Medians over 199 replicates, on the four observational specimens. "Preserved in expectation" is not
preserved, and this is what says so.

### 4. Controlled fixtures

Contamination of the plant, measured against the same background walks: transition TV **0.1321** and
**0.2472**, marginal TV **0.0500** and **0.1111**, and **no transition outside the background chain's
support** — asserted by a test, not claimed.

| control | k=3 | k=4 | k=6 | k=8 | k=12 | rule |
|---|---|---|---|---|---|---|
| **first-order negative** `p̂` | 0.238 | 0.612 | 0.557 | 0.356 | 0.800 | **PASS** — no `k` exceptional |
| **first-order positive** `p̂` | 0.814 | 0.424 | 0.048 | **0.002** | **0.001** | **PASS** — exceptional at the planted length |

At `k = 12` the positive control's `T = 31.6793` against a null maximum of `26.0202`: **no first-order
null search in 999 reached it.** The machinery recovers a long-range figure whose every transition the
null already reproduces, and does not flag a specimen drawn from the null.

**The bridge.** sprint:19's i.i.d. controls under this null behave as they did under theirs: negative
ordinary at every `k` (largest 0.034), positive exceptional at `k = 8` (0.002) and `k = 12` (0.001).

**One measurement worth keeping.** The first-order *negative* control — a specimen drawn from a chain,
containing nothing — scores `p̂ = 0.015` at `k = 8` against sprint:19's order null and `0.356` against
the null that matches its own law. No `k` crosses the threshold, so it is a tendency and not a flag,
but it is sprint:19's confound reproduced in a fixture with no planted structure at all.

### 5. The corpus, paired

30 cells, `B = 999`, the same `T` in both columns by construction. **23 separate under the order null;
0 under the first-order null.**

| pair | k | order `p̂` | 1st-order `p̂` | median shift |
|---|---|---|---|---|
| `8b68dece × 57f18ff9` | 3 / 4 / 6 / 8 / 12 | 0.157 / **0.004** / **0.005** / **0.004** / **0.007** | 1.000 / 0.744 / 0.773 / 0.764 / 0.675 | +3.46 … +8.29 |
| `8b68dece × f5c18299` | 3 / 4 / 6 / 8 / 12 | **0.004** / **0.003** / 0.015 / **0.002** / 0.075 | 0.742 / 0.815 / 0.804 / 0.675 / 0.905 | +3.88 … +7.99 |
| `8b68dece × 7d95c414` | 3 / 4 / 6 / 8 / 12 | 0.133 / 0.031 / 0.027 / **0.003** / **0.002** | 0.182 / 0.263 / 0.493 / 0.570 / 0.477 | −2.43 … +6.83 |
| `57f18ff9 × f5c18299` | 3 / 4 / 6 / 8 / 12 | 0.035 / **0.001** / **0.001** / **0.001** / **0.001** | 0.986 / 0.929 / 0.633 / **0.045** / 0.060 | +3.48 … +14.66 |
| `57f18ff9 × 7d95c414` | 3 / 4 / 6 / 8 / 12 | **0.002** / **0.001** / **0.001** / **0.001** / **0.001** | 0.452 / 0.578 / 0.839 / 0.611 / 0.468 | +2.46 … +9.92 |
| `f5c18299 × 7d95c414` | 3 / 4 / 6 / 8 / 12 | **0.001** ×5 | 0.442 / 0.376 / 0.624 / 0.540 / 0.430 | +4.94 … +11.06 |

**The preregistered change quantities.** Retention `0/23 = 0.0000`. Median over cells of
(first-order null median − order null median) = **+6.3399 nats**, range −2.4314 to +14.6604. Median
over cells of (first-order percentile − order percentile) = **−0.6106**, range −0.9519 to −0.0440.
**Zero of 30 observed values sit at or below the order null's median; 19 of 30 sit at or below the
first-order null's.** The lowest first-order tail on the corpus is `0.045`, four and a half times the
threshold; the highest is `1.000`, where every one of 999 replicates matched or beat the observation.

**No cell was excluded.** Degeneracy at `B = 999`: `8b68dece` 352 identical of 999 (0.3524, 648
distinct), `7d95c414` 0.0100, `57f18ff9` 0.0030, `f5c18299` 0.0010. The largest pairwise product is
`0.3524 × 0.0100 = 0.0035`, below the 0.01 attainability rule, so all 30 cells are eligible and the
verdict counts all of them.

### 6. The reference null nobody's criteria read

The fitted-chain null was run and is reported because §PHASE 1 promised it. **1 of 30 cells separates**
under it. More usefully, its tail lies **between** the order null's and the exact null's in **30 of 30
cells, without exception**: the more first-order structure a null preserves, the less this corpus
separates from it, monotonically, cell by cell. That ordering is the round's result stated a second way,
and it came from a construction disqualified for inadequacy — so it is reported as a description, and
no criterion reads it.

### 7. Top-k descriptive comparison

**Absent / ambiguous**, and D7 explains why: 16 of 30 cells carry at least one rank clearing 0.01, but
the zeros concentrate at ranks the deduplication cannot supply. Reported as ambiguity rather than
resolved, exactly as §PHASE 6 preregistered, and read by no verdict branch.

### 8. Specimen-level verdicts

Every pair collapses. `57f18ff9 × f5c18299` — decision:8's known runbook siblings, and the pair whose
longest shared run is the only one outside its null's entire range — retains the most, reaching 0.045
at `k = 8` and 0.060 at `k = 12` without crossing the threshold. `8b68dece × 57f18ff9` at `k = 3` moves
furthest, from 0.157 to 1.000. No specimen separates anywhere, so the corpus is not forced to one
verdict over a disagreement: there is no disagreement.

`8b68dece`'s 35% degeneracy makes its ten cells the weakest evidence in this branch, as §PHASE 9 said
in advance. The other 20 cells collapse without that caveat.

### 9. The narrowest supported claim

> **On this corpus, the separation sprint:19 measured is fully explained by first-order categorical
> transition structure.** Holding each recording's transition counts, mark marginals, length and
> endpoints exactly fixed, and destroying only longer-range reuse, the complete search's maxima become
> ordinary in all 30 specimen–span cells, from 23 exceptional. The controls establish that the
> procedure still recovers a planted long-range figure under this null, and does not flag a specimen
> drawn from it.

### 10. Explicitly unsupported

R1 is **not** shown to be wrong, and this round measures no error of it. What is not earned: that any
span is a motif; that no longer-range structure exists in agent recordings; that a larger or different
corpus would collapse the same way; that the search is useless. A negative result on four recordings
from two projects is an envelope, not a distribution. sprint:19's own claim is **not retracted** — it
said the search detects structure not explained by exchangeable ordering, and that remains true; this
round establishes that the structure in question is first-order.

Also not earned, and now carrying a measurement: **absence of separation is not absence of
structure.** Both nulls destroy the mark-to-gap association, and `align()` reads gaps; and the exact
null is partly degenerate on the largest specimen. Neither is repaired here.

### 11. One next experiment

Not another null. The corpus is the binding constraint: four recordings, two projects, 32 to 169
events, and a vocabulary-to-length ratio at which conditioning on first-order counts leaves a 169-event
sequence with 648 distinct alternatives and 352 copies of itself. Before any further calibration,
**measure what recording length and vocabulary size a first-order-null calibration can resolve at all**
— a power study on synthetic first-order backgrounds with planted figures, sweeping length and
vocabulary, reporting the smallest planted figure the procedure recovers at each. That is a controlled
question, needs no new specimen, and would say whether this corpus could have produced an A verdict
under any circumstances.

### 12. Gates

`scripts/check.sh` green and unweakened. **417 tests**, up from 393; 24 new in
`tests/transition_null.rs`. `scarp doctor` clean. Nothing pushed. Counts, scores, ranks, quantiles,
lengths and exceedances only, per decision:8 — no discovered span was named, described or inspected,
and no recording content appears in any artifact.

Two cards render on the existing evidence page. The construction card sits **above** the calibration
card deliberately: a reader who sees the new tails without the fidelity measurement has no way to tell
why the null changed. The calibration card draws both nulls for one specimen on **one shared axis**
with the observed `T` marked, because the round's whole question is whether the second distribution
moves onto the observation — and on all six pairs it visibly does.

Commits: preregistration `d2dc21d`, alone and before any calibration code existed; construction and
measurement `58cbf07` before it; the calibration and rendering below.

### 13. Scarp desire paths

**idea:1 recurred, as it has every round.** This Result was written to the task file with a shell
append before `scarp close task:30`, because closing and recording a result are two writes and only one
of them is a Scarp command. Nothing new to add to the idea.

**idea:5 recurred, and this round is the strongest case for it so far.** The whole evidential weight of
§PHASE 4 onward rests on those criteria having been fixed before any `T` was computed under any
first-order null, and the only mechanism available to demonstrate that is commit ordering: `58cbf07`
for the constructions and their measurements, `d2dc21d` for the preregistration alone, the calibration
after. That works, and a reader who does not have the repository has to take the ordering on trust.
A sealed section would carry the claim in the artifact rather than in the history around it.

The workflow attempted was ordinary — measure, preregister, execute — and the friction is that Scarp
records *what* was written but not *when it stopped being writable*. No workaround beyond the commit
discipline was used, and none is needed for this repository; the affordance would matter to a reader
outside it.
