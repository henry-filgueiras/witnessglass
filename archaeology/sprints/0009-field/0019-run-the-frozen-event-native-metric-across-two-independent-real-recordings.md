---
id: tsk_01KZ9TDVCCED0WD6ZV6RS188XR
sequence: 19
kind: task
status: closed
sprint: spr_01KZ9TDVC2YRR1SK15MTGT5DTG
created: 2026-08-05
closed: 2026-08-05
---

# Run the frozen event-native metric across two independent real recordings

## Objective

Run task:18's frozen event-native metric across two independent real recordings, at a small
preregistered event-count ladder, against a deterministic order null and a preregistered manual
inspection rubric, and report whether anything survives.

Everything below the `### Preregistration` heading was written and committed to disk **before any
cross-recording ranking was computed**. What *was* inspected first is stated in §1 of the
preregistration, because recording selection required looking at the recordings.

## Acceptance criteria

- Two real recordings selected on inspected evidence, with independence argued; the rejected ones
  named with the reason. Aggregate metadata only in this artifact.
- The frozen metric confirmed field by field.
- A cross-recording ranking that never compares two windows of the same recording, with provenance on
  every candidate.
- Unrestricted rankings reported, and distinct-mark strata reported beside them as diagnostics.
- A deterministic order null run through the identical path, with separation reported per rung.
- Marginal mark frequencies for both recordings.
- Manual classifications against the four-category rubric, recorded before distances were revealed if
  blinding proves cheap, and recorded as unblinded if it does not.
- Whether fixed window boundaries appear to be a material failure mode, observed and recorded.
- A supported/mixed/falsified verdict against the criteria below.
- `scripts/check.sh` passes unweakened; no existing test changed; nothing pushed.

### Preregistration

#### 1. Recording selection, and what was inspected to make it

Four recordings exist untracked in the local workspace. Selecting two required characterizing all
four, so the following **was** inspected before this preregistration was written: record counts,
channel counts, event-kind counts, delivered tool-name counts, timestamps, session ids, and — for the
two small recordings — the verbatim observed mark sequence. No distance was computed and no ranking
existed.

| recording | records | observed | span | what it is |
|---|---|---|---|---|
| `8b68dece` | 234 | 169 | 17 m 34 s | the first-contact development session sprint:4–8 have all used |
| `57f18ff9` | 39 | 32 | 2 m 20 s | a hostile-recording protocol run |
| `f5c18299` | 40 | 33 | 1 m 53 s | a second run of that same protocol |
| `c3afa0ca` | 1 | 1 | 0 s | a lone `session_ended`; not a session |

**`c3afa0ca` is rejected**: one record supports no window at any rung.

**`57f18ff9` and `f5c18299` are two executions of one runbook and must not be the primary pair.**
Their observed mark sequences agree in **27 of 32** positions, with the divergences localized to the
opening and one reordering. Comparing them and calling the result cross-recording structure would be
comparing a script to itself. This is the "copies or replays of the same session" exclusion, in its
near-miss form.

**Primary pair, therefore:**

- **A = `8b68dece`** — 169 observed events, 17 m 34 s, 2026-08-02, ordinary development work.
- **B = `57f18ff9`** — 32 observed events, 2 m 20 s, 2026-08-04, a hostile-protocol run.

Two days apart, different tasks, different protocols, no shared prompt. **The independence that does
not hold** is recorded now rather than discovered later: both are Claude Code sessions in one
repository, driven by the same agent product, over the same five delivered tool names. That is not
independence of vocabulary, and §7 below exists because of it.

**Secondary, and clearly labelled as a positive control:** `57f18ff9` against `f5c18299`, the two
runbook siblings. It is preregistered because it answers the question a negative primary result
cannot answer on its own — whether the frozen metric can match two recordings *known* to contain the
same figure. If the control fails, a negative primary result says nothing about reality.

**One characterization fact that matters for reading everything below.** The Claude adapter emits
session and subagent lifecycle records on the `observed` channel, where the synthetic oracle's
generator used `recorder`. So a real recording's observed-scope sequence contains `session_started`,
`session_ended`, `subagent_started`, and `subagent_stopped` marks that the fixtures' observed scope
did not. Nothing is adjusted for this; it is a difference between the fixture and reality, and it is
stated so that a match involving those marks is read correctly.

#### 2. The frozen metric

Unchanged from task:18, field by field, verifiable by `git diff` over
`src/experiment/event_sequence.rs`:

| what | value |
|---|---|
| representation | `MarkedEvent { sequence, mark, offset_ms, gap_from_previous_ms }` |
| mark | (schema-tagged event kind, verbatim delivered tool name or none) |
| channel scope | `observed` |
| substitution | `1.0` |
| insertion / deletion | `1.0` each |
| timing weight | `0.5` |
| timing term | `min(1, |ln((gₐ+100)/(g_b+100))| / ln 4)` |
| `event_norm` | `event_cost / max(lₐ, l_b)` |
| `timing_norm` | `timing_cost / (0.5 × timed_pairs)` |
| `total` | `(event_cost + timing_cost) / (L + 0.5(L−1))` |

No weight is changed, no facet is added, and nothing is tuned against A or B. Additions this round
are confined to *which pairs get compared* and *what gets printed*.

#### 3. The event-count ladder

**k = 3, 4, 6, 8, 12.**

| k | why this rung |
|---|---|
| **3** | task:18's short control. It recovered the planted figure and showed no null separation worth anything — the rung that says what a trivial sequence scores |
| **4** | the sparse oracle's planted figure length, where the method is known to work |
| **6** | between the two known-good lengths, with no prior attached |
| **8** | the legible oracle's planted figure length, the other known-good point |
| **12** | the richest figure B can support: 21 windows at 32 observed events. Chosen as the largest rung leaving a search space two orders of magnitude above the inspection set |

No rung outside this list is scanned. 16 and above are excluded in advance: B would yield 17 windows
or fewer, and a reality check whose search space is smaller than a page of output is not one.

#### 4. Candidate de-duplication, and why it is preregistered rather than discovered

Window `i↔j` and window `i+1↔j+1` share `k−1` events on both sides and will score almost identically.
Without a policy, "the top five candidates" would mean "one candidate, reported five times".

**Policy:** rank every cross pair by `total`, then walk the ranking greedily and keep a candidate only
if **neither** of its windows overlaps a window of an already-kept candidate. The kept list is the
candidate set; the unrestricted ranking is reported too, so the de-duplication can be seen rather
than trusted.

This is a reporting policy. It changes no distance.

#### 5. The null

**Order null, applied independently to both sides**, using task:18's existing fixed-seed Fisher–Yates
`order_null`: marks permuted across the whole sequence, every gap and offset left in place, mark
multiset and timing profile preserved, receipts dropped. The comparison, per rung:

```text
separation_order = best_total(order_null(A) × order_null(B)) − best_total(A × B)
```

Both sides nulled, because the coherent control for "do two recordings share a figure" is "neither
recording's ordering carries information". One-sided nulls are not run.

**Timing null reported beside it**, using task:18's `timing_null` on both sides, because it is one more
call of the same shape and task:18 predicted and measured it to be nearly inert. It is a diagnostic,
not a criterion.

Both nulls are also run for the positive control pair.

#### 6. Views to be reported

- **View A — unrestricted.** The top cross-recording pairs under the frozen metric, de-duplicated per
  §4 and nothing else removed.
- **View B — diagnostic strata.** The same ranking restricted to pairs whose *smaller* window carries
  at least 2, 3, and 4 distinct marks, reported as three additional slices beside the unrestricted
  one. **These are slices, not definitions of motifhood**, and the verdict may not be read off
  whichever slice looks best. If trivial sequences own the whole ranking, that is the result.
- Per candidate: `k`, both window positions and session ids, distinct marks per side, span per side,
  the verbatim mark sequence per side, `event_norm`, `timing_norm`, `total`.
- Marginal mark frequencies for A and B, so §7's hypothesis can be judged.

#### 7. The frequency hypothesis, named in advance

The hypothesis this round most expects to confirm:

> Strong cross-recording matches are produced by common vocabulary alone — a window that is
> `tool_requested/Bash → tool_succeeded/Bash` repeated, matching another window that is the same,
> because that is what both recordings are mostly made of.

A's delivered tool names are dominated by one string; B's vocabulary is five strings over 32 events.
If the strongest candidates are repetitions of the one or two commonest marks, the round says so
plainly and the verdict follows §10.

#### 8. Manual inspection rubric, fixed before any candidate is seen

For each rung, the **top three de-duplicated candidates** are inspected, using the mark sequences and
gaps the projection produces. Each is classified as exactly one of:

- **TRIVIAL** — mathematically real, but explained by an impoverished or repetitive mark sequence that
  is little evidence of a richer recurring figure.
- **STRUCTURALLY SIMILAR** — recognizably similar multi-event marked structure, comparable ordering,
  no obvious trivial explanation. **This does not mean the two regions performed the same semantic
  workflow**, and no category asserting that exists.
- **AMBIGUOUS** — some resemblance, but the representation does not carry enough to judge whether it
  reflects a meaningful recurring figure.
- **NOT SIMILAR** — the ranking does not correspond to a convincing structural resemblance.

Classifications are recorded before distances are revealed, if that is cheap. If blinding costs more
than a printing flag, inspection is recorded as unblinded.

Additionally recorded per candidate, because it is what the *next* round needs: whether the fixed `k`
window appears to capture a shared core with irrelevant context attached or a piece missing — the
boundary-failure observation. Nothing is done about it this round.

#### 9. Criterion-feasibility check, on cardinalities alone

Performed before the matcher ran and using no distance or ranking. Search-space sizes, from the two
recordings' observed event counts (169 and 32):

| k | A windows | B windows | cross pairs | control pairs (32 × 33 events) |
|---|---|---|---|---|
| 3 | 167 | 30 | 5010 | 930 |
| 4 | 166 | 29 | 4814 | 870 |
| 6 | 164 | 27 | 4428 | 756 |
| 8 | 162 | 25 | 4050 | 650 |
| 12 | 158 | 21 | 3318 | 462 |

Three questions asked, and their answers:

1. **Is an inspection set of three per rung well defined?** Only after §4. Every rung's raw ranking is
   thousands of pairs in which shifted copies of one candidate are adjacent and near-tied, so without
   de-duplication a top-three is not three candidates. **This check is what produced §4**, and §4 is
   preregistered rather than introduced when the output looked repetitive.
2. **Does any criterion require a specific labelled pair to reach a specific rank?** No — and this is
   the difference from sprint:6 §4 and sprint:8 §9, both of which set a rank cutoff that fixture
   combinatorics made unreachable. There is no ground truth here, so no pair is owed a rank. The
   criteria below are a threshold on separation and a classification under §8, and neither can be
   obstructed by how many other pairs happen to rank above something.
3. **Is the ranking deterministic under ties?** Yes. task:18's ordering is total: `total`, then the
   A-window start, then the B-window start.

Recorded as performed. The reusable lesson — that a rank-based criterion needs a cardinality check and
a de-duplication policy before it means anything — is noted in the Result as process friction rather
than built into machinery this round.

#### 10. Verdict criteria, fixed before running

**Supported** requires at least one cross-recording candidate that satisfies **all** of:

- its rung's `separation_order ≥ 0.05` — the same threshold task:18 used, on the same scale;
- it is classified **STRUCTURALLY SIMILAR** under §8;
- its smaller window carries **at least 3 distinct marks**, so it is not explained by one- or
  two-mark vocabulary;
- it was produced with the task:18 metric unchanged.

**Falsified** if any of:

- every inspected candidate at every rung is TRIVIAL or NOT SIMILAR;
- no rung reaches `separation_order ≥ 0.05`;
- the strongest candidates are repetitions of the one or two commonest marks in both recordings.

**Mixed** is anything else — including genuine resemblance with weak null separation, results that
depend heavily on `k`, or classifications that come out AMBIGUOUS.

The positive control is **not** part of the verdict. It is diagnostic: it says whether a Falsified
primary result is about reality or about the metric, and those are different findings.

#### 11. What this task will not do, whatever it finds

No variable-length or boundary discovery, no subsequence growth, no hierarchical motifs, no motif
families, no corpus clustering, no third recording. No path, extension, working-set, payload,
edit-magnitude, intent, semantic-category, hierarchy, duration, or learned facet — including as a
rescue for a negative result. No metric change. No dependency, no product CLI surface, no viewer or
Spectroscope change. No real recording committed, copied, or reproduced, and no absolute path, prompt,
response, command, or file content in this artifact. Nothing pushed.

## Result

Delivered. **Supported** by the letter of the preregistered criteria, and the positive control — which
the preregistration excluded from the verdict on purpose — shows that support is weak. Both are
reported, neither is allowed to overwrite the other.

The short version: two independent real recordings do contain cross-recording window pairs that beat
the order null and survive blind inspection as structurally similar. They score six times worse than
two recordings that genuinely share a figure, timing actively hurts across independent sessions
rather than helping, and the single most persistent match is half an artefact of how the adapter
emits subagent lifecycle records.

### 1. The metric did not move

`git diff 8bec896..f226b3b -- src/experiment/event_sequence.rs` touches only `cross_pairs`,
`CrossPair`, and `dedupe_overlapping`. `align`, `timing_term`, `SUBSTITUTION`, `GAP`,
`TIMING_WEIGHT`, `TIMING_FLOOR_MS`, `TIMING_RATIO_FULL`, the normalization, and `project` are
byte-identical to task:18. No weight was changed, no facet added, and nothing was tuned against A or
B — before or after seeing any ranking.

### 2. The two recordings, and the two that were rejected

| | records | observed retained | filtered by channel | span | distinct marks |
|---|---|---|---|---|---|
| **A** `8b68dece` | 234 | 169 | 65 | 1053.8 s | 14 |
| **B** `57f18ff9` | 39 | 32 | 7 | 140.5 s | 16 |

**Rejected, with reasons.** `c3afa0ca` holds one `session_ended` record and supports no window.
`f5c18299` is a second execution of the same runbook as B — their observed mark sequences agree in
**27 of 32** positions — so pairing them would have been comparing a script to itself. It is used as
the preregistered positive control instead.

Marginal mark frequencies, which §7 needs:

```text
A (169 events, 14 marks)          B (32 events, 16 marks)
  37.9%  tool_requested/Bash        18.8%  tool_requested/Bash
  37.9%  tool_succeeded/Bash        12.5%  tool_succeeded/Bash
   4.1%  tool_requested/Read         9.4%  tool_requested/Read
   4.1%  tool_succeeded/Read         6.2%  subagent_stopped
   3.0%  Edit/Write req + succ       6.2%  Edit/Write/Read req + succ
   1.2%  subagent_stopped            3.1%  session_started, session_ended,
   0.6%  session_started,                   subagent_started, tool_failed/Bash,
         session_ended,                     tool_failed/Read,
         subagent_started,                  tool_requested/Agent,
         tool_requested/Agent,              tool_succeeded/Agent
         tool_succeeded/Agent
```

**A is 75.8% one tool name.** That is the condition under which "matching" can mean nothing, and it is
why §7 named the frequency hypothesis in advance.

### 3. View A — unrestricted, and View B — diagnostic strata

Primary pair, A × B. Strata are the best distance among pairs whose *smaller* window carries at
least that many distinct marks.

```text
    k  A win  B win   pairs     best      >=2      >=3      >=4  null-ord  null-tim  sep-ord  sep-tim
    3    167     30    5010    0.022    0.022    0.031        -     0.043     0.012   +0.021   -0.010
    4    166     29    4814    0.113    0.113    0.113    0.113     0.219     0.103   +0.106   -0.010
    6    164     27    4428    0.306    0.306    0.306    0.306     0.333     0.247   +0.026   -0.060
    8    162     25    4050    0.369    0.369    0.369    0.369     0.433     0.365   +0.064   -0.004
   12    158     21    3318    0.436    0.436    0.436    0.436     0.524     0.437   +0.088   +0.001
```

The `-` at `k = 3, ≥4` is a structural impossibility, not missing data: a three-event window cannot
hold four distinct marks.

**The strata answer §7 directly, and the answer is split by rung.** At `k = 3` the unrestricted best
*is* wallpaper — `Bash req → Bash succ → Bash req`, two marks, each 37.9% of A — and the `≥3` stratum
costs 0.031 against 0.022 to escape it. At `k = 4` and above, `best` and the `≥4` stratum are the
**same number**: the unrestricted top candidate already carries four or more distinct marks, so no
filtering is doing any work and the strongest matches are not one- or two-mark vocabulary.

`sep-ord ≥ 0.05` holds at `k = 4, 8, 12` and fails at `k = 3` and `k = 6`.

**`sep-tim` is negative at four of five rungs.** Permuting the gaps produced a *better* best match
than the real timing did. Across two independent recordings, real relative timing actively penalizes
matching rather than helping it — which is the opposite of what the same null showed inside one
recording in task:18, and the opposite of what it shows in the control below.

### 4. Manual inspection — blind, then revealed

Thirteen candidates, three per rung after §4's de-duplication. Classifications were written to disk
from a packet with distances withheld, and the distances were revealed only afterwards. **Self-blinding
by the same agent that wrote the metric is weak evidence, and this sentence is the disclosure that
task:19 §9 asks for rather than a claim that the protocol was strong.**

| id | k | classification (blind) | revealed `ev / tm / tot` | marks (A·B) |
|---|---|---|---|---|
| k3-c1 | 3 | TRIVIAL | 0.000 / 0.087 / **0.022** | 2·2 |
| k3-c2 | 3 | AMBIGUOUS | 0.000 / 0.124 / 0.031 | 3·3 |
| k3-c3 | 3 | TRIVIAL | 0.000 / 0.364 / 0.091 | 3·3 |
| **k4-c1** | 4 | **STRUCTURALLY SIMILAR** | 0.000 / 0.416 / **0.113** | 4·4 |
| k4-c2 | 4 | TRIVIAL | 0.000 / 0.447 / 0.122 | 3·3 |
| k4-c3 | 4 | AMBIGUOUS | 0.000 / 0.725 / 0.198 | 4·4 |
| k6-c1 | 6 | STRUCTURALLY SIMILAR | 0.167 / 0.641 / **0.306** | 4·5 |
| k6-c2 | 6 | STRUCTURALLY SIMILAR | 0.167 / 0.738 / 0.335 | 4·5 |
| k6-c3 | 6 | AMBIGUOUS | 0.333 / 0.369 / 0.344 | 4·6 |
| **k8-c1** | 8 | **STRUCTURALLY SIMILAR** | 0.250 / 0.641 / **0.369** | 5·7 |
| k8-c2 | 8 | AMBIGUOUS | 0.375 / 0.420 / 0.389 | 4·7 |
| k8-c3 | 8 | AMBIGUOUS | 0.500 / 0.431 / 0.479 | 4·8 |
| k12-c1 | 12 | AMBIGUOUS | 0.417 / 0.479 / **0.436** | 4·9 |

Four STRUCTURALLY SIMILAR, five AMBIGUOUS, three TRIVIAL, no NOT SIMILAR. The blind ordering tracked
the metric's ordering at every rung — the classification never disagreed with the ranking about which
of three was best — which is mild evidence the metric is measuring something a reader also sees, and
is also exactly what a self-blinding protocol would produce if it failed.

**The two candidates that satisfy every Supported condition:**

```text
k4-c1   A idx 53 [233.7s]  ·  B idx 17 [74.4s]        rung separation +0.106, min marks 4
  A[8b68dece] tool_requested/Agent --1.8s-> subagent_started --3.3s-> tool_requested/Bash --0.0s-> tool_succeeded/Bash
  B[57f18ff9] tool_requested/Agent --0.0s-> subagent_started --2.6s-> tool_requested/Bash --0.0s-> tool_succeeded/Bash

k8-c1   A idx 112 [657.0s] ·  B idx 6 [52.4s]         rung separation +0.064, min marks 5
  A[8b68dece] tool_succeeded/Write --3.2s-> tool_requested/Bash --1.8s-> tool_succeeded/Bash --4.9s->
              tool_requested/Edit --0.0s-> tool_succeeded/Edit --12.7s-> tool_requested/Bash --2.2s->
              tool_succeeded/Bash --4.8s-> tool_requested/Bash
  B[57f18ff9] tool_succeeded/Write --1.8s-> tool_requested/Read --1.3s-> tool_succeeded/Read --2.7s->
              tool_requested/Edit --1.5s-> tool_succeeded/Edit --1.9s-> tool_requested/Bash --0.0s->
              tool_succeeded/Bash --4.8s-> tool_requested/Bash
```

**k4-c1 is anchored by the rarest mark in A** — `tool_requested/Agent`, one occurrence in 169 events —
so it is the opposite of vocabulary-driven. **And its first two events are an adapter emission
pattern**: the Claude adapter writes `subagent_started` immediately after an `Agent` request, always,
so half of that four-event figure is a property of the integration rather than of anything an agent
chose to do. This was recorded in the blind classification, before any distance was seen, and it is
the single most important caveat in the round.

k8-c1 has no such defect. Seven of eight positions match; only the second call pair differs, `Bash`
against `Read`. It is a write, a tool call and its outcome, an edit and its outcome, then shell calls
— in two sessions two days apart with no shared prompt.

### 5. The positive control, and what it does to the reading

`57f18ff9` × `f5c18299`, the two runbook executions. **Not part of the verdict**, by preregistration.

```text
    k  A win  B win   pairs     best  null-ord  null-tim  sep-ord  sep-tim
    3     30     31     930    0.007     0.281     0.020   +0.273   +0.012
    4     29     30     870    0.014     0.321     0.022   +0.307   +0.007
    6     27     28     756    0.033     0.377     0.083   +0.344   +0.050
    8     25     26     650    0.063     0.502     0.079   +0.438   +0.016
   12     21     22     462    0.078     0.582     0.096   +0.504   +0.017
```

Every top candidate has `ev 0.000` — identical mark sequences — and the whole distance is timing. The
metric independently recovered the alignment of the two executions: at `k = 8` it matched A index 23
to B index 23, at `k = 12` index 19 to index 19, at `k = 4` index 10 to index 10, without being told
they were related.

Side by side, at the same rungs:

| k | primary best | control best | primary `sep-ord` | control `sep-ord` |
|---|---|---|---|---|
| 3 | 0.022 | **0.007** | +0.021 | **+0.273** |
| 4 | 0.113 | **0.014** | +0.106 | **+0.307** |
| 6 | 0.306 | **0.033** | +0.026 | **+0.344** |
| 8 | 0.369 | **0.063** | +0.064 | **+0.438** |
| 12 | 0.436 | **0.078** | +0.088 | **+0.504** |

The control is what "these two recordings genuinely contain the same figure" looks like through this
metric: distances five to twenty times lower and separations four to thirteen times larger. **The
metric is not the limiting factor.** A Falsified primary result would have been about reality; a
Supported one has to be read against this scale, and against it the primary support is thin.

`sep-tim` is *positive* in the control at every rung and *negative* in the primary at four of five.
Timing helps when two sequences really are the same figure and hurts when they are not — which is a
coherent thing for a timing term to do, and it means the negative primary values are informative
rather than a defect.

### 6. Verdict: **Supported**

The preregistered condition is "at least one cross-recording candidate that satisfies **all** of" four
things, and two candidates do:

| condition | k4-c1 | k8-c1 |
|---|---|---|
| rung `separation_order ≥ 0.05` | +0.106 ✓ | +0.064 ✓ |
| classified STRUCTURALLY SIMILAR | ✓ | ✓ |
| smaller window ≥ 3 distinct marks | 4 ✓ | 5 ✓ |
| task:18 metric unchanged | ✓ | ✓ |

No Falsified clause fires: not every candidate is TRIVIAL or NOT SIMILAR, three rungs clear the
separation threshold, and above `k = 3` the strongest candidates are not one- or two-mark vocabulary.

**Supported, and weakly.** Three things a reader should carry with the verdict, none of which changes
it:

1. **The control is six times better on the same scale.** §5.
2. **One of the two qualifying candidates is half adapter artefact.** §4.
3. **The criteria as drafted had no effect-size condition.** A separation threshold of 0.05 was
   imported from task:18, where distances between planted occurrences were 0.000–0.043. Cross-recording
   distances are an order of magnitude larger, so 0.05 means something much weaker here than it did
   there. That is a drafting defect, recorded in §9 as friction rather than repaired by moving the
   threshold after seeing the numbers.

### 7. Does common-mark frequency explain the strongest matches?

**At `k = 3`, yes.** The unrestricted best is the two commonest marks in A.

**At `k ≥ 4`, no**, and the strata are what say so: `best` and the `≥4` stratum are the same number at
every rung from 4 upward, so the top candidate already carries four or more distinct marks with no
filtering applied. The strongest candidate of the round is anchored by the *rarest* mark in A.

The frequency hypothesis is therefore **rejected above `k = 3` and confirmed at `k = 3`**, which is
also a statement about how short a window has to be before a recording's marginal vocabulary decides
the answer.

### 8. Fixed window boundaries are the dominant failure mode at `k ≥ 6`

This is what the round was told to look for, and it is the clearest thing it found.

One figure — `tool_requested/Agent → subagent_started → tool_requested/Bash → tool_succeeded/Bash` —
appears as the anchor of a top candidate at `k = 3` (partially), `k = 4` (exactly), `k = 6`, `k = 8`,
and `k = 12`. At `k = 4` it is the whole window and scores 0.113. At every longer rung the same core
is present with **divergent context attached on both sides**, and the distance degrades accordingly:

```text
k=4   the core exactly                                    tot 0.113   sub 0
k=6   the core + 2 events, A continues with shell calls,
      B closes the subagent                               tot 0.344   sub 2
k=8   the core at positions 3-6, different context both   tot 0.479   sub 4
      sides
k=12  the core embedded in 12 events of largely
      unrelated surroundings                              tot 0.436   sub 5
```

Four of the nine candidates at `k = 6, 8, 12` are this shape: a shared core the fixed window cannot
stop at. k8-c2 is a second instance with a different core (`Edit req → Edit succ → Bash req → Bash
succ`).

**Nothing was done about it.** It is recorded because it is the evidence that would justify
commissioning boundary discovery, and because it was observed under a fixed-`k` protocol that had no
way to exploit it.

### 9. Criterion-feasibility check, and what it caught

Performed on cardinalities alone, before the matcher ran, and recorded in the preregistration at
`8bec896`. It caught one thing and missed one thing.

**Caught, and fixed in advance:** without a de-duplication policy, a "top three" would have been one
candidate reported three times, because windows `i↔j` and `i+1↔j+1` share `k − 1` events on both sides
and near-tie. §4 of the preregistration exists because of this check. In the run, de-duplication never
displaced rank 1 — the unrestricted best and the best kept candidate are the same pair at every rung —
so the policy cost nothing and bought three distinct candidates instead of three views of one.

**Missed:** the check asked whether the criteria were *reachable* and not whether they were
*meaningful at this scale*. A separation threshold carried over from a round whose distances were an
order of magnitude smaller passes trivially without saying much. That is the third preregistration
defect in four rounds — sprint:6 §4 (rank cutoff too tight), sprint:8 §9 (rank cutoff unreachable),
and now a threshold that is reachable but weak.

**The generalization, recorded as friction rather than built:** a rank-based criterion needs a
cardinality check *and* a de-duplication policy; a threshold-based criterion needs a scale check
against the distribution the threshold will actually face. Neither is machinery this round should
build. See §11.

### 10. Excluded facets that looked useful during inspection

**Interpretation, and future feature opportunities. Not used, and not permitted to rescue anything.**

- **k8-c1 and k6-c2 both pair a `Write` or `Edit` with shell calls.** Whether the two regions touched
  the same or comparable repository regions would have made the difference between AMBIGUOUS and
  STRUCTURALLY SIMILAR for at least k6-c2. The facet would be *delivered path strings present in the
  payloads* — not "files changed on disk", which no event in either recording establishes.
- **k4-c3 has identical marks and a 300× timing disagreement** (30.4 s against 0.1 s on the first
  gap). `recorded_response_json_bytes` on the two `Bash` completions would say whether the long gap
  accompanied a large recorded response, which is a fact about the recording rather than about how
  long anything ran.
- **The `Agent` figure at every rung would be far more informative with the subagent's own event
  stream attributed to it.** `agent_id` is on the ledger and is licensed; the representation
  deliberately does not carry it. Using it would need its own argument, because "these events belong
  to the subagent" is a claim about attribution rather than a raw field.

None was used. All three would need their own preregistration.

### 11. Desire-path friction

**A fourth consecutive round with the preregistration in a `###` subsection of `Acceptance
criteria`.** Same as sprint:5, sprint:6, sprint:8. The evidence it predates the run is `8bec896`, a
commit containing nothing else. **idea:5**, fourth occurrence, no new information.

**The criterion-drafting problem has now happened three times and changed shape.** sprint:6 and
sprint:8 were rank cutoffs that fixture combinatorics made wrong. This round's feasibility check —
which task:19 was explicitly asked to perform — caught the rank problem and produced §4's
de-duplication policy before any output existed, which is the check working. It then missed a
*threshold* problem of the same family. The reusable shape is one sentence long: **a preregistered
numeric criterion needs a check against the scale of the distribution it will face, not only against
whether it is reachable.** Recorded here as friction. Not promoted to an idea, because it is a
research-process discipline rather than a Scarp affordance — Scarp cannot check a threshold's
calibration, and idea:5's sealed section would not have caught it either.

**Appending a Result is still `cat >>` on this machine.** `scarp` 0.2.0, `scarp close` offers only
`--resolved-by`; maintenance:1 records that upstream shipped result-on-close. Version lag, not a gap.

**`scarp list` still requires a collection**, second occurrence, still five seconds of workaround.
Noted; still not promoted.

### 12. Strongest limitation

**Two recordings is not a sample, and one of them is 32 events long.**

Every number above rests on a single pair, and B contributes 21 to 30 windows per rung against A's 158
to 167. The strongest candidate of the round is one pair out of 4,814. Nothing here establishes a rate,
a distribution, or a base expectation for how often independent sessions share a figure — only that at
least one pair does, above a null, in this one comparison. A third recording would not fix that; a
corpus would, and building one is explicitly not this round's business.

The second limitation is the one §4 discloses: the inspection was self-blinded by the agent that wrote
the metric.

### 13. Recommendation: exactly one next experiment

**Variable-length boundary discovery, over the same two recordings, seeded from the fixed-`k` cores
this round found.**

§8 is the reason and it is now evidence rather than a hunch: a four-event core recurred across two
independent recordings and every longer fixed window degraded it by attaching context that did not
match. The failure mode is the window, not the metric — the metric found the core five times and was
then forced to carry rubbish either side of it. That is precisely the condition boundary discovery
exists for, and it is now measured rather than assumed.

Scope it as narrowly as this round was: grow and shrink a match around a seed until the distance stops
improving, no motif families, no corpus, no new facet, and the same null and blind-inspection
discipline. The falsifiable question is whether variable boundaries recover a longer shared figure than
`k = 4` did, or whether four events is genuinely all these two recordings share.

**Two things this recommendation does not do.** It does not reach for paths or payloads: §10's
observations are logged and stay logged, because adding a facet and changing the window shape in one
round would leave neither attributable. And it does not propose a third recording — the sample-size
limitation in §12 is real and a corpus is a different, larger commitment than this line of work has
earned.

### What this task did not do

No variable-length or boundary discovery, no subsequence growth, no hierarchical motifs, no motif
families, no corpus clustering, no third recording. No path, extension, working-set, payload,
edit-magnitude, intent, semantic-category, hierarchy, duration, or learned facet — including as a
rescue. No metric change: the alignment, timing policy, costs, and normalization are byte-identical to
task:18. No dependency, no product CLI surface, no viewer or Spectroscope change, no UI for the
inspection protocol. No existing test altered and no check weakened. No real recording committed,
copied, or reproduced; no absolute path, prompt, response, command, or file content appears in this
artifact. The local output under the session scratchpad is derived from real recordings, is as
sensitive as they are, and is not redacted. Nothing pushed.
