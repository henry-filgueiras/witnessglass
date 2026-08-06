---
id: dec_01KZA7RWZS0FY7427FZ9ZT6Q9X
sequence: 7
kind: decision
status: accepted
created: 2026-08-05
---

# Write every experimental criterion in the vocabulary of the computed quantity

## Context

Ten preregistered experimental rounds — sprint:6 through sprint:16 — produced **eight** defects in
criterion writing. Not one was a defect in an experiment. Every one was a defect in how a prediction, a
feasibility check, or a verdict rule was *worded*, discovered only when the result arrived and the
sentence turned out not to mean what its author thought.

Verified against the artifacts rather than from recollection:

| round | recorded defect | shape |
|---|---|---|
| sprint:6 | rank cutoff too tight; fixture combinatorics made it unreachable | missing check |
| sprint:8 | rank cutoff unreachable again; 29 correct answers preceded the one asked for | missing check |
| sprint:9 | threshold inherited from a round whose distances were ~10× smaller | missing check |
| sprint:10 | the feasibility check derived the disproof of its own criterion; the implication was not applied to it | **unpropagated** |
| sprint:11 | criterion said "most exceptional" without naming which statistic made a candidate so | **unpropagated** |
| sprint:12 | verdict ladder did not tile the outcome space; one FAIL with zero MIXED fit no cell | **unpropagated** |
| sprint:13 | arithmetic correct, applied to the wrong quantity — the span rather than the delta between spans | **unpropagated** |
| sprint:15 | prediction asked whether an invariant broke *anywhere in a sweep*; the rule asked whether it broke *at the nominal point* | **unpropagated** |

sprint:7, sprint:14, and sprint:16 recorded none.

**One correction this decision makes to the record.** sprint:15's Result calls itself *"the ninth defect
in ten rounds"*. Counting the artifacts, it is the **eighth**: sprint:10 is explicitly the fourth,
sprint:11 the fifth, sprint:12 the sixth, sprint:13 the seventh, and nothing between sprint:13 and
sprint:15 recorded one. The off-by-one is left in sprint:15's Result, which is not rewritten, and is
corrected here.

**The division in the table is the whole finding.** The first three are *missing feasibility checks* —
nobody asked whether the criterion was reachable. The last five are worse and more interesting: a
feasibility check **ran, succeeded, and produced a mechanism**, and that mechanism was not carried back
through the rest of the preregistration. sprint:10 is the sharpest case — its check derived the exact
algebra that invalidated its own criterion, and the criterion was written anyway.

Adding a feasibility step therefore does not fix this. Every round from sprint:10 onward had one.

## Decision

Two rules govern every preregistered experiment in this repository. They are short on purpose.

**Rule 1 — write in the vocabulary of the computed quantity.**

> Every prediction, feasibility check, measurement, and verdict criterion must be written in the exact
> vocabulary of the quantity the code will compute.

A criterion that says "most exceptional", "clearly better", "breaks in the sweep", or "the pick" must
name the function, the field, and the comparison that decide it. If the sentence cannot be read against
a value the run will produce, it is not yet a criterion.

**Rule 2 — propagate the feasibility pass.**

> After feasibility analysis, re-read **every** prediction and criterion against **every** mechanism the
> feasibility check discovered — including mechanisms discovered while investigating a different
> criterion — and either amend the criterion before execution or record why the mechanism does not
> apply to it.

The pass is written down. A round that performed it says so and shows the enumeration; a round that
found nothing to amend says that, which is a different claim from having skipped it.

**What this decision is not.** It is not a methodology framework, a template engine, or a required
document. It is two sentences and a habit, and it is recorded as a decision because that is where this
repository keeps settled rules. `CLAUDE.md` §7 points at it in one line so a future round meets it while
reading the contract it already reads.

## Consequences

- Every preregistration from sprint:17 onward carries an explicit propagation pass, and the pass is part
  of the preregistration commit rather than the Result.
- A round that skips the pass is incomplete in the same way a round that skips `scripts/check.sh` is.
- The rules are testable only by the next defect. If a ninth appears in a round that performed both, the
  rules are insufficient and this decision needs revisiting rather than repeating.
- Nothing about existing artifacts changes. sprint:15's off-by-one stays where it is; §7's requirement to
  preserve history outranks tidiness, and the correction lives here.
- **idea:5 remains unmet and is not replaced by this.** A rule about wording does not make a prediction
  legible as predating its result; only a sealed section does. These are different guarantees and both
  are still wanted.
