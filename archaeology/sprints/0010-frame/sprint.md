---
id: spr_01KZ9VQEDS3B4Z66483RNNRFKN
sequence: 10
kind: sprint
status: active
created: 2026-08-05
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
