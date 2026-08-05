---
id: tsk_01KZ9ZARD2H8KY5NGRT3982KFN
sequence: 22
kind: task
status: closed
sprint: spr_01KZ9ZARCM0SDS94AQQ54DQYCS
created: 2026-08-05
closed: 2026-08-05
---

# Attack the null-relative boundary hypothesis with a controlled adversarial gauntlet

## Objective

Build a deterministic adversarial gauntlet of synthetic specimen families with known boundary
semantics, run the frozen sprint:8–11 machinery over hundreds of trials, and report whether the
null-relative statistic behaves as its interpretation claims — or where it does not.

Everything below the `### Preregistration` heading was written and committed to disk **before any
gauntlet trial was generated or run**. The one exception is §2's splinter diagnosis, which is analysis
of task:20's already-committed output, is labelled as such, and is what the hypothesis in §1 is
phrased against.

## Acceptance criteria

- The frozen machinery verifiable by diff; the gauntlet calls it and changes none of it.
- Eight families implemented as specified below, deterministic, with every seed recorded.
- Directional expectations and the uniform pass rule below applied without modification.
- Counterexamples surfaced per family, not aggregated away.
- The three existing specimens re-run and reported separately from the controlled families.
- A self-contained report with scorecard and quadrant scatter.
- A verdict on the four-step ladder, with the strongest counterexample named.
- `scripts/check.sh` passes unweakened; no existing test changed; nothing pushed.

### Preregistration

#### 1. The hypothesis under test

Written in terms of what the machinery computes. For a pair of spans, `total` is the frozen combined
alignment distance and `z` is `(null_mean − observed) / null_stddev` over an order-null ensemble of both
recordings.

> **H.** Let `core` be a pair of spans that genuinely share structure, and `core⁺` the same pair
> extended by one event on each side. When the added event is *informative* — the same mark on both
> sides, and rare in the two recordings' background — `z(core⁺) > z(core)` should tend to hold **even
> when `total(core⁺) > total(core)`**. When the added event is noise, or is the same mark on both sides
> but ubiquitous in the background, `z` should show no systematic improvement.

The claim is about a **tendency across trials**, not about any single transition, because the statistic
is an estimate from a finite ensemble and one specimen is an anecdote.

**Three claims kept apart.** (1) *statistically unusual agreement* — sprint:11 supports this. (2) *this
boundary is preferable* — what H tests. (3) *an automatic boundary-selection policy* — not claimed, not
built here, and out of scope even on a clean pass.

#### 2. The planted-boundary / Pareto splinter, diagnosed

Computed from task:20's committed enumeration before this preregistration was written; nothing was
changed to produce it.

The planted span is `A[20..28) ↔ B[162..170)`, at `ev 0.000, tm 0.087, tot 0.02649`. It is enumerated,
it is not missing, and exactly **three** candidates dominate it:

| dominating candidate | retained | ev | tm | total | how it dominates |
|---|---|---|---|---|---|
| `A[20..29) ↔ B[162..171)` | 9 | 0.000 | 0.078 | **0.02391** | longer **and** lower |
| `A[21..29) ↔ B[163..171)` | 8 | 0.000 | 0.086 | **0.02606** | same length, lower |
| `A[20..32) ↔ B[162..174)` | 12 | 0.000 | 0.084 | **0.02646** | longer **and** lower |

**It is not a bug and it is not a frontier defect.** All four spans agree exactly on marks — `ev 0.000`
— so the ranking among them is decided entirely by timing agreement and the length normalization. The
second row is the sharpest: the *same length*, shifted one event to the right, has better timing
agreement than the planted span.

**It is evidence, and the evidence is this:** in a fixture whose figure repeats exactly every eight
events, the planted boundary is not identifiable from agreement alone. The objective has no notion of
where a repeating figure begins; it measures how well two spans agree, and many spans agree perfectly.
sprint:10's criterion assumed the semantic boundary would be recoverable from the objective, and the
fixture's own periodicity makes that impossible in principle rather than in practice. Recorded, not
repaired — the fixture is unchanged and the metric is unchanged.

This is also why the gauntlet below constructs **non-periodic** backgrounds: a family whose context
repeats would inherit exactly this ambiguity and measure it a second time.

#### 3. Frozen

Unchanged and verifiable by `git diff` over `src/experiment/event_sequence.rs`: `Mark`, `MarkedEvent`,
`align`, `timing_term`, `SUBSTITUTION`, `GAP`, `TIMING_WEIGHT`, `TIMING_FLOOR_MS`, `TIMING_RATIO_FULL`,
`event_norm`, `timing_norm`, `total`, `project`, `refine`, `enumerate_candidates`, `pareto_frontier`,
`REFINE_RADIUS`, `LENGTH_FLOOR`, `order_null_seeded`, `ORDER_NULL_SEED`, `null_seed`, `null_ensemble`,
`null_evidence`, `NULL_HISTOGRAM_BINS`, and the observed channel scope.

The gauntlet lives in a new disposable module and calls this machinery. It does not extend it.

#### 4. Specimen construction

Every trial builds two synthetic NDJSON recordings and runs them through the ordinary
replay → inspect → project path, so the gauntlet exercises the real pipeline rather than a shortcut.
Every tool name contains `Synthetic`.

A trial's two sequences each have the shape `context · core · boundary · context`, where:

- **context** is `context_len` events drawn from a background vocabulary of six marks by a fixed-seed
  generator, **independently on each side**, with non-repeating gaps — so nothing in the context is
  shared and nothing is periodic;
- **core** is `core_len` events carrying the *same* marks in the *same* order on both sides, with
  identical gaps, so the core's own `total` is at or near zero;
- **boundary** is the one event under test, which is what the family varies.

`core` spans and `core⁺` spans are known by construction. The null ensemble is built from the whole
sequences, so background prevalence — the lever families C and D turn — is prevalence over the whole
generated recording.

#### 5. The eight families

| family | boundary construction | expected direction |
|---|---|---|
| **A informative** | same mark both sides, absent from the background, gaps disagreeing by a factor so that `total` worsens | `Δz > 0` |
| **B noise** | *different* marks on the two sides, both drawn from the background vocabulary | no systematic `Δz > 0` |
| **C common** | same mark both sides, **also injected into the background** at high prevalence | (paired with D) |
| **D rare** | same mark both sides, absent from the background — otherwise **identical to C**: same seed, same core, same gaps, so `Δtotal` is identical by construction | `Δz(D) > Δz(C)` |
| **E redundant** | same mark both sides, but a mark **already present in the core**; paired against an A-style trial with a novel rare mark, same seed | `Δz(redundant) < Δz(novel)` |
| **F accidental** | two **independent** streams with nothing planted; refine from an arbitrary seed and take the best-`z` candidate found by chance; paired against a genuine planted core of the same length | `z(chance) < z(planted)` |
| **G diluted** | a planted core inside `context_len ∈ {10, 20, 40, 80}` of unrelated context | the best-`z` candidate overlaps the planted core, at every context length |
| **H competing** | one short exact core built from **common** marks and one longer imperfect core (one substitution) built from **rare** marks, in the same pair | `z(longer, rarer) > z(shorter, tighter)` |

Family C and D are the round's central measurement: **identical raw agreement, different background
prevalence.** If `Δz(D)` is not reliably above `Δz(C)`, sprint:11's stated mechanism is wrong.

#### 6. The sweep

Bounded, deterministic, and recorded. Seeds are trial indices fed to the same LCG family the fixtures
use; every reported trial carries its seed so any failure can be regenerated.

| family | grid | trials |
|---|---|---|
| A | core_len {3,4,5} × context {20,40} × gap ratio {2,4} × seed 0–4 | 60 |
| B | core_len {3,4,5} × context {20,40} × gap ratio {2,4} × seed 0–4 | 60 |
| C/D | core_len {3,4,5} × context {20,40} × seed 0–4, run as matched pairs | 60 |
| E | core_len {3,4,5} × context {20,40} × seed 0–4, matched pairs | 60 |
| F | core_len {3,4,5} × context {20,40} × seed 0–4, matched pairs | 60 |
| G | core_len {4,6} × context {10,20,40,80} × seed 0–4 | 40 |
| H | context {20,40} × seed 0–9 | 20 |

≈ **360 trials**. Null ensemble: **1 000 realizations per trial**, both sides, built once per trial and
shared by that trial's candidates. At sprint:11's measured throughput this is roughly one to two
milliseconds of alignment per trial, so the whole gauntlet is a second of arithmetic.

The empirical tail floor at 1 000 realizations is `1/1001 ≈ 1.0e-3`, and sprint:11 already measured that
`empirical_p` saturates on specimens of this kind. `Δp` is therefore **reported but not scored**, and
`z` is the statistic the expectations are written against — recorded here so that a saturated `p` is
not later presented as a failure of a criterion that was never placed on it.

#### 7. Scoring, fixed before any aggregate is seen

One rule, applied to every family alike, over that family's trials:

- **PASS** — the median delta has the expected sign **and** at least **two thirds** of trials show the
  expected sign.
- **MIXED** — the median has the expected sign but fewer than two thirds of trials agree.
- **FAIL** — the median has the wrong sign or is exactly zero.

For family **B**, whose expectation is an *absence* of effect, the rule inverts: **PASS** when the median
`Δz ≤ 0` **and** at most half of trials show `Δz > 0`; **MIXED** when the median is `≤ 0` but more than
half of trials are positive; **FAIL** when the median `Δz > 0`.

For family **G**, the per-trial outcome is boolean (does the best-`z` candidate overlap the planted
core), so the rule reads on the fraction alone: PASS at ≥ 2/3, MIXED at ≥ 1/2, FAIL below.

Reported per family regardless of outcome: trial count, fraction with the expected sign, median and
quartiles of `Δz`, median `Δtotal` beside it, the count of trials where `z` was undefined because the
null had zero variance, and the **three worst counterexamples** by signed magnitude — worst meaning most
contrary to the expectation, listed with their seeds and parameters.

No bootstrap. The trial counts are large enough for quartiles and small enough that a confidence
interval would be decorative.

#### 8. What the report shows

Extends `boundary_page`. A scorecard near the top: family, expected behaviour, result, trials, fraction
with expected sign, median `Δz`. Then per family a **Δtotal against Δz scatter**, with the four quadrants
distinguished and the interesting one — *raw distance worsens, surprise improves* — marked, since that
is the phenomenon under test. Then the counterexample tables. Then, under a separate heading that says
it establishes no ground truth, the three real specimens from sprint:10 and sprint:11.

#### 9. Verdict ladder

- **FALSIFIED** — the statistic routinely behaves contrary to its interpretation: families A or C/D fail,
  or B shows a systematic positive tendency.
- **WEAK / FRAGILE** — the phenomenon exists but only narrowly, or has serious counterexamples: two or
  more families MIXED or FAIL.
- **PROMISING** — expected directional behaviour survives most attacks with understandable limitations:
  at most one family MIXED, none FAIL.
- **STRONG ENOUGH TO INVESTIGATE A SELECTION POLICY** — every family PASSes, including C/D and F.

A clean pass does **not** authorize building a selector; it authorizes a round that asks whether one can
be built without overfitting this gauntlet.

#### 10. What this task will not do

No selector, no motif score, no ranking rule. No change to the metric, null, search, or representation.
No information-theoretic weighting. No new facet, no variable-length discovery, no motif families, no
corpus, no fourth real specimen, no product CLI surface, no dependency, no Spectroscope change. No
fixture altered to make a planted answer land on a frontier. No real recording committed, copied, or
reproduced. Nothing pushed.

## Result

Delivered. **WEAK / FRAGILE.** The headline phenomenon survived every attack; the interpretation
attached to it did not.

Two sentences:

> Adding a shared informative boundary made raw agreement worse and surprise better in **60 of 60**
> trials — the sprint:11 observation is not a coincidence and reproduces unanimously. But the
> statistic cannot tell a **novel** rare mark from one the core **already contains** — median
> `−0.003` over 30 matched pairs — so what it measures is not informativeness.

### 1. Frozen, verified

`git diff 288229f..HEAD -- src/experiment/event_sequence.rs` is **empty**. Every cost, weight, timing
term, normalization, search radius, floor, null constant, and statistic is exactly as sprint:11 left
it. The gauntlet lives in `src/experiment/gauntlet.rs`, calls the machinery, and parameterizes none of
it. A test recomputes a trial's raw distance with `align` directly and asserts it matches what the
gauntlet reported.

### 2. The scorecard

300 trials, 1 000 order-null realizations each, one preregistered rule applied to every family alike.

| family | expectation | trials | frac | median | median Δtotal | result |
|---|---|---|---|---|---|---|
| informative | `Δz > 0` even where raw worsens | 60 | **1.000** | **+0.515** | +0.047 | **PASS** |
| noise | no systematic `Δz > 0` | 60 | **1.000** | −0.770 | +0.192 | **PASS** |
| rare vs common | rare carries more evidence | 30 | 0.700 | +0.073 | +0.033 | **PASS** |
| redundant | novel beats repeated | 30 | 0.467 | **−0.003** | +0.033 | **FAIL** |
| accidental | coincidence attracts less | 30 | 0.967 | +2.403 | 0.000 | **PASS** |
| diluted | surprise stays on the motif | 40 | **1.000** | +1.000 | +0.027 | **PASS** |
| competing | evidential weight beats raw fit | 20 | **1.000** | +0.427 | 0.000 | **PASS** |

No trial anywhere produced an undefined `z`; no null was degenerate.

### 3. The phenomenon reproduces, unanimously

**Family A, 60 of 60.** Median `Δtotal = +0.047` — raw agreement got *worse* — and median
`Δz = +0.515`. Every point lands in the quadrant the round was built to test. This is exactly the 3→4
transition from the independent-real specimen, and it is not an anecdote: it survives every core
length, every context length, both gap ratios, and every seed.

**Family B, 60 of 60 in the right direction.** Adding an unrelated boundary event costs `Δz = −0.770`
at the median and never once helps. So the statistic is not simply rewarding length.

**Family F, 29 of 30.** A genuine planted core attracts `z` about **2.4 higher** than the best match a
coincidence can produce between two independent streams. The opposite failure mode — a tiny accidental
coincidence canonized as a profound motif — does not occur here.

**Family G, 40 of 40.** The best-`z` candidate overlaps the planted core at every context length from
10 to 80 events per side. The surprising region does not drift with specimen size.

**Family H, 20 of 20.** A six-event imperfect core built from rare marks beats a three-event exact core
built from common ones, by `+0.427` at the median. Evidential weight beats raw fit, which is the whole
conceptual conflict this line of work has been circling.

### 4. The failure, which is the round's most valuable output

**Family E: FAIL.** `Δz(novel) − Δz(redundant)` has median **−0.003** and only **14 of 30** pairs have
the expected sign. That is not a weak effect; it is *no effect*. The statistic assigns essentially the
same evidential weight to:

```text
core:      Core0 · Core1 · Core2
extended:  Core0 · Core1 · Core2 · BoundaryRare     <- a mark seen nowhere else
extended:  Core0 · Core1 · Core2 · Core0            <- a mark the core already carries
```

**Why, mechanically.** Under an order null the marks are permuted across the whole sequence, so the
probability that a *specific* mark lands in a *specific* slot depends only on that mark's global
prevalence — not on whether the span already contains a copy of it. `Core0` is exactly as rare in the
recording as `BoundaryRare`, so a null realization is exactly as unlikely to reproduce either. The null
has no notion of the *information already carried by the span*, and there is no arrangement of this
null that could have one.

**This is a real limitation of the interpretation, not of the arithmetic.** "Surprise" here means
*unlikely under a permutation of marks*, and that is a strictly weaker notion than *informative about a
shared figure*. sprint:11's Result described the mechanism correctly — rarity — and this round shows
rarity alone does not distinguish new evidence from repeated evidence.

**Strongest counterexample**, seed 1, core 3, context 20: `Δz(novel) − Δz(redundant) = −0.3126`. The
redundant extension — a fourth event repeating `Core0` — scored *more* surprising than the novel rare
mark, by a third of a standard deviation.

### 5. The central mechanism is supported, and only weakly

**Family C/D: PASS at 0.700, median `+0.073`, first quartile `−0.031`.** The two families are identical
by construction in marks, gaps, core, seed, and raw agreement — a test asserts `|Δtotal(rare) −
Δtotal(common)| < 1e-6` over all 30 pairs — and differ only in whether the boundary mark also appears in
the background at 35% prevalence.

So background prevalence does move the statistic in the predicted direction, but **9 of 30 pairs go the
wrong way** and the median effect is an order of magnitude smaller than family A's. The mechanism
sprint:11 named is real and is not reliable at the level of one boundary event. Worst counterexample:
seed 1, core 5, context 40, `−0.2027`.

### 6. A defect in the gauntlet, found and reported rather than quietly fixed

The first run scored **noise** at `frac 0.667`, which would have been a MIXED-adjacent PASS and a real
weakness. Inspecting the counterexamples showed `SyntheticBg3` on **both** sides of a "different marks"
trial. The generator drew the two marks with two independent calls, which collided in **20 of 60**
trials — a third of the noise family was silently informative.

Split by that contamination, the pre-fix numbers are unambiguous:

```text
20 contaminated trials  ->  20 with Δz > 0     (all of them)
40 genuine noise trials ->   0 with Δz > 0     (none of them)
```

The fix applies the offset to the first draw so a collision is impossible by construction, and the
corrected family is 60 of 60. **This was a change made after seeing results**, and it is recorded that
way. The justification is that it is a specification violation provable without reference to any
outcome — the family is documented as "different marks on the two sides" and it demonstrably produced
the same mark — and that the fix is to the *specimen generator*, never to the metric or the null, which
the round's binding constraint protects. Both sets of numbers are above. A test now walks the entire
noise family and asserts no collision, because one collision in sixty is precisely what went unnoticed.

### 7. The planted-boundary splinter, resolved

Preregistered §2, from task:20's committed enumeration. The planted span `A[20..28) ↔ B[162..170)` is
enumerated and is dominated by exactly three candidates:

| candidate | retained | total | how |
|---|---|---|---|
| `A[20..29) ↔ B[162..171)` | 9 | 0.02391 | longer **and** lower |
| `A[21..29) ↔ B[163..171)` | 8 | 0.02606 | **same length**, lower |
| `A[20..32) ↔ B[162..174)` | 12 | 0.02646 | longer **and** lower |

All four agree exactly on marks (`ev 0.000`), so the ranking among them is decided entirely by timing
agreement and length normalization. The second row is the sharpest: the same length, shifted one event
right, has better timing agreement than the planted span.

**Not a bug, not a frontier defect, and not a reason to change anything.** In a fixture whose figure
repeats exactly every eight events, the planted boundary is not identifiable from agreement alone —
many spans agree perfectly and the objective has no notion of where a repeating figure begins.
sprint:10's criterion assumed recoverability that the fixture makes impossible in principle. Recorded
as evidence; the fixture and the metric are untouched. The gauntlet's backgrounds are non-periodic
precisely so no family inherits the same ambiguity.

### 8. Observations on the real specimens

Re-run and kept in the report under a heading that says what they are. sprint:11's numbers are
unchanged: the four-event core at `total 0.113, emp-p 7.0e-4, z 4.91` against the three-event suffix at
`total 0.031, emp-p 3.0e-3, z 4.37`. **These establish no ground truth about boundary correctness** and
are not scored. What the gauntlet adds is that the *shape* of that transition — raw worse, surprise
better — is now reproducible on demand, 60 times out of 60, in specimens where the answer is known.

The gauntlet also sharpens the caveat sprint:11 recorded. The real core's rarest mark,
`tool_requested/Agent`, is an adapter emission. Family E shows the statistic cannot distinguish novel
evidence from repeated evidence, and family C/D shows it responds to prevalence only weakly. Neither
finding rescues the real specimen from that caveat, and one of them makes it worse: a mark that is rare
*because of how events are written down* is exactly as surprising to this null as one that is rare
because of what an agent did.

### 9. Verdict: **WEAK / FRAGILE**

By the ladder task:22 fixed:

- Not **FALSIFIED**: family A passes unanimously, C/D passes, and B shows no systematic positive
  tendency — the three conditions that would have falsified it.
- Not **PROMISING**: that rung requires no family to FAIL, and family E does.
- Not **STRONG ENOUGH TO INVESTIGATE A SELECTION POLICY**: that requires every family to pass.

**A gap in my own ladder, recorded rather than smoothed.** The WEAK/FRAGILE rung's count clause reads
"two or more families MIXED or FAIL", and this round produced exactly one FAIL and zero MIXED — a cell
the counts do not cover. Its gloss does: "the phenomenon exists but only narrowly, or **has serious
counterexamples**." One family failing outright is a serious counterexample, the cell is strictly worse
than PROMISING's stated bar, and the verdict is placed on the gloss. This is the **sixth** criterion
defect in seven rounds and the third distinct shape — after a disproved criterion, an unreachable rank
cutoff, an inherited threshold, an unapplied mechanism, and an under-specified statistic, this one is a
ladder whose rungs do not tile.

**What the verdict means in one line.** The phenomenon is real and robust. The interpretation —
*surprise measures how informative a boundary is* — is not supported, because the statistic cannot see
redundancy at all and sees rarity only weakly.

### 10. Strongest counterexample

Family E, seed 1, core 3, context 20:

```text
core       Core0 · Core1 · Core2                          z = z₀
+ novel    Core0 · Core1 · Core2 · BoundaryRare           Δz = +0.42
+ repeat   Core0 · Core1 · Core2 · Core0                  Δz = +0.73
                                                          difference −0.3126
```

Repeating a mark the span already carries bought **more** surprise than adding a mark seen nowhere else
in either recording. If the statistic were measuring evidence, that could not happen.

### 11. Desire-path friction

**Seventh consecutive round with the preregistration in a `###` subsection.** `288229f` contains
nothing else. **idea:5**.

**The verdict ladder did not tile, and no Scarp affordance would have caught it.** §9. The check that
would is arithmetic on the criterion set itself: enumerate the cells the rungs claim to cover and
confirm the cover is exhaustive. Recorded, not built.

**Appending a Result is still `cat >>`** — `scarp` 0.2.0, version lag, maintenance:1.

**One thing that went well.** `--gauntlet` produced 300 trials in 3.2 s including 300 000 null
realizations, so the whole attack is re-runnable in the time it takes to read its own scorecard. A
probe that is cheap to repeat is a probe that gets repeated.

### 12. Strongest limitation

**The gauntlet's families are built from a six-mark vocabulary with no internal structure**, which is
generous to the null in one specific way: a permutation of six roughly-equiprobable marks produces a
well-spread distribution, and every trial had a usable `z`. Real recordings are dominated by one mark at
38% and carry lifecycle marks that appear once. Family C/D probes prevalence directly and finds the
effect weak; whether it becomes weaker still at real vocabularies is not measured here.

Secondly, every family varies **one** boundary event. Nothing here says how the statistic behaves when
two boundaries move at once, which is what an actual selection policy would have to do.

### 13. Recommendation: exactly one next experiment

**Test whether the statistic can see redundancy at all, by replacing the order null with a
within-span-preserving null — and predict, in advance, that it cannot.**

Family E's failure has a mechanical explanation: the null permutes marks globally, so the probability of
a specific mark landing in a specific slot depends only on global prevalence. A null that instead
permutes marks *within the observed span's own multiset* would leave a redundant extension almost
unchanged — the multiset already contains that mark — while a novel extension would change it. If that
null separates novel from redundant and the current one does not, redundancy is representable and the
fix is a different null. If neither separates them, redundancy is not visible to any permutation-based
null and the next lever is genuinely the representation.

It is the smallest experiment that resolves what family E exposed, it reuses the gauntlet unchanged as
its measuring instrument, and it is falsifiable in one round. **Not recommended:** a selection policy.
The round's own verdict forbids it, and a policy built on a statistic that cannot see redundancy would
select redundant boundaries.

### What this task did not do

No selector, no motif score, no ranking rule. No change to the alignment, costs, timing policy,
normalization, marks, representation, search radius, length floor, or null construction — the diff over
`event_sequence.rs` is empty. No information-theoretic weighting. No new facet, no variable-length
discovery, no motif families, no corpus, no fourth real specimen, no product CLI surface, no dependency,
no Spectroscope change. No fixture altered to make a planted answer land on a frontier. No existing test
changed and no check weakened. No real recording committed, copied, or reproduced. Nothing pushed.
