---
id: tsk_01KZC40AJFCRAXA1M14THM6YYZ
sequence: 28
kind: task
status: pending
sprint: spr_01KZC3WV6DKR99RKFDMMQVT0RN
created: 2026-08-06
---

# Commission R1 pooled sum against a gauntlet that can distinguish it from S0

## Objective

Settle whether R1 `pooled sum` has a coherent evidential interpretation, and whether that
interpretation survives constructions specifically capable of distinguishing pooled symmetric rarity
from one-sided rarity.

`### Preregistration` — the adjudication, the derivation, every construction, every analytic
prediction, and the propagation pass — was written and committed **before any fixture was implemented
or any candidate scored**.

## Acceptance criteria

- Crossing theorem adjudicated by proof; the criterion defect it exposes recorded, not smoothed.
- R1 frozen exactly, defined completely, and derived or honestly declared heuristic.
- Every fresh family preregistered with construction, computed quantity, analytic prediction,
  semantic expectation, and a PASS / FAIL / LIMITATION rule stated in the vocabulary of the quantity.
- Propagation pass recorded before execution; any criterion it disproves repaired first, with the
  correction visible.
- Old gauntlets unchanged and green as regressions; R1 not tuned after results.
- Verdict A/B/C on stated grounds. Nothing adopted.
- `scripts/check.sh` passes unweakened; nothing pushed; no recording content committed.

### Preregistration

#### PHASE 1 — Adjudicating the crossing theorem

**The two requirements, formally.** Let `S = Σ over agreeing positions of w(m)`, `w > 0` a function of
the mark alone — the form of S0, R1, and every statistic this project has considered.

- **(A) Rarity informativeness.** `w` is non-constant, and rarer marks carry strictly greater weight.
- **(B) Agreement dominance.** For all agreeing multisets `X`, `Y` with `|X| < |Y|`, `S(X) < S(Y)`.

**Claim. A and B are mutually exclusive, not merely in tension.**

> *Proof.* Let `r` be any mark and `c` any other. Apply B to `X` = `k` copies of `r` and `Y` = `k+1`
> copies of `c`: it requires `k·w(r) < (k+1)·w(c)`, i.e. `w(r)/w(c) < (k+1)/k`, **for every** `k`.
> Letting `k → ∞` gives `w(r) ≤ w(c)`. Exchanging the roles of `r` and `c` gives `w(c) ≤ w(r)`. So
> `w(r) = w(c)` for every pair of marks: `w` is constant, and `S` counts agreements while ignoring
> rarity entirely. B therefore holds only for the statistic A forbids. ∎

**Answering the four questions.**

1. **Yes, and it is forced.** If rare agreements carry more evidence than common ones, then
   sufficiently many rare agreements outscoring more common ones is not an artefact to be tolerated —
   it is the content of the claim. Denying it denies A.
2. **Not without an explicit bound.** B is recoverable on a *declared* maximum span `K` if and only if
   `w_max/w_min < 1 + 1/K`. At the `K = 12` this project's frozen ladder reaches, that admits a weight
   ratio below `1.0833` — a frequency ratio below `1.78×` at `N = 1000`. Rarity weighting purchased
   at that price is indistinguishable from none. **This is the tradeoff, quantified.**
3. **The intended semantics require A and must abandon B.** The relation being scored is *these two
   spans agree more than chance would produce*. Agreement on a mark occurring once in a thousand is
   stronger evidence of shared structure than four agreements on marks occurring five hundred times in
   a thousand, which chance produces continuously. B is an intuition about motif *size*, not about
   evidence, and this project scores evidence.
4. **What R1 ranks.** Not amount of agreement, not average quality of agreement, and not a calibrated
   probability. **Total self-information, under a pooled i.i.d. model, of the marks the two spans
   agree on.** §PHASE 2 derives this and bounds what it licenses.

**The criterion defect this exposes — the tenth.** Four of the ten sprint:15 families preregister
requirement B verbatim:

| family | preregistered invariant | requirement |
|---|---|---|
| AG1 | "a substantially stronger repeated motif beats a lone accidental agreement" | **B** |
| AG2 | "one weak agreement must not dominate without bound" | **B** |
| AG4 | "a repeated figure of common marks stays recoverable; rarity is not motif-ness" | **B** |
| AG8 | "four independent moderate agreements outweigh one spectacular coincidence" | **B** |

These four demand of a rarity-weighted statistic something the claim above proves no rarity-weighted
statistic can supply. **Their MIXED and FAIL verdicts are not evidence of a defect in S0 or R1.** They
are evidence that the family asks for B. Two rounds read them the other way, and sprint:16's
accumulation classification — **L1 observed/reachable**, "warrants a repair round before adoption is
discussable" — was built on that reading.

**Premises corrected before preregistration, per decision:7:**

- sprint:16's **L1 accumulation classification is withdrawn as a defect finding.** Its measurements
  stand unaltered — 13 crossings, largest margin +4.128 nats — and are re-read as a measurement of how
  often the corpus exercises requirement A, not of how often a statistic fails.
- **AG1, AG2, AG4 and AG8 are demoted to regression fixtures.** They are run unchanged and their
  verdicts recorded, but no verdict of theirs counts for or against any statistic in this round. They
  are not modified: task:27 §J's reasoning holds, and a family edited to fit a conclusion is worthless.
- Any criterion below that would have read "more agreements must win" is struck in advance. **F7 is
  the family this governs**, and it is written in §PHASE 3 against the analytic ordering instead.

#### PHASE 2 — Deriving R1

**The frozen definition.** Exactly as `src/experiment/repair.rs` implements it, unchanged in this
round:

```text
p̂(m)  =  ( ĉ_A(m) + ĉ_B(m) ) / ( N_A + N_B )
R1     =  Σ over positions i where āᵢ = b̄ᵢ  of  −ln p̂(āᵢ)
```

- **What enters the pool.** Whole-recording mark counts for both recordings and both recording lengths
  in the projected scope. Not span-local counts.
- **An agreeing position.** Index `i` with `āᵢ = b̄ᵢ`, marks compared for equality only, over spans of
  equal length. Unequal lengths yield `None`: a positional question needs a positional correspondence.
- **Zero and impossible counts.** A count absent from either map is read as 1, and every count is
  floored at 1, so `p̂ > 0` always and `−ln p̂` is finite. A mark present at an agreeing position always
  occurs in both recordings, so the floor is unreachable there; it exists to make the function total.
- **Candidate length.** Enters **only** through the number of agreeing terms `k`. `L` appears nowhere
  else — that is precisely what distinguishes R1 from the rejected R3.

**The derivation.** Two models over an aligned position:

```text
M0  independence : both marks drawn i.i.d. from p̂       P(agree on m) = p̂(m)²
M1(λ) copy-mixture: with probability λ the position is copied, else independent
                     P(agree on m)   = λ p̂(m) + (1−λ) p̂(m)²
                     P(a ≠ b)        = (1−λ) p̂(a) p̂(b)
```

The log-likelihood ratio of `M1(λ)` against `M0` over a span is

```text
LLR(λ)  =  Σ over agreeing of ln( λ/p̂(m) + 1−λ )   +   (L − k)·ln(1−λ)
```

As `λ → 1` the agreeing term tends to `−ln p̂(m)` and the disagreeing term tends to `−∞`.

> **R1 is exactly the `λ → 1` limit of the agreement side of `LLR(λ)`, with the disagreement penalty
> discarded.** Equivalently and exactly: `R1 = −ln ∏ over agreeing of p̂(m)` — the self-information,
> under pooled i.i.d., of the marks the two spans agree on.

**R1 is therefore the numerator half of a likelihood ratio, not a likelihood ratio.** Discarding
`(L−k)·ln(1−λ)` is what makes R1 satisfy "rarity that does not agree contributes nothing"; the same
discard is why R1 cannot distinguish three agreements in three positions from three in fifty.

**Assumptions, and where real event sequences violate them.**

| assumption | violation in this domain |
|---|---|
| Positions within a span are i.i.d. | Event sequences are strongly locally dependent: a tool use is followed by its own result, edit–test–edit recurs. Adjacent marks are near-deterministic, so `k` agreeing positions are worth far fewer than `k` independent observations. **F8 exhibits this.** |
| Marks are drawn from one shared distribution `p̂` | That is the hypothesis under test. The null is estimated from both recordings, so the estimate absorbs part of whatever shared structure exists. |
| `p̂` is known rather than estimated | With `N_A + N_B ≈ 200` and vocabularies of 12–17, a singleton mark's `p̂ ≈ 0.01` carries relative variance of order 1. No estimation-error correction exists. |
| Positions are aligned independently of the marks | **False by construction.** `cross_pairs` selects spans *because* they agree. Every score is computed on a maximum found by search. |

**The non-claim, stated in advance.** "R1 is self-information under model M0/M1" is supportable and is
what §PHASE 3 tests. "R1 is calibrated statistical evidence that two spans share a motif" is **not**
supportable, and this round will not assert it whatever the gauntlet returns. The selection effect
alone voids it, independently of every other assumption.

#### PHASE 3 — The fresh discriminating gauntlet

Nine families. Constructions where A and B share marginals appear only as **F0**, a control. Every
prediction below was computed analytically before any fixture existed. Scores in nats.

**F0 — shared-marginal control.** `ĉ_B = ĉ_A`, `N_B = N_A`.
*Quantity:* `S0 − R1`. *Prediction:* exactly `0` at every point. *Rule:* **PASS** iff `max|S0 − R1| = 0`.
Establishes that the fresh harness reproduces sprint:17 §D4 and that later differences come from the
marginals and not the harness.

**F1 — argument reversal.** Asymmetric marginals, spans fixed.
*Quantity:* `max |S(A,B) − S(B,A)|`. *Prediction:* R1 exactly `0`; S0 equal to
`|Σ over agreeing of ln( (ĉ_B/N_B)/(ĉ_A/N_A) )|`, non-zero.
*Rule:* **PASS** iff R1's maximum is `0` **and** S0's matches the closed form to `1e-12`. Invariance
check only; §PHASE 4 M-A records that it is weak evidence on its own.

**F2 — same local agreement, different A marginal.** `ĉ_A(m) ∈ {10, 100, 500}`, `N_A = N_B = 1000`,
`ĉ_B(m) = 10` fixed.
*Quantity:* `ΔS0` and `ΔR1` against the `ĉ_A = 10` baseline.
*Prediction:* `ΔS0 = −ln(ĉ_A′/10)`; `ΔR1 = −ln((ĉ_A′+10)/20)`. Both strictly decreasing, `|ΔR1| < |ΔS0|`.
*Semantic expectation:* a mark commoner in A makes agreement easier, so evidence must fall. Both do;
R1 is damped because A supplies only half the pooled estimate.
*Rule:* **PASS** iff both match their closed forms to `1e-12` and both are strictly decreasing.

**F3 — same local agreement, different B marginal.** `ĉ_B(m) ∈ {10, 100, 500}`, `ĉ_A(m) = 10`,
`N_A = N_B = 1000` **held equal** (see §PHASE 4 M3).
*Quantity:* `ΔS0`, `ΔR1`.
*Prediction:* `S0 = 4.6052` at all three points, `ΔS0 = 0` exactly. `R1 = 4.6052, 2.9004, 1.3665`.
*Semantic expectation:* a mark ubiquitous in B makes agreement easy; evidence must fall. **S0 cannot
see it. R1 can.** This is the family that carries the round.
*Rule:* **PASS** iff `ΔS0 = 0` at every point and R1 matches the three predicted values to `1e-12`
while strictly decreasing.

**F4 — balanced countervailing marginals.** `ĉ_A + ĉ_B = 510` and `N_A + N_B = 2000` both held fixed;
`(ĉ_A, ĉ_B) ∈ {(10,500), (255,255), (500,10)}`.
*Quantity:* `max R1 − min R1` across the three; and the corresponding S0 range.
*Prediction:* R1 `= 1.3665` at all three, range exactly `0`. S0 `= 4.6052, 1.3665, 0.6931`, range `3.9120`.
*Semantic expectation:* **contested, and deliberately not resolved by fiat.** Under the shared-source
model the three are identical and R1 is right. Read as evidence, "rare in A but ubiquitous in B" is a
case where B supplies the mark constantly and agreement is cheap — not obviously the same evidence as
"moderately common in both".
*Rule:* **LIMITATION**, confirmed iff R1's range is exactly `0` while S0's is not. Recorded as a
documented blindness of pooling, not as a failure, because no clause of the contract adjudicates it.

**F5 — corpus-size imbalance.** `f_A = 0.01`, `f_B = 0.50` fixed;
`(N_A, N_B) ∈ {(1000,1000), (100,10000), (10000,100), (100,100000)}`.
*Quantity:* `p̂` and `R1` at each point.
*Prediction:* `p̂ = (N_A·f_A + N_B·f_B)/(N_A + N_B)` — the **length-weighted mean** of the two relative
frequencies. Values `0.2550, 0.4951, 0.0149, 0.4995`; `R1 = 1.3665, 0.7029, 4.2097, 0.6941`.
*Semantic expectation:* the larger recording dominates the pooled estimate, by exactly the ratio
`N_A : N_B`. **R1 is exchange-invariant in value but not equal-influence.** Under the shared-source
model this is correct — more data is more informative — but it means R1 measures rarity chiefly in the
longer recording whenever lengths are unequal.
*Rule:* **LIMITATION**, confirmed iff `p̂` matches the weighted-mean form to `1e-12` at every point and
`R1` moves by more than `1` nat between the balanced and the `100:100000` case. Not a failure: no
clause requires equal influence, and this round declines to invent one after the fact.

**F6a — duplicate A's background only.** `D ∈ {1000, 5000, 20000}` irrelevant events appended to A;
agreeing marks' counts untouched; `k = 2`.
*Quantity:* `ΔS0`, `ΔR1`. *Prediction:* `ΔS0 = k·ln((N_A+D)/N_A)`, `ΔR1 = k·ln((N+D)/N)` with
`N = N_A+N_B`; `ΔS0 = +1.3863, +3.5835, +6.0890` and `ΔR1 = +0.8109, +2.5055, +4.7958`. **Both rise.**
*Semantic expectation:* a mark appearing the same number of times in a longer recording is genuinely
rarer, so a rise is defensible; but it means scores are not comparable across recordings of different
length.
*Rule:* **LIMITATION**, confirmed iff both match their closed forms to `1e-12` and both are positive.
§PHASE 4 M5 struck the discrimination criterion this family originally carried.

**F6b — duplicate B's background only.** Same `D`, appended to B.
*Quantity:* `ΔS0`, `ΔR1`. *Prediction:* `ΔS0 = 0` exactly; `ΔR1 = k·ln((N+D)/N)`, the same values as F6a.
*Rule:* **PASS** iff `ΔS0 = 0` at every point and `ΔR1` matches F6a's values to `1e-12`. Discriminating:
S0's blindness to B is a blindness to specimen size on one side only.

**F7 — rare few versus common many.** `k_r ∈ 1..8` agreements at pooled `p_r ∈ {0.001, 0.01, 0.05}`
against `k_c ∈ 1..8` at `p_c = 0.25`.
*Quantity:* `sign( R1(rare) − R1(common) )` at every one of the swept points, against
`sign( k_r·ln(1/p_r) − k_c·ln(1/p_c) )`.
*Prediction:* identical at all points; the crossover surface is the line
`k_r/k_c = ln(1/p_c)/ln(1/p_r)`, at `0.2007, 0.3010, 0.4628` for the three `p_r`.
*Semantic expectation:* **per PHASE 1, crossings are required, not tolerated.** The criterion is
agreement with the analytic ordering, never "more agreements must win" — that phrasing is struck.
*Rule:* **PASS** iff the observed and analytic signs agree at every swept point, and the empirical
crossover ratio matches the closed form to `1e-12`.

**F8 — dependent repetition.** `N_A = N_B = 1000`, every mark at count 10, so `p̂ = 0.01` throughout.
X: one rare mark repeated five times inside the span by a planted deterministic run. Y: five distinct
rare marks planted independently.
*Quantity:* `R1(X) − R1(Y)`, plus a mechanical check that both sides' pooled probabilities are equal.
*Prediction:* both `= 23.0259`; difference exactly `0`.
*Semantic expectation:* the two generating processes differ by a factor of five in the number of
independent events, and R1 cannot tell them apart, because the i.i.d. assumption of §PHASE 2 is false
here. **This family tests the limits of the interpretation and does not demand R1 solve dependency.**
*Rule:* **LIMITATION**, confirmed iff the difference is exactly `0` and the equal-`p̂` precondition
holds mechanically (§PHASE 4 M7).

#### PHASE 4 — Feasibility propagation pass

Every mechanism found in PHASE 1 and PHASE 2, against every prediction and criterion it touches.
Recorded **before execution**.

| mechanism | criteria/predictions touched | disposition |
|---|---|---|
| **M1** A and B mutually exclusive | F7's criterion; AG1/AG2/AG4/AG8's standing; sprint:16's L1 classification | **Struck and rewritten before trials.** F7 is scored against the analytic ordering, never against "more agreements wins". The four families are demoted to regressions and sprint:16's L1 is withdrawn as a defect finding. Had this been left, F7 would have been a criterion that no correct statistic could pass. |
| **M2** R1 is the numerator half of an LLR | any criterion assuming R1 penalizes disagreement; the verdict's wording | **Corrected.** No family below scores disagreement count. The A verdict is redefined in §PHASE 6 to exclude any calibration claim, since M2 shows the denominator is absent by construction. |
| **M3** `p̂` is the length-weighted mean of `f_A`, `f_B` | **F3's magnitudes** | **Repaired before trials.** F3 originally left `N_A`, `N_B` free, which would have confounded the B-marginal effect with the imbalance effect and made its predicted values wrong. `N_A = N_B = 1000` is now fixed, and the imbalance is isolated in F5 where it belongs. |
| **M3** | F5's rule | **Corrected.** Written as LIMITATION, not FAIL: exchange invariance is a statement about value, and no preregistered clause has ever required equal influence. Inventing one now to fail R1 would be tuning the criterion to the result. |
| **M4** F4 needs `ĉ_A+ĉ_B` **and** `N_A+N_B` both fixed | F4's "R1 invariant" prediction | **Kept, with a mechanical precondition.** The fixture asserts both sums; without it the predicted invariance would be untestable and a passing result would prove nothing. |
| **M5** both statistics move under one-sided A duplication | **F6's original single criterion** | **Split before trials.** F6a is not discriminating and is written as LIMITATION; F6b is discriminating and is written as PASS. A single F6 asserting "R1 differs from S0" would have been false on the A side. |
| **M6** spans are selected by search for agreeing | every real-corpus criterion; the verdict | **Struck.** The replay may report mechanical distributions only. No significance, calibration, or "more surprising than chance" claim is admissible from it, in this round or in the report. |
| **M7** F8 needs equal `p̂` on both sides | F8's "identical scores" prediction | **Kept, with a mechanical precondition.** Without it an equality could be arithmetic coincidence rather than the intended blindness. |
| **M8** F0/F1 are implied by construction | the weight F0 and F1 carry | **Corrected.** Both are correctness checks. Neither counts toward the verdict; §PHASE 6 draws only on the discriminating and limitation families. |

**Reachability.** Every PASS rule above names a quantity the code computes and a value the arithmetic
can attain: F0 and F1's zeros are exact by construction, F2/F3/F6b/F7's targets are the closed forms
themselves. No rule requires a statistic to satisfy both A and B.

**Tiling.** §PHASE 6's partition is by precedence and covers every result; the argument is given there.

#### PHASE 5 — Execution

Old gauntlets — the sprint:12 gauntlet, the ten sprint:15 families, and sprint:17's contract — run
unchanged as regressions, against frozen S0 and frozen R1. R1 is not modified after any result. If a
PASS family fails, the counterexample is minimized and preserved rather than explained away.

Real corpus: the minimum replay that characterizes consequences already measurable without inspecting
contents, under decision:8 and §PHASE 4 M6. Counts, frequencies and margins only. Picks that move are
reported as counts and never rationalized by looking at what moved.

#### PHASE 6 — Verdict partition

Exactly one, by precedence, so the partition tiles:

1. **C — REJECT.** Any family carrying a PASS rule fails. *(Checked first; if so, nothing below applies.)*
2. **A — COHERENT SURVIVOR.** Every PASS family passes **and** no confirmed LIMITATION materially bounds
   what the score may be claimed to mean.
3. **B — USEFUL HEURISTIC / MODEL LIMITATION.** Every PASS family passes, and at least one confirmed
   LIMITATION does materially bound the claim.

**Predicted: B.** §PHASE 2 already establishes that R1's probabilistic reading rests on an i.i.d.
assumption this domain violates, on a null estimated from the data under test, and on spans selected
for agreeing. F5, F6a and F8 are expected to confirm as limitations. Predicting the verdict in advance
is the point: **if the run returns A, §PHASE 2's derivation is wrong**, and that is the finding.

A does not mean adopted. A earns a subsequent adoption experiment and nothing else.

#### PHASE 7 — What this task will not do

No adoption of R1; no R4 or any new formula; no modification of R1, S0, the incumbent selector, the
production statistic, the representation, the metric, or the null. No modification of any prior family,
fixture or expectation, including the four demoted to regressions. No threshold fitted to the corpus,
no treatment of it as ground truth, and no calibration or significance claim from it. No recording
content — prompts, commands, responses, file contents, or absolute paths — in any artifact. Nothing
pushed.
