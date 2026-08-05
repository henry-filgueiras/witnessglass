---
id: spr_01KZA5SR3F4HX6ZQY7XC8T8DE9
sequence: 16
kind: sprint
status: active
created: 2026-08-05
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
