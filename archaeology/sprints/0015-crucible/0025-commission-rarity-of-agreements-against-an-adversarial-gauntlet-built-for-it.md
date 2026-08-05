---
id: tsk_01KZA3K3N4ABJAZM31CXSH5SZM
sequence: 25
kind: task
status: pending
sprint: spr_01KZA3K3MQ3KS31VSN2QGQAZER
created: 2026-08-05
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
