---
id: spr_01KZA0ZWNEG3KMC464BN7CWNKD
sequence: 13
kind: sprint
status: closed
created: 2026-08-05
closed: 2026-08-05
---

# Epicycle

## Goal

sprint:12 broke half the house of cards. Family E — a boundary event that repeats a mark the core
already carries, against one seen nowhere else — came out at median `−0.003` over 30 matched pairs,
14 of 30 in the expected direction. Not a weak effect; no effect.

This sprint asks one question and is prepared to answer it either way:

> Is Family E's failure a limitation of the current **global order-permutation null**, or evidence
> that null-relative boundary evidence itself is insufficient for the job?

**The goal is not to rescue the approach.** A round that manufactures a challenger to make E pass has
learned nothing. The round is structured so that "no principled challenger survives review" is a
publishable outcome reached before any code is written, and so that a challenger which repairs E by
regressing families the incumbent already handles has visibly *paid* for the repair.

## Rationale

**Derive before building.** sprint:12's failure has a mechanical cause, and the cause determines
whether a repair exists. So the first phase is analysis: what the current null preserves, what it
destroys, why that makes E unidentifiable, and what the *minimum* additional conditioning would be.
Several constructions are considered and most are rejected — on stated grounds, before any of them is
measured against a specimen.

**A challenger must earn admission on its mechanism, not its results.** Anything that smuggles in
WitnessGlass or Claude semantics, a rarity weight chosen by hand, a threshold, a tunable window, or any
knowledge of the planted answer is rejected in review regardless of how well it would score. What
survives has to be explainable without reference to the specimens it will be run against.

**The whole gauntlet, not just the failure.** The challenger is run against all seven scored families,
side by side with the incumbent, on identical trials. A repair that costs the noise family, or the
accidental family, or the dilution family, has bought E at a price, and the price appears in the
verdict rather than in a footnote.

**And a separate question, asked explicitly.** Whatever survives — if anything — is assessed for
whether it still looks like a domain-neutral primitive over arbitrary timestamped categorical event
sequences, or whether it has started to require epicycles specific to this project's data.

## Success criteria

- A mechanical derivation of the incumbent null's blindness, written and committed **before** any
  challenger is implemented or run, including exactly what the null preserves and destroys and the
  minimum conditioning a challenger would need.
- Multiple candidate constructions considered, with the rejected ones named and the grounds recorded.
- The alignment metric and its constants frozen; the sprint:12 families, their generation, their
  directional expectations, the uniform pass rule, and the trial and realization counts all unchanged.
- Both nulls run over the entire gauntlet on identical trials, reported per family as distributions
  rather than as an aggregate pass count.
- Counterexamples and near-failures inspected as aggressively as sprint:12 inspected its own — which
  found a defect in its own generator by reading them.
- Any post-hoc correction preserved beside the original result and labelled as post-hoc.
- A verdict that distinguishes *current-null limitation* from *deeper insufficiency* from
  *inconclusive*, and that is allowed to be more precise than those three if the evidence warrants.
- A separate assessment of whether the surviving machinery is still domain-neutral.
- `scripts/check.sh` passes unweakened. No existing test changed.

## Non-goals

- Any production selector or boundary-selection policy.
- Any change to the alignment metric, its constants, the timing policy, the normalization, the
  representation, the search, or the sprint:12 families and their expectations.
- Inverse-frequency weighting, hand-added rarity bonuses, TF-IDF, entropy scores, or any weight chosen
  rather than derived.
- Threshold tuning against observed results.
- WitnessGlass or Claude semantic categories anywhere in the statistical machinery.
- Weakening or rewriting Family E because the incumbent statistic cannot satisfy it.
- New facets, variable-length discovery, corpus accumulation, a fourth real specimen, a product CLI
  surface, a dependency, or a Spectroscope change.
- Committing a real recording, or any prompt, response, command, file content, or sensitive path.

## Outcome

One task, closed. **B — deeper insufficiency**, with a precise reading and one real gain the label hides.

The sprint asked whether Family E's failure belonged to the global order null or to the approach. The
answer is neither, exactly, and the round is worth more for that than it would have been for a clean
repair.

**What conditioning fixed.** Holding one side's observed span fixed and permuting only the other —
one change, exact, no parameter — took the rare-versus-common family from 0.700 of pairs at a median of
`+0.073` to **30 of 30 at `+3.034` nats**. task:23 §1.2 argued before running that sprint:11's stated
mechanism was the wrong conditioning and that its rarity effect was a whole-recording artefact. That
argument is now measured, at forty times the effect size.

**What conditioning did not fix, and why it could not.** Family E stayed at FAIL, and the failure is
exact rather than noisy: every one of thirty matched pairs came out at `±0.0000`. The challenger *does*
see the redundancy — `S(core)` is lower by precisely `ln 2` in the arm whose recording holds two copies
of the repeated mark — but that same `ln 2` discounts the core and the extension equally and cancels in
the delta. What an added event costs is *(unused copies of its mark remaining) / (N − L)*, and in both
arms exactly one unused copy remains.

So Family E asks about **information a span already carries**, and a permutation null expresses
**availability in the population**. Those coincide only when the recording holds more copies than the
span uses, which in Family E it never does. No statistic that is a function of a span and a recording's
marginals, scored as a difference between nested spans, can separate the arms. Family E is unchanged;
this is a diagnosis, not a repair.

### Success criteria, against evidence

- **Derived before building.** Phase 1 was committed at `61cf0d7` with no code beside it: what the null
  preserves and destroys, why the double permutation's tail is a function of `c^L` and never mentions
  the observed span, and the correction to sprint:11's stated mechanism.
- **Six candidates considered, five rejected on stated grounds** — within-span permutation discards the
  background, windowed permutation needs a free parameter, paired permutation does not condition,
  inverse-frequency weighting is a chosen weight, an analytic tail of the same double permutation
  removes saturation but not blindness.
- **Frozen and pinned.** The diff over `event_sequence.rs` is empty, and a test holds sprint:12's seven
  incumbent rows to within `5e-4` so a later round cannot perturb them while claiming to add a column.
- **Both nulls over the whole gauntlet**, per family, with distributions and counterexamples — and no
  family regressed in verdict, though one lost 60% of its coverage to the challenger's indel gap.
- **A verdict more precise than its label**, and a separate domain-neutrality assessment.

### What the sprint found that it was not looking for

**My own derivation was arithmetically correct and applied to the wrong quantity.** §1.6 computed that
the redundant span is exactly twice as reachable as the novel one — true, and now pinned by a test — and
concluded the family would show `ΔS ≈ ln 2`. It does not follow: the family scores a delta between
nested spans and the factor appears in both terms. Seventh defect in eight rounds and a new shape. The
step that would have caught it is one line: *when a criterion scores a difference, do the algebra on the
difference, not on either term.*

**Exact statistics are testable in a way sampled ones are not.** The closed form was checked against
brute-force enumeration of every permutation on small populations, and that test found a genuine
underflow at `k = 0` within minutes — on a path no specimen reaches. That is an argument for the
challenger's construction independent of how it scored.

**Where the epicycles would begin.** A second conditioning layer — excluding the core's own events from
the population before scoring the extension — would separate Family E's arms. It would also be a
correction motivated by a residual rather than by a mechanism, which is the definition the sprint was
named for. The round declines it and says so.

### What this sprint deliberately leaves open

The one recommended next experiment: stop adding statistics and ask whether the *representation* can
carry the question, by measuring whether any function of the two spans' marks alone can separate Family
E's arms — by enumeration over a small candidate family rather than by inventing one.

The challenger's coverage gap, which is load-bearing: undefined wherever an alignment contains an indel,
which is 60% of the one family that searches over variable spans. A statistic that cannot score
variable-length candidates is a poor foundation for variable-length boundary discovery.

Nothing here changed the raw format, the schema, the recorder, `inspection`, the viewer, the workbench,
the Spectroscope, or the product CLI's verbs, and no dependency was added.
