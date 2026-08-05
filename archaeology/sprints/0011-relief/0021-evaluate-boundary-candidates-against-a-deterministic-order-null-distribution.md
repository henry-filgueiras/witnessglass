---
id: tsk_01KZ9XFS9REDP6BNFCMKMXJVWF
sequence: 21
kind: task
status: closed
sprint: spr_01KZ9XFS9ED6GPVPF0K7HZGWC9
created: 2026-08-05
closed: 2026-08-05
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

## Result

Delivered. **Supported**, and the central prediction held with a margin the mechanism predicted in
advance.

The one-line finding:

> The four-event core is 4.3× rarer under the null than the three-event suffix that raw distance
> prefers — and standardized separation has an interior maximum at exactly that core, over all 2304
> candidates, without being told it existed.

The finding that matters as much: **the three statistics disagree sharply**, and one of them —
unstandardized separation — reproduces raw distance's bias exactly. Naming a score in advance would
have had a one-in-three chance of being right.

### 1. Frozen, and the one extension

`git diff bff44d6..5646595 -- src/experiment/event_sequence.rs` leaves `align`, `timing_term`, the
five constants, `event_norm`, `timing_norm`, `total`, `Mark`, `MarkedEvent`, `project`, `refine`,
`pareto_frontier`, `REFINE_RADIUS`, and `LENGTH_FLOOR` byte-identical. `refine`'s enumeration moved
into `enumerate_candidates` and `refine` now calls it — behaviour unchanged, and task:20's
thirty-three tests pass untouched, which is the assertion that says so.

**The extension:** `order_null_seeded(sequence, seed)`, with `order_null(s) =
order_null_seeded(s, ORDER_NULL_SEED)` at task:19's own constant. A test asserts the two agree, so
task:19's null keeps its exact meaning and is realization zero of this one.

### 2. The null construction, exactly

Per realization, per side: marks permuted by fixed-seed Fisher–Yates; every `gap_from_previous_ms` and
`offset_ms` left where it was; mark multiset, length, and channel scope preserved; receipts dropped.
Both sides nulled independently. One ensemble per specimen, shared by every candidate.

```text
seed(realization r, side s) = ORDER_NULL_SEED
                            ^ r · 0x9E37_79B9_7F4A_7C15
                            ^ s · 0xD1B5_4A32_D192_ED03
```

No ambient RNG, no clock, no thread state. A test checks for collisions **after** the generator's
`seed | 1` normalization, because two seeds differing only in bit 0 drive the same stream and would
have put duplicate realizations in the ensemble unnoticed. Twenty thousand realizations × two sides:
no collision.

### 3. Scope and cost

| scope | candidates | realizations | measured |
|---|---|---|---|
| geometry | every valid candidate (2401 / 1764 / 2304) | 1 000 | ≈ 2.5 s per specimen |
| resolution | the Pareto frontier (8 / 7 / 10) | 10 000 | ≈ 2.1 s per specimen |

Both preregistered from the benchmark, both run comprehensively as preferred. Total ≈ 14 s for three
specimens in a release build.

### 4. The feasibility review's mechanism, confirmed empirically

The preregistration derived, before anything ran, that because the null permutes marks and leaves gaps
in place, a null realization reaches an exact-agreement candidate's observed total **precisely when its
marks match positionally** — so the empirical tail probability of such a candidate is the probability
of positional mark agreement, roughly `c^L`.

The measurement bears it out. On specimen C the frontier's exact-agreement candidates:

```text
retained 4   emp-p 7.0e-4        c⁴ predicted ≈ 3.1e-4
retained 3   emp-p 3.0e-3        c³ predicted ≈ 2.4e-3
```

Same order of magnitude, same ratio direction, from marginals computed by hand in advance. And the
review's two consequences both landed: the resolution scope was raised to 10 000 *because* 1 000 could
not separate `c³` from `c⁴`, and the synthetic criterion was written around fixture repetition rather
than against it.

### 5. Specimen C — independent real, the central prediction

Frontier at 10 000 realizations:

```text
   retained       A span       B span     tot  null-mu  null-sd    emp-p      sep        z
         14     [48..62)     [12..26)   0.544    0.706    0.055  5.50e-3    0.162     2.94
         13     [48..61)     [12..25)   0.508    0.696    0.058  3.20e-3    0.188     3.22
         12     [48..60)     [12..24)   0.466    0.682    0.062  2.00e-3    0.216     3.48
         11     [48..59)     [12..23)   0.446    0.691    0.066  9.00e-4    0.245     3.73
         10     [48..58)     [12..22)   0.419    0.701    0.070  4.00e-4    0.282     4.05
          9     [48..57)     [12..21)   0.372    0.700    0.075  3.00e-4    0.328     4.39
          7     [50..57)     [12..21)   0.331    0.636    0.071  1.00e-4    0.304     4.27
          5     [53..58)     [17..22)   0.266    0.721    0.108  1.10e-3    0.455     4.23
          4     [53..57)     [17..21)   0.113    0.718    0.123  7.00e-4    0.605   * 4.91
          3     [54..57)     [18..21)   0.031    0.674    0.147  3.00e-3    0.643     4.37
```

**The core against the suffix, under every reported statistic:**

| | 4-event core `[53..57)`↔`[17..21)` | 3-event suffix `[54..57)`↔`[18..21)` | who wins |
|---|---|---|---|
| raw `total` | 0.113 | **0.031** | suffix, by 3.6× |
| `event_norm` | 0.000 | 0.000 | tie |
| `timing_norm` | 0.416 | **0.124** | suffix |
| `null_mean` | 0.718 | 0.674 | — |
| `null_stddev` | 0.123 | 0.147 | — |
| `empirical_p` | **7.0e-4** | 3.0e-3 | **core, by 4.3×** |
| `separation` | 0.605 | **0.643** | suffix |
| `standardized_separation` | **4.91** | 4.37 | **core** |

**P3 confirmed.** The core is strictly rarer under the null despite scoring 3.6× worse on raw
agreement. Two of the four null statistics prefer it; two do not.

### 6. Specimens A and B

**A — synthetic**, frontier at 10 000: `emp-p` is at the floor `1.0e-4` for **all eight** candidates —
completely saturated and useless here. `separation` rises monotonically to retained 9 (0.563). `z` has
an interior maximum at **retained 13, `A[20..33)` ↔ `B[162..175)`** — which begins at exactly the
planted left boundary, index 20 and index 162.

```text
   retained    tot  null-mu  null-sd    emp-p      sep        z
         18  0.277    0.613    0.050  1.00e-4    0.336     6.75
         15  0.139    0.596    0.057  1.00e-4    0.457     8.04
         13  0.029    0.582    0.063  1.00e-4    0.553   * 8.85
         12  0.026    0.584    0.066  1.00e-4    0.557     8.45
          9  0.024    0.587    0.079  1.00e-4    0.563     7.15
```

**P1 confirmed as written**, including its prediction that surprise would prefer the longest exact
match rather than the planted span itself — a swing, not a solution, and predicted as such.

**B — positive control:** `emp-p` saturated at the floor for all seven. `z` peaks at **retained 10,
`A[5..15)` ↔ `B[5..15)`** — the longest span in which the two runbook executions agree exactly,
starting at the first index past their divergence. **P2 confirmed**: ten events, well above the six
the criterion asked for.

### 7. The geometry scope: the argmax over every candidate

The frontier is 8–10 points. Over the **whole neighbourhood** at 1 000 realizations:

| specimen | candidates | `max z` | `max separation` | `min emp-p` |
|---|---|---|---|---|
| A | 2401 | `A[20..33) B[162..175)` ret 13 — **starts at the planted boundary** | `A[21..27)` ret 6 | 1914 of 2401 tied at the floor |
| B | 1764 | `A[5..15) B[5..15)` ret 10 — **the agreement span** | `A[5..9)` ret 4 | 1086 of 1764 tied at the floor |
| C | 2304 | `A[53..57) B[17..21)` ret 4 — **the previously observed core, exactly** | `A[54..57)` ret 3 | 38 of 2304 tied at the floor |

**Standardized separation's global argmax is the meaningful span on all three specimens**, chosen from
thousands of candidates with no anchor, no ground truth passed in, and no metric change. On the
independent-real specimen it is the four-event core task:19 observed and task:20 recovered.

### 8. Does an interior optimum appear? Yes — in exactly one statistic

**Yes for `z`**, on all three specimens: 2.94 → 4.91 → 4.37 on C, 6.75 → 8.85 → 7.15 on A,
8.25 → 9.81 → 9.26 on B. Each rises with shortening spans, peaks, and falls. task:20's finding was
that *every* frontier descended monotonically to the floor with no knee; there is now a knee.

**No for `separation`**, which is monotonic toward short spans on all three and therefore reproduces
raw distance's bias. **No for `emp-p`**, which saturates at the ensemble floor on two specimens.

**Why `z` has the knee, stated so it is not mistaken for magic.** `z = separation / null_stddev`, and
the null's standard deviation *grows* as spans shorten — 0.055 at retained 14 to 0.147 at retained 3
on specimen C. Short spans match by chance more variably, so their good agreement is less exceptional.
The knee is where separation stops outgrowing that variance. That is a mechanism, not a discovery, and
it is the reason `z` behaves differently from `separation` rather than a reason to trust it.

### 9. Do the statistics agree? No, and that is the round's second finding

```text
                    specimen A        specimen B        specimen C
raw total           shortest          shortest          shortest
separation          shortest-ish      shortest-ish      shortest
empirical p         saturated         saturated         core (4.3x)
standardized sep    planted boundary  agreement span    the core
```

Three null-relative statistics computed from the same distribution point in three directions. Had this
round preregistered one as *the* motif score, it would have had a one-in-three chance of picking the
one with useful geometry — and `separation` is the most obvious choice a person would reach for.

**This is why §10 forbade writing a selector**, and the prohibition held: nothing in this round ranks
candidates by any of these, and no `choose_motif` exists.

### 10. Degenerate behaviour, reported rather than hidden

**`empirical_p` saturates badly.** 1914 of 2401 candidates on specimen A and 1086 of 1764 on B sit at
the ensemble floor at N = 1 000, and every frontier candidate on both sits at the floor at N = 10 000.
The statistic cannot order candidates on either specimen. It is reported at the floor as *rarer than
this ensemble can resolve*, which is what the evidence supports, rather than as `p = 0`.

**No zero-variance null occurred on any specimen.** Standard deviations ran 0.049–0.147 throughout. The
path is tested anyway: a fixture whose every observed record carries one mark produces a point-mass
null, and `standardized_separation` is `None` there rather than infinity.

**No NaN or infinity reaches serialization or the page**, asserted by test at both finite-sample edges.

### 11. Verdict: **Supported**

| clause | outcome |
|---|---|
| **C** — core's `empirical_p` strictly below the suffix's | 7.0e-4 < 3.0e-3 ✓ |
| **A** — most exceptional candidate begins at the planted left boundary | ✓ under `z` (retained 13) and under `separation` (retained 9); undefined under saturated `emp-p` |
| **B** — most exceptional candidate retains ≥ 6 events | ✓ 10 under `z`, 9 under `separation` |
| no change to metric or representation | ✓ by diff |

No Falsified clause fires: the suffix is not at least as exceptional as the core; the most exceptional
candidate is at the length floor on **one** specimen only (C, under `separation`) and the clause needs
two; no specimen had a degenerate null; and two of three statistics order candidates fine.

**One drafting imprecision, recorded rather than smoothed.** Clauses A and B say "the most exceptional
candidate" without naming which statistic makes a candidate most exceptional. On specimen A three
statistics give three answers. Both clauses hold under both non-saturated statistics and are undefined
under the saturated one, so the ambiguity did not bite — but it is an under-specification of the same
family as the four previous defects, and it is the fifth. §13.

### 12. What the null adds beyond raw similarity

**It supplies the quantity task:20 found missing, and it does so without being told anything about
rarity.** No `−log p(mark)`, no inverse-frequency weight, no entropy term, no change to the alignment.
The rarity enters through the permutation: a mark occurring once in 169 events almost never lands in
your window under the null, so a span containing it is hard to match by chance. That is why the core
beats the suffix, and it is stronger evidence than a weighting invented to produce that answer would
have been — the null was built in a different round for a different question.

**And the interpretation this round is required to keep separate.** The core is
`tool_requested/Agent → subagent_started → tool_requested/Bash → tool_succeeded/Bash`, and task:19
recorded that the first two are a deterministic adapter emission. The finding here is that the core is
**statistically distinctive**. It is not a finding that it is **behaviourally meaningful**, and the
adapter-emission question stays deferred exactly as preregistered. A representation in which the
rarest mark is an artefact of how events are written down will make artefacts look exceptional, and
that is now the most urgent open question in this line of work.

### 13. Desire-path friction

**The section-template refusal returned, for the first time in three rounds.** `scarp new sprint`
rejected a `## A distinction this sprint must keep` heading — sprints have exactly Goal, Rationale,
Success criteria, Non-goals. One failed command, fold the material into Rationale as a bolded
paragraph, move on. Already **idea:2**; recorded as a fourth occurrence across the project.

**Sixth consecutive round with the preregistration in a `###` subsection.** `bff44d6` contains nothing
else. **idea:5**.

**The fifth criterion imprecision, and it is a new shape again.** Not a disproved criterion, not an
inherited threshold, not an unapplied mechanism — an under-specified one: "the most exceptional
candidate" without naming the statistic, in a round whose whole point was that several statistics
exist. The closing step task:20 recommended (re-read every criterion against each mechanism found)
would not have caught this; what would is asking, of every criterion, *which computed quantity does
this sentence name?* Recorded, not built.

**Appending a Result is still `cat >>`**: `scarp` 0.2.0 on this machine, version lag, maintenance:1.

### 14. Strongest limitation

**`z` has a knee for a reason that is about variance, not about meaning.** The null's spread grows as
spans shorten because short spans have fewer positions to disagree in. That makes `z` a *length-aware*
statistic almost by accident, and its interior maximum could be a length preference wearing a
statistical costume. The evidence against that reading is that the maximum lands on the meaningful
span on all three specimens rather than at a fixed length — 13, 10, and 4 events respectively — but
three specimens cannot distinguish "finds the figure" from "prefers a length that happens to be right
three times".

Secondly, and it follows from §12: the null measures surprise *given the recording's marginal
vocabulary*, so anything that makes a mark rare makes spans containing it exceptional — including an
adapter that emits a mark exactly once per subagent launch.

### 15. Recommendation: exactly one next experiment

**Test whether `z`'s interior maximum survives when the rare mark is removed from the representation.**

Rebuild specimen C's sequences with `subagent_started`, `subagent_stopped`, and `tool_requested/Agent`
excluded — the three adapter-lifecycle marks task:19 flagged — recompute the same frontier and the same
null evidence, and ask whether `z` still peaks at a four-event span, peaks somewhere else, or loses its
knee entirely.

It is the smallest experiment that separates the two readings §14 leaves open. If the knee survives, the
geometry is about behaviour and the adapter emission was incidental. If it collapses, then what this
round found is that adapter artefacts are statistically distinctive — which is a real and useful
finding, and it would make representation the next problem rather than statistics. Either way it costs
one flag, no new machinery, no new specimen, and no metric change.

**Not recommended:** an information-theoretic weighting. The null already supplies rarity, and adding a
`−log p(mark)` term now would answer the same question twice while making it impossible to tell which
mechanism produced the answer.

### 16. The visualization

`src/experiment/boundary_page.rs`, extended rather than replaced: two stacked SVG panels over a shared
axis — raw `total` and standardized separation against retained length, longest span on the left — with
every evaluated candidate as a faint dot, the frontier highlighted, and the planted figure, a
caller-supplied marked span, and the raw-distance-preferred span drawn on top. Below them the evidence
table with all five statistics, and below that each marked candidate's null distribution drawn as a
20-bin histogram with the observed value marked on the same axis.

The raw panel slopes down to the right on all three specimens. The surprise panel rises and then falls,
with the green marker at its apex on specimen C. The two panels are the round's result, side by side.

**One defect found by looking at the page and not by reading the source**: the x axis plotted short
spans on the left while its own caption said long-to-short. Fixed by flipping the axis to match the
caption, which is also the direction the argument reads in.

Fidelity is tested: a real refinement is scored against a real ensemble, serialized, rendered, and every
computed mean, standard deviation, separation, and tail probability is asserted present at the precision
the page prints, along with the histogram bars.

**The generator is committed and its output is not**, as in task:20.

### What this task did not do

No selector, no motif score, no `choose_motif`, no ranking by a null statistic. No inverse-frequency,
TF-IDF, `−log p(mark)`, entropy, mutual-information, or learned weighting. No new facet, no mark change,
no timing change, no removal of adapter emissions, no boundary moved on this round's output. No
variable-length discovery, no motif families, no corpus, no fourth specimen. No product CLI surface, no
stable public statistical API, no recording-format change, no dependency, no new page, no Spectroscope
change. No existing test altered and no check weakened. No real recording committed, copied, or
reproduced; no absolute path, prompt, response, command, or file content in this artifact. The
unrelated `cargo build --examples` / `spectroscope.rs` `required-features` defect did not obstruct
validation and stays out of scope. Nothing pushed.
