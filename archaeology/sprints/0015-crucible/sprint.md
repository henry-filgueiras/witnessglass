---
id: spr_01KZA3K3MQ3KS31VSN2QGQAZER
sequence: 15
kind: sprint
status: closed
created: 2026-08-05
closed: 2026-08-05
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

## Outcome

One task, closed. **B — two specific failure modes, banked, with the repair deferred.**

The statistic that passed seven of seven on the suite that discovered it fails or wobbles on seven of
ten families built against it. That contrast is the round's whole point, and it is why sprint:14's
result was recorded as discovery evidence rather than validation before this round ran.

**One mechanism explains six of the ten families.** `rarity_of_agreements` scores a candidate at
`k·ln N − Σ ln cᵢ`, so two candidates are ordered by `(k_X − k_Y)·ln N + (Σ ln c_Y − Σ ln c_X)`. That
changes sign at a corpus size fixed entirely by the marks' counts and by nothing about either candidate.
Whichever candidate has more agreements eventually wins as the corpus grows; whichever has fewer
eventually wins as it shrinks. Every boundary the sweep found matches the closed form exactly — `c =
N^{3/4}` for one-against-four at fixed counts, `N = p^{−k}` at fixed frequency, `N = 3 674` for the
vocabulary-growth pair.

**A separate defect is the only outright FAIL.** The implementation reads `a_counts` and `a_total` and
never B's, so an agreement on a mark occurring once in A and five hundred times in B — trivial for B to
supply — scores *identically* to one rare on both sides. Exactly equal, at every swept point. That one is
cheaply repairable; sprint:13 already symmetrized the same way.

**And the rarity idea itself passed the families aimed at it.** Rarity that does not agree contributes
exactly nothing; duplicating the whole corpus leaves every score unchanged to `1e-9`; a candidate at
fixed relative frequency is stable across sample sizes from 1.5× to 10×. The failures are about how
per-position scores are *combined*, not about what a position is worth.

### Success criteria, against evidence

- **The frozen definition recorded first**, including that it never reads the second recording's
  marginals — which turned into the round's only FAIL.
- **What it approximates, and how loosely**: an i.i.d. likelihood under A's empirical marginals, whose
  independence assumption a figure is precisely a violation of.
- **The old prohibition quoted and explained**, with no mechanical change needed to evaluate a flagged
  probe: the `probe` flag stays, its test stays, no history rewritten, scope alone changed.
- **Discovery separated from validation before any new result**, and the old gauntlets re-run unchanged
  as regression with every number identical and pinned rows intact.
- **Ten families with construction, invariant, mechanism of risk, nominal point, sweep, and rule**, all
  fixed in advance, and three minimized counterexamples banked as tests.
- **Failures preserved rather than repaired**, and no replacement enumerated.

### What the sprint found that it was not looking for

**Six families, one cause.** The round expected a scatter of soft spots and found a single structural
fact with six faces — which is a better result than six independent defects, because one mechanism can
be reasoned about and six cannot.

**A prediction vocabulary that did not match its own scoring rule.** Three families were predicted FAIL
and came out MIXED: the predictions asked whether the invariant breaks *anywhere in the sweep*, the rule
asks whether it breaks *at the nominal point*. The mechanism was predicted correctly every time and the
label was not. Ninth defect in ten rounds, third of this exact shape, and all three would have been
caught by reading each prediction against the quantity the code will actually compute.

**Building specimens at the representation level paid off.** Every counterexample is one readable line
and every phase boundary is analytically checkable. sprint:12 lost a family to a generator confound;
this round had no generator to be confounded by.

### What this sprint deliberately leaves open

The repair, both of them, and adoption — all out of scope by construction.

The one recommended next experiment, which is not a repair: **do the failure boundaries derived here
actually bite at the corpus sizes and mark frequencies of the real recordings, or are they asymptotic
curiosities outside the operating range?** The arithmetic is suggestive — the 234-record session's
commonest mark occurs 64 times against an `N^{3/4}` boundary near 47 — and suggestive is not measured. If
they sit outside the operating range, both defects are acceptable consequences of the statistic's
semantics and adoption becomes discussable. If inside, the length-dependence must be addressed first,
and that is the round to propose a repair in.

Nothing here changed the raw format, the schema, the recorder, `inspection`, the viewer, the workbench,
the Spectroscope, or the product CLI's verbs, and no dependency was added.
