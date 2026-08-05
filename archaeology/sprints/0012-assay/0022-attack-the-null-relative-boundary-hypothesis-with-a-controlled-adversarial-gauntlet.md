---
id: tsk_01KZ9ZARD2H8KY5NGRT3982KFN
sequence: 22
kind: task
status: pending
sprint: spr_01KZ9ZARCM0SDS94AQQ54DQYCS
created: 2026-08-05
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
