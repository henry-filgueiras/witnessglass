---
id: spr_01KZ9ZARCM0SDS94AQQ54DQYCS
sequence: 12
kind: sprint
status: closed
created: 2026-08-05
closed: 2026-08-05
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

## Outcome

One task, closed. **WEAK / FRAGILE.** The phenomenon survived; its interpretation did not.

The sprint set out to break sprint:11's result and half succeeded, which is the outcome it was
commissioned for.

**What survived, unanimously.** Adding a shared informative boundary made raw agreement worse and
surprise better in **60 of 60** trials. Adding an unrelated one helped in **0 of 60**. A genuine
planted core out-scored the best a coincidence between two independent streams could manage in **29 of
30**. The surprising region stayed on the planted motif at every context length from 10 to 80, **40 of
40**. A longer imperfect core of rare marks beat a shorter exact core of common ones, **20 of 20**. The
3→4 transition that motivated the round is not an anecdote; it reproduces on demand.

**What broke.** The statistic cannot distinguish a **novel** rare mark from one the core **already
carries**: median `−0.003` over 30 matched pairs, 14 of 30 in the expected direction. Not a weak
effect — no effect. Mechanically it could not have gone otherwise: an order null permutes marks
globally, so the chance of a specific mark landing in a specific slot depends only on its global
prevalence, never on what the span already contains. "Surprise" here means *unlikely under a
permutation*, which is strictly weaker than *informative about a shared figure*.

**And the mechanism is weaker than advertised.** Rare against common — identical raw agreement by
construction, differing only in background prevalence — passes at 0.700 with a median of `+0.073` and a
first quartile below zero. Prevalence moves the statistic in the predicted direction and does so
unreliably at the scale of one boundary event.

### Success criteria, against evidence

- **The metric frozen**, and the diff over `event_sequence.rs` is empty. A test recomputes a trial's
  raw distance with `align` directly and asserts the gauntlet reported the same number.
- **Eight families, 300 trials, every seed recorded**, re-runnable in 3.2 seconds including 300 000
  null realizations.
- **One rule scored every family alike**, written before any trial ran and applied without amendment.
- **Counterexamples surfaced, not averaged away** — three worst per family, with their seeds,
  parameters, and verbatim spans, in the terminal and on the page.
- **The planted-boundary splinter explained**: the planted span is dominated by three candidates, all
  agreeing exactly on marks, one of them the same length shifted one event right. In a fixture that
  repeats every eight events the planted boundary is not identifiable from agreement alone. Neither the
  fixture nor the metric was touched.
- **A report with a scorecard and a Δraw-against-Δsurprise scatter**, the interesting quadrant shaded,
  with controlled synthetic validation separated from observations on real specimens.

### What the sprint found that it was not looking for

**A defect in its own gauntlet, caught by reading counterexamples rather than aggregates.** The first
run scored the noise family at 0.667 — a real-looking weakness. The counterexample table showed the
same background mark on *both* sides of a "different marks" trial: the generator drew them
independently and collided in 20 of 60 trials, making a third of the noise family silently informative.
Split by contamination the pre-fix numbers are stark — all 20 contaminated trials helped, none of the
40 genuine ones did. The generator was corrected, both sets of numbers are in the Result, and the
change is recorded as one made after seeing results, justified because it is a specification violation
provable without reference to any outcome and because it touches the specimen builder rather than the
metric or the null.

**The verdict ladder did not tile.** One FAIL and zero MIXED is a cell the rungs' counts do not cover.
The verdict rests on the WEAK/FRAGILE gloss rather than its clause. Sixth criterion defect in seven
rounds, and a new shape: not a wrong criterion but an incomplete cover.

### What this sprint deliberately leaves open

A selection policy, which the verdict forbids and which would select redundant boundaries if built on
this statistic as it stands.

The one recommended next experiment: replace the order null with a within-span-preserving null and ask
whether redundancy becomes visible — with the prediction, recorded in advance, that it may not, in which
case the next lever is the representation rather than the statistic.

The adapter-emission question, which family E makes worse rather than better: a mark that is rare
because of how events are written down is exactly as surprising to this null as one that is rare because
of what an agent did.

Nothing here changed the raw format, the schema, the recorder, `inspection`, the viewer, the workbench,
the Spectroscope, or the product CLI's verbs, and no dependency was added.
