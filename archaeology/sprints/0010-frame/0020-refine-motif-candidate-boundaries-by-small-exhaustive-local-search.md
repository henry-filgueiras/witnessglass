---
id: tsk_01KZ9VQEE3FZ6MEG639Z03QP7B
sequence: 20
kind: task
status: closed
sprint: spr_01KZ9VQEDS3B4Z66483RNNRFKN
created: 2026-08-05
closed: 2026-08-05
---

# Refine motif candidate boundaries by small exhaustive local search

## Objective

Implement the smallest deterministic local boundary-refinement procedure, run it against three
preregistered specimens with three different evidentiary roles, and report whether adjusting a
candidate's boundaries buys anything the fixed window did not already have.

Everything below the `### Preregistration` heading was written and committed to disk **before any
refinement was run**. What *was* inspected first is stated in §4, because specimen selection required
reading committed fixture indices and task:19's committed Result.

## Acceptance criteria

- The metric frozen and the freeze verifiable by diff.
- Exhaustive enumeration over the preregistered neighbourhood, deterministic in ranking and in ties.
- Unequal-length refined spans supported and reported.
- The anti-collapse policy implemented as preregistered, with the negative control run.
- Full decomposition and boundary deltas per seed and per candidate.
- The three specimens run, the stability probe run if refinement succeeds and stays cheap.
- A verdict against the criteria below, and a criterion-feasibility check recorded.
- A small visualization generated from computed output, or a recorded reason it was skipped.
- `scripts/check.sh` passes unweakened; no existing test changed; nothing pushed.

### Preregistration

#### 1. The frozen metric

Unchanged from task:18 and task:19, and verifiable by `git diff` over
`src/experiment/event_sequence.rs`: `MarkedEvent`, `Mark`, `align`, `timing_term`, `SUBSTITUTION`
(1.0), `GAP` (1.0), `TIMING_WEIGHT` (0.5), `TIMING_FLOOR_MS` (100), `TIMING_RATIO_FULL` (4),
`event_norm`, `timing_norm`, `total`, and the observed channel scope.

Three things are explicitly **deferred, not fixed**:

- `tool_requested/Agent → subagent_started` is partly a deterministic adapter emission. task:19
  recorded it; this round does not exclude it and does not adjust for it.
- `distinct_marks` stays a diagnostic and does not become a definition of motifhood.
- Timing stays in. Removing it would make boundary refinement look better and would answer a
  different question.

#### 2. The local search

`refine(a, span_a, b, span_b, radius)` enumerates every combination of four independent boundary
offsets in `[−radius, +radius]`:

```text
a.start + i,  a.end + j,  b.start + p,  b.end + q       i, j, p, q ∈ [−R, +R]
```

**R = 3.** `(2·3+1)⁴ = 2401` combinations per seed, each an `O(k²)` alignment — microseconds, so
brute force is a feature rather than a compromise. R is 3 rather than 2 because task:19's ladder
evidence says the independent-real seed needs a `(+2, −2, +2, −2)` adjustment, and a radius that
exactly equals the required adjustment would put the known answer at the corner of the search space.
At R = 3 it is interior, so the search can overshoot and has to choose not to.

A combination is **valid** when both spans lie inside their sequences, `start < end`, and both satisfy
the length floor below. Invalid combinations are skipped, not clamped — clamping would silently
re-enter the neighbourhood from outside it.

No gradient, no optimizer, no dynamic programming. Spans on the two sides move independently, so
refined lengths may differ; the alignment already carries insertions and deletions and both lengths
are reported.

#### 3. Anti-collapse: a floor, and a frontier

**The degeneracy is a certainty, not a risk, and the algebra says so before anything runs.** A
one-event span has no within-window gap, so its timing component is structurally absent and `total`
reduces to `event_cost / 1` — exactly `0` when the two marks are equal. In recording A, 37.9% of all
events carry `tool_requested/Bash`, so a zero-distance one-event pair exists inside essentially any
neighbourhood. Unconstrained minimization does not merely risk collapse; it must collapse.

**Floor: each refined span must contain at least 3 events.** Chosen on a property of the metric rather
than by preference: three events is the shortest span at which the timing component has **two** gaps to
speak with, so it is the shortest span at which both channels of the metric — identity and timing —
carry more than a single comparison. Hand-checked degeneracies at each candidate floor, before running:

| floor | what remains degenerate |
|---|---|
| 1 | any shared mark scores exactly 0; timing structurally absent |
| 2 | `Bash req → Bash succ` occurs 64 times in A; one gap only |
| **3** | `Bash req → Bash succ → Bash req` — task:19 measured such a pair at **0.022** |

**The floor does not remove degeneracy, and this is stated in advance so the result is not read as if it
did.** It removes the arithmetically guaranteed collapse and nothing more.

**Primary reporting form: a Pareto frontier**, not a scalar. Candidates are non-dominated on two axes:

```text
total distance          ↓ lower is better
retained events         ↑ higher is better,  measured as min(len_a, len_b)
```

`min` rather than the sum, because the shorter side bounds how much figure is actually shared.
`distinct_marks` is **reported per candidate but is not a frontier axis** — adding axes only grows a
frontier, and a two-axis frontier is one a reader can check by eye. No regularization coefficient is
introduced anywhere.

**Designated pick, so criteria have something to name:** the frontier point with the greatest retained
events among those whose `total ≤ the seed's total`; if none qualifies, the frontier point with the
lowest `total`. Deterministic, coefficient-free, and it states a preference in words — *the longest span
that is at least as good as what we started with* — rather than in a constant.

#### 4. The specimens, and what was inspected to choose them

Read before writing this preregistration: the committed fixture generator constants, the observed-index
maps of the two synthetic fixtures and three real recordings, and task:19's committed Result. No
refinement was run and none existed.

**Specimen A — synthetic, known answer.** Legible oracle, observed scope, both sides from the same
recording. Not a cross-recording claim: a boundary-recovery test on a figure whose boundaries are
decided in `oracle`'s constants.

| | span | contents |
|---|---|---|
| truth A | `L[20..28)` | the first planted occurrence, exactly periodic region |
| truth B | `L[162..170)` | a planted occurrence in the jittered recurrence region |
| **seed A** | `L[18..30)` | truth plus 2 baseline events before and 2 of the next occurrence after |
| **seed B** | `L[160..172)` | truth plus 2 events of the previous occurrence and 2 of the next |

Contamination is 2 events on each side of each span; R = 3 contains it with room. The two sides are
drawn from regions with different timing character — region A exactly periodic, region B jittered by up
to 700 ms per instance start — so extension across an instance boundary carries a real timing cost. A
specimen whose both sides were exactly periodic would have made unlimited extension free.

**Specimen B — positive control, runbook pair.** `57f18ff9` × `f5c18299`, task:19's two executions of one
runbook. Their observed marks agree everywhere except positions 1–4, where one opens with a `Skill` call
and the other with a `Bash` call and a `Read` call.

| | span |
|---|---|
| **seed A** | `57f18ff9[2..12)` |
| **seed B** | `f5c18299[2..12)` |

Chosen because it straddles the one known divergence: positions 2, 3, 4 differ and 5 onward agree. A
known shared sequence inside surrounding context, which is exactly what §7B asks for.

**Specimen C — independent real, the one that motivated the round.** task:19's `k8-c3`, `8b68dece` ×
`57f18ff9`.

| | span | task:19 |
|---|---|---|
| **seed A** | `8b68dece[51..59)` | `ev 0.500 tm 0.431 tot 0.479`, 8 events |
| **seed B** | `57f18ff9[15..23)` | same pair |
| persistent core | `8b68dece[53..57)` ↔ `57f18ff9[17..21)` | `ev 0.000 tm 0.416 tot 0.113` at `k = 4` |

The core is `tool_requested/Agent → subagent_started → tool_requested/Bash → tool_succeeded/Bash`. The
required adjustment is `(+2, −2)` on both sides, interior to R = 3.

**Only these three.** No seed was tried and discarded, and none will be added after refinement runs.

#### 5. Criterion-feasibility check, on structure and algebra alone

No refinement output was consulted. Three findings, and the third changed a criterion before it was
written down.

**(a) Neighbourhood cardinality.** 2401 combinations per seed, of which the valid subset is smaller.
Every specimen's target adjustment is strictly interior. No criterion is unreachable for want of search
space.

**(b) The collapse degeneracy is arithmetic, not empirical.** §3. This is why a floor exists at all.

**(c) The normalization has a length preference, and its direction depends on the seed's own distance.**
With `total = (E + T) / (1.5L − 0.5)`, appending one *matching* event whose timing term costs `t` gives

```text
new total < old total   ⟺   t < 1.5 × old total
```

So a span already at distance 0 is worsened by any extension, while a span at distance `d > 0` is
*improved* by any extension whose added timing cost falls below `1.5d`. Two consequences, both recorded
before running:

- A criterion of the form "the pick equals the planted boundaries exactly" is **not safe**. On specimen A
  the truth sits at a small non-zero distance and the neighbouring events are a repetition of the same
  figure, so extension into equivalent repeated context may legitimately score better. The criterion is
  therefore written as *the planted boundaries appear on the frontier, and the pick contains them*.
- Absolute thresholds imported from earlier rounds are refused. task:19's Result already records that
  importing `0.05` from task:18 gave a criterion that was reachable but weak. Every criterion below is
  **relative** — `refined < seed`, or membership of a frontier, or containment of a known span.

Recorded as performed. This is the fourth round to run such a check and the second in which it changed
the preregistration before execution rather than after.

#### 6. Predictions

**P1 — synthetic.** The planted pair `L[20..28)` ↔ `L[162..170)` appears on specimen A's frontier, and
the designated pick contains it on both sides. Any extension beyond it is expected to lie in the
*repeated* direction — into the next or previous occurrence — because that is where the marks continue to
match, and not into the baseline, where they do not.

**P2 — positive control.** Both starts move right, past the divergence at positions 2–4, so both refined
spans begin at index 5 or later. The pick retains at least 3 distinct marks per side and its total does
not exceed the seed's.

**P3 — independent real, stated exactly.** The pick is `8b68dece[53..57)` ↔ `57f18ff9[17..21)`: the
persistent core, exactly. The prediction is exact rather than approximate because the algebra permits it
— both events adjacent to the core are substitutions on both sides, so any extension adds a full unit of
event cost, while shrinking below four events removes a zero-cost gap and shrinks the denominator faster
than the numerator. Seed total 0.479; predicted refined total ≈ 0.11.

**P4 — degeneracy, run as a negative control.** With the floor removed, specimen C collapses to a
one-event pair at total exactly 0.000.

**P5 — stability.** Three seeds for specimen C perturbed by one event — left edge moved in, left edge
moved out, right edge moved out — converge on refined A-spans that overlap `[53..57)` in at least 3 of
its 4 events.

#### 7. Verdict criteria, fixed before running

**Supported** requires all four:

- **A** — the planted boundary pair lies on the frontier, **and** the pick's spans contain the planted
  figure on both sides, **and** the pick's total is strictly below the seed's;
- **B** — the pick retains at least 3 distinct marks on each side and its total does not exceed the
  seed's;
- **C** — the pick's spans each overlap the persistent core, **and** the pick's total is strictly below
  the seed's;
- **no pick sits at the length floor with 2 or fewer distinct marks**, on any specimen.

**Falsified** if any of:

- specimen A's planted boundaries are absent from the frontier;
- two or more picks sit at the length floor with 2 or fewer distinct marks;
- no specimen's pick improves on its seed.

**Mixed** is anything else — including boundaries that are unstable under P5's probe, a frontier whose
points are indistinguishable in distance, a positive control that shrinks to the floor, or an
independent-real pick that overlaps the core but is dominated by something unrelated.

The negative control (P4) is **not** part of the verdict. It exists to show the policy is load-bearing.

#### 8. Stability probe

Run only if refinement succeeds and only for specimen C. Three perturbed seeds, one event each:
`A[52..59)`, `A[54..59)`, `A[51..60)`, with `57f18ff9[15..23)` held fixed. No sweep, no parameters.

#### 9. Visualization scope and budget

**One small static artifact, generated from computed output, or nothing.** The order of preference is
the round's: reuse existing machinery if genuinely easy, otherwise a standalone static HTML page,
otherwise SVG, otherwise ASCII and stop.

Budget: it must consume the same structures the experiment computed — a test asserts that the rendered
numbers are the computed numbers and not transcribed ones — and it must not become a framework. The
Behavioral Spectroscope is not touched. Seed and refined boundaries must be distinguishable, marks and
order inspectable, seed and refined distances both shown, synthetic truth shown where it exists, each
specimen labelled synthetic / positive control / independent real, and no figure given a semantic
workflow name.

**Hygiene:** the generator is committed; generated output over real specimens is not. task:19's policy
allows mechanically derived marks and timings, and the page carries exactly that and nothing else — but
a rendered file is a different kind of object from a paragraph in a Result, and the conservative choice
costs nothing here because the page regenerates from one command.

#### 10. What this task will not do

No global variable-length discovery, no arbitrary subsequence mining, no motif families, no corpus
accumulation, no hierarchical motifs, no semantic naming. No path, file, payload, edit-intensity,
intent, or embedding facet. No timing-policy change, no mark change, no adapter-ontology change. No
`MotifDiscoveryEngine`, no product CLI surface, no recording-format change, no dependency, no
visualization framework, no Spectroscope change. No real recording committed, copied, or reproduced.
The pre-existing `cargo build --examples` / `spectroscope.rs` `required-features` defect is unrelated
and is not repaired here. Nothing pushed.

## Result

Delivered. **Falsified** by the letter of the preregistered criteria — and the reason is not that the
search failed. The search recovered every answer it was pointed at. What failed is the rule for
choosing a point off the frontier, and the criterion that assumed the planted answer would be on one.

The one-sentence finding, which is worth more than the verdict:

> Local boundary refinement finds the right core on every specimen, and this metric has no scale at
> which a figure is complete — every frontier improves monotonically down to the floor, so nothing in
> the objective ever says *stop here*.

### 1. The metric did not move

`git diff 246a4e8..1982ba5 -- src/experiment/event_sequence.rs` adds `refine`, `RefinedCandidate`,
`Refinement`, `BoundaryDelta`, `pareto_frontier`, and two constants. `align`, `timing_term`,
`SUBSTITUTION`, `GAP`, `TIMING_WEIGHT`, `TIMING_FLOOR_MS`, `TIMING_RATIO_FULL`, `event_norm`,
`timing_norm`, `total`, `Mark`, `MarkedEvent`, and `project` are byte-identical to task:18. The
`Agent → subagent_started` adapter-emission question is deferred as preregistered, `distinct_marks`
stayed a diagnostic, and timing stayed in.

### 2. The search

Four independent boundary offsets in `[−3, +3]`; `7⁴ = 2401` combinations per seed, each an `O(k²)`
alignment; invalid combinations skipped rather than clamped. Enumeration cost is microseconds, which
is why brute force is the right shape here and not a compromise.

| specimen | scored | rejected |
|---|---|---|
| A synthetic | 2401 | 0 |
| B positive control | 1764 | 637 |
| C independent real | 2304 | 97 |

### 3. Specimen A — synthetic, known answer

Legible oracle, both spans from one fixture. Seed contaminated by two events on each side of each
span.

```text
  seed      A[18..30) len 12   B[160..172) len 12    ev 0.167  tm 0.165  tot 0.166
  pick      A[18..33) len 15   B[160..175) len 15    ev 0.133  tm 0.151  tot 0.139
  truth     A[20..28)          B[162..170)           on frontier: NO

   retained       A span       B span      ev      tm     tot   deltas
         18     [15..33)   [157..175)   0.278   0.276   0.277   -3 +3 -3 +3
         17     [16..33)   [158..175)   0.235   0.230   0.234   -2 +3 -2 +3
         16     [17..33)   [159..175)   0.188   0.208   0.194   -1 +3 -1 +3
         15     [18..33)   [160..175)   0.133   0.151   0.139   +0 +3 +0 +3   <- pick
         14     [19..33)   [161..175)   0.071   0.122   0.088   +1 +3 +1 +3
         13     [20..33)   [162..175)   0.000   0.091   0.029   +2 +3 +2 +3
         12     [20..32)   [162..174)   0.000   0.084   0.026   +2 +2 +2 +2
          9     [20..29)   [162..171)   0.000   0.078   0.024   +2 -1 +2 -1
```

**The planted left boundary is recovered exactly.** Between retained 14 and retained 13 the A start
moves from index 19 to index 20 — the planted boundary — and the event component drops from 0.071 to
**0.000** while the total falls by two thirds. Every frontier point from there down starts at exactly
20 on the A side and exactly 162 on the B side. The search found the left edge of the planted figure
without being told it existed.

**The planted pair is nevertheless absent from the frontier, and legitimately so.** Scored directly:

```text
  A[20..28) B[162..170)   the planted figure       ev 0.000  tm 0.087  tot 0.026
  A[20..29) B[162..171)   the same, plus one event ev 0.000  tm 0.078  tot 0.024
```

One more matching event scores strictly better, so the planted pair is dominated on both axes and is
correctly excluded. This is exactly the mechanism §5 of the preregistration derived — `new < old` iff
the added event's timing term is below `1.5 × old total` — applied to a fixture whose figure repeats
every eight events. Extension went in the *repeated* direction, as P1 predicted, and never into the
baseline.

**The pick is bad.** `A[18..33)` still carries both contaminating baseline events. The rule chose it
because it was the longest span scoring at or below the seed's 0.166, and the seed was bad enough to
make that bar meaningless.

### 4. Specimen B — positive control

Two executions of one runbook, seeded across their single known divergence at positions 2–4.

```text
  seed      A[2..12) len 10   B[2..12) len 10    ev 0.300  tm 0.251  tot 0.285
  pick      A[1..15) len 14   B[2..15) len 13    ev 0.214  tm 0.360  tot 0.243

   retained       A span      B span      ev      tm     tot   deltas
         15      [0..15)     [0..15)   0.267   0.393   0.289   -2 +3 -2 +3
         14      [0..14)     [0..14)   0.286   0.338   0.286   -2 +2 -2 +2
         13      [1..15)     [2..15)   0.214   0.360   0.243   -1 +3 +0 +3   <- pick
         12      [1..15)     [3..15)   0.143   0.360   0.194   -1 +3 +1 +3
         11      [4..15)     [4..15)   0.091   0.296   0.155   +2 +3 +2 +3
         10      [5..15)     [5..15)   0.000   0.319   0.099   +3 +3 +3 +3
          9      [5..14)     [5..14)   0.000   0.234   0.072   +3 +2 +3 +2
```

**P2 confirmed exactly.** Both starts move to index 5 — the first position at which the two runbook
executions agree — and the event component reaches **0.000** and stays there. Retained 12 is also
worth reading: it is the first point where the two sides take *different* starts (`A[1..)`, `B[3..)`),
which is the alignment absorbing the offset the divergence introduced. Refinement did not collapse:
the answer it converges on is a nine-to-ten-event figure with eight or nine distinct marks.

**The pick is bad again**, and for the same reason: 0.243 clears the seed's 0.285, so the longest such
point wins and the divergence stays in.

### 5. Specimen C — independent real

task:19's `k8-c3`, the seed whose core persisted while larger `k` degraded it.

```text
  seed      A[51..59) len 8   B[15..23) len 8    ev 0.500  tm 0.431  tot 0.479
  pick      A[48..60) len 12  B[12..24) len 12   ev 0.500  tm 0.391  tot 0.466

   retained       A span       B span      ev      tm     tot   deltas
         14     [48..62)     [12..26)   0.571   0.485   0.544   -3 +3 -3 +3
         13     [48..61)     [12..25)   0.538   0.442   0.508   -3 +2 -3 +2
         12     [48..60)     [12..24)   0.500   0.391   0.466   -3 +1 -3 +1   <- pick
         11     [48..59)     [12..23)   0.455   0.427   0.446   -3 +0 -3 +0
         10     [48..58)     [12..22)   0.400   0.461   0.419   -3 -1 -3 -1
          9     [48..57)     [12..21)   0.333   0.459   0.372   -3 -2 -3 -2
          7     [50..57)     [12..21)   0.333   0.435   0.331   -1 -2 -3 -2
          5     [53..58)     [17..22)   0.200   0.431   0.266   +2 -1 +2 -1
          4     [53..57)     [17..21)   0.000   0.416   0.113   +2 -2 +2 -2   <- the persistent core
          3     [54..57)     [18..21)   0.000   0.124   0.031   +3 -2 +3 -2
```

**P3 confirmed exactly, to the index.** Retained 4 is `A[53..57)` ↔ `B[17..21)` at `ev 0.000`,
`tot 0.113` — the four-event core task:19 observed persisting across its whole fixed-`k` ladder,
recovered from a seed at 0.479 by a search that was never told the core existed. The prediction named
those spans and that distance in advance.

**And the frontier does not stop there.** Retained 3, `A[54..57)` ↔ `B[18..21)`, scores **0.031** —
3.6 times better than the core — by discarding `tool_requested/Agent`, which is the rarest mark in
recording A at one occurrence in 169 events and the single most distinctive thing about the match.
The remaining three events are `subagent_started → tool_requested/Bash → tool_succeeded/Bash`, whose
gaps happen to agree closely.

That is the round's central finding, and it is a finding about the **objective**, not the search. The
metric has no notion of how much a mark is worth, so it will always trade away a rare mark for better
timing agreement among common ones.

### 6. The negative control, and a feasibility miss inside it

**P4 was unreachable as written, and the geometry says so.** From an 8-event seed at radius 3 the
shortest achievable span is `8 − 6 = 2` events, so the one-event collapse P4 predicted cannot occur
for specimen C at all. The preregistration's cardinality check counted combinations and did not check
the *reachable length range*. Recorded as a miss.

Demonstrated properly from a three-event seed, where length 1 is reachable:

```text
floor 1                                                floor 3 (preregistered)
   retained    A span      B span      tot                retained    tot
          3  [54..57)   [18..21)    0.031                        3  0.031
          2  [57..59)   [19..21)    0.002                        — removed
          1  [53..54)   [17..18)    0.000                        — removed
```

Retained 1 is a single `tool_requested/Agent` matched against a single `tool_requested/Agent`, at
distance **exactly zero**. The floor is load-bearing and the arithmetic collapse is real rather than
hypothetical. `tests/event_sequence.rs` pins both halves.

### 7. Stability probe

Four seeds for specimen C, differing by one event:

```text
  seed A[51..59)  ->  frontier terminal  A[54..57)  B[18..21)  tot 0.031
  seed A[52..59)  ->                     A[54..57)  B[18..21)  tot 0.031
  seed A[54..59)  ->                     A[54..57)  B[18..21)  tot 0.031
  seed A[51..60)  ->                     A[54..57)  B[18..21)  tot 0.031
```

Identical, to the index, from every perturbed seed. **P5 holds with room to spare** — the prediction
asked for 3-of-4 overlap and got exact agreement. Whatever else is wrong here, the search is not
fragile.

### 8. Verdict: **Falsified**

Against the criteria fixed at `246a4e8`:

| clause | outcome |
|---|---|
| **A** — planted pair on the frontier | **NO** — dominated by a nine-event extension, §3 |
| **A** — pick contains the planted figure | yes, `A[18..33) ⊇ [20..28)` |
| **A** — pick total below seed | yes, 0.139 < 0.166 |
| **B** — pick ≥ 3 distinct marks per side, total ≤ seed | yes, 9 and 10 marks, 0.243 ≤ 0.285 |
| **C** — pick overlaps the core, total below seed | yes, and 0.466 < 0.479 |
| no pick at the floor with ≤ 2 marks | yes, none |

Supported fails on A's first clause. And the Falsified clause **"specimen A's planted boundaries are
absent from the frontier"** fires directly. Neither the categories nor the clauses have been altered.

**What Falsified does and does not mean here.** It does not mean local boundary adjustment buys
nothing — §3, §4, and §5 are three recoveries of an intended answer that a fixed window could not
reach. It means the round's own criterion for "recovered" was written against an object, the frontier,
that provably cannot contain the planted pair whenever an adjacent event matches well enough. The
verdict is correct as a verdict on what was asked; the finding is in the sections above it.

### 9. The criterion defect, which is the fourth and the most instructive

sprint:6 §4 set a rank cutoff fixture combinatorics made unreachable. sprint:8 §9 did it again.
sprint:9 §9 imported a threshold from a round whose distances were an order of magnitude smaller. This
round did something worse and more interesting.

**The feasibility check derived the exact mechanism that invalidates the criterion, and the criterion
was written anyway.** §5 of the preregistration states, before anything ran, that appending a matching
event improves the total whenever its timing cost is below `1.5 ×` the current total. On a fixture
whose figure repeats every eight events, that implies the planted span is dominated by the planted
span plus one event — which implies it cannot be on the frontier. The check produced the disproof and
the conclusion drawn from it was "so do not require the pick to *equal* the truth" rather than "so the
truth will not be *on the frontier* either".

The reusable step is one line, and it is the missing end of the procedure this project has now run
four times: **after identifying a mechanism, apply it to every criterion already written, including
the ones the mechanism was not raised about.** Recorded as friction in §12. Not built into machinery.

### 10. The pick rule is wrong, and the frontier is why we can tell

The designated pick — *the longest span scoring no worse than the seed* — failed on all three
specimens, and failed the same way each time: a bad seed sets a low bar, so the longest barely-passing
span wins.

```text
              seed    pick    best on frontier    the answer the frontier contains
  A          0.166   0.139   0.024               planted left boundary at retained 13
  B          0.285   0.243   0.072               both starts at index 5, ev 0.000
  C          0.479   0.466   0.031               the persistent core at retained 4
```

The pick is between four and fifteen times worse than the frontier's own best on every specimen.

**This is the round's methodological result and it vindicates the design decision.** Reporting a
frontier instead of a scalar is what makes "the search works" and "the selection rule does not"
separable claims. A round that had reported `score = distance + λ · discarded` would have reported one
number per specimen, all three of them wrong, with no way to see that the search underneath was
finding exactly the right spans.

**Every frontier here is monotone to the floor.** There is no knee, no interior optimum, nothing that
distinguishes "the figure ends here" from "we have not thrown away enough yet". That is not a property
of these three specimens; it follows from the normalization, which divides by length and therefore
always pays for a badly-matching event and always rewards removing one.

### 11. Deliberately excluded, and visible in the failure

**Marginal mark frequency.** Specimen C's frontier trades `tool_requested/Agent` — one occurrence in
169 events — for closer timing agreement among `Bash` marks that occur 64 times each. An objective
that knew how surprising a mark is would not make that trade. This is *not* a new facet: mark
frequency is computed from the same marks the representation already carries, and task:19 already
reports it. It is the obvious next lever and it was not pulled.

**Deferred as preregistered:** the `Agent → subagent_started` adapter emission, which is two of the
four events in specimen C's recovered core. It is still half an artefact, and this round did not fix
it. Paths, payload sizes, edit magnitude, and intent were not consulted.

### 12. Desire-path friction

**Fifth consecutive round with the preregistration in a `###` subsection of `Acceptance criteria`.**
The evidence it predates the run is `246a4e8`, a commit containing nothing else. **idea:5**, fifth
occurrence.

**The criterion-feasibility check needs a closing step, and that is new.** The four previous defects
were *missing* checks or *mis-scaled* thresholds. This one is a check that succeeded and was then not
applied to its own siblings. The affordance that would have caught it is procedural rather than
tooling: the check should end by re-reading every criterion against each mechanism it found. Recorded
here rather than promoted, because it is a research-process discipline and no Scarp feature — sealed
sections included — would have caught it.

**Appending a Result is still `cat >>`** on this machine: `scarp` 0.2.0, version lag, maintenance:1
records upstream shipped result-on-close.

### 13. Strongest limitation

**The objective has no stopping rule, and this round has no candidate for one.** Every frontier
descends to the floor, and the floor is a guard chosen from the metric's structure rather than a
statement about where figures end. Refinement can therefore *propose* boundaries but cannot *choose*
them, which is exactly one half of the capability the round set out to acquire:

```text
"these two k-event windows look similar"                          <- had this
"there appears to be a recurring figure here"                     <- have this now, from the frontier
"and these may be its boundaries"                                 <- still missing
```

Secondly: three specimens, one of them synthetic and one a runbook pair, is an existence demonstration
and not a rate.

### 14. Recommendation: exactly one next experiment

**Evaluate the existing order null at every boundary combination, and ask whether a null-referenced
objective has an interior optimum where the raw one does not.**

The reason is §10: the frontier has no knee because `total` measures agreement without asking how
surprising that agreement is. A three-event span of two common marks agreeing well is unremarkable; a
four-event span containing a mark that occurs once in 169 events is not, and the current objective
cannot tell them apart. task:19's `order_null` already answers exactly that question — *what would a
span like this score if the ordering carried no information* — and running it per candidate turns the
frontier's y-axis from "distance" into "distance relative to chance".

It is the smallest possible next step: no new facet, no coefficient, no tuning, no new machinery — the
null already exists, the specimens already exist, and the same three seeds and the same frontier
reporting answer it. The falsifiable question is whether a null-referenced frontier has an interior
optimum at the planted boundaries on specimen A and at the four-event core on specimen C. If it does,
this project has a stopping rule. If it does not, boundary discovery needs richer marks, and that is
worth knowing before variable-length search is attempted at any scale.

**Not recommended yet:** general variable-length discovery. This round shows the search half is easy
and the deciding half is not, and mining arbitrary subsequences with an objective that cannot say when
to stop would produce a great many confident, wrong boundaries.

### 15. The visualization

Built, and small. `src/experiment/boundary_page.rs` renders one self-contained static page from the
specimen documents `event-motif --refine --json` produces. It holds no measurement: the fidelity test
runs a real refinement, serializes it, renders the page, and asserts every frontier total and both
spans appear in the output.

```text
cargo run --example event-motif -- --refine --recording <A> --against <B> \
    --seed-a 51..59 --seed-b 15..23 --label C --role "independent real" --json > c.json
cargo run --example event-motif -- --render out.html --from a.json --from b.json --from c.json
```

Seed, pick, and planted boundaries are three distinguishable bands against a shared event-index scale;
the frontier is a table with the pick and the planted row marked; marks are the delivered kind and
tool-name string, verbatim; each specimen is labelled synthetic, positive control, or independent
real; and no figure is given a workflow name. The Behavioral Spectroscope was not touched and nothing
is served.

**The generator is committed and its output is not.** A page over specimens B and C carries a real
recording's delivered marks and timings, and while task:19's policy permits that information in a
Result, a rendered file is a different kind of object. Regenerating it is one command. A
synthetic-only snapshot could be committed later if a rendered artifact in the repository is ever
wanted; that is a decision, not an oversight.

### What this task did not do

No global variable-length discovery, no arbitrary subsequence mining, no motif families, no corpus
accumulation, no hierarchical motifs, no semantic naming. No path, file, payload, edit-intensity,
intent, or embedding facet. No timing-policy change, no mark change, no adapter-ontology change. No
`MotifDiscoveryEngine`, no product CLI surface, no recording-format change, no dependency, no
visualization framework, no Spectroscope change. No existing test altered and no check weakened. No
real recording committed, copied, or reproduced; no absolute path, prompt, response, command, or file
content in this artifact. The pre-existing `cargo build --examples` / `spectroscope.rs`
`required-features` defect did not obstruct validation and was left alone. Nothing pushed.
