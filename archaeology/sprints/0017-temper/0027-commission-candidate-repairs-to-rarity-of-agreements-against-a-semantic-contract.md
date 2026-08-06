---
id: tsk_01KZA8C47ZYK0V8ZC4GQPTH2D9
sequence: 27
kind: task
status: pending
sprint: spr_01KZA86CT59H3WXB1BZ33F61RG
created: 2026-08-05
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
