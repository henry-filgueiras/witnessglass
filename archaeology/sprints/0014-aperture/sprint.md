---
id: spr_01KZA2DNPW9HXNQYWAKTBBG68T
sequence: 14
kind: sprint
status: active
created: 2026-08-05
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
