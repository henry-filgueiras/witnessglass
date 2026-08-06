---
id: spr_01KZA5SR3F4HX6ZQY7XC8T8DE9
sequence: 16
kind: sprint
status: closed
created: 2026-08-05
closed: 2026-08-05
---

# Envelope

## Goal

sprint:15 established two mechanically understood defects in `rarity_of_agreements` and located their
failure surfaces analytically. This sprint asks where those surfaces lie relative to the recordings this
project actually has:

> Where do the accumulation/length-dependence defect and the A/B asymmetry defect lie relative to the
> empirical operating envelope of the WitnessGlass recordings we actually possess?

**An exposure study, not a statistic-design round.** The statistic is frozen and is not repaired,
normalized, symmetrized, replaced, or adopted. The deliverable is a map from two known failure surfaces
onto measured corpus quantities, with a bounded classification for each — and the two classifications
are kept apart rather than combined into one verdict.

## Rationale

**A defect's mechanism and its exposure are different questions, and sprint:15 only answered the first.**
It derived `c = N^{(k−1)/k}` and friends from the frozen definition and confirmed every boundary against
a sweep. What it could not say is whether real recordings sit near those boundaries or nowhere near them.
Its own closing note flagged one suggestive coincidence — a commonest mark at 64 against a boundary near
47 — and explicitly called it unmeasured. This round measures it, **from source data rather than by
carrying the estimate forward**.

**The corpus is not ground truth and must not be used as any.** These recordings have no known true motif
boundaries. Nothing here measures whether the statistic is *right*; it measures only how close the
recordings come to configurations where the statistic is known to misorder. A threshold tuned on this
corpus and then declared validated by it would be the same error sprint:12 was built to catch.

**Asymmetry is judged on intent first, magnitude second.** If the relation being scored is meant to be
symmetric under exchanging the two occurrences, then argument-order dependence is an invariance defect
whatever its current empirical size, and a small measured discrepancy does not excuse it. The intent is
settled in the preregistration on design grounds, before the measurement, precisely so the measurement
cannot be used to infer a convenient directionality.

**The corpus grew since the last round, and that is why this round is worth running now.** A real Claude
session has been recorded in an external project, so the study covers two independent repositories rather
than one — which is the difference between characterizing a project's own idiom and characterizing a
range.

## Success criteria

- The frozen definition, the preserved counterexamples, and the analytical boundaries restated before any
  measurement.
- A corpus inventory naming every recording included and every one excluded, with reasons.
- Per-recording characterization: record and event counts, vocabulary size, every mark's empirical
  frequency, extremes, quantiles, and the span and agreement counts the existing machinery actually
  produces.
- The accumulation boundary evaluated against those measurements, reported as absolute and relative
  margins rather than as a binary label, with the closest observed configuration identified.
- The distinction between *the corpus contains a crossing* and *the corpus contains parameter values from
  which a crossing could be constructed* kept explicit throughout.
- Asymmetry measured over real candidate pairs produced by existing machinery, with the fraction at zero,
  the distribution of nonzero values, the maximum, and whether any ordering or designated pick changes.
- Classifications fixed before the data is seen, and reported separately as L1/L2/L3 and S1/S2/S3.
- The synthetic gauntlets re-run unchanged as regressions, with their known failures intact.

## Non-goals

- Repairing, modifying, normalizing, symmetrizing, replacing, or adopting `rarity_of_agreements`; or
  choosing among pooling constructions if symmetry is found to be required.
- Treating the corpus as ground truth for boundary correctness, or tuning any threshold on it.
- Manufacturing recordings to increase sample count.
- Combining the accumulation and asymmetry results into a single verdict.
- Any change to the incumbent selector, production behaviour, the representation, or existing
  expectations.
- Committing a real recording, or any prompt, response, command, file content, or sensitive path — from
  either repository. Mechanically derived mark frequencies and counts only.

## Outcome

One task, closed. **L1** for accumulation, **S1** for asymmetry, reported separately.

sprint:15 closed by asking whether its failure surfaces bite at real corpus sizes or are asymptotic
curiosities. They bite.

**Accumulation is observed, not merely reachable.** Both preregistered clauses fire independently. The
unmodified machinery, on real recordings at sprint:9's frozen ladder, returns **13 candidate pairs in
which fewer agreements outscore more** — the largest by 4.128 nats, ten agreements beating eleven. And
two of four included recordings, from two independent projects, hold both ingredients for a constructed
crossing: a mark above `N^{(k−1)/k}` and a singleton. The L2 margin test fails by a factor of four.

**The two large recordings straddle the surface.** At `k = 5`, `8b68dece` sits 3.4 counts above its
boundary and `7d95c414` sits 3.3 below. The envelope does not approach this boundary; it contains it.

**Asymmetry is an invariance defect by intent, and empirically large.** The intent was settled in the
preregistration on design grounds before any measurement: the relation is a claim about an unordered
pair, `cross_pairs` gives the two recordings no distinct roles, and sprint:13 symmetrized `surprisal`
for exactly this reason. The measurement then found **0 of 118 real candidate pairs symmetric**, a median
discrepancy of 0.851 nats, a maximum of 4.082, and the designated pick moving in 3 of 29 candidate sets.
Which window pair a reader is shown as *the* candidate depends on which recording was passed first.

**sprint:15's carried-forward estimate was recomputed from source and was correct**: `N = 169` exactly,
commonest count `= 64` exactly, `169^{3/4} = 46.94`. It is now a measurement rather than a recollection,
and a test pins the formula.

### Success criteria, against evidence

- **A corpus inventory with reasons**, spanning two repositories after log:1's addendum recorded a real
  Claude session in an external project. Four included, two excluded as lone records, none manufactured.
- **Per-recording characterization** of events, vocabulary, every mark's frequency, extremes, deciles,
  and the spans and agreement counts the frozen machinery produces.
- **Margins as quantities, not labels** — absolute and relative, per `k`, per recording — with the
  closest approach in each direction named.
- **The two claims kept distinct throughout**: a corpus that *contains* a crossing is a different and
  stronger finding than one that *could supply* an adversarial one, and both were established.
- **Classifications fixed before the data** and reported apart; neither combined into a single verdict.
- **All three synthetic gauntlets re-run unchanged**, known failures intact.

### What the sprint found that it was not looking for

**Two unrelated repositories have almost the same shape.** `8b68dece` and `7d95c414` — different
projects, different languages, different tasks — put 0.3787 and 0.3766 of their observed events on the
same delivered tool name, with its completion close behind. The operating envelope characterized here is
not one project's idiom, which is the difference between a measurement and an anecdote, and it exists
only because the corpus grew.

**The corpus itself is now durable knowledge with nowhere to live.** Which recordings exist, in which
repositories, and which are suitable for study is project knowledge that currently survives only inside
a task Result. First occurrence; recorded, not promoted.

### What this sprint deliberately leaves open

The repair, both of them. L1 makes the accumulation repair a blocker and S1 makes the symmetry repair
required, and this round chose neither mechanism — it is forbidden to.

The recommended sequencing, with its reason: **accumulation first**. It is the defect that makes scores
incomparable *within* a single argument order, so fixing symmetry first would produce a statistic that is
consistently wrong in both directions. Its repair round should preregister candidates before measuring
and be scored against both existing gauntlets plus the 13 crossings banked here.

Nothing here changed the raw format, the schema, the recorder, `inspection`, the viewer, the workbench,
the Spectroscope, or the product CLI's verbs, and no dependency was added.
