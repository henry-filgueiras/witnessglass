---
id: tsk_01KZA2DNQ48TTH1PSXZQVAPMVN
sequence: 24
kind: task
status: pending
sprint: spr_01KZA2DNPW9HXNQYWAKTBBG68T
created: 2026-08-05
---

# Audit whether Family E is identifiable from the mark-only representation

## Objective

Audit whether Family E's distinction is identifiable from the current mark-only representation:
formalize what a scorer can see, attempt a collision certificate, and — only if none exists — run a
preregistered enumeration of simple domain-neutral functions over that representation.

`### Phase A` was written and committed **before any enumeration code existed**. `### Phase B` was
written in the same commit, after Phase A settled the collision question and before any function was
evaluated.

## Acceptance criteria

- `R` formalized with its invariance.
- The collision question settled constructively, either way.
- If no collision: the function family preregistered here, evaluated against Family E and the whole
  gauntlet by the existing rule, with no function added or removed after evaluation.
- Incumbent, challenger, families, generation, expectations, and pass rule frozen; pinned outputs
  unperturbed.
- The `k = 0` defect preserved as a named regression test, separate from the representation result.
- `scripts/check.sh` passes unweakened; no existing test changed; nothing pushed.

### Phase A — the representation, and the collision question

#### A.1 What a scorer can see

A candidate is a pair of spans. Restricted to the current mark-only representation, a scoring function
receives exactly:

```text
R(candidate) = ( ā , b̄ , ĉ_A , ĉ_B , N_A , N_B )
```

- `ā = (ā₀ … ā_{L−1})` — span A's marks, in order;
- `b̄` — span B's marks, in order;
- `ĉ_A : mark → count` — how many times each mark occurs in the whole of recording A; likewise `ĉ_B`;
- `N_A`, `N_B` — the recordings' lengths.

**The invariance is what makes this a representation.** A mark is an opaque label: a scorer may test two
marks for equality and may look up a count, and may do nothing else with them — no parsing, no ordering,
no interpretation. So `R` is defined **up to a bijective relabelling of the mark alphabet applied
consistently** to `ā`, `b̄`, `ĉ_A`, and `ĉ_B`. Two candidates presenting relabelled-equal data present
the same `R`.

Equivalently, and more usefully: **`R` determines exactly the equality pattern of the two spans —
which positions share a mark, within each span and across the two — together with each occurring mark's
recording count and the two lengths.** Nothing else survives the relabelling.

Everything sprints 8 to 13 built is a function of `R` **except the incumbent `z`**, which also reads the
timing policy. The challenger `S` is a function of `R`. This round's enumeration stays inside `R`.

#### A.2 The collision attempt, and its outcome

A collision certificate would be two cases with `R(case A) = R(case B)` whose desired orderings differ.
None exists for Family E's arms, and the proof is constructive.

The two arms, as sprint:12 generates them:

```text
novel      ā = (x, y, z, w)     ĉ_A = {x:1, y:1, z:1, w:1, …background}
redundant  ā = (x, y, z, x)     ĉ_A = {x:2, y:1, z:1,      …background}
```

Their equality patterns differ in the first row of the within-span matrix — `[T,F,F,F]` against
`[T,F,F,T]` — and the equality pattern is relabelling-invariant. So the arms are **not** `R`-identical,
and the discriminator

```text
is_redundant(candidate)  =  ā_{L−1} ∈ { ā₀ … ā_{L−2} }
```

is a function of `R`: it tests only mark equality, never mark identity, and survives any relabelling.

**Conclusion of Phase A: no collision exists, and Family E's arms are identifiable from `R`.** The
failures in sprint:12 and sprint:13 were therefore *not* identifiability failures. Per the round's
instruction, the enumeration in Phase B is the required path.

#### A.3 A collision that does exist, for a different and stronger claim

Recorded here because it bounds what any `R`-based scorer can mean by the word, and because it was found
while attempting A.2 rather than after seeing any result.

Consider two candidates, both with `ā = b̄ = (x, y, z, x)` and identical recording counts:

- **P** — generated as a three-event core `(x, y, z)` extended by a repeat of `x` that adds nothing.
- **Q** — generated as a four-event figure whose defining property is *returning to `x`* after `y` and
  `z`; the final `x` is the informative event.

`R(P) = R(Q)` exactly — same spans, same counts, same lengths — while the desired orderings are
opposite. **So "redundant" in the semantic sense — *this event adds no information about the figure* —
is not a function of `R`.** What `R` determines is the equality pattern, and Family E's arms are
separable only because sprint:12 *defined* redundancy syntactically, as a repeat.

This is a statement about the word, not about Family E, and it does not license changing Family E.

### Phase B — preregistered enumeration

#### B.1 The family of functions

Ten, fixed here, all simple, all functions of `R`, all relabelling-invariant. Each maps a candidate to a
score where **higher means more evidence**. None is added, removed, or edited after evaluation.

| # | name | definition |
|---|---|---|
| 1 | `agreements` | `k` = number of positions where `āᵢ = b̄ᵢ` |
| 2 | `agreement_rate` | `k / L` |
| 3 | `distinct_agreements` | number of **distinct** marks among agreeing positions |
| 4 | `distinct_agreement_rate` | `distinct_agreements / L` |
| 5 | `span_distinct` | number of distinct marks in `ā` (sprint:8's diagnostic, as a score) |
| 6 | `first_occurrence_agreements` | agreeing positions `i` whose mark does not occur in `ā₀…ā_{i−1}` |
| 7 | `negative_repeats` | `−(L − span_distinct)` |
| 8 | `surprisal` | sprint:13's conditional match surprisal |
| 9 | `rarity_of_agreements` | `Σ over agreeing positions of −ln( ĉ_A(āᵢ) / N_A )` |
| 10 | `novel_rarity` | the same sum, restricted to first-occurrence agreeing positions |

**Functions 9 and 10 are probes, not candidates for adoption.** They are inverse-frequency weightings,
which sprints 12 and 13 forbid as *repairs*. They are admissible here because the question is whether
the representation carries the distinction, not which scorer to adopt, and excluding them would bias an
identifiability answer. Nothing in this round may propose either as a statistic.

#### B.2 How they are evaluated

Each function replaces `z` and `S` as the scored quantity, and **everything else is the frozen
machinery**: the same 300 trials, the same eight families, the same per-family quantities and directional
expectations, and the same single pass rule. For the dilution family the argmax is recomputed under each
function, as sprint:13 did for the challenger, rather than inheriting another statistic's answer.

Reported as a matrix: ten functions × seven scored families, with each cell's verdict, and with the
fraction and median available beneath it.

#### B.3 Predictions

**P1 — several functions separate Family E, and trivially.** `span_distinct`, `distinct_agreements`,
`first_occurrence_agreements`, `negative_repeats`, and `novel_rarity` all differ between the arms by
construction: the novel extension contributes a new distinct mark and the redundant one does not.
Phase A already proves the information is present; this predicts the obvious functions expose it.

**P2 — and they will not survive the rest of the gauntlet.** A function that rewards a new distinct mark
rewards a *noise* boundary equally, because a mismatched pair of background marks also adds distinct
marks. `span_distinct` is predicted to repair E and fail `noise`.

**P3 — the discriminating question is whether any single function passes all seven.** Predicted: **none
does.** The families pull in incompatible directions — `noise` demands that adding an unmatched event
not help, `informative` demands that adding a matched one does, `rare vs common` demands sensitivity to
background prevalence, and `redundant` demands insensitivity to a repeat. Predicted zero functions clean.

**P4 — the incumbent and the challenger keep their existing rows exactly.** They are in the matrix for
comparison and their numbers must not move; a test pins them.

**P5 — no function fails for want of definition.** All ten are total over equal-length spans; where a
candidate's spans differ in length, `agreements` and everything derived from it are undefined, exactly as
`surprisal` is, and the counts are reported per function.

#### B.4 What the round concludes, in each case

- **If some function passes all seven families**: the representation carries the distinction *jointly*,
  and the next question is whether that function survives an adversarial extension of the gauntlet — a
  different round, and the function is **not** adopted here.
- **If no function passes all seven** — the predicted case: the representation carries Family E's
  distinction *in isolation* (Phase A) but no simple function of `R` reconciles it with the other
  families. The deliverable is then the smallest additional domain-neutral observable that would, named
  and argued but **not implemented**.

#### B.5 The `k = 0` defect, kept separate

sprint:13's brute-force test found `agreement_tail` underflowing at `k = 0`, where Jordan's formula's
`C(j−1, k−1)` is undefined. The fix — returning `P(≥0) = 1` directly — was applied in sprint:13 and is
semantically unambiguous. This round adds a **named regression test** for it and nothing else. It does
not touch the experimental question, does not weaken the gate, and the pinned incumbent rows prove it
perturbed nothing.

#### B.6 What this task will not do

No new scoring statistic offered as a repair, no selector, no policy. No timing, path, payload, tool,
channel, adapter, schema, or agent observable. No implementation of whatever observable is recommended.
No change to the incumbent, the challenger, the families, their generation, their expectations, or the
pass rule. No weakening or reinterpretation of Family E. No corpus, no variable-length discovery, no
fourth real specimen, no product CLI surface, no dependency, no Spectroscope change. No real recording
committed, copied, or reproduced. Nothing pushed.
