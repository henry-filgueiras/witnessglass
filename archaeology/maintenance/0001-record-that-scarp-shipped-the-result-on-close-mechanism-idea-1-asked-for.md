---
id: mnt_01KZ79DAZDTE50GWJQ4RWN5YVQ
sequence: 1
kind: maintenance
status: closed
created: 2026-08-04
closed: 2026-08-04
---

# Record that Scarp shipped the result-on-close mechanism idea 1 asked for

## Work

Record on [[ide_01KZ1SSY9D03KFBH9D2NE1Q4MQ|Scarp: record a task result in the same write that closes it]] that upstream Scarp now does what it asks: a task's
result rides the close transition in one write.

The idea was filed 2026-08-02 after closing sprint:1's task:1 and
task:2, both of which needed the append-then-close workaround. Upstream
shipped the mechanism on 2026-08-04. Recording it here keeps the idea
honest — a parked idea that has silently been satisfied is worse than no
idea, because the next person to hit the friction will read it and
believe the gap is still open.

What to record: the flag as it actually shipped, which parts of the
sketch it covers, and the one part it does not. Disposition of the idea
is deliberately left to its owner; this item only supplies the evidence.

## Result

Recorded on [[ide_01KZ1SSY9D03KFBH9D2NE1Q4MQ|Scarp: record a task result in the same write that closes it]] as a dated section: what shipped, the flag's
actual spelling, each Sketch and Boundaries point it satisfies, and the
one form it does not — the standalone `scarp result`, for recording an
outcome before a task is ready to close. Disposition left to this
project rather than decided here.

Two observations from doing it, both about Scarp rather than about
WitnessGlass.

**This item is the first thing in this repository that no sprint
commissioned.** All five sprints are closed. Under the shape that
produced sprint:3, recording a one-paragraph note on a parked idea would
have meant commissioning a sixth sprint, or doing it untracked. That is
the friction task:12 recorded, and it is gone.

**Scarp introduced a reference syntax this corpus does not use.** Before
this item, the archaeology contained zero `[[...]]` markers across
thirty-three artifacts; references are written as plain prose (`task:4`,
`sprint:3`). The sugar in this item's `## Work` was rewritten to a bound
marker at write time, so the first wikilink in this repository was
written by the tool rather than chosen by the project. It is correct,
robust, and resolves — and it is still a house style arriving by
default. The dated section appended to idea 1 above deliberately follows
this repository's prose convention instead, because that append happened
outside Scarp and nothing forced the choice.
