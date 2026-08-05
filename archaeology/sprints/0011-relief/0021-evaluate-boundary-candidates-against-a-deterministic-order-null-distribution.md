---
id: tsk_01KZ9XFS9REDP6BNFCMKMXJVWF
sequence: 21
kind: task
status: pending
sprint: spr_01KZ9XFS9ED6GPVPF0K7HZGWC9
created: 2026-08-05
---

# Evaluate boundary candidates against a deterministic order-null distribution

## Objective

Evaluate every boundary candidate from task:20's three specimens against a deterministic distribution
of order-null realizations, report several null-relative statistics side by side without choosing one,
and show the resulting geometry.

Everything below the `### Preregistration` heading was written and committed to disk **before any
candidate's null distribution was computed**. What *was* run first is the throughput benchmark in §4,
which measures the machinery and no specimen.

## Acceptance criteria

- task:20's machinery frozen except the null's seed, verifiable by diff.
- The order null reused, its extension to multiple realizations explicit and tested, and its meaning
  unchanged for the single-realization call task:19 makes.
- A deterministic null distribution per candidate at the preregistered realization counts and scopes.
- Every statistic in §6 reported per candidate, with degenerate cases handled honestly rather than
  hidden.
- Predictions and verdict criteria fixed below; the six-question feasibility review answered per
  prediction, before committing.
- The task:20 page extended so the geometry is visible, consuming computed output, with a test.
- `scripts/check.sh` passes unweakened; no existing test changed; nothing pushed.

### Preregistration

#### 1. Frozen

Unchanged from task:20 and verifiable by `git diff` over `src/experiment/event_sequence.rs`:
`MarkedEvent`, `Mark`, `align`, `timing_term`, `SUBSTITUTION` (1.0), `GAP` (1.0), `TIMING_WEIGHT`
(0.5), `TIMING_FLOOR_MS` (100), `TIMING_RATIO_FULL` (4), `event_norm`, `timing_norm`, `total`,
`project`, `refine`, `pareto_frontier`, `REFINE_RADIUS` (3), `LENGTH_FLOOR` (3), the observed channel
scope, and the three specimens with their seed spans:

| specimen | role | seed A | seed B |
|---|---|---|---|
| **A** | synthetic, known planted answer | legible oracle `[18..30)` | legible oracle `[160..172)` |
| **B** | positive control, two runs of one runbook | `57f18ff9[2..12)` | `f5c18299[2..12)` |
| **C** | independent real | `8b68dece[51..59)` | `57f18ff9[15..23)` |

No boundary is moved on the strength of this round's output.

#### 2. The order null, restated exactly

Read from the committed implementation rather than from memory. `order_null` today:

- **permutes** the `mark` field across every event of one sequence, by fixed-seed Fisher–Yates;
- **preserves** every `gap_from_previous_ms` and every `offset_ms`, exactly where they were;
- **preserves** the sequence's whole mark multiset, its length, and its channel scope;
- **drops** each event's `sequence` receipt, because a permuted mark is not what that record carried;
- is applied to **one** sequence and produces **one** realization.

**The extension, stated so it cannot be mistaken for a redefinition:** the seed becomes a parameter.
`order_null_seeded(sequence, seed)` does exactly the above with a caller-supplied seed, and
`order_null(sequence)` becomes `order_null_seeded(sequence, ORDER_NULL_SEED)` with task:19's original
constant. A test asserts the two agree, so task:19's null keeps its exact meaning and remains
realization zero of this one.

**Both sides are nulled, independently.** A candidate is a pair of spans, and the question is how
surprising their agreement is; nulling only one side would ask a different question. Each sequence
gets its own derived seed so the two permutations are never the same.

**Timing does not travel with the marks.** Gaps stay at their positions, so a nulled span keeps the
observed span's exact timing and receives randomised identity. This is a property of task:19's null,
not a choice made here, and §5 shows it has a consequence worth knowing in advance.

**Nulls are shared across candidates, not regenerated per candidate.** One ensemble of `N` nulled
`(A, B)` pairs per specimen; every candidate is scored against the same ensemble. This is what makes
comprehensive evaluation affordable, and it means candidates within a specimen are compared against
the same null world rather than against `N` different ones.

#### 3. Deterministic seed policy

```text
realization r of sequence side s ∈ {0, 1}:
    seed = ORDER_NULL_SEED  ^  (r as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
                            ^  (s as u64).wrapping_mul(0xD1B5_4A32_D192_ED03)
```

Two odd 64-bit constants, XORed into task:19's own seed. No ambient RNG, no clock, no thread state.
The same recording, candidate, realization count, and scope reproduce the same evidence byte for byte,
and a test pins it.

#### 4. Realization counts, benchmarked before being chosen

Measured on this machine, release build, before any specimen was evaluated:

```text
1000 whole-sequence order nulls (186 events)   460 µs
100k alignments, span length  4                 14.5 ms   (145 ns each)
100k alignments, span length  8                 36.3 ms   (363 ns each)
100k alignments, span length 12                 70.1 ms   (701 ns each)
100k alignments, span length 18                140.5 ms  (1405 ns each)
```

Two scopes, both fixed here:

| scope | candidates | realizations | cost per specimen |
|---|---|---|---|
| **geometry** | every valid candidate in the radius-3 neighbourhood (≈2400) | **1 000** | ≈ 2400 × 1000 × 700 ns ≈ **1.7 s** |
| **resolution** | the Pareto frontier only (≈8–10 candidates) | **10 000** | ≈ 10 × 10000 × 700 ns ≈ **0.07 s** |

Comprehensive evaluation is preferred and is affordable, so the geometry scope takes every candidate
rather than a sample. The resolution scope exists because §5's feasibility review found that the
central prediction needs a finer tail than 1/1000 can resolve — see there. Both counts are fixed
before any specimen runs and are not revisited.

Ensemble memory at N = 1000 is roughly 1000 × 200 events × ~40 bytes ≈ 9 MB, which is why the
realizations are materialized once and shared.

#### 5. Criterion-feasibility review — six questions per prediction

Performed on structure, cardinality, and algebra alone. No candidate's null distribution was computed.
It found one thing that **changed a parameter** and one that **changed a criterion**, both before
commitment.

**The mechanism, derived first.** Under this null a candidate's span keeps its observed gaps and
receives permuted marks. For a pair of equal-length spans, an alignment with zero substitutions and
zero indels is a positional match, and in that case the timing cost is *exactly* the observed timing
cost, because the gaps never moved. Therefore:

```text
null_total ≤ observed_total    ⟺    the null realization's marks match positionally
```

whenever the observed candidate has `event_norm = 0` and `observed_total < 1/(1.5L − 0.5)` — which
holds for every exact-agreement candidate here, since one substitution alone costs
`1/(1.5L − 0.5)` and that exceeds every observed total in question. **So the empirical tail
probability of an exact-agreement candidate is exactly the probability of positional mark agreement
under the null**, which is approximately `c^L` where `c = Σ_m p_A(m)·p_B(m)` is the marginal collision
probability of the two recordings.

That single fact answers most of the review:

| question | answer |
|---|---|
| 1. Is the claimed candidate reachable? | Yes for all three specimens: A's planted span and its extensions, B's `[5..)` agreement spans, and C's four-event core are all inside the radius-3 neighbourhood and above the floor — task:20 found every one of them. |
| 2. Is the comparison candidate reachable? | Yes: C's three-event suffix is on task:20's frontier at retained 3. |
| 3. Does the construction permit the predicted ordering? | Yes, and it predicts it: `c^4 < c^3`, so the four-event core is strictly rarer under the null than the three-event suffix. P3 follows from the mechanism rather than from hope. |
| 4. Does fixture repetition dominate the "correct" span? | **Yes, on specimen A, and it defeats the obvious criterion.** task:20 established that the planted span is dominated by the planted span plus one repeated event. The same repetition means `c^13 < c^8`, so surprise will *prefer* the longest exact match over the planted one. A criterion requiring the planted span to maximize any statistic is therefore invalid and is **not** written. §7's A-clause asks instead whether the most exceptional candidate *begins at the planted left boundary*, which is reachable and which task:20 showed the search does find. |
| 5. Does candidate length make the null degenerate? | Partly. Estimating `c` from the two recordings' marginals gives `c ≈ 0.13` for specimen C, so `c³ ≈ 2.4e-3` and `c⁴ ≈ 3.1e-4`. At N = 1000 the expected counts are ~2 and ~0.3 — too coarse to separate. **This is why the resolution scope uses N = 10 000**, where expected counts are ~24 and ~3. The parameter was chosen by this check, not after seeing results. |
| 6. Is the requested threshold meaningful given N? | With the conservative finite-sample estimator `p̂ = (1 + count)/(1 + N)`, N = 10 000 gives a floor of `1e-4`, below `c⁴`. Zero-count candidates are reported as *at or below the floor* rather than as `p = 0`, and the standardized separation does not saturate at all. No criterion below rests on a rank or an inherited absolute threshold. |

**No absolute threshold is inherited from any previous round.** Every criterion in §7 is a comparison
between two candidates of the same specimen under the same ensemble.

#### 6. Statistics reported, none of them preregistered as the score

For every evaluated candidate, the existing `event_norm`, `timing_norm`, `total`, retained count,
distinct marks, and exact boundaries are kept unchanged. Added beside them, computed for **both** the
combined `total` and the `event_norm` component — the second because §5's mechanism lives in identity
agreement and hiding it inside the combined figure would repeat task:18's mistake:

- `null_mean`, `null_stddev`, `null_min`, `null_max`;
- `at_or_below` — the raw count of realizations with `null ≤ observed`;
- `empirical_p` — `(1 + at_or_below) / (1 + realizations)`, the conservative finite-sample estimator;
- `separation` — `null_mean − observed`;
- `standardized_separation` — `(null_mean − observed) / null_stddev`, reported as **absent** when
  `null_stddev` is zero rather than as infinity;
- a fixed **20-bin histogram over `[0, 1]`**, so a distribution can be drawn rather than summarized.

Samples are not retained per candidate; the histogram plus the five summary numbers is what travels.
**None of these is the motif score, and no output ranks candidates by any of them alone.**

#### 7. Predictions

**P1 — synthetic.** Null-relative evidence orders candidates differently from raw distance, and
specifically does not prefer the shortest. Given §5 question 4, the *predicted* preference is for the
longest exact-agreement span rather than for the planted one; the planted left boundary should still
be where exact agreement begins. A swing from "shortest wins" to "longest wins" is a real finding and
is **not** a solution.

**P2 — positive control.** The most exceptional candidate retains at least six events, so the
null-relative view keeps a nontrivial shared runbook figure rather than the shortest allowed
agreement.

**P3 — independent real, the round's central prediction.** The four-event core `8b68dece[53..57)` ↔
`57f18ff9[17..21)` is strictly more exceptional than the three-event suffix `[54..57)` ↔ `[18..21)`
under the empirical tail probability, despite scoring 3.6× worse on raw distance.

**P4 — raw monotonicity survives.** The raw frontier is unchanged from task:20, candidate for
candidate. Null evidence is added beside it and alters nothing about it.

**P5 — an interior optimum is not guaranteed.** Null-relative evidence may be monotonic or flat. If
the geometry contains no knee, that is recorded as the result and no optimum is manufactured.

#### 8. Verdict criteria

**Supported** requires all four:

- **C** — the four-event core's `empirical_p` is strictly below the three-event suffix's, under the
  resolution scope;
- **A** — the most exceptional candidate on the synthetic specimen begins at the planted left
  boundary on both sides (`A` index 20, `B` index 162);
- **B** — the most exceptional candidate on the positive control retains at least six events;
- and all of it without any change to the alignment metric or the representation.

**Falsified** if any of:

- the three-event suffix is at least as exceptional as the four-event core under the empirical tail
  probability;
- the most exceptional candidate sits at the length floor on two or more specimens;
- null distributions are degenerate — zero variance — on two or more specimens;
- the statistics cannot order candidates at all.

**Mixed** is anything else, including agreement on C with failure on A or B, statistics that disagree
with one another, or neighbouring spans that remain indistinguishable.

Where the reported statistics disagree, the disagreement is reported and not resolved. `empirical_p`
is named in the criteria because §5 derives its behaviour in advance; naming it is not a claim that it
is the right score, and §10 forbids building a selector on it.

#### 9. Visualization plan and budget

Extend `src/experiment/boundary_page.rs`. Not a new page, not a framework, no dependency, inline
SVG and CSS only.

Per specimen, added to what task:20 already renders:

- **two series against retained length**, longest to shortest: raw `total`, and null-relative surprise.
  The visual question is whether raw similarity keeps improving while surprise peaks somewhere richer.
- **known-answer overlays**, using the words the round is allowed to use: `planted figure` on the
  synthetic specimen, and on the independent-real one `previously observed core` and
  `raw-distance-preferred suffix`. Neither of the latter is called a true motif.
- **the candidate table gains** `null mean`, `null stddev`, `empirical p`, `separation`, and
  `standardized separation`, all from computed values.
- **one null distribution drawn**, not summarized: the 20-bin histogram for the frontier's marked
  candidates, with the observed value marked on the same axis.

Fidelity: a test renders a real evidence document and asserts the page carries its computed numbers.

Budget: if this grows beyond a sidecar to the existing page, it stops and the round reports the
geometry as text. Hygiene is task:19's and task:20's — the generator is committed, output over real
specimens is not, and a page carries only mechanically derived marks, timings, and statistics.

#### 10. What this task will not do

No selector, no motif score, no `choose_motif`, no ranking by a null statistic. No inverse-frequency,
TF-IDF, `−log p(mark)`, entropy, mutual-information, or learned weighting. No new facet, no mark
change, no timing change, no removal of adapter emissions, no boundary moved on this round's output.
No variable-length discovery, no motif families, no corpus, no fourth specimen. No product CLI
surface, no stable public statistical API, no recording-format change, no dependency, no new page. No
real recording committed, copied, or reproduced. The unrelated `cargo build --examples` /
`spectroscope.rs` `required-features` defect stays out of scope. Nothing pushed.
