---
id: dec_01KZ1SQTYTXQTNPSQGSCRNM9BR
sequence: 2
kind: decision
status: accepted
created: 2026-08-02
---

# Keep reported intent separate from observed facts

## Context

WitnessGlass records two kinds of information about an agent session, and they are not the
same kind of thing.

**Reported** information is what the agent says: intent, hypotheses, decisions, plans, and
friendly descriptions of what it is about to do or believes it just did. This is the layer
a diff destroys entirely, and it is the only layer that can answer *why*. It is also, by
construction, a claim. A cooperative agent can report an intent it never acts on, describe
a test it never ran, or narrate a success that did not occur — not necessarily
dishonestly, but because narration and action are produced by different steps and can
diverge.

**Observed** information is what the surrounding machinery can see: tool invocation and
completion, commands, exit status, file mutation, test execution. Within its coverage it is
operationally solid. Outside its coverage it is silent, and it is epistemically poor in a
different way: an exit code is a fact about a process and says nothing about why anyone
wanted it run. A sequence of writes does not tell you what they were for.

The failure mode is obvious once stated and easy to commit accidentally. A correlator sees
a reported intent "run the test suite" near an observed `cargo test` exit 0 and writes a
single tidy event meaning "the agent ran the tests and they passed". That event is now
unfalsifiable: the case where the agent claimed to run tests and did not, and the case
where it did, have become indistinguishable in the record. The most valuable thing the
recording could have told anyone has been destroyed by the act of tidying it.

Merging is tempting because merged data is easier to display. Timelines look cleaner,
summaries read better, and there is exactly one row per "thing that happened".

## Decision

Reported and observed information remain distinct channels in the raw record, permanently.

- Raw session events are immutable and append-oriented.
- Every event retains explicit source and fidelity provenance: which channel produced it,
  via which adapter, and what that adapter could actually see.
- Reported intent is never promoted to ground truth because it is confident, structured, or
  convenient.
- Observed process facts are never promoted to intent because they are adjacent in time to
  a claim.
- Spans, timelines, landmarks, findings, and summaries are derived projections, rebuildable
  from the raw stream and freely discardable. They are never the record itself.
- Correlation may produce a view. It may not rewrite, overwrite, or reconcile the evidence
  it correlated.

## Consequences

- Derived views must carry their inputs' provenance through to the surface, so a reader can
  always ask "who said this" and get an answer.
- Disagreement between channels is a first-class, preservable finding — arguably the most
  interesting output the system can produce — rather than an inconsistency to be resolved
  at write time.
- A projection that cannot be rebuilt from raw events indicates either that the raw stream
  is missing something or that the projection is inventing something. Both are defects, and
  this gives a concrete test for either.
- The raw format is more verbose than a merged one, and display code carries the burden of
  presenting two channels legibly. That cost is accepted.
- Correlation heuristics can be changed, improved, or thrown away after the fact without
  invalidating existing recordings, because they never touched the evidence.
- This constrains the capture layer too: an adapter that cannot say what it did *not* see
  cannot honestly populate fidelity, which is why per-adapter blind spots are a stated
  requirement rather than documentation polish.
