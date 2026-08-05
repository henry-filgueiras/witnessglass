---
id: spr_01KZ9XFS9ED6GPVPF0K7HZGWC9
sequence: 11
kind: sprint
status: active
created: 2026-08-05
---

# Relief

## Goal

task:20 ended on a clean split:

> The search can find the core. Agreement alone cannot tell us when the figure is complete.

Its local boundary search recovered the synthetic planted left boundary exactly, converged on the
first agreement point of the repeated-runbook control, and recovered task:19's four-event real core to
the index from four differently perturbed seeds. Then the objective kept improving past all of it. On
the independent-real specimen:

```text
8-event seed      total 0.479
4-event core      total 0.113
3-event suffix    total 0.031
```

The three-event suffix scores 3.6× better by discarding `tool_requested/Agent`, the rarest mark in
that recording. The metric measures **how strongly two sequences agree** and not **how surprising
that agreement is given the recordings they came from**.

task:19 already built an order null that destroys temporal structure while preserving marginal event
identity. This sprint asks one question:

> If every boundary candidate is evaluated against that null, does the previously observed core become
> distinguishable from shorter but more commonplace matches?

## Rationale

**This round is descriptive and must stay descriptive.** It does not design a motif score, does not
write a selector, and does not decide how raw agreement, retained structure, and surprise should
collapse into one number. It computes the third quantity and shows its shape beside the other two.
Deciding before looking is how the previous round's pick rule came to be wrong on all three specimens.

**The null is preferred over hand-designed rarity weighting, and the order of attempts matters.** The
obvious repair for task:20's failure is to weight marks by `−log p(mark)` so that discarding a rare
one costs something. That repair is *deliberately not attempted here*. If the existing order null
already separates "a common sequence happened to match" from "an unusually structured sequence
matched", that is stronger evidence than a weighting invented to produce the answer we want — because
the null was built for a different round, for a different question, and has no knowledge of this one.

**A null that is not a distribution is not evidence.** task:19 used one shuffled realization to answer
a coarser question. One realization cannot say whether a distance is surprising. This sprint's only
extension to the machinery is to make the null's seed a parameter so a distribution can be estimated,
and that extension is stated, tested, and prevented from silently redefining what "order null" means.

**A negative result is a real result here.** It is entirely possible that null-relative evidence is
also monotonic toward short spans, or flat. If both the four-event core and the three-event suffix
turn out equally unsurprising, that is a finding about what the representation can support, and it
would redirect the next round toward richer marks rather than better statistics.

**A distinction this sprint must keep.** The independent-real core contains `tool_requested/Agent → subagent_started`, which task:19 recorded
as partly a deterministic adapter emission. That representation question stays deferred. If
null-relative evidence favours the four-event core *because* of the rare `Agent` mark, the finding is
that the core is **statistically distinctive**. It is not a finding that the core is **behaviourally
meaningful**, and no output this round may blur the two.

## Success criteria

- Every piece of task:20's machinery frozen except the null's seed: representation, marks, alignment,
  costs, timing policy and weight, normalization, search radius, candidate enumeration, length floor,
  the three specimens, and their seed spans.
- The order null reused rather than reinvented, with exactly what it permutes and what it preserves
  restated, and with the extension to multiple realizations made explicit and tested.
- A deterministic null *distribution* per candidate, with the realization count benchmarked and
  preregistered rather than guessed.
- Several null-relative statistics reported side by side, with none of them preregistered as the
  motif score, and with disagreements between them reported rather than resolved.
- The three task:20 specimens and seeds, unchanged.
- A criterion-feasibility review answering six explicit questions per prediction, performed before the
  preregistration is committed, and any invalid criterion fixed rather than knowingly committed.
- The task:20 static page extended — not replaced, not generalized — so the geometry is visible: raw
  agreement and null-relative surprise against retained length, on the same page, with known-answer
  overlays and at least one candidate's null distribution shown rather than summarized.
- `scripts/check.sh` passes unweakened. No existing test changed.

## Non-goals

- Any selector, motif score, or rule that collapses the three quantities into one.
- Information-theoretic weighting of any kind: inverse frequency, TF-IDF, `−log p(mark)`, entropy,
  mutual information, learned rarity. Those are candidates for a *later* round and only if the null
  fails or explains why they would help.
- New similarity facets, changes to marks, changes to timing, removal of adapter emissions, or any
  change to the alignment metric.
- General variable-length motif discovery, motif families, corpus accumulation, or a fourth specimen.
- A visualization framework, a charting dependency, a new page, a product CLI surface, a stable public
  statistical API, or a recording-format change.
- Committing a real recording, or any prompt, response, command, file content, or sensitive path.
