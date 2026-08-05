---
id: tsk_01KZA2DNQ48TTH1PSXZQVAPMVN
sequence: 24
kind: task
status: closed
sprint: spr_01KZA2DNPW9HXNQYWAKTBBG68T
created: 2026-08-05
closed: 2026-08-05
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

## Result

Delivered. **The representation is sufficient.** Family E is identifiable from `R`, by proof and then by
measurement, and no additional observable is needed — so this round has no `φ` to propose, which is the
opposite of the outcome it was commissioned to expect.

The three findings, in order of how much they change the picture:

> 1. **Family E's arms are not `R`-identical.** No collision certificate exists for them; the equality
>    pattern separates them and a one-line function of `R` computes it. sprint:12's and sprint:13's
>    failures were never identifiability failures.
> 2. **Two of the ten preregistered functions pass all seven families cleanly** — against P3, which
>    predicted none would. Both are the ones sprints 12 and 13 forbade as repairs.
> 3. **A collision does exist for the stronger claim**, with a minimal witness: *semantic* redundancy is
>    not a function of `R`. Family E is answerable only because sprint:12 defined redundancy
>    syntactically.

### 1. The representation available to a scorer

```text
R(candidate) = ( ā , b̄ , ĉ_A , ĉ_B , N_A , N_B )
```

the two spans' marks in order, each mark's count over the whole recording it came from, and the two
recording lengths — **up to a bijective relabelling of the mark alphabet applied consistently to all
four**, because a mark is an opaque label a scorer may compare for equality and look up, and nothing
else. Equivalently: `R` determines the equality pattern of the two spans, plus each occurring mark's
count and the two lengths.

`Observation` is that value, and it is closed: nothing a scorer receives can reach a timestamp, a path,
a payload, a channel, an adapter, a schema version, an agent, or the text of a tool name. A test asserts
every preregistered function is invariant under renaming the entire alphabet, which is the property that
makes each one a function of the representation rather than of the data behind it.

### 2. No collision certificate exists for Family E

Constructively. The arms' equality patterns differ in the first row of the within-span matrix:

```text
novel      ā = (x, y, z, w)   within-span row 0 = [T, F, F, F]   ĉ_A = {x:1, y:1, z:1, w:1, …}
redundant  ā = (x, y, z, x)   within-span row 0 = [T, F, F, T]   ĉ_A = {x:2, y:1, z:1,      …}
```

and the discriminator

```text
is_redundant(candidate)  =  ā_{L−1} ∈ { ā₀ … ā_{L−2} }
```

tests only mark equality, never mark identity, so it survives any relabelling. It is therefore a
function of `R`, and it separates the arms exactly.

**Family E is identifiable from the current representation.** Pinned by
`family_es_arms_differ_inside_the_representation`.

### 3. The enumeration, since no collision existed

Ten preregistered functions, the frozen 300 trials, the frozen families and expectations, the single
pass rule. Nothing added, removed, or edited after evaluation.

```text
  function                      informative        noise         rare    redundant   accidental      diluted    competing   clean
  agreements                           PASS         PASS         FAIL         FAIL         PASS         PASS         PASS      no
  agreement_rate                       FAIL         PASS         FAIL         FAIL         PASS         PASS         FAIL      no
  distinct_agreements                  PASS         PASS         FAIL         PASS         PASS         PASS         PASS      no
  distinct_agreement_rate              FAIL         PASS         FAIL         PASS         PASS         PASS        MIXED      no
  span_distinct                        PASS         FAIL         FAIL         PASS         PASS         PASS         PASS      no
  first_occurrence_agreements          PASS         PASS         FAIL         PASS         PASS         PASS         PASS      no
  negative_repeats                     FAIL         PASS         FAIL         PASS        MIXED         PASS        MIXED      no
  surprisal                            PASS         PASS         PASS         FAIL         PASS         PASS         PASS      no
  rarity_of_agreements *               PASS         PASS         PASS         PASS         PASS         PASS         PASS     YES
  novel_rarity *                       PASS         PASS         PASS         PASS         PASS         PASS         PASS     YES
```

Family E's column in detail:

| function | trials | frac | median | verdict |
|---|---|---|---|---|
| agreements | 30 | 0.000 | 0.0000 | FAIL |
| agreement_rate | 30 | 0.000 | 0.0000 | FAIL |
| distinct_agreements | 30 | 1.000 | 1.0000 | PASS |
| distinct_agreement_rate | 30 | 1.000 | 0.2000 | PASS |
| span_distinct | 30 | 1.000 | 1.0000 | PASS |
| first_occurrence_agreements | 30 | 1.000 | 1.0000 | PASS |
| negative_repeats | 30 | 1.000 | 1.0000 | PASS |
| surprisal | 30 | 0.333 | −0.0000 | FAIL |
| **rarity_of_agreements** | 30 | 1.000 | **0.6931** | **PASS** |
| **novel_rarity** | 30 | 1.000 | 4.1297 | PASS |

**Six of ten separate Family E**, confirming §2 empirically. **P1 held; P2 held** — `span_distinct`
repairs E and fails `noise`, for exactly the predicted reason: a function that rewards a new distinct
mark rewards a mismatched noise boundary too. **P3 was wrong.**

### 4. Why the two probes pass, and why it is not luck

`rarity_of_agreements` scores Family E at a median of exactly **0.6931 = ln 2** — the same quantity
sprint:13 derived from the falling factorial and then lost to cancellation. It does not cancel here, and
the reason is precise and mechanical:

```text
without replacement (sprint:13's permutation null)
    the 2nd copy of x is the 2nd of 2 available   → contributes (2−1) = 1, same as a fresh mark
    so the core and the extension are discounted equally and the ln 2 cancels in the delta

with replacement (an i.i.d. model with the empirical marginals)
    every draw of x costs −ln(2/N) regardless of how many were drawn before
    so the repeated position costs ln 2 less than a fresh one, and the ln 2 survives
```

`rarity_of_agreements` is the negative log-likelihood of the agreeing marks under an i.i.d. multinomial
with the recording's own marginals. **It is a surprisal under a different null, not a hand-added
weight** — and the two nulls disagree precisely about what a repeat costs.

That relocates the whole problem. sprint:13 concluded that Family E's question was not answerable by "a
statistic that is a function of a span and a recording's marginals, scored as a nested delta". That
conclusion was **too strong**: it is true of *without-replacement* permutation nulls and false of
*with-replacement* ones, and sprint:13 did not distinguish the two because it only ever considered
permutations. Recorded as a correction to sprint:13's Result, which stands unedited.

The other six families pass for coherent reasons rather than by accident: a mismatched boundary
contributes no agreeing position so `noise` sees `Δ = 0`; a common boundary mark has a large count so
`rare vs common` sees the difference of two logs; the competing family's rare six-event core outscores
its common three-event one on both count and length.

### 5. And a collision that does exist, for a stronger claim

Found while attempting §2 and recorded in the preregistration before any function ran. Two candidates:

```text
P   ā = b̄ = (x, y, z, x)   counts {x:2, y:1, z:1, bg:40}   a core (x,y,z) plus a repeat that adds nothing
Q   ā = b̄ = (x, y, z, x)   counts {x:2, y:1, z:1, bg:40}   a figure whose defining property is returning to x
```

`R(P) = R(Q)` exactly — same spans, same counts, same lengths — and the desired orderings are opposite.
`the_witness_pair_is_identical_inside_the_representation` asserts the equality and then asserts that all
ten functions return identical values for both, which they must.

**So *semantic* redundancy — "this event adds no information about the figure" — is not a function of
`R`.** What `R` determines is the equality pattern. Family E's arms are separable only because sprint:12
defined redundancy syntactically, as a repeat. This bounds what any `R`-based scorer can mean by the
word, and it is **not** grounds to change Family E, which stands exactly as written.

### 6. The strongest conclusion the evidence supports

> **Family E is identifiable from the current representation** — by construction, and by six of ten
> ordinary functions of it. The representation is not the limiting factor and needs no additional
> observable. What sprint:13 diagnosed as a limit of "marginal-based statistics scored as nested deltas"
> is in fact a limit of **sampling without replacement**: the permutation null charges a repeated mark
> nothing extra because the recording already contains the copy, and an i.i.d. null over the *same*
> representation charges it in full. Two preregistered functions built on the second null pass all seven
> families cleanly.
>
> A second, weaker representation does fail: *semantic* redundancy is not `R`-determined, with a witness.
> Family E is answerable because its notion of redundancy is syntactic.

### 7. No `φ` is proposed, and that is the finding

task:24 §B.4 committed to naming a minimum additional observable **only if** the representation proved
insufficient. It did not. Proposing one anyway would be answering a question the evidence closed.

**What the next experiment should be instead**, stated as a question rather than a construction: the two
functions that pass are inverse-frequency weightings, which sprints 12 and 13 forbade as repairs — a
constraint written when the only null under consideration was a permutation. This round shows that
constraint excluded the only null that can see a repeat. **Whether to lift it is an adjudication for the
human, not a decision for this round**, and it is listed under uncertainties rather than acted on. If it
is lifted, the round that follows should put `rarity_of_agreements` through an adversarial extension of
the gauntlet built against *its* failure modes rather than the permutation null's — because passing a
gauntlet designed against a different statistic is weak evidence, and sprint:12 exists to make that point.

Nothing here adopts either probe. They remain flagged as probes in the code, and a test pins that flag.

### 8. The `k = 0` defect, kept separate

sprint:13's brute-force check found `agreement_tail` underflowing at `k = 0`, where Jordan's formula's
`C(j−1, k−1)` is undefined. The fix — returning `P(≥ 0) = 1` — was applied in sprint:13 and is
semantically unambiguous.

This round adds `agreeing_in_at_least_zero_positions_is_certain_rather_than_an_underflow`, a named
regression test that checks the value is exactly 1 across several spans and that it still agrees with
brute force at `k = 0`, which is how it was found. **It changed no experimental question, weakened no
check, and perturbed no pinned output** — `adding_the_challenger_left_the_incumbents_numbers_exactly_
where_they_were` still holds sprint:12's seven rows to `5e-4`.

### 9. Counterexamples and near-failures

**`negative_repeats` is MIXED on `accidental` and on `competing`**, and it is the one function whose
failures are interesting rather than structural: it scores only repetition and is blind to agreement, so
a chance match between independent streams that happens to contain no repeats outscores a planted core
that does. That is the mirror image of Family E's problem and it is why a repetition-only function is not
a candidate for anything.

**`agreement_rate` and `distinct_agreement_rate` fail `informative`** because dividing by span length
makes adding a matching event neutral or negative. Rates are the wrong shape for a question about
*extending* a span, which the enumeration exposes at no cost.

**No function failed for want of definition.** P5 held: all ten are total over equal-length spans and all
ten decline together across an indel, asserted by test.

### 10. Desire-path friction

**Ninth consecutive round with the preregistration in a `###` subsection.** `e5cdf47` contains nothing
else. **idea:5**.

**The round needed a place for a proof, and there wasn't one.** Phase A is not a prediction — it is a
constructive argument whose conclusion determined which of two experiments the round ran. It sits in the
same `###` block as the preregistered function family, so the record cannot distinguish *this is what we
proved* from *this is what we committed to before looking*. sprint:13 noted the same gap for analysis;
this is its second occurrence and the objects are now three: analysis, proof, and prediction, all sharing
one undifferentiated subsection.

**Appending a Result is still `cat >>`** — `scarp` 0.2.0, version lag, maintenance:1.

**One thing that went well.** The relabelling-invariance test is what makes this round an audit rather
than an assertion: it mechanically enforces that every function is a function of `R`, so "restricted to
the mark-only representation" is checked rather than promised.

### 11. Strongest limitation

**Ten functions is not a proof of anything about functions in general.** §2's collision argument is a
proof about identifiability; the enumeration is an existence result — *these* two pass — and its
non-results are not impossibility claims. A function outside the ten might pass without being an
inverse-frequency weighting, and this round cannot say it does not.

Secondly, **the gauntlet the two probes passed was built against the permutation null's failure modes**,
not against theirs. That is the weakest link in the round's strongest finding, it is why §7 declines to
propose either as a statistic, and it is exactly the objection sprint:12 was commissioned to raise.

### What this task did not do

No new scoring statistic offered as a repair, no selector, no policy, and neither passing probe adopted.
No timing, path, payload, tool-semantic, channel, adapter, schema, or agent observable. No implementation
of any additional observable — none is proposed, because none is needed. No change to the incumbent, the
challenger, the families, their generation, their expectations, or the pass rule; the pinned incumbent
rows are unmoved. No weakening or reinterpretation of Family E, which this round explains and leaves
alone. No corpus, no variable-length discovery, no fourth real specimen, no product CLI surface, no
dependency, no Spectroscope change. No real recording committed, copied, or reproduced. Nothing pushed.
