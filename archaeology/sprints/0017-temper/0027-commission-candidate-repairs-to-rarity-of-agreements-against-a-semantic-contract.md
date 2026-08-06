---
id: tsk_01KZA8C47ZYK0V8ZC4GQPTH2D9
sequence: 27
kind: task
status: closed
sprint: spr_01KZA86CT59H3WXB1BZ33F61RG
created: 2026-08-05
closed: 2026-08-06
---

# Commission candidate repairs to rarity_of_agreements against a semantic contract

## Objective

State what `rarity_of_agreements` is supposed to *mean*, derive candidate repairs from that meaning,
and measure what the meaning costs — against every prior adversarial family and against sprint:16's
exact real candidate sets.

`### Preregistration` was written and committed **before any candidate was implemented or scored**.
What was computed first is §A: a reproduction of sprint:16's published measurements from the same
recordings, because using them as premises requires establishing that they still hold.

## Acceptance criteria

- Semantic contract fixed before any candidate is scored; each clause names its witnessing quantity.
- Candidates derived from the contract, each with an interpretation stated independent of any test.
- Feasibility derivation followed by an explicit propagation pass, recorded before trials.
- Every prior adversarial family run unchanged against the frozen original and every candidate.
- Candidates replayed against sprint:16's exact candidate sets, same machinery, same parameters.
- Selection by stated eligibility, not by pass count. Nothing adopted.
- `scripts/check.sh` passes unweakened; nothing pushed; no recording content committed.

### Preregistration

#### A. The premises, reproduced before use

Every sprint:16 number this round relies on was recomputed from the same four recordings before this
preregistration was written, and reproduced exactly:

| session | events `N` | vocab | `c_max` | max freq | singletons |
|---|---|---|---|---|---|
| `8b68dece` | 169 | 14 | 64 | 0.3787 | 5 |
| `57f18ff9` | 32 | 16 | 6 | 0.1875 | 7 |
| `f5c18299` | 33 | 17 | 6 | 0.1818 | 9 |
| `7d95c414` | 77 | 12 | 29 | 0.3766 | 4 |

- **Asymmetry.** `delta = 0` in **0 of 118** real candidate pairs; median 0.851, max 4.082 nats;
  designated pick changed in **3 of 29** candidate sets; **27 of 195** pairwise orders reversed.
- **Accumulation.** **13** observed crossings across the same 29 sets; largest margin **+4.128** nats,
  where 10 agreements outrank 11.

Also reproduced: all ten sprint:15 family verdicts, unchanged.

**One structural fact about the crossings, established here and load-bearing below.** `cross_pairs(a,
b, k, ·)` returns candidates of a *single* span length `L = k`. Agreement counts vary within a
candidate set; span length does not. Every observed crossing is therefore a comparison **at fixed
`L`**.

#### B. The semantic contract

What the statistic is for: *these two spans, one from each recording, agree more than chance would
produce.* Six clauses, each with the quantity that would witness its violation.

| | clause | witness |
|---|---|---|
| **C1** | **Exchange invariance.** `S(A,B) = S(B,A)`. The relation is a statement about an unordered pair; decision:8 §5 settled this on design grounds before any measurement. | `max │S(A,B) − S(B,A)│` over all pairs; must be 0 |
| **C2** | **Agreement monotonicity.** Converting one disagreeing position into an agreeing one, everything else fixed, must not lower the score. | sign of `S(after) − S(before)` |
| **C3** | **Rare agreement is more informative.** Agreement on a rarer mark scores strictly higher than agreement on a commoner one, all else fixed. | sign of `S(rare) − S(common)` |
| **C4** | **Disagreement neutrality.** Changing the marks at a disagreeing position, while it stays disagreeing, must not change the score — rarity that does not agree contributes nothing. | `│S(after) − S(before)│`; must be 0 |
| **C5** | **Proportional duplication invariance.** Scaling every count and both totals by `t` leaves the score unchanged. | `│S(scaled) − S(base)│`; must be 0 |
| **C6** | **Rarity is not motif-ness.** A candidate with zero agreements must never outscore one with agreements, at equal length. | sign of `S(k>0) − S(k=0)` |

**C2 and C3 are stated separately on purpose.** §D shows they conflict, and the conflict is this
round's central question, not an accident to be smoothed over.

#### C. Candidate repair families

Write `p̂(m) = (ĉ_A(m) + ĉ_B(m)) / (N_A + N_B)` — the maximum-likelihood frequency of mark `m` under
the hypothesis that both recordings are draws from one shared distribution. Pooling is not a taste
among {mean, min, max, geometric}; it is the estimator that hypothesis licenses, and it is the only
symmetric construction below that requires no free choice. Let `k` be the agreement count and `L` the
span length.

| | statistic | interpretation, stated independent of any test |
|---|---|---|
| **S0** | `Σ_{i: āᵢ=b̄ᵢ} −ln( ĉ_A(āᵢ)/N_A )` | frozen incumbent. Total surprisal of the agreeing marks, measured against recording **A**'s marginals alone. |
| **R1** | `Σ_{i: āᵢ=b̄ᵢ} −ln p̂(āᵢ)` | total surprisal of the agreeing marks under the shared-source model. Changes exactly one thing about S0: whose marginals the surprisal is measured against. |
| **R2** | `(1/k) Σ_{i: āᵢ=b̄ᵢ} −ln p̂(āᵢ)`, `0` when `k = 0` | surprisal of a *typical* agreeing position. Answers "how surprising is an agreement here", not "how much surprise in total". |
| **R3** | `(1/L) Σ_{i: āᵢ=b̄ᵢ} −ln p̂(āᵢ)`, `L ≥ 1` | surprisal per position *examined* — evidence per unit of opportunity, so a long span is not rewarded for having had more chances. |

Each is one minimal, separable change: R1 fixes whose marginals are read; R2 divides by opportunity
taken; R3 divides by opportunity offered. No candidate is included because it passes anything.

#### D. Feasibility derivation

**D1 — every candidate satisfies C1, C4, C5 by construction.** All three read `p̂`, which is symmetric
in `A` and `B` and invariant under scaling both sides by `t`; all three sum over agreeing positions
only, and `k` and `L` are order-symmetric.

**D2 — the crossing theorem.** Let `S = Σ_{agreeing} w(m)` with `w(m) > 0` a function of the mark
alone. If `w` is non-constant — which is exactly what C3 demands — then for every sufficiently long
`L` there exist candidates `X`, `Y` at that same `L` with `k_X < k_Y` and `S(X) > S(Y)`.

> *Proof.* Let `r` be the rarest mark and `c` the commonest, so `w(r) > w(c) > 0`. Take `X` with `k`
> agreements all on `r` and `Y` with `k+1` all on `c`, both padded to length `L` with disagreements.
> Then `S(X) − S(Y) = k·w(r) − (k+1)·w(c)`, which is positive for any `k > w(c)/(w(r) − w(c))`. ∎

So an accumulation crossing is **not a defect of `rarity_of_agreements`**. It is a necessary
consequence of weighting positions by rarity at all. Any statistic of this form exhibits it, and any
statistic that eliminates it has abandoned C3. C2 and C3 are jointly unsatisfiable in the strong form
"more agreements always wins".

The same argument covers R2: a mean of `w` over the agreeing positions ignores `k` entirely, so
`R2(X) = w(r) > w(c) = R2(Y)` regardless of `k`. **A mean crosses more readily, not less.**

**D3 — R3 cannot change any observed crossing.** At fixed `L`, `R3 = R1/L` is a positive constant
multiple of `R1`, so R1 and R3 induce the *same order* within any candidate set. §A established that
every observed crossing is a comparison at fixed `L`. Therefore R3 reproduces R1's crossings exactly —
same sets, same pairs, same count.

**D4 — R1 collapses onto S0 wherever the two recordings share marginals.** If `ĉ_B = ĉ_A` and
`N_B = N_A` then `p̂(m) = 2ĉ_A(m) / 2N_A = ĉ_A(m)/N_A`, so `R1 ≡ S0` **numerically**. Nine of the ten
sprint:15 families build `B` from `A`'s counts; only AG3b overrides them.

**D5 — AG3b contains a degenerate sweep point.** Its first point sets `count_B = 1`, at which
`ubiquitous_in_b` and `rare_in_both` are *the same case*, and the test is the strict `rare > ubiquitous`.
`X == Y` there, so **no statistic whatever can pass that point.** It is a property of the fixture, not
of anything under test. sprint:15's "first failing point `count_B=1`" for S0 is therefore uninformative
about S0. AG3b is **not modified** — the user's constraint forbids rewriting a family to accommodate a
candidate, and a preregistration that quietly repaired its own gauntlet would be worthless. It is run
unchanged and its verdict is read with this defect stated in advance.

#### E. Propagation pass — decision:7

Every mechanism §D found, against every criterion it could touch. Recorded **before any trial ran**.

| mechanism | criteria it touches | disposition |
|---|---|---|
| **D2** crossing theorem | any prediction that a candidate *removes* accumulation crossings | **Struck before trials.** No such prediction is made for any candidate. The outcome partition §H gains **O3**, which did not exist when this round was commissioned. |
| **D2** | the framing "two known defects, both to be repaired" inherited from the commissioning prompt | **Corrected before trials.** This round repairs *one* defect and *characterizes* the other as a contract conflict. Stated in the report, not discovered by it. |
| **D3** R3 ≡ R1 at fixed `L` | any prediction distinguishing R1 from R3 on the real envelope | **Struck.** Replaced by P2b, an exact-equality prediction, which is a sharper test than the vague one it replaces. |
| **D3** | the value of running R3 on the envelope at all | **Kept, as a falsification target.** If R3's crossings differ from R1's by even one, D3 is wrong and the derivation is unsound. |
| **D4** R1 ≡ S0 on shared marginals | every per-family prediction for R1 | **Corrected.** R1's verdicts are predicted *identical* to S0 on nine families; only AG3b can distinguish them. Without this pass, nine "R1 passes as well as S0" results would have read as evidence for R1 and are in fact evidence of nothing. |
| **D5** AG3b degenerate point | the AG3b verdict for every candidate, and sprint:15's recorded S0 verdict | **Corrected.** No candidate is predicted to pass AG3b outright. The reachable target is the three non-degenerate points. |
| **D1** C1/C4/C5 by construction | any claim that passing C1, C4 or C5 is *evidence for* a candidate | **Corrected.** These are checked as implementation verification — they can only reveal a coding error — and are excluded from eligibility in §I. |
| pooled `p̂` uses `ĉ_B` | AG6b, AG7, AG5, which vary background or sample size | **Kept, undetermined.** These vary `N` and vocabulary and may or may not keep `ĉ_B = ĉ_A`; §D4's collapse is predicted per family from the fixture, not assumed. |

#### F. Predictions

Fixed before implementation. Each names the computed quantity that decides it.

- **P1** — R1, R2, R3 each give `max │S(A,B) − S(B,A)│ = 0` over all 118 real pairs; pick changes
  `0 of 29`; pairwise reversals `0 of 195`. Exact, and by D1 a check on the code, not on the idea.
- **P2a** — R1 shows **strictly more than zero** accumulation crossings on the real envelope.
- **P2b** — R3's crossing count, sets, and pairs are **exactly equal** to R1's. Falsifies D3 if not.
- **P2c** — R2 also shows strictly more than zero crossings.
- **P3** — R1 is numerically identical to S0 on every family point where `ĉ_B = ĉ_A` and `N_B = N_A`,
  and the count of such points is reported.
- **P4** — on AG3b, R1 holds at `count_B ∈ {10, 100, 500}` and fails at `count_B = 1` for D5's reason.
- **P5** — R2 fails AG1 at **every** point: `R2(lone) = ln N` and `R2(motif) = ln(N/c) < ln N` for all
  `c > 1`, so the invariant `Y > X` can never hold.
- **P6** — R3's verdict equals R1's on every family whose two candidates have equal span length, and
  may differ only where they do not. The per-family length comparison is reported.
- **P7** — C2 is violated by R2 (adding an agreement below the running mean lowers it) and satisfied
  by R1 and R3 (a non-negative term is added to a fixed denominator).

#### G. Rejection criteria

A candidate is **rejected** if it: violates C1, C4, or C5 (implementation error); violates C6; loses a
family that S0 held, other than by a mechanism §D derived in advance; or can be justified only by the
tests it passes. **No candidate is rejected for exhibiting D2's crossings**, since the theorem shows
every rarity-weighted statistic does.

#### H. Outcome partition

Exactly one, by precedence, so the partition tiles:

1. **O5 Inconclusive** — the machinery yields no comparisons, or predictions cannot be evaluated.
2. **O3 Contract conflict** — two contract clauses are shown incompatible and no candidate satisfies
   all; the incompatibility is exhibited concretely. *(Predicted, on D2.)*
3. **O1 Repaired** — some candidate satisfies every clause and shows no crossing on the real envelope.
4. **O2 Partially repaired** — some candidate satisfies C1 exactly and every non-conflicting clause.
5. **O4 Regressed** — every candidate loses a property S0 had, with nothing gained.

**The predicted outcome is O3, with R1 partially repairing under O2's description.** Predicting the
outcome in advance is the point: if the run returns O1, the derivation in §D is wrong.

#### I. Eligibility for an adoption experiment

Not a pass count. A candidate is eligible only if it: satisfies C1 exactly on real pairs; satisfies
C2, C3, C6; loses no family S0 held except by a derived mechanism; and has an interpretation that
survives being stated without reference to any test. C1, C4, C5 confer no credit — D1 makes them
free. Multiple eligible candidates are all preserved; at most one is proposed for a separate adoption
*experiment*, and nothing is adopted in this round.

#### J. What this task will not do

No adoption; no change to the incumbent selector, the production statistic, the representation, the
metric, or the null. No modification of any prior family, fixture, or expectation — including AG3b,
whose defect is reported rather than fixed. No new search procedure. No threshold tuned on the corpus,
and no treatment of the corpus as ground truth. No recording content — prompts, commands, responses,
file contents, or absolute paths — in any artifact. Nothing pushed.

## Result

**Outcome: O2 — partially repaired. The predicted outcome was O3, and predicting it was a criterion
defect of exactly the kind decision:7 exists to prevent. §7 below is the most important part of this
result.**

Every premise reproduced before use, and the frozen statistic's row in every table below reproduces
sprint:16 exactly: 118 pairs, `delta = 0` in 0 of them, median 0.851, max 4.082 nats, 13 crossings,
pick moved in 3 of 29 sets, 27 of 195 pairwise orders reversed.

### 1. The semantic contract

`(free)` marks a clause §D1 derived to be free by construction, which §I therefore excludes from
conferring eligibility.

| candidate | C1 *(free)* | C2 | C3 | C4 *(free)* | C5 *(free)* | C6 |
|---|---|---|---|---|---|---|
| **S0** `rarity_of_agreements` | **NO** | ok | ok | ok | ok | ok |
| **R1** pooled sum | ok | ok | ok | ok | ok | ok |
| **R2** pooled mean | ok | **NO** | ok | ok | ok | ok |
| **R3** pooled density | ok | ok | ok | ok | ok | ok |

- S0 violates C1 at `max │S(A,B) − S(B,A)│ = 6.5101` nats. This is the commissioning defect.
- R2 violates C2 at `min ΔS = −3.1073` nats, witness *add a count-500 agreement to {count 1}*. Adding
  an agreement **lowers** the score, which is the price of dividing by the agreement count.

### 2. The crossing theorem, exhibited

§D2 predicted that *every* rarity-weighted statistic admits a candidate with fewer agreements
outscoring one with more. Constructed at rare `c=1` against common `c=500`, `N=100 000`:

| candidate | fewer `k` | more `k` | fewer | more | crosses |
|---|---|---|---|---|---|
| S0 | 1 | 2 | 11.513 | 10.597 | **yes** |
| R1 | 1 | 2 | 11.513 | 10.597 | **yes** |
| R2 | 1 | 2 | 11.513 | 5.298 | **yes** |
| R3 | 1 | 2 | 5.756 | 5.298 | **yes** |

No candidate escapes, including the mean built to bound accumulation. **Accumulation is not a defect
of `rarity_of_agreements`. It is a consequence of weighting positions by rarity at all**, and any
statistic that eliminates it has abandoned C3.

### 3. The ten sprint:15 families, constructions unchanged

| family | S0 | R1 | R2 | R3 |
|---|---|---|---|---|
| AG1 singleton vs motif | MIXED | MIXED | **FAIL** | MIXED |
| AG2 rarity explosion | MIXED | MIXED | **FAIL** | MIXED |
| AG3 rare disagreement | PASS | PASS | PASS | **FAIL** |
| AG3b one-sided rarity | FAIL | MIXED | MIXED | MIXED |
| AG4 common but structural | MIXED | MIXED | **FAIL** | MIXED |
| AG5 vocabulary growth | MIXED | MIXED | PASS | MIXED |
| AG6a whole-corpus duplication | PASS | PASS | PASS | PASS |
| AG6b background duplication | MIXED | MIXED | PASS | MIXED |
| AG7 sample-size stability | PASS | PASS | PASS | PASS |
| AG8 coincidence vs repetition | MIXED | MIXED | **FAIL** | MIXED |

**§D4 confirmed exactly.** S0 and R1 are numerically identical at **65 of 68** family points, and
differ *only* at AG3b's `count_B ∈ {10, 100, 500}` — the three points where the two recordings'
marginals differ at all. Without the propagation pass, nine families' worth of "R1 does as well as S0"
would have read as evidence for R1 and is in fact evidence of nothing.

**§D5 confirmed.** Every candidate's only AG3b failure is `count_B=1`, the degenerate point where the
family compares a case to itself under a strict inequality. **R1 holds every reachable AG3b point**,
which is the repair working. AG3b was not modified.

**§P5 confirmed.** R2 fails AG1 at its very first point, `N=100 c=10`, as derived.

### 4. The real operating envelope — sprint:16's exact candidate sets

| candidate | pairs | `delta = 0` | median | max | crossings | picks moved | orders reversed |
|---|---|---|---|---|---|---|---|
| **S0** | 118 | 0/118 | 0.851 | 4.082 | 13 | 3/29 | 27/195 |
| **R1** | 118 | **118/118** | 0.000 | 0.000 | 13 | **0/29** | **0/195** |
| **R2** | 118 | **118/118** | 0.000 | 0.000 | **25** | **0/29** | **0/195** |
| **R3** | 118 | **118/118** | 0.000 | 0.000 | 13 | **0/29** | **0/195** |

- **P1 confirmed.** All three candidates are exchange-invariant on every real pair. The asymmetry
  defect is fully repaired, and by the cheapest available change.
- **P2b confirmed, and §D3 survived its falsification target.** R1's and R3's crossings are identical
  set-for-set and pair-for-pair, 13 and 13, because every observed crossing is a comparison at fixed
  span length and `R3 = R1/L` there.
- **P2c confirmed, sharply.** R2 — the candidate built to bound accumulation — produces **25**
  crossings, nearly twice S0's 13. A mean crosses *more* readily, exactly as §D2 derived.

### 5. The undeviated loss: R3 fails AG3

§D derived nothing about this and it is the round's one surprise. AG3 compares a candidate carrying a
count-1 mark at a **disagreeing** position against one not carrying it at all — spans of length 3 and
2. R3 divides by span length, so a disagreeing position *dilutes* the score by 33%. The invariant AG3
states — *rarity that does not agree contributes nothing* — is violated.

**The contract did not catch this and the prior family did.** C4 as operationalized varied the
disagreeing marks' counts while holding their *number* fixed, so R3 passed it. AG3 varies the number.
A clause whose title says "contributes nothing" was implemented as the strictly weaker "changing its
marks changes nothing". Running the prior families unchanged is the only reason this surfaced, and it
is the concrete argument for §J's refusal to touch them.

By §G, R3 loses a family S0 held by a mechanism not derived in advance, and is **rejected**. Banked as
`tests/repair.rs::dividing_by_span_length_lets_a_disagreement_change_the_score`.

### 6. Eligibility — §I, not a pass count

| candidate | verdict | reason |
|---|---|---|
| **R1** pooled sum | **eligible** | C1 exact on all 118 real pairs; C2, C3, C6 hold; loses no family S0 held and repairs every reachable AG3b point; its interpretation — *surprisal under the shared-source model* — stands without reference to any test. |
| **R2** pooled mean | rejected | violates C2 at −3.107 nats, and loses AG1, AG2, AG4 and AG8 outright. Its gains on AG5 and AG6b do not enter, since §I is not a pass count. |
| **R3** pooled density | rejected | loses AG3, which S0 held, by an underived mechanism (§5). |

R2's and R3's results are preserved rather than discarded: both are informative about the shape of the
problem, and R2 in particular is the obvious naive repair whose failure is worth having on record.

**Exactly one candidate is eligible. R1 earns an ADOPTION EXPERIMENT, and is not adopted.** Nothing in
this round changed the incumbent selector, the production statistic, the representation, the metric,
or the null; `UNDER_TEST` and `UNDER_STUDY` are still `rarity_of_agreements`, and a test asserts it.

### 7. The ninth criterion defect, and it is this round's own

decision:7 banked eight criterion defects across sprints 6–15. **This round contributes a ninth, and
it is mine.**

§H predicted **O3 — contract conflict**, on the strength of §D2's theorem. The partition returns
**O2**. The precedence rule that decides it is `O3` *only if* "two clauses are incompatible **and no
candidate satisfies all**" — and R1 satisfies all six. The prediction assumed C2 meant *more
agreements always wins*, which is the form §D2's theorem is genuinely incompatible with C3. The C2 I
actually wrote and implemented is the weaker *one candidate gaining an agreement must not lose score*,
which R1 satisfies comfortably. §B even footnoted "C2 and C3 conflict" while defining a C2 that does
not conflict with C3 at all.

**A criterion that did not mean what the prediction assumed, in the first round conducted under the
decision written to stop exactly that.** Recorded here rather than resolved by adjusting either the
clause or the partition after the fact. What decision:7 bought is not immunity: it is that the defect
was caught by the round's own machinery, named against a preregistered rule, and reported in the same
breath as the result it distorted — instead of surfacing eight rounds later.

The substantive finding stands on its own and does not depend on which outcome label applies: **the
strong form of C2 and the whole of C3 are provably incompatible, and §2 exhibits it.**

### 8. Numbers

`scripts/check.sh` green, unweakened. **358 tests**, up from 343; 15 new, all in `tests/repair.rs`.
`scarp doctor`: no problems. Nothing pushed. No recording content in any artifact — counts,
frequencies and margins only, per decision:8.
