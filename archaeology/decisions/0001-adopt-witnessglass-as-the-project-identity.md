---
id: dec_01KZ1SQTYM09GJYV1MQ006MZ07
sequence: 1
kind: decision
status: accepted
created: 2026-08-02
---

# Adopt WitnessGlass as the project identity

## Context

The project was carried under the working name *AgentScope* before this bootstrap. It had
no repository, no crate, and no published artifacts under that name — it existed only as a
working label for the idea.

That name is not available in any meaningful sense. AgentScope is an established,
well-known multi-agent framework at <https://github.com/agentscope-ai/agentscope>. Shipping
a differently-scoped tool under a colliding name would produce permanent search ambiguity,
mistaken issue reports in both directions, and a false implication of association. The cost
of the collision is highest exactly where this project wants to be legible: someone trying
to find out what it does.

A replacement name needed to describe the actual function — retaining evidence of a process
for later inspection — without implying a general agent platform.

In optical manufacturing, a *witness glass* is a sample coupon placed alongside a real part
and subjected to the same process: the same coating run, the same furnace, the same bath.
The part ships; the witness is retained so the process can be measured afterward and the
result traced back to the conditions that produced it. That is precisely the intended
relationship between this tool and an agent session — the agent does the work, and the
recording is the retained sample of what the run actually consisted of.

The name was checked before adoption: no GitHub repository named `witnessglass` existed
(global case-insensitive search returned zero results), and no `witnessglass` crate existed
on crates.io.

## Decision

Adopt **WitnessGlass** as the project identity, with this exact mapping used consistently:

- project in prose: **WitnessGlass**
- repository: `henry-filgueiras/witnessglass`
- crate, library, and eventual binary: `witnessglass`

Adopt the positioning line:

> WitnessGlass is a flight recorder for coding agents: declared intent, observed activity,
> and temporal replay.

AgentScope is retired as a name for this project. It is not retained as an alias.

## Consequences

- Prose, package metadata, repository name, and binary name are fixed and should not drift
  into alternative spellings, casings, or short forms.
- AgentScope survives only as historical provenance: it may appear in archaeology or in a
  README sentence that explains the rename and links the unrelated established framework,
  and nowhere else. It must never be used to refer to this project in the present tense.
- The name carries a claim the project now has to honor. A witness glass is worthless if it
  was not exposed to the same process as the part, and worse than worthless if it is
  quietly polished before measurement. That obligation is made concrete in the decision to
  keep reported intent separate from observed facts.
- The name is metaphorical rather than descriptive, so the positioning line has to do the
  explanatory work wherever the name appears without context.
- Availability was true at bootstrap time, not permanently guaranteed. Publishing the crate
  — which is not authorized and not a near-term goal — would be the point at which the
  crates.io name is actually claimed.
