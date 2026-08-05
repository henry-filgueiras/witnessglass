---
id: spr_01KZ9VQEDS3B4Z66483RNNRFKN
sequence: 10
kind: sprint
status: closed
created: 2026-08-05
closed: 2026-08-05
---

# Frame

## Goal

Find out whether a machine can trim its own frame.

task:19's most useful output was not its verdict. It was this, measured across a preregistered ladder
on two independent real recordings:

> Fixed event-count boundaries became the dominant observed failure mode at `k ≥ 6`. A four-event core
> persisted across multiple rungs, while larger windows progressively degraded the match by attaching
> surrounding context that did not match.

The question this sprint asks is much smaller than variable-length motif discovery:

> Given a plausible fixed-window match, can a tiny deterministic local boundary-refinement procedure
> recover a better-supported variable-length core without being told the correct boundaries?

Nothing about the representation or the metric moves. The only new thing in the world is a small
exhaustive search over where a candidate's four boundaries sit.

## Rationale

This is the smallest possible step between two capabilities, and the gap between them is the whole
point:

```text
"these two k-event windows look similar"
        ↓
"there appears to be a recurring figure here, and these may be its boundaries"
```

**The methodological risk is not the search; it is the objective.** Minimizing alignment distance over
boundary choices rewards deleting evidence. In this metric the degeneracy is not a risk but a
certainty: a one-event span has no gaps at all, so any two spans carrying the same single mark are at
distance exactly zero, and in a recording that is 37.9% one tool name such a pair always exists. Any
round that reported "refinement improved the distance" without confronting that would be reporting an
artefact.

So the anti-collapse policy is preregistered, its degeneracies are hand-checked before running, and the
primary reporting form is a **Pareto frontier over distance and retained events** rather than a scalar
with an invented regularization coefficient. A frontier can be read; `distance + 0.173 × discarded` can
only be believed.

**A known answer exists for one specimen and must be used.** The synthetic fixture's planted figure has
boundaries decided in generator constants. Handing the algorithm a deliberately contaminated seed and
asking whether it finds them back is the only true known-answer boundary test available, and it is what
separates this round from a plausible story.

**Three specimens with three different evidentiary roles.** A synthetic one where the answer is known, a
positive control where a shared figure is known to exist inside divergent context, and the independent-real
candidate that motivated the round. Any one alone would be unfalsifiable.

## Success criteria

- The task:18/task:19 metric frozen and verifiable by diff: representation, marks, alignment, costs,
  timing term and weight, normalization, and the three-part decomposition all untouched. The
  `Agent → subagent_started` adapter-emission question explicitly deferred, not fixed.
- A local boundary search that is exhaustive over a small preregistered neighbourhood, deterministic,
  and brute-force by design. No optimizer, no dynamic programming, no global subsequence mining.
- Refined spans allowed to differ in length between the two sides, with both lengths reported.
- An anti-collapse policy preregistered with its reasoning, its degeneracies hand-checked in advance,
  and no tuning after results are seen.
- Full decomposition per seed and per refined candidate, including the boundary deltas, so what changed
  is visible rather than inferred.
- Three specimens chosen before running, with their provenance, their original `k`, their original
  distances, and what was known about each recorded first.
- Predictions and a three-way verdict fixed in advance; a criterion-feasibility check performed on
  structure and cardinality alone.
- A negative control demonstrating the degeneracy the policy exists to prevent, rather than asserting it.
- `scripts/check.sh` passes unweakened. No existing test changed.

## Non-goals

- Global variable-length motif discovery, arbitrary subsequence mining, motif families, corpus
  accumulation, hierarchical motifs, or semantic motif naming.
- Any richer similarity facet: paths, files, payload magnitude, edit intensity, intent, embeddings.
- Any change to the timing policy, the marks, the costs, the normalization, or the adapter's event
  vocabulary. Any promotion of `distinct_marks` into a definition of motifhood.
- A general `MotifDiscoveryEngine`, a product CLI surface, a change to the canonical recording format,
  or a new dependency.
- A generalized visualization framework. One small static artifact generated from computed output is
  authorized; anything larger is recorded as a follow-up and replaced with text.
- Committing a real recording, or any prompt, response, command, file content, or sensitive absolute
  path — in archaeology or in a rendered artifact.

## Outcome

One task, closed. **Falsified** — and the verdict is the least interesting thing the round produced.

The question was whether a machine can trim its own frame. The answer turns out to be two answers,
and the sprint's design is what separated them:

- **The search can.** On the synthetic specimen it found the planted left boundary exactly, from a
  seed contaminated on both sides, and the event cost fell to zero the moment it got there. On the
  positive control it moved both starts to the first index at which two runbook executions agree. On
  the independent-real specimen it recovered task:19's four-event persistent core to the index, from
  a seed scoring 0.479, having never been told the core existed — and four seeds perturbed by one
  event each converged on the identical answer.
- **Nothing can choose.** Every frontier descends monotonically to the length floor. There is no
  knee, no interior optimum, and nothing in the objective that distinguishes "the figure ends here"
  from "not enough has been thrown away yet". On the independent-real specimen the frontier improves
  3.6× *past* the recovered core by discarding `tool_requested/Agent` — the rarest mark in that
  recording, one occurrence in 169 events — because the three common marks left behind have gaps that
  agree more closely.

The capability the round set out to acquire was: *there appears to be a recurring figure here, and
these may be its boundaries*. It got the first clause and not the second.

### Success criteria, against evidence

- **The metric frozen**, verifiable by diff. The alignment, timing policy, five constants, and
  normalization are byte-identical to task:18; the additions decide which spans get scored.
- **Exhaustive, deterministic, brute-force local search.** 2401 combinations per seed at radius 3,
  invalid ones skipped rather than clamped, no optimizer anywhere.
- **Unequal-length spans supported and reported**, with boundary deltas beside them.
- **The anti-collapse policy preregistered with its degeneracies hand-checked**, and demonstrated
  rather than asserted: with the floor removed, a one-event span matches a one-event span at distance
  exactly zero, because a one-event span has no gap for the timing component to use.
- **Three specimens with three evidentiary roles**, all chosen and recorded before running, none
  added or discarded afterwards.
- **A criterion-feasibility check that found three things** — and, this round, missed a fourth in a
  new way.
- **A small static page generated from computed output**, with a test that asserts it holds no
  measurement of its own. Generator committed; output over real specimens not.

### What the sprint found that it was not looking for

**The Pareto frontier earned its place, and a scalar objective would have hidden the entire result.**
The designated pick — the longest span scoring no worse than the seed — was four to fifteen times
worse than its own frontier's best on all three specimens, every time for the same reason: a bad seed
sets a low bar. Had this round reported one number per specimen, all three would have been wrong and
nothing would have revealed that the search underneath was finding exactly the right spans.

**A feasibility check can derive the disproof of its own criterion and still not apply it.** The
preregistration worked out, before anything ran, that appending a matching event improves the total
whenever its timing cost falls below 1.5× the current total. On a fixture whose figure repeats every
eight events that implies the planted span is dominated by the planted span plus one event — hence
cannot be on the frontier — and the criterion "the planted boundaries appear on the frontier" was
written anyway. Fourth criterion defect in five rounds and the first of this shape: not a missing
check, but a successful one whose consequence was not carried to the neighbouring clauses.

### What this sprint deliberately leaves open

The stopping rule. The one recommended next experiment is to evaluate task:19's existing order null
at every boundary combination and ask whether a null-referenced objective has an interior optimum
where the raw one has none — no new facet, no coefficient, no new machinery, the same three seeds.

General variable-length discovery, which this round makes a worse idea than it looked: the search
half is easy and the deciding half is unsolved, and mining arbitrary subsequences under an objective
that cannot say when to stop would produce a great many confident, wrong boundaries.

Marginal mark frequency, which is visible in the failure and is not a new facet, and the deferred
`Agent → subagent_started` adapter emission, which is still two of the four events in the recovered
core.

Nothing here changed the raw format, the schema, the recorder, `inspection`, the viewer, the
workbench, the Spectroscope, or the product CLI's verbs, and no dependency was added.
