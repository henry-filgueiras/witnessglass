---
id: drg_01KZ1SQTYZE1MQBGP6WQSHSYXX
sequence: 1
kind: dragon
status: open
created: 2026-08-02
---

# Claude integration may not expose complete tool ground truth

## Context

WitnessGlass intends to record a real Claude coding session as its first adapter, and the
whole value proposition rests on the recording being an honest account of what happened.

At bootstrap time, nobody here has measured what a cooperative Claude integration actually
exposes. The plausible sensor surfaces — agent hooks fired around tool use, or an explicit
semantic emission API the agent calls — have not been enumerated against a real session,
let alone tested for coverage. Assumptions about them are currently just assumptions.

Several things are unknown in ways that matter:

- Which lifecycle points are observable at all, and whether both the start and the
  completion of a tool call are visible, or only one side.
- Whether the payload carries enough to identify the work (command, arguments, exit status,
  affected paths) or only enough to know that *something* happened.
- Whether a hook can be dropped, coalesced, reordered, or silently skipped under load,
  interruption, or cancellation.
- Whether the same surface exists across the CLI, IDE integrations, and other hosts, or
  varies per host and per platform.
- Whether a session that ends abnormally leaves a truncated tail and what a reader is
  entitled to conclude from one.

The alternative some tools reach for — attach to the agent's process, trace descendants,
reconstruct everything from syscalls — is not a promise this project can keep. It is not
portable across macOS, Linux, and Windows; it requires privileges that many environments
will not grant; it degrades badly under sandboxing and containerization; and even where it
works it produces process facts that cannot recover intent. Shipping it as a v0 claim would
mean shipping a claim that fails silently on someone else's machine, which is precisely the
failure this project exists to make visible.

## Question

What does a cooperative Claude integration actually let WitnessGlass observe, how much of a
real session does that cover, and where are its blind spots?

## Constraints

- Cooperative hooks or an explicit semantic emission API are the primary sensor. Reported
  intent can only come from cooperation; there is no other channel for it.
- OS/process observation is optional, secondary, and corroborating. It is not the v0
  mechanism.
- v0 must not claim it can attach to an arbitrary agent process and observe every
  descendant process.
- Whatever the integration turns out to be, reported and observed events stay in separate
  channels with source and fidelity provenance intact.
- No completeness claim without measurement. "We did not see this" must be a supported,
  publishable result.
- Portability matters: an adapter that works only on one platform must say so rather than
  degrading quietly.

## Candidate direction

Cooperative hooks first, documented corroboration second.

Build the smallest adapter against one supported cooperative path. Record what it emits,
compare that against what the session demonstrably did, and write the gap down as the
adapter's declared fidelity — including the specific things it cannot see. Only after that
gap is characterized does it make sense to ask whether limited process-level corroboration
is worth adding, and it would then be additive evidence in its own channel, never a
retroactive patch over a hole in the cooperative record.

## Resolution criteria

This dragon is resolved when:

- At least one cooperative integration path has been exercised against a real Claude
  session, not a synthetic fixture.
- The adapter's coverage is stated in measured terms: which lifecycle events were observed,
  which fields were populated, and against what the comparison was made.
- The adapter's blind spots are enumerated explicitly, including behavior under abnormal
  termination and any host- or platform-specific variation encountered.
- The recording produced under that adapter carries fidelity provenance a reader can act
  on, such that no consumer can mistake partial coverage for complete observation.
- The decision about whether to pursue process-level corroboration is made on the basis of
  that measured gap rather than on speculation.
