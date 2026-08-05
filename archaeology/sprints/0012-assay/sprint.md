---
id: spr_01KZ9ZARCM0SDS94AQQ54DQYCS
sequence: 12
kind: sprint
status: active
created: 2026-08-05
---

# Assay

## Goal

Try to break sprint:11's result.

sprint:11 found that on one real specimen, adding a fourth event made raw agreement **worse** (0.031 →
0.113) while making the agreement **more exceptional** relative to the order null (z 4.37 → 4.91), and
that standardized separation's global maximum over 2304 candidates landed on the previously observed
core. That is one interesting specimen. It is not evidence for a boundary-selection rule, and this
sprint exists to attack it rather than extend it.

**Nothing is invented this round.** The event-native representation, the alignment, every cost, the
timing policy, the normalization, the boundary search, the null model, and the null-relative statistics
are frozen. What is built is a gauntlet: families of synthetic specimen pairs in which *we know by
construction* which boundary events should carry evidence and which should not, run in the hundreds
with recorded seeds, scored against directional expectations written down first.

The most valuable outcome available here is discovering that the 3→4 observation was a coincidence or
an artefact. That would be progress.

## Rationale

**Three claims, and we have evidence for one of them.** They are routinely conflated and this sprint
keeps them apart:

1. *These sequences exhibit statistically unusual agreement.* Supported by sprint:11 across three
   specimens.
2. *This particular boundary is preferable.* Tantalizing, from one transition on one real specimen and
   one argmax on each of three.
3. *We have an automatic boundary-selection policy.* Not claimed, not built, and not to be built this
   round even if the statistic survives.

**The gauntlet must be able to fail.** Every family below has a directional expectation recorded before
any trial runs, a uniform pass rule applied to all of them alike, and a requirement that counterexamples
be shown rather than averaged away. A family that exposes a weakness in the null model is a result; the
model does not move to accommodate it.

**Rarity is the mechanism under test.** sprint:11's explanation was that a mark occurring once in 169
events almost never lands in a window under a permutation, so spans containing it are hard to match by
chance. If that is right, agreement on a ubiquitous event must contribute substantially less evidence
than agreement on a rare one — and families C and D are constructed as a matched pair to measure
exactly that, holding raw agreement identical by construction and varying only background prevalence.
If the null cannot represent that distinction, the round says so.

**Synthetic validation and real observation are different kinds of claim.** The three existing
specimens are re-run and kept in the report, but under a heading that says they cannot establish
ground-truth boundary correctness. Only the constructed families can.

## Success criteria

- A compact falsifiable hypothesis written before implementation, stated in terms of quantities the
  machinery actually computes.
- The metric, null, boundary search, and statistics frozen, verifiable by diff.
- Eight controlled families, deterministic and seed-recorded, exercising informative, noise, common,
  rare, redundant, accidental, diluted, and competing-motif boundaries.
- Hundreds of trials from a bounded grid, not three hand-authored examples.
- Directional expectations and a single uniform pass rule recorded before any aggregate is examined.
- The planted-boundary / Pareto splinter explained concretely — which candidates dominated, on which
  axes, and whether it is a bug, an expected disagreement, or evidence — without changing the fixture
  or the metric to make the planted answer land.
- A self-contained report with a scorecard, a Δraw-versus-Δsurprise scatter with the interesting
  quadrant marked, and every MIXED or FAIL family's counterexamples inspectable rather than aggregated
  away.
- An epistemic verdict on a four-step ladder, earned from the results, with the strongest counterexample
  named explicitly.
- `scripts/check.sh` passes unweakened. No existing test changed.

**The binding constraint:** the metric and null model are not tuned to pass these tests. If a family
fails, the failure is preserved prominently and the machinery stays as it was.

## Non-goals

- Any boundary-selection policy, selector, motif score, or rule that collapses the statistics — even if
  the gauntlet is survived. The next question would be whether the surviving signal supports a policy
  without overfitting this gauntlet, and that is a different round.
- Any change to the alignment, costs, timing policy, normalization, marks, representation, search
  radius, length floor, or null construction.
- Information-theoretic weighting of any kind.
- New similarity facets, variable-length discovery, motif families, corpus accumulation, a fourth real
  specimen, a product CLI surface, a new page, or a dependency.
- Committing a real recording, or any prompt, response, command, file content, or sensitive path.
