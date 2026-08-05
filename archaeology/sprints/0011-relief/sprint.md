---
id: spr_01KZ9XFS9ED6GPVPF0K7HZGWC9
sequence: 11
kind: sprint
status: closed
created: 2026-08-05
closed: 2026-08-05
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

## Outcome

One task, closed. **Supported.** The missing quantity was surprise, and the existing null already had it.

task:20 ended with a search that could find the core and an objective that could not stop at it. This
sprint added no facet, changed no weight, and touched the alignment not at all; it made the null's seed
a parameter, estimated a distribution per candidate, and looked at the shape.

**The central number.** The four-event core is 4.3× rarer under the null than the three-event suffix
that raw distance prefers — `empirical_p` 7.0e-4 against 3.0e-3 — despite scoring 3.6× worse on
agreement. The preregistration derived that ordering from the null's own construction before anything
ran, and predicted its magnitude to within an order of magnitude from marginals computed by hand.

**The number that was not predicted.** Standardized separation has an interior maximum, and its global
argmax over *every* candidate is the meaningful span on all three specimens: the span beginning at the
planted left boundary on the synthetic one, the exact agreement span on the runbook control, and — on
the independent-real specimen, out of 2304 candidates with no anchor and no ground truth supplied — the
four-event core itself. task:20 found every frontier descending monotonically to the floor with no knee.
There is now a knee, and it lands where three previous rounds said the figure was.

**Rarity arrived without being invited.** No `−log p(mark)`, no inverse-frequency weight, no entropy
term. A mark occurring once in 169 events almost never lands in a window under a permutation, so a span
containing it is hard to match by chance. That is stronger evidence than a weighting designed to
produce the answer, because the null was built in a different round for a different question.

### Success criteria, against evidence

- **Frozen except the seed**, verifiable by diff, with task:20's thirty-three tests passing untouched as
  the assertion that `refine` did not move when its enumeration was split out.
- **The null reused, its extension explicit**: `order_null` is `order_null_seeded` at task:19's own
  constant, and a test says so. Seed collisions are checked *after* the generator's low-bit
  normalization, because two seeds differing in bit 0 would have silently duplicated realizations.
- **A distribution, not a shuffled answer**: 1 000 realizations over every candidate, 10 000 over the
  frontier, both benchmarked before being chosen and both run comprehensively.
- **Several statistics, none named as the score** — and this mattered more than expected. See below.
- **A six-question feasibility review** that changed a parameter and a criterion before commitment.
- **The page extended, not replaced**, with the geometry visible in two stacked panels and one null
  distribution drawn per marked candidate.

### What the sprint found that it was not looking for

**The statistics disagree, and the obvious one is wrong.** Unstandardized separation is monotonic toward
short spans on all three specimens — it reproduces raw distance's bias exactly. The empirical tail
saturates at the ensemble floor on two of three specimens and cannot order candidates there at all. Only
standardized separation has usable geometry. A round that had preregistered one of the three as *the*
motif score would have had a one-in-three chance, and `separation` is what a person reaches for first.
The prohibition on writing a selector this round was not caution; it was load-bearing.

**The knee has a mechanical explanation, and it cuts both ways.** The null's spread grows as spans
shorten, so `z` is length-aware almost by accident. Its maximum landing on the meaningful span at three
different lengths — 13, 10, and 4 events — is the evidence against reading it as a disguised length
preference, and three specimens cannot settle that.

**The rarest mark in the independent-real specimen is an adapter emission.** The null makes spans
containing rare marks exceptional, and `tool_requested/Agent` is rare because of how the integration
writes subagent launches down. The core is statistically distinctive; whether it is behaviourally
meaningful is now the most urgent open question in this line of work, and it is the one recommended next
experiment.

### What this sprint deliberately leaves open

Whether `z`'s interior maximum survives removing the three adapter-lifecycle marks from the
representation — one flag, no new machinery, and it separates "finds the figure" from "finds the
artefact".

Any selector. The three quantities are computed and shown side by side; how or whether they should
collapse into one is not decided here, and task:20 is the reason.

Information-theoretic weighting, which the null makes unnecessary to try next and would confound if
tried now.

Nothing here changed the raw format, the schema, the recorder, `inspection`, the viewer, the workbench,
the Spectroscope, or the product CLI's verbs, and no dependency was added.
