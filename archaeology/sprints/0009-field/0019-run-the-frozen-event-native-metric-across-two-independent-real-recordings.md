---
id: tsk_01KZ9TDVCCED0WD6ZV6RS188XR
sequence: 19
kind: task
status: pending
sprint: spr_01KZ9TDVC2YRR1SK15MTGT5DTG
created: 2026-08-05
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
