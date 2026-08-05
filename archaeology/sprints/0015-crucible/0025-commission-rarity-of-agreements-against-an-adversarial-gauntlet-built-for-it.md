---
id: tsk_01KZA3K3N4ABJAZM31CXSH5SZM
sequence: 25
kind: task
status: closed
sprint: spr_01KZA3K3MQ3KS31VSN2QGQAZER
created: 2026-08-05
closed: 2026-08-05
---

# Commission rarity_of_agreements against an adversarial gauntlet built for it

## Objective

Commission `rarity_of_agreements` adversarially: state exactly what it is, why the old rule forbade it,
why the old gauntlet cannot validate it, and then run a preregistered gauntlet built against
inverse-frequency weighting's own failure modes — with the statistic frozen throughout.

`### Phase 1` and `### Phase 2` were written and committed **before any adversarial family was
implemented or evaluated**.

## Acceptance criteria

- The frozen definition, its evidential reading, and the old prohibition all recorded first.
- Eight adversarial families with construction, invariant, mechanism of risk, and quantitative rule,
  fixed before evaluation and unchanged after.
- Sweeps where a phase transition may exist; any failure boundary minimized and preserved.
- The old gauntlet re-run unchanged; no expectation weakened; pinned rows unmoved.
- `k = 0` coverage preserved; `scripts/check.sh` passes unweakened; nothing pushed.

### Phase 1 — what is being commissioned

#### 1.1 The exact frozen definition

From `src/experiment/identifiability.rs`, unchanged for this round:

```text
rarity_of_agreements(ā, b̄, ĉ_A, N_A)  =  Σ over positions i where āᵢ = b̄ᵢ  of  −ln( ĉ_A(āᵢ) / N_A )
```

in nats, higher meaning more evidence, and `None` when the two spans differ in length.

**What it reads:** span A's marks, span B's marks (only to test equality), recording A's mark counts,
recording A's length.

**What it does not read, and this is load-bearing:** `ĉ_B` and `N_B`. The implementation touches
`a_counts` and `a_total` only. **The statistic is asymmetric** — swapping the two arguments can change
its value — where sprint:13's `surprisal` was deliberately symmetrized. §2.4's family AG3b is built on
that.

Also absent by construction: timing, paths, payloads, channels, adapters, schemas, agents, and any
interpretation of a mark beyond equality and a count lookup. It is a function of the representation
sprint:14 formalized.

#### 1.2 What it is approximating

It is the **negative log-likelihood, in nats, of the agreeing marks under an i.i.d. categorical model
with recording A's empirical marginals**. Read aloud: *if marks were drawn independently at the rates A
exhibits, how improbable would it be to draw exactly the marks on which the two spans agreed?*

Three ways that approximation is loose, recorded now so that a failure attributable to one of them is
not later reported as a surprise:

- **Independence.** Positions are treated as independent draws. Real event streams are not
  independent, and a figure is precisely a violation of independence — so the model under which
  surprise is computed is one the data is known to violate.
- **With replacement.** Unlike sprint:13's permutation null, drawing a mark does not consume it. That
  is exactly why the statistic sees a repeat where the permutation null cannot (sprint:14 §4), and it
  is also why nothing bounds the contribution of a mark whose count is one.
- **One-sided.** The likelihood is computed under A's marginals alone, so an agreement on a mark that
  is rare in A and ubiquitous in B is scored as though it were rare on both sides.

#### 1.3 The old prohibition, quoted and explained

sprint:12's non-goals: *"Information-theoretic weighting of any kind: inverse frequency, TF-IDF,
`−log p(mark)`, entropy, mutual information, learned rarity. Those are candidates for a later round and
only if the null fails or explains why they would help."* sprint:13 repeated it as a constraint.

**Its historical purpose was to prevent post-hoc repair, not to claim the class is wrong.** Both rounds
were falsification rounds with a family that failed, and the obvious way to make a failing family pass
is to add a term that rewards exactly what it is failing on. The rule existed so that a repair could not
be fitted to a gauntlet. Its own wording anticipated this round: *"candidates for a later round."*

sprint:14 did not violate its purpose: `rarity_of_agreements` entered as one of ten functions
preregistered before evaluation, in an identifiability audit rather than as a repair, and was flagged
`probe: true` and explicitly not adopted.

**The narrowest change this round makes: none to the code, and none to history.** No mechanism ever
prevented evaluation — sprint:14 already evaluated it — so nothing is deleted or rewritten. The `probe`
flag stays and a test still pins it. What changes is scope: this sprint declares evaluation of a flagged
probe to be in scope, and records that the statistic **violates the sprint:12 rule as written** and is
being tested precisely to find out whether that rule should survive. Adoption remains out of scope.

#### 1.4 Discovery is not validation, recorded before any new result

sprint:12's gauntlet was constructed against the **permutation null's** failure modes. sprint:14 found
`rarity_of_agreements` by scoring ten functions against that gauntlet and observing which passed. A
statistic selected on a test suite is not validated by it.

Therefore, for this round:

```text
old gauntlet  =  discovery evidence, and now a regression suite
new gauntlet  =  fresh commissioning evidence
```

Its seven PASSes are not counted toward this round's conclusion. Re-running it unchanged shows only that
nothing broke.

### Phase 2 — the preregistered adversarial gauntlet

#### 2.1 How specimens are built, and why differently from sprint:12

`rarity_of_agreements` is a function of the representation and of nothing else, so specimens are
constructed **directly as `Observation` values** rather than generated as recordings and projected.
That removes the generator as a confound — sprint:12 lost a family to one — and makes every case
readable in a line. The cost is that the pipeline is not exercised; it does not need to be, because the
pipeline is not what is on trial and sprint:12's gauntlet still runs through it.

Notation below: a case names the agreeing marks by their **count in A**, the number of disagreeing
positions, and `N_A`.

#### 2.2 The rule, fixed for every family

Each family is a set of comparisons, each with an expected ordering, evaluated at a **nominal point** and
over a **sweep**.

- **PASS** — the expected ordering holds at the nominal point **and** at every swept point.
- **MIXED** — it holds at the nominal point but fails somewhere in the sweep. The boundary is located
  and reported.
- **FAIL** — it fails at the nominal point.

These constructions are deterministic, so no fraction-of-trials rule is needed: a single violation
anywhere in the sweep is a real violation and is reported as one.

#### 2.3 Predictions are recorded per family below

Several families are predicted to FAIL or come out MIXED. That is the point of an adversarial round, and
recording the predictions in advance is what distinguishes a discovered failure from a manufactured one.

#### 2.4 The eight families

**AG1 — singleton against a repeated motif.**
*Construction:* X = one agreement on a mark of count 1; Y = four agreements on marks of count `c`.
*Invariant:* Y > X — a substantially stronger repeated motif must beat a lone accidental agreement.
*Mechanism of risk:* X scores `ln N`, which grows without bound in corpus size, while Y scores
`4·ln(N/c)`.
*Nominal:* `N = 1000`, `c = 50` → X = 6.91, Y = 11.98.
*Sweep:* `N ∈ {10², 10³, 10⁴, 10⁵, 10⁶}` × `c ∈ {10, 50, 200}`.
*Prediction:* **MIXED** — holds at nominal, flips at large `N` where `ln N > 4·ln(N/c)`.

**AG2 — rarity explosion.**
*Construction:* X = one agreement on a mark of count 1 in a corpus of growing size; Y = four agreements
at a fixed relative frequency `p = 0.05`.
*Invariant:* one weak agreement must not acquire unbounded dominance; concretely X < Y for all
`N ≤ 10⁶`.
*Mechanism of risk:* `−ln(1/N) = ln N` is unbounded, while Y is `4·ln(1/p)` and constant in `N`.
*Nominal:* `N = 1000`.
*Sweep:* `N ∈ {10², 10³, 10⁴, 10⁵, 10⁶}`.
*Prediction:* **FAIL** at the upper end — `ln N` exceeds `4·ln 20 = 11.98` once `N > e^{11.98} ≈ 1.6×10⁵`.

**AG3 — rare disagreement.**
*Construction:* X = spans containing a count-1 mark at a position where the two sides **disagree**, plus
two agreements on common marks; Y = the same two common agreements and no rare mark anywhere.
*Invariant:* X = Y — rarity that does not agree must contribute nothing.
*Mechanism of risk:* a statistic that summed span rarity rather than agreement rarity would reward X.
*Nominal and sweep:* rare mark count 1, common counts `∈ {20, 100}`, `N ∈ {10³, 10⁵}`.
*Prediction:* **PASS**, structurally — the sum ranges over agreeing positions only.

**AG3b — one-sided rarity.**
*Construction:* X = one agreement on a mark with count 1 in A and count `N_B/2` in B; Y = one agreement
on a mark with count 1 in both.
*Invariant:* Y > X — an agreement on a mark that is ubiquitous in B is easy to obtain and must count for
less than one rare on both sides.
*Mechanism of risk:* §1.1 — the statistic never reads `ĉ_B`.
*Nominal:* `N_A = N_B = 1000`.
*Sweep:* B-count `∈ {1, 10, 100, 500}`.
*Prediction:* **FAIL**, and identically at every point: X = Y exactly, because B's counts are invisible.

**AG4 — common but structural.**
*Construction:* X = six agreements on marks of relative frequency `p = 0.2`; Y = one agreement on a
count-1 mark.
*Invariant:* X > Y — a genuinely repeated figure of common marks must remain recoverable; rarity must
not become synonymous with motif-ness.
*Mechanism of risk:* X scores `6·ln 5 = 9.66` and is fixed; Y scores `ln N` and grows.
*Nominal:* `N = 1000` → X = 9.66, Y = 6.91.
*Sweep:* `N ∈ {10², 10³, 10⁴, 10⁵, 10⁶}`, `p ∈ {0.1, 0.2, 0.35}`.
*Prediction:* **MIXED** — the same `ln N` crossing as AG1 and AG2.

**AG5 — vocabulary growth.**
*Construction:* two candidates with `k_X = 1` and `k_Y = 3` agreements, unchanged, in a corpus to which
`M` events carrying entirely new marks are appended. Candidate marks' counts do not change; `N` does.
*Invariant:* the ordering of X and Y must not change, since neither candidate's evidence changed.
*Mechanism of risk:* each agreement's term is `ln N − ln c`, so growing `N` adds `k·ln(N′/N)` — an
increase **proportional to the number of agreements**, which is not order-preserving between candidates
of different `k`.
*Nominal:* `N = 1000`, `M = 0`, with X's mark count 2 and Y's counts 300 each.
*Sweep:* `M ∈ {0, 10³, 10⁴, 10⁵}`.
*Prediction:* **FAIL** — X leads at `M = 0` (6.21 against 3.61) and Y overtakes as `M` grows.

**AG6 — corpus duplication.**
*Construction:* (a) duplicate the **whole** corpus: every count and `N` double. (b) duplicate
**background only**: `N` grows, candidate marks' counts unchanged.
*Invariant:* (a) every score exactly unchanged, since `c/N` is unchanged. (b) stated, not assumed — the
underlying empirical distribution *has* changed, so a change in score is defensible; what must not
happen is a **reordering** of two candidates whose own evidence is untouched.
*Mechanism of risk:* (b) is AG5's mechanism under a different name.
*Nominal:* base `N = 1000`; (a) ×2; (b) +1000 background events.
*Sweep:* replication factor `∈ {1, 2, 4, 10}` for (a); background additions `∈ {0, 10³, 10⁴}` for (b).
*Prediction:* (a) **PASS**, exactly invariant. (b) **FAIL** on reordering, same as AG5.

**AG7 — sample-size instability.**
*Construction:* the same underlying mark distribution observed at `N` and at `N′`, with counts scaled
proportionally and rounded. Two candidates: one whose agreeing marks have relative frequency `0.05`, one
whose agreeing mark is a **singleton** (count 1 at every `N`, so its relative frequency changes with
sample size).
*Invariant:* the non-singleton candidate's score must be stable to within `0.1` nats per agreement
across `N`, and the ordering must not change.
*Mechanism of risk:* a singleton's term is `ln N` by definition, so it is not stable in `N` at all.
*Nominal:* `N = 1000` against `N′ = 2000`.
*Sweep:* `N′/N ∈ {1.5, 2, 5, 10}`.
*Prediction:* **PASS** for the proportional candidate, **FAIL** for the singleton — and the split itself
is the finding.

**AG8 — rare coincidence against repeated moderate evidence.**
*Construction:* X = one agreement on a count-1 mark; Y = four agreements on marks of count `c = 5`.
*Invariant, with the reason preregistered:* **Y > X.** Four independent agreements are four independent
opportunities to have been wrong, and the joint improbability of four moderate coincidences is stronger
evidence of a shared figure than one spectacular coincidence — which, at a rate of `1/N` in a corpus of
`N`, is expected to occur about once per corpus **by construction**. A single event at its own expected
rate is not evidence.
*Mechanism of risk:* the statistic sums log-probabilities without regard to how many terms the sum has,
so a single large term and several moderate ones are interchangeable to it.
*Nominal:* `N = 1000` → X = 6.91, Y = 4·5.30 = 21.2.
*Sweep:* `N ∈ {10², 10³, 10⁴, 10⁶, 10⁹}`, `c ∈ {5, 50}`.
*Prediction:* **MIXED** — Y wins comfortably at realistic `N`, X overtakes at extreme `N`.

#### 2.5 The regression suite

sprint:12's gauntlet runs unchanged, all seven families, under both the incumbent `z` and the challenger
`surprisal`, and the pinned incumbent rows must still hold to `5e-4`. Nothing about it is weakened,
reinterpreted, or re-scored, and **its results are not counted toward this round's conclusion**.

#### 2.6 Conclusions available

- **A** — survives the preregistered adversarial gauntlet, and deserves a subsequent head-to-head
  adoption experiment.
- **B** — exposes a specific repairable failure mode, with the counterexample banked.
- **C** — has a structural failure serious enough to reject the candidate.

The round may not conclude the statistic is generally correct because it passed a synthetic gauntlet, and
may not adopt it under any outcome.

#### 2.7 What this task will not do

No adoption, no change to the incumbent selector or production behaviour, no repair of the statistic, no
re-run against a repaired version, no enumeration of replacements. No richer observable. No weakening of
any existing expectation. No corpus, no variable-length discovery, no fourth real specimen, no product
CLI surface, no dependency, no Spectroscope change. No real recording committed, copied, or reproduced.
Nothing pushed.

## Result

Delivered. **B — a specific failure mode, banked, with the repair deferred.** Two of them, in fact, and
the larger one has a single mechanical cause that explains six of the ten families at once.

The headline:

> `rarity_of_agreements` scores a candidate at `k·ln N − Σ ln cᵢ`. The `k·ln N` term means **the
> ordering of two candidates with different numbers of agreements is a function of corpus size**, and
> nothing about either candidate. Six families break on that one fact. A seventh breaks on a separate
> defect: the statistic never reads the second recording's marginals at all.

### 1. Results, against predictions fixed before running

| family | predicted | result | first failing point |
|---|---|---|---|
| AG1 singleton vs motif | MIXED | **MIXED** | `N=100 c=50` |
| AG2 rarity explosion | FAIL | **MIXED** | `N=1000000` |
| AG3 rare disagreement | PASS | **PASS** | — |
| AG3b one-sided rarity | FAIL | **FAIL** | `count_B=1` |
| AG4 common but structural | MIXED | **MIXED** | `N=1000 p=35%` |
| AG5 vocabulary growth | FAIL | **MIXED** | `M=10000` |
| AG6a whole-corpus duplication | PASS | **PASS** | — |
| AG6b background duplication | FAIL | **MIXED** | `+10000 background` |
| AG7 sample-size stability | PASS | **PASS** | — |
| AG8 coincidence vs repetition | MIXED | **MIXED** | `N=100 c=50` |

Three PASS, six MIXED, one FAIL. Seven of ten predictions exact; three (AG2, AG5, AG6b) predicted FAIL
and came out MIXED — see §6, which is a defect in how I wrote the predictions and not in what happened.

### 2. The one mechanism behind six families

Every agreement contributes `−ln(c/N) = ln N − ln c`. So a candidate with `k` agreements scores

```text
k · ln N  −  Σ ln cᵢ
```

and two candidates are ordered by

```text
(k_X − k_Y) · ln N  +  (Σ ln c_Y − Σ ln c_X)
```

which **changes sign at a corpus size determined entirely by the marks' counts**. The statistic is not
scale-free across candidates of different length: whichever candidate has more agreements eventually
wins as `N` grows, and whichever has fewer eventually wins as `N` shrinks — regardless of what either
candidate contains.

Every boundary the sweep found matches the closed form exactly:

| family | comparison | analytic boundary | observed |
|---|---|---|---|
| AG1, AG8 | `k=1` against `k=4`, counts fixed | `c = N^{3/4}` | `N=100` fails at `c≥50` (`N^{3/4}=31.6`); `N=1000` fails at `c=200` (`177.8`); `N=10⁴` holds to `c=200` (`1000`) |
| AG2 | `k=1` against `k=4` at fixed frequency `p=0.05` | `N = p^{−4} = 1.6×10⁵` | holds at `10⁵` (11.513 < 11.983), fails at `10⁶` |
| AG4 | `k=1` against `k=6` at fixed frequency `p` | `N = p^{−6}` | `p=0.35 → 544`, fails from `N=10³`; `p=0.20 → 15 625`, fails at `10⁵`; `p=0.10 → 10⁶`, fails at `10⁶` |
| AG5, AG6b | `k=1` (`c=2`) against `k=3` (`c=300`) | `N = 3 674` | holds at `N=2 000`, fails at `N=11 000` |

**This is one finding seen six ways, not six findings.** It also explains the two invariance PASSes:
AG6a and AG7 are exactly the transformations that hold every `c/N` fixed, and under those the statistic
is exactly invariant. **`rarity_of_agreements` is invariant under transformations preserving all relative
frequencies, and unstable under any that move the denominator without moving a candidate's counts.**

### 3. The separate defect: AG3b, the only outright FAIL

The statistic reads `a_counts` and `a_total` and never `b_counts` or `b_total`. So an agreement on a
mark that occurs **once in A and five hundred times in B** — trivially easy for B to supply — scores
identically to an agreement on a mark that occurs once in both.

```text
rare in both       ā = (m), ĉ_A(m) = 1, ĉ_B(m) = 1      score = ln 1000 = 6.908
ubiquitous in B    ā = (m), ĉ_A(m) = 1, ĉ_B(m) = 500    score = ln 1000 = 6.908
```

Exactly equal, at every swept value of B's count. `counterexample_ag3b_one_sided_rarity_is_exactly_
blind_to_the_other_side` pins it.

This one is **cheaply repairable** — sprint:13's `surprisal` already symmetrizes over both directions,
and the same move applies here. The repair is not made, per task:25.

### 4. What passed, and why it is worth stating

**AG3 — rarity that does not agree contributes nothing.** Exact equality at every point. The statistic
sums over *agreeing* positions, so a rare mark sitting at a disagreeing position is correctly worth zero.
A naïve "sum the span's rarity" would have failed this, and it is the one adversarial family aimed at a
mistake the statistic does not make.

**AG6a — whole-corpus duplication is exactly invariant.** Multiplying every count and `N` by 2, 4, or 10
leaves every score unchanged to `1e-9`. The statistic depends on frequencies, not on absolute counts,
which is the right dependence and is not automatic.

**AG7 — sample-size stability.** A candidate whose marks hold a fixed relative frequency is stable to
well inside the preregistered `0.1` nats per agreement across sample-size ratios from 1.5 to 10. The
same row reports the singleton's drift beside it, which is `ln(N′/N)` by construction and is the same
unboundedness AG2 measures.

### 5. Minimized counterexamples, banked

Three, each reduced to a line and pinned by test so a later round cannot lose them.

```text
AG3b   A: (m) with ĉ_A(m)=1, N=1000        B-count 1 → 6.908
                                            B-count 500 → 6.908        exactly equal

AG1    N=100.  one agreement on a count-1 mark          = ln 100      = 4.605
               four agreements on count-50 marks        = 4·ln 2      = 2.773
               the singleton wins by 66%

AG5    two candidates, neither changed in any way:
               X: one agreement,  count 2       at N=1 000  → 6.215   X leads
               Y: three agreements, count 300   at N=1 000  → 3.612
               X: one agreement,  count 2       at N=11 000 → 8.613   Y leads
               Y: three agreements, count 300   at N=11 000 → 10.806
               only unrelated events were appended
```

### 6. My predictions were imprecise in a way worth recording

Three families (AG2, AG5, AG6b) were predicted **FAIL** and came out **MIXED**. The predictions
reasoned about *whether the invariant breaks anywhere in the sweep* — it does, in all three — while §2.2's
rule keys FAIL to the **nominal point**, which held in all three. The mechanism was predicted correctly
every time; the label was not.

That is the **ninth** defect in ten rounds and it is the same family as sprint:11's: a criterion and a
prediction that name different quantities without noticing. sprint:11's was *"which computed quantity
does this sentence name?"*; this one is *"which point of the sweep does this prediction refer to?"*. The
step that would have caught it: **write each prediction in the same vocabulary as the rule that will
score it.** Recorded, not built.

### 7. Regression: the old gauntlets are unchanged

sprint:12's gauntlet, all seven families under both `z` and `surprisal` — every number identical to
sprint:13's, including the pinned incumbent rows to `5e-4`. sprint:14's ten-function matrix — identical,
including `rarity_of_agreements` and `novel_rarity` clean across all seven.

**Those results are not counted toward this round's conclusion**, per §1.4. They show only that nothing
broke. The contrast is the point: the statistic passes seven-for-seven on the suite that discovered it
and fails or wobbles on seven of ten families built against it.

### 8. Is the old prohibition justified, overbroad, or unresolved?

**Vindicated in its caution and overbroad in its stated reason.**

sprint:12 forbade *"information-theoretic weighting of any kind"*. This round shows the specific
candidate has two real defects, so the caution was warranted — a round that had adopted it on the
strength of sprint:14 would have adopted a statistic whose orderings move with corpus size.

But the defects are **not** "rarity weighting is wrong". AG3 and AG6a show the rarity part behaving
exactly as it should. The failures are (a) summing an unbounded per-position surprisal across candidates
with different numbers of terms, without any correction for the differing number of opportunities, and
(b) a one-sided likelihood. Neither is about inverse frequency; the first is about **comparing sums of
different lengths** and would afflict any additive per-position score, and the second is a plain
asymmetry.

So the prohibition caught the right candidate for the wrong reason. Its own wording — *"candidates for a
later round"* — anticipated exactly this, and the round it anticipated has now happened.

### 9. Conclusion: **B**, narrowly

> `rarity_of_agreements` exposes **two specific failure modes**, both mechanically understood, both with
> minimized counterexamples banked, and both repairable in principle — the asymmetry cheaply, the
> length-dependence not cheaply. It **does not** survive the adversarial gauntlet built for it, and it
> **is not** rejected outright: the rarity mechanism itself passes the families aimed at it, and the
> failures are about how per-position scores are combined rather than about what a position is worth.

Not **A**: six of ten families wobble and one fails, so it has not earned a head-to-head adoption
experiment yet. Not **C**: the failures are localized to the combination rule and one missing argument,
and AG3, AG6a, and AG7 show the underlying idea behaving correctly where it is aimed at directly.

**And nothing here says the statistic is generally correct**, which §2.6 forbids and which passing a
synthetic round could not establish anyway.

### 10. Desire-path friction

**Tenth consecutive round with the preregistration in a `###` subsection.** `ec44550` contains nothing
else. **idea:5**.

**A prediction vocabulary that did not match its scoring rule.** §6. Third occurrence of the general
shape — sprint:11's under-specified statistic, sprint:12's non-tiling ladder, this round's
nominal-versus-sweep mismatch — and all three would have been caught by the same discipline: read each
criterion and each prediction against the exact quantity the code will compute.

**Appending a Result is still `cat >>`** — `scarp` 0.2.0, version lag, maintenance:1.

**One thing that went well.** Building the specimens directly as representation values rather than
generating recordings made every counterexample a single readable line and every phase boundary
analytically checkable. sprint:12 lost a whole family to a generator confound; this round had no
generator to be confounded by.

### 11. Strongest limitation

**Ten families is a gauntlet, not a proof.** It was built against the failure modes I could anticipate
for inverse-frequency weighting, and a mode I did not anticipate would not appear in it — which is the
same objection this round levels at sprint:12's gauntlet, and it applies here with equal force.

Secondly, **every family is synthetic and constructed at the representation level.** The sweeps show
where the orderings flip in corpus size and mark frequency, but nothing here says where real recordings
sit relative to those boundaries. The 234-record session has `N ≈ 169` and marks at frequencies from
`0.006` to `0.379` — inside the region where AG1's boundary `c = N^{3/4} ≈ 47` bites, since its commonest
mark occurs 64 times. That is a suggestive coincidence and not a measurement, and this round did not
make it one.

### 12. Is another experiment warranted, and what exactly should it ask?

**Yes, one, and it is not a repair.**

> Do the failure boundaries this round located analytically actually bite at the corpus sizes and mark
> frequencies of the real recordings — or are they asymptotic curiosities outside the operating range?

§11's arithmetic suggests they may bite: the real session's commonest mark sits above AG1's boundary at
that corpus size. If the boundaries are outside the operating range, both defects are acceptable
consequences of the statistic's semantics and the next question is adoption. If they are inside it, the
length-dependence must be addressed before adoption is discussable, and *that* is the round to propose a
repair in.

It needs no new statistic, no new representation, and no new gauntlet — only the boundaries already
derived, evaluated at the three real specimens' actual `N` and mark frequencies. **This round does not
implement it.**

### What this task did not do

No adoption of `rarity_of_agreements`, no change to the incumbent selector or any production behaviour,
no repair of the statistic, no re-run against a repaired version, no enumeration of replacements. No
richer observable, and the representation-sufficiency question was not reopened. No existing expectation
weakened, reinterpreted, or rewritten; sprint:12's gauntlet and sprint:14's enumeration both reproduce
identically, pinned rows included. The `probe` flag and its test remain, and no history was rewritten.
The `k = 0` regression coverage is intact. No corpus, no variable-length discovery, no fourth real
specimen, no product CLI surface, no dependency, no Spectroscope change. No real recording committed,
copied, or reproduced. Nothing pushed.
