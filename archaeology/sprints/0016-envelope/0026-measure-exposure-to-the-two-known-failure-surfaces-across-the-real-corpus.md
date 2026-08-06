---
id: tsk_01KZA5SR41C9H7H7NMR6JNJVES
sequence: 26
kind: task
status: closed
sprint: spr_01KZA5SR3F4HX6ZQY7XC8T8DE9
created: 2026-08-05
closed: 2026-08-05
---

# Measure exposure to the two known failure surfaces across the real corpus

## Objective

Measure where sprint:15's two known failure surfaces lie relative to the empirical operating envelope of
every suitable real recording available, and classify exposure to each — separately, on criteria fixed
before the data was seen.

`### Preregistration` was written and committed **before any mark frequency was computed**. What *was*
inspected first is the corpus inventory in §F: record counts, observed-event counts, spans, session ids,
and adapters, which is what deciding inclusion requires.

## Acceptance criteria

- A–E below recorded before measurement; classifications not chosen after seeing data.
- Every available recording included or excluded with a reason.
- Both defects measured, reported separately, and classified L1/L2/L3 and S1/S2/S3.
- Margins reported as quantities, not as binary labels; the closest observed configuration named.
- Synthetic gauntlets re-run unchanged; known failures intact.
- `scripts/check.sh` passes unweakened; nothing pushed; no recording content committed.

### Preregistration

#### A. The exact frozen definition

Unchanged since sprint:14 and frozen for this round:

```text
rarity_of_agreements(ā, b̄, ĉ_A, N_A)  =  Σ over positions i where āᵢ = b̄ᵢ  of  −ln( ĉ_A(āᵢ) / N_A )
```

in nats, higher meaning more evidence, `None` across unequal span lengths. It reads span A's marks, span
B's marks for equality only, recording **A's** mark counts, and recording **A's** length. It does not
read `ĉ_B` or `N_B`.

#### B. The preserved counterexamples and boundaries

From sprint:15, banked in `tests/adversarial.rs` and unmodified:

- **AG3b, the asymmetry witness.** `ā = (m)` with `ĉ_A(m) = 1`, `N_A = 1000`: scores `ln 1000 = 6.908`
  whether `ĉ_B(m)` is 1 or 500. Exactly equal.
- **AG1, the accumulation witness.** `N = 100`: one agreement on a count-1 mark scores `ln 100 = 4.605`;
  four agreements on count-50 marks score `4·ln 2 = 2.773`. The singleton wins by 66%.
- **AG5, the reordering witness.** Two candidates unchanged in every respect; only unrelated events
  appended. `k=1, c=2` leads at `N = 1 000` and loses at `N = 11 000`.

The general ordering rule, derived there: two candidates are ordered by

```text
(k_X − k_Y) · ln N  +  (Σ ln c_Y − Σ ln c_X)
```

#### C. Which empirical quantities determine proximity to each surface

**Accumulation.** Specialize the ordering rule to the sharpest form sprint:15 minimized: a **single**
agreement on a mark of count `c₁` outscores `k` agreements each on marks of count `c` exactly when

```text
c^k / c₁  >  N^{k−1}        and, for a singleton (c₁ = 1),        c  >  N^{(k−1)/k}
```

So proximity is determined by three measured quantities per recording: **`N`** (events in the scope the
machinery uses), **the largest mark count `c_max`**, and **whether any mark has count 1**. The boundary
`N^{(k−1)/k}` is evaluated for each `k` the machinery actually produces.

**Asymmetry.** Determined by how much the two recordings' *relative* frequencies differ on the marks a
candidate agrees on, since

```text
score(A,B) − score(B,A)  =  Σ over agreeing marks of  ln( (ĉ_B(m)/N_B) / (ĉ_A(m)/N_A) )
```

So the measured quantity is per-mark relative-frequency ratio between recordings, aggregated over the
agreeing positions of real candidates.

#### D. What is measured, per included recording

Record count; observed-scope event count `N`; vocabulary size; every mark's count and empirical
frequency; `c_max` and its mark; minimum nonzero count; the count of singleton marks; quartiles and
deciles of the frequency distribution; the span lengths the existing boundary machinery produces; and the
agreement counts of the candidates it produces.

Nothing new is searched for: spans and agreements come from `refine` at sprint:10's frozen radius and
floor, and from `cross_pairs` at sprint:9's frozen ladder `k ∈ {3, 4, 6, 8, 12}`, both unmodified.

#### E. Classification criteria, fixed before the data

**Accumulation — exactly one of:**

- **L1 observed/reachable** — either the corpus contains an **actual** candidate pair produced by the
  unmodified machinery in which a candidate with strictly fewer agreements outscores one with strictly
  more; **or** it contains, in a single recording, both a mark of count 1 and a mark whose count exceeds
  `N^{(k−1)/k}` for some `k` within the span range the machinery actually produced. The two are reported
  distinctly, as §C requires, but either warrants a repair round before adoption is discussable.
- **L2 comfortably outside** — no observed crossing, and for every `k` in the observed span range,
  `c_max ≤ ½ · N^{(k−1)/k}` — a relative margin of at least 2× on the count — in every included
  recording.
- **L3 unresolved** — fewer than two included recordings, or the machinery yields no candidates to
  examine.

Anything that is neither L1 nor L2 by these tests is reported as **L3 with the reason**, rather than
being placed by judgement.

**Asymmetry — exactly one of:**

- **S1 intended symmetric** — the relation is meant to be invariant under exchanging the two occurrences,
  so argument-order dependence is an invariance defect requiring eventual repair whatever its measured
  size.
- **S2 intentionally directional** — the two arguments have documented distinct semantic roles that
  justify a directional score.
- **S3 unresolved** — intent is not settled and needs an explicit design decision.

**The intent is settled here, before measurement, on design grounds.** The relation being scored is *"a
window in one recording and a window in another agree more than chance"* — a statement about an unordered
pair. Nothing in `cross_pairs` gives the two recordings distinct evidential roles: it takes two sequences,
requires only that they differ, and returns pairs. sprint:13 symmetrized `surprisal` explicitly for this
reason, calling the asymmetric form a different question. No artifact anywhere in this project documents a
directional reading of the relation.

**Therefore the preregistered intent is symmetric, and the expected classification is S1.** The empirical
measurement is collected anyway, because the *magnitude* of an invariance defect is decision-relevant to
sequencing a repair even when its existence is not in question. **A small measured discrepancy will not
be used to reclassify this as acceptable**, which §E forbids in advance.

#### F. Corpus inventory, and the inspection that produced it

Six recordings exist across two established locations. Deciding inclusion required record counts,
observed counts, spans, sessions, and adapters, and those were inspected before this preregistration was
written. No mark frequency was.

| recording | project | records | observed | span | decision |
|---|---|---|---|---|---|
| `8b68dece` | witnessglass | 234 | 169 | 1054 s | **include** |
| `57f18ff9` | witnessglass | 39 | 32 | 141 s | **include** |
| `f5c18299` | witnessglass | 40 | 33 | 113 s | **include** |
| `c3afa0ca` | witnessglass | 1 | 1 | 0 s | **exclude** — a lone `session_ended`; no vocabulary, no span |
| `7d95c414` | cuecraft | 106 | 77 | 859 s | **include** |
| `6a8a02cc` | cuecraft | 1 | 1 | 0 s | **exclude** — same reason |

Four included, from **two independent repositories**. `57f18ff9` and `f5c18299` remain two executions of
one runbook, as sprint:9 established; they are included as separate recordings for envelope
characterization, and that dependence is carried in the report rather than forgotten.

No recording is manufactured. The cuecraft session is read in place, from the location log:1 records, and
nothing from it is committed.

#### G. What this task will not do

No repair, modification, normalization, symmetrization, replacement, or adoption of the statistic, and no
choice among pooling constructions. No treatment of the corpus as ground truth, and no threshold tuned on
it. No new search procedure. No combining of the two classifications. No change to the incumbent
selector, production behaviour, the representation, or any existing expectation. No recording content —
prompts, responses, commands, file contents, or absolute paths — in any artifact. Nothing pushed.

## Result

Delivered. **L1** for accumulation and **S1** for asymmetry — reported separately, as required, and each
on its own preregistered criteria.

The two sentences:

> The accumulation defect is **not** an asymptotic curiosity. Both L1 clauses fire independently: the
> corpus contains **13 actual crossings** produced by unmodified machinery, and **two of four
> recordings — from two independent projects** — hold both ingredients for a constructed one.
> The asymmetry is nonzero on **118 of 118** real candidate pairs, and it moves the designated pick in
> 3 of 29 candidate sets.

### 1. Corpus inventory

Six recordings across two established locations; four included.

| recording | project | records | observed | span | decision |
|---|---|---|---|---|---|
| `8b68dece` | witnessglass | 234 | 169 | 1054 s | **included** |
| `57f18ff9` | witnessglass | 39 | 32 | 141 s | **included** |
| `f5c18299` | witnessglass | 40 | 33 | 113 s | **included** |
| `7d95c414` | **cuecraft** | 106 | 77 | 859 s | **included** |
| `c3afa0ca` | witnessglass | 1 | 1 | 0 s | excluded — a lone `session_ended`; no vocabulary, no span |
| `6a8a02cc` | cuecraft | 1 | 1 | 0 s | excluded — same |

**The corpus grew since sprint:15**, and that is why this round could be run at all. log:1's addendum
records a real Claude session in an external project, so the study covers **two independent
repositories** rather than one. Nothing was manufactured; the cuecraft session is read in place and
nothing from it is committed.

**One dependence carried rather than forgotten:** `57f18ff9` and `f5c18299` are two executions of one
runbook (sprint:9). They are counted as two recordings for envelope characterization and as one source
of idiom, and both sit far from the accumulation surface anyway, so the dependence does not carry the
result.

### 2. The empirical operating envelope

Observed scope, mechanically derived counts only.

| session | events | vocabulary | max count | max freq | singletons | median freq |
|---|---|---|---|---|---|---|
| `8b68dece` | 169 | 14 | **64** | 0.3787 | 5 | 0.0296 |
| `57f18ff9` | 32 | 16 | 6 | 0.1875 | 7 | 0.0625 |
| `f5c18299` | 33 | 17 | 6 | 0.1818 | 9 | 0.0303 |
| `7d95c414` | 77 | 12 | **29** | 0.3766 | 4 | 0.0260 |

The two larger sessions, from **different projects**, have almost identical shape: a single delivered
tool name carrying ~38% of all observed events, its completion carrying nearly as much, and a long tail.

```text
8b68dece   64 (0.3787)  tool_requested/Bash     7d95c414   29 (0.3766)  tool_requested/Bash
           64 (0.3787)  tool_succeeded/Bash                27 (0.3506)  tool_succeeded/Bash
            7 (0.0414)  tool_requested/Read                 5 (0.0649)  subagent_stopped
```

That two unrelated repositories land within 0.002 of each other on the top mark's frequency is the
single most useful number in this round: the envelope is not one project's idiom.

### 3. The analytical failure surfaces

**Accumulation.** A single agreement on a mark of count `c₁` outscores `k` agreements each on marks of
count `c` exactly when `c^k / c₁ > N^{k−1}`, so for a singleton `c₁ = 1` the boundary is
**`c > N^{(k−1)/k}`**.

**sprint:15's carried-forward estimate, recomputed from source rather than inherited:** it said
*N ≈ 169, commonest count ≈ 64, boundary ≈ 47*. Measured: `N = 169` exactly, commonest count `= 64`
exactly, `169^{3/4} = 46.94`. **The estimate was correct** and is now a measurement. A test pins the
formula and that value.

**Asymmetry.** `score(A,B) − score(B,A) = Σ over agreeing marks of ln((ĉ_B(m)/N_B)/(ĉ_A(m)/N_A))`, which
is zero only when every agreeing mark holds the same relative frequency in both recordings.

### 4. Accumulation exposure: **L1**

Both clauses of the preregistered L1 test fire, independently.

**Clause one — the corpus contains actual crossings.** Over 29 candidate sets produced by the unmodified
`cross_pairs` at sprint:9's frozen ladder, **13 crossings** appear in **6 distinct sets**: a candidate
with strictly fewer agreements outscoring one with strictly more.

```text
8b68dece × 57f18ff9  k=4     3 agreements scored  9.888, beating 4 agreements at 6.434   (+3.455)
8b68dece × 57f18ff9  k=6     4 agreements scored 12.202, beating 5 agreements at 9.954   (+2.248)
8b68dece × 7d95c414  k=12   10 agreements scored 14.809, beating 11 agreements at 10.681  (+4.128)   ← largest
```

Nothing was built to provoke these. They are what the frozen machinery already returns on real
recordings.

**Clause two — the parameter values are present.** Per-recording, against the boundary at each `k` the
machinery actually produces:

| session | k=3 | k=4 | k=5 | k=6 | k=8 | k=12 |
|---|---|---|---|---|---|---|
| `8b68dece` boundary | 30.6 | 46.9 | 60.6 | 71.9 | 89.0 | 110.2 |
| `8b68dece` rel. margin | **2.09** | **1.37** | **1.06** | 0.89 | 0.72 | 0.58 |
| `7d95c414` boundary | 18.1 | 26.0 | 32.3 | 37.3 | 44.7 | 53.6 |
| `7d95c414` rel. margin | **1.60** | **1.12** | 0.90 | 0.78 | 0.65 | 0.54 |
| `57f18ff9` rel. margin | 0.60 | 0.45 | 0.37 | 0.33 | 0.29 | 0.25 |
| `f5c18299` rel. margin | 0.58 | 0.44 | 0.37 | 0.33 | 0.28 | 0.24 |

`8b68dece` is **above** the boundary at `k = 3, 4, 5` and holds 5 singletons; `7d95c414` is above at
`k = 3, 4` and holds 4. Both are marked constructible. **The L2 test — `c_max ≤ ½·N^{(k−1)/k}` for every
`k` in every recording — fails by a factor of four at its worst.**

**Closest approach in each direction**, reported as margins rather than as a label:

```text
closest crossing from above:  8b68dece  k=5   boundary 60.6, max count 64   margin +3.4   ratio 1.06
closest approach from below:  7d95c414  k=5   boundary 32.3, max count 29   margin −3.3   ratio 0.90
```

At `k = 5` the two large recordings sit on opposite sides of the surface by about three counts each. The
operating envelope does not merely approach this boundary; it straddles it.

**The two claims, kept distinct as §C requires.** *The corpus contains configurations that cross the
surface* — yes, 13 of them, observed. *The corpus contains parameter values from which an adversarial
candidate could be constructed* — also yes, in two recordings from two projects. The first is the
stronger finding and it does not depend on the second.

### 5. Asymmetry exposure: **S1**

**The intent was settled in the preregistration, on design grounds, before measuring.** The relation is
*"a window in one recording and a window in another agree more than chance"* — a claim about an unordered
pair. `cross_pairs` gives the two recordings no distinct evidential roles; it requires only that they
differ. sprint:13 symmetrized `surprisal` for exactly this reason, calling the asymmetric form a
different question. No artifact in this project documents a directional reading.

**So the classification is S1 regardless of magnitude** — and the magnitude was measured anyway, because
it is decision-relevant to sequencing a repair.

Over **118 real candidate pairs** from the unmodified machinery, across all six unordered recording
pairs at the frozen ladder:

```text
delta = 0        0 of 118   (0.0%)
quantiles        min 0.016   q1 0.247   median 0.851   q3 1.642   p90 2.396   max 4.082   nats
largest          f5c18299 × 7d95c414, k=4, span 4, 4 agreements
                 forward 11.907   backward 15.989   delta 4.082
```

**Not one pair is symmetric.** The median discrepancy of 0.851 nats is a factor of 2.3 in likelihood; the
maximum of 4.082 nats is a factor of 59.

**And it changes the answer, not merely the number:**

```text
designated pick changed in   3 of 29 candidate sets
pairwise orders reversed    27 of 195   (13.8%)

  8b68dece × 57f18ff9  k=6    forward picks #3, backward picks #1
  8b68dece × f5c18299  k=6    forward picks #2, backward picks #0
  8b68dece × 7d95c414  k=8    forward picks #3, backward picks #4
```

Which window pair a reader would be shown as *the* candidate depends on which recording was passed
first. Those are the minimized real examples §5 asked for; each is a real pair of real recordings at a
frozen ladder rung.

**This is not classified as acceptable on the strength of a small magnitude**, which §E forbade in
advance — and the magnitude is not small.

### 6. Failure surface against operating envelope

| | **accumulation / length dependence** | **A/B asymmetry** |
|---|---|---|
| **mechanism** | each agreement adds `−ln(c/N)`, so a candidate scores `k·ln N − Σ ln cᵢ` and ordering between different `k` depends on `N` | `score` reads `ĉ_A, N_A` only, so exchanging the arguments changes the claim |
| **synthetic boundary** | `c > N^{(k−1)/k}` for a singleton against a `k`-motif | any two recordings with differing relative frequencies on the agreeing marks |
| **empirical range** | `N ∈ [32, 169]`, `c_max ∈ [6, 64]`, `c_max/N ∈ [0.18, 0.38]`, singletons 4–9 | 118 candidate pairs across 6 recording pairs |
| **closest approach / crossing** | **crossed**: 13 observed crossings; closest straddle at `k=5`, ratios 1.06 above and 0.90 below | **crossed everywhere**: 0 of 118 symmetric; max delta 4.082 nats |
| **decision relevance** | reorders real candidates by up to 4.128 nats | moves the designated pick in 3 of 29 sets |
| **epistemic status** | **L1** — observed and reachable in the current envelope | **S1** — invariance defect by intent, and empirically large |

### 7. Regressions, unchanged

sprint:15's adversarial gauntlet — all ten families identical, including the one FAIL and six MIXED.
sprint:12's gauntlet — all fourteen rows identical, pinned incumbent rows intact. sprint:14's enumeration
— identical, both probes still clean across all seven, `rarity_of_agreements` still at `0.6931` on
Family E. **The frozen statistic retains its known failures**, which is what a regression suite is for.

### 8. Conclusions, kept apart

**Accumulation: L1.** Observed and reachable in the current operating envelope. A repair round is
warranted before adoption can be discussed. Not L2 — the L2 margin test fails by a factor of four. Not
L3 — four recordings from two projects, 29 candidate sets, and 13 observed crossings is not insufficient
evidence.

**Asymmetry: S1.** The relation is intended symmetric, so argument-order dependence requires eventual
repair regardless of magnitude — and here the magnitude is also large and decision-relevant. Not S2: no
artifact documents a directional reading and none was inferred to rescue the formula. Not S3: the intent
question is settled, on design grounds recorded before the data.

**Neither is combined with the other**, and neither is a verdict on the statistic's correctness, which
this corpus cannot establish and this round did not attempt.

### 9. Desire-path friction

**Eleventh consecutive round with the preregistration in a `###` subsection.** `8b76fa5` contains nothing
else. **idea:5** — and log:1's addendum independently made the same observation from the other side,
noting that its prediction is legible as predating its result "only because the two sit in the same
artifact in written order, which is precisely the property idea:5 wants to stop depending on good
manners." Two independent occurrences in one day.

**The corpus is now multi-repository and the archaeology has no place to say so.** This round's inventory
spans two projects, and which recordings exist, where they live, and which project produced them is
durable project knowledge that currently lives only inside a task Result. log:1 records the commissioning
and the addendum records the session; the *corpus as an inventory* is recorded nowhere. Noted as
first-occurrence friction; not promoted, since one round is not a pattern.

**Appending a Result is still `cat >>`** — `scarp` 0.2.0, version lag, maintenance:1.

**One thing that went well.** Reading the repository before trusting the prompt caught that HEAD had
moved and that a fourth real recording existed. The round's most useful single number — two unrelated
projects agreeing to within 0.002 on their top mark's frequency — exists only because of that check.

### 10. Strongest limitation

**Four recordings is an envelope, not a distribution.** Two of them are runbook siblings and two are the
only substantial sessions available, so "the envelope straddles the boundary at `k=5`" rests on two
independent data points. A fifth recording could move the picture, and nothing here estimates how much.

Secondly, **the observed crossings are crossings of the *statistic's* ordering, not of any known truth.**
That a 3-agreement candidate outscores a 4-agreement one is a fact about `rarity_of_agreements`; whether
the 3-agreement candidate is *worse* is exactly what this corpus cannot say. The round measures exposure
to a known misordering mechanism, and stops there.

### 11. The narrowest justified next experiment

**A preregistered repair round for the accumulation defect — and only that one.**

L1 makes it a blocker, and the repair has a well-posed question that this round is forbidden from
answering: how should scores over different numbers of agreements be made comparable, without becoming a
rate (which sprint:14 measured failing the `informative` family) and without a coefficient fitted to a
gauntlet? That round should preregister its candidates before measuring and be scored against **both**
existing gauntlets plus the crossings banked here.

**The asymmetry repair is deferred behind it, deliberately.** S1 makes it required, but the accumulation
defect reorders candidates by up to 4.128 nats against asymmetry's 4.082 and is the one that makes the
score incomparable *within* a single argument order. Fixing symmetry first would produce a statistic that
is consistently wrong in both directions. **This round does not choose the symmetrization mechanism**,
per §G.

### What this task did not do

No repair, modification, normalization, symmetrization, replacement, or adoption of the statistic; no
choice among pooling constructions. No treatment of the corpus as ground truth, and no threshold tuned on
it. No new search procedure — every candidate came from `cross_pairs` at sprint:9's frozen ladder. No
recording manufactured. No combining of L and S. No change to the incumbent selector, production
behaviour, the representation, or any existing expectation; all three regression suites reproduce
identically. No recording content — prompt, response, command, file content, or absolute path — appears
in this artifact, from either repository. Nothing pushed.
