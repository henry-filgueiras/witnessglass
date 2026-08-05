---
id: spr_01KZA3K3MQ3KS31VSN2QGQAZER
sequence: 15
kind: sprint
status: active
created: 2026-08-05
---

# Crucible

## Goal

sprint:14 discovered that `rarity_of_agreements` passes all seven families of sprint:12's gauntlet. This
sprint asks the only question that matters next:

> Does `rarity_of_agreements` survive adversarial tests designed **specifically against
> inverse-frequency weighting** — rather than merely passing the gauntlet that helped us discover it?

**The statistic is frozen for the entire round and is not adopted by it.** No production behaviour
changes, no incumbent selector moves, and nothing here makes `rarity_of_agreements` the boundary
statistic. The strongest outcome available is *"it deserves a subsequent head-to-head adoption
experiment"*, and that experiment is not this one.

## Rationale

**The old gauntlet is discovery evidence, not validation.** sprint:12 built it against the *permutation
null's* failure modes; sprint:14 found `rarity_of_agreements` by running ten preregistered functions
over it. A statistic found by searching a test suite has not been validated by that suite — it has been
selected on it. Re-reporting those seven PASSes as evidence of merit would be testing on the training
set, and sprint:12 exists to make exactly that objection. The old gauntlet runs again this round, but as
a **regression suite**: its role is to show nothing broke, not to show anything works.

**Adversarial means built against this statistic's own mechanism.** Inverse-frequency weighting has
predictable soft spots — unbounded growth as a mark's frequency approaches zero, sensitivity to a
denominator that changes when unrelated events are added, indifference to how many independent
agreements support a conclusion, and a definition that in this case reads only one side's marginals. The
families below are constructed against those, with expected orderings fixed in advance and quantitative
rules, and several of them are predicted to fail.

**Failure is preserved, not repaired.** If a family fails, the round minimizes the counterexample,
explains the mechanism, and classifies it — fatal, repairable, or an acceptable consequence of the
statistic's intended semantics. It does not fix the statistic and re-run, because a repaired statistic
that passes the gauntlet built against its predecessor has been validated by nothing at all.

**And no replacement is enumerated.** sprint:14 was an enumeration; this is a commissioning test of one
candidate. Proposing alternatives here would turn a bounded question back into a search.

## Success criteria

- The exact frozen definition recorded, including what it reads and what it does not.
- The evidential intuition it approximates stated, and the ways that approximation is loose.
- The old inverse-frequency prohibition quoted, its historical purpose explained, and the narrowest
  possible change made to permit evaluation — with the violation preserved rather than deleted.
- The discovery-versus-validation distinction recorded explicitly, before any new result.
- A small, interpretable adversarial gauntlet with constructions, invariants, mechanisms of risk, and
  quantitative rules, all fixed before evaluation.
- Parameter sweeps where a phase transition might exist, with any failure boundary minimized and banked.
- The old gauntlet re-run unchanged as regression, with no expectation weakened.
- The `k = 0` regression coverage preserved, and every existing check run.
- A bounded conclusion — A, B, or C — claiming no more than the evidence supports.

## Non-goals

- Adopting `rarity_of_agreements`, changing the incumbent selector, or altering any production
  behaviour.
- Repairing the statistic if it fails, or re-running the new gauntlet against a repaired version.
- Enumerating replacement statistics.
- Any richer observable: timing, paths, payloads, semantic labels, adapter identity, agent identity,
  channel, or schema. The representation-sufficiency question is closed and is not reopened.
- Weakening, reinterpreting, or rewriting any existing expectation, including sprint:12's families.
- Concluding that the statistic is generally correct because it passed a synthetic round.
- Corpus accumulation, variable-length discovery, a fourth real specimen, a product CLI surface, a
  dependency, or a Spectroscope change. No real recording committed, copied, or reproduced.
