---
id: tsk_01KZA5SR41C9H7H7NMR6JNJVES
sequence: 26
kind: task
status: pending
sprint: spr_01KZA5SR3F4HX6ZQY7XC8T8DE9
created: 2026-08-05
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
