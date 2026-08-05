---
id: spr_01KZA2DNPW9HXNQYWAKTBBG68T
sequence: 14
kind: sprint
status: closed
created: 2026-08-05
closed: 2026-08-05
---

# Aperture

## Goal

A representation audit, not another statistic.

```text
representation R0
    → can / cannot answer question Q
    → if cannot, identify the missing information
    → propose the minimum additional observable φ
    → a future experiment tests R1 = R0 + φ
```

`Q` is sprint:13's unresolved question, restated as identifiability rather than as scoring:

> Does the current mark-only representation contain enough information, **in principle**, to
> distinguish Family E's novel-boundary arm from its redundant-boundary arm?

sprint:12 found no statistic that separated the arms; sprint:13 built a principled challenger and it did
not separate them either, for a reason it diagnosed exactly. Both were searches over scoring functions.
This round asks the prior question — whether the information is there at all — and answers it about the
representation rather than about any scorer.

## Rationale

**The two failures so far were about statistics, and a statistic can only lose information the
representation already has.** Before proposing a fourth scoring quantity it is worth establishing which
of the two situations we are in: the distinction is present in `R` and no one has found the function, or
the distinction is absent from `R` and no function exists. Those call for opposite next moves, and the
project has spent three rounds without asking which one it is in.

**The audit's strongest possible output is a collision certificate.** Two cases whose representation is
identical but whose desired orderings differ would settle the question by construction: no function of
`R` could separate them, because a function cannot distinguish inputs it cannot tell apart. That is a
proof, not an empirical failure, and it is worth more than any number of unsuccessful searches.

**If no collision exists, the fallback is an enumeration and not a hunt.** Its purpose is not to find a
clever winner — a winner selected after inspection would be exactly the overfitting sprint:12 was built
to catch — but to test whether the representation *visibly* carries the distinction under a small,
preregistered, ordinary family of functions, and at what cost to the families the existing machinery
already handles.

**Nothing richer enters the representation this round.** No timing, no paths, no tool semantics, no
payloads, no channel, adapter, schema, or agent identity. If the audit concludes the representation is
insufficient, the deliverable is a *named* minimum additional observable and a recommendation, not an
implementation.

## Success criteria

- The representation available to a scorer formalized precisely, including the invariance that makes it
  a representation rather than a view of the raw data.
- The collision question settled one way or the other, constructively.
- If no collision: a small explicit family of simple domain-neutral functions preregistered before
  evaluation, and every one of them run against Family E **and** the whole existing gauntlet by the
  existing rule.
- The incumbent, the challenger, the gauntlet families, their generation, their expectations, and the
  pass rule all frozen, with the pinned incumbent outputs unperturbed.
- A conclusion stronger than "no good statistic was found": either *identifiable*, or *not identifiable
  with a witness showing why*.
- The `k = 0` permutation-underflow defect preserved as a named regression test, kept clearly separate
  from the representation result.
- `scripts/check.sh` passes unweakened. No existing test changed.

## Non-goals

- Any new scoring statistic offered as a repair, any selector, any boundary-selection policy.
- Any richer observable: timing, paths, tool semantics, payload magnitude, channel, adapter, schema, or
  agent identity. Any semantic interpretation smuggled into the evidence representation.
- Implementing whatever additional observable the round recommends. That is the *next* experiment's
  subject, and stating it is this round's deliverable.
- Weakening, reinterpreting, or rewriting Family E or any other acceptance criterion.
- Corpus accumulation, variable-length discovery, a fourth real specimen, a product CLI surface, a
  dependency, or a Spectroscope change.
- Committing a real recording, or any prompt, response, command, file content, or sensitive path.

## Outcome

One task, closed. **The representation is sufficient**, which is the opposite of what the audit was
commissioned to expect, and the round has no `φ` to propose.

```text
representation R0  →  can answer question Q  →  no missing information  →  no φ
```

**Family E's arms are not `R`-identical.** Their equality patterns differ, and `ā_{L−1} ∈ ā₀…ā_{L−2}`
separates them while testing only mark equality and surviving any relabelling. No collision certificate
exists for them, so three rounds of failure were never identifiability failures. Six of ten preregistered
functions separate the arms empirically, confirming the proof.

**Two functions pass all seven families cleanly**, against a prediction that none would — and both are
the ones sprints 12 and 13 forbade as repairs. `rarity_of_agreements` scores Family E at exactly
`ln 2 = 0.6931`: the very quantity sprint:13 derived from the falling factorial and then lost to
cancellation.

**And the reason relocates the problem.** A permutation null samples *without* replacement, so the second
copy of a mark is the second of two already in the recording and costs nothing extra — the discount falls
on the core and the extension alike and vanishes from the delta. An i.i.d. null over the *same*
representation charges every draw at `−ln(c/N)` regardless, so the repeat costs `ln 2` less than a fresh
mark and the signal survives. sprint:13's conclusion — that no marginal-based statistic scored as a
nested delta can separate the arms — was **too strong**: it holds for without-replacement nulls and fails
for with-replacement ones, and sprint:13 never distinguished them because it only considered permutations.
That Result stands unedited with the correction recorded here.

**A collision does exist for a stronger claim.** Two candidates with identical spans and identical counts
— one a core plus a repeat that adds nothing, one a figure whose defining property is returning to its
first mark — are indistinguishable inside `R` and have opposite desired orderings. So *semantic*
redundancy is not `R`-determined, and Family E is answerable only because sprint:12 defined redundancy
syntactically. The witness is preserved in the module and asserted by test, and it is not grounds to
change Family E.

### Success criteria, against evidence

- **`R` formalized with its invariance**, and mechanically enforced: a test renames the entire mark
  alphabet and asserts every function's value is unmoved, so "restricted to the mark-only representation"
  is checked rather than promised.
- **The collision question settled constructively**, both ways — none for Family E's arms, one for the
  semantic claim, with a minimal witness.
- **Ten functions preregistered at `e5cdf47` and evaluated unchanged**, over the frozen 300 trials, the
  frozen families and expectations, and the single pass rule, with the dilution argmax recomputed per
  function rather than inherited.
- **A conclusion stronger than "no good statistic was found"**: identifiable, with the proof and the
  measurement agreeing.
- **The `k = 0` underflow preserved as a named regression test**, changing no experimental question,
  weakening no check, and perturbing no pinned output.

### What the sprint found that it was not looking for

**A prediction of "none will pass" that was wrong, and instructively.** Two functions pass all seven, and
both are inverse-frequency weightings — a class forbidden by a constraint written when the only null
under consideration was a permutation. That constraint, it turns out, excluded the only null that can see
a repeat at all. Whether to lift it is an adjudication for the human, and the round declines to make it.

**The archaeology now needs three kinds of section and has one.** sprint:13 wanted somewhere for an
analysis; this round wanted somewhere for a *proof*, whose conclusion chose which of two experiments ran.
Both share the `###` subsection with the preregistration, so the record cannot distinguish what was
proved from what was predicted. Second occurrence, third object.

### What this sprint deliberately leaves open

Whether the inverse-frequency constraint should be lifted, and if so, whether `rarity_of_agreements`
survives an adversarial gauntlet built against *its* failure modes rather than the permutation null's.
Passing a gauntlet designed against a different statistic is weak evidence, which is the objection
sprint:12 exists to raise, and it is the weakest link in this round's strongest finding.

Nothing here changed the raw format, the schema, the recorder, `inspection`, the viewer, the workbench,
the Spectroscope, or the product CLI's verbs, and no dependency was added.
