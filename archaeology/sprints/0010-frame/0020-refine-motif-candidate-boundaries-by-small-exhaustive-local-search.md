---
id: tsk_01KZ9VQEE3FZ6MEG639Z03QP7B
sequence: 20
kind: task
status: pending
sprint: spr_01KZ9VQEDS3B4Z66483RNNRFKN
created: 2026-08-05
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
