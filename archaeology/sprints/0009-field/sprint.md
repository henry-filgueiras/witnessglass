---
id: spr_01KZ9TDVC2YRR1SK15MTGT5DTG
sequence: 9
kind: sprint
status: active
created: 2026-08-05
---

# Field

## Goal

Make sprint:8's successful representation touch real dirt, before anything is built on top of it.

sprint:8 froze a question and answered it: event identity and relative timing, compared as short
marked sequences, recover a planted figure that sampled Matrix Profile could not. Every number in
that round came from a fixture whose structure was decided in constants, or from a single real
recording compared against itself.

This sprint asks the cheapest question that could invalidate the direction:

> Do two independent real WitnessGlass recordings contain cross-recording fixed-event-count window
> pairs that survive basic anti-triviality controls and look structurally meaningful under manual
> inspection?

**The metric does not move.** Representation, channel scope, event identity, timing policy,
substitution cost, indel cost, normalization, and combined ranking distance are exactly task:18's,
and nothing is tuned against the real recordings. If the metric fails on reality, the failure is the
result.

## Rationale

The risk this sprint exists to defuse is specific and this project has a history with it. sprint:3
exists because a chain of individually reasonable steps produced a false conclusion inside one system
asking itself whether it was complete. sprint:8 produced a clean, checked, preregistered success on
two fixtures **that were built to contain the thing it found**, and the natural next move —
variable-length boundary discovery — would be a substantially larger piece of work erected on that
one result.

So: one cheap reality check first, with a negative outcome treated as a real finding rather than as a
setback to be rescued.

**The most useful possible outcome may be the negative one.** If the metric recognizes a known figure
when one exists but natural recordings do not repeat themselves enough for event identity and timing
to expose cross-session structure, then the next problem is representation *richness* — paths,
payload magnitudes, working sets — rather than detector cleverness. That would redirect the whole
line of work, and it is worth one small round to find out. No excluded facet may be reached for to
rescue such an outcome in this round.

**`distinct marks` is not a definition of interestingness.** task:18 was explicit that its
degenerate-window diagnostic fits those fixtures rather than stating a principle, and this round must
not quietly promote it into one. Rankings are reported unrestricted first, and stratified by
distinct-mark count only as a diagnostic slice, with the slice never chosen for how good it makes the
round look.

**Common vocabulary is the null hypothesis, not a nuisance.** Both recordings are Claude Code sessions
in one repository, and a tool-name vocabulary of five strings dominated by one of them is exactly the
condition under which "matching" can mean nothing at all. The round has to be able to say
`Bash request → Bash success` out loud if that is what it found.

## Success criteria

- Two real recordings identified from the local workspace on inspected evidence, with independence
  argued rather than assumed, and characterized by aggregate metadata only. Neither committed,
  copied into tracked storage, nor reproduced.
- The task:18 metric used unchanged, with the freeze stated field by field and verifiable by diff.
- A cross-recording comparison that ranks **only** A-window against B-window pairs, carrying
  provenance so every candidate says which recording and which window each side came from.
- A small event-count ladder, preregistered with its justification, chosen from task:18's results and
  the two recordings' sizes rather than swept until something looks good.
- A deterministic order null, preregistered, with separation reported rather than a low absolute
  distance treated as a discovery.
- Marginal mark frequencies for both recordings, so a reader can judge whether common vocabulary
  explains the strongest matches.
- A manual-inspection rubric with four conservative categories, recorded before any candidate is
  seen, and classifications made against it.
- A criterion-feasibility check performed on cardinalities alone, before the matcher runs, and
  recorded — including whatever it finds.
- A three-way verdict fixed in advance and not weakened afterwards.
- `scripts/check.sh` passes unweakened. No existing test changed.

## Non-goals

- Variable-length or boundary discovery, subsequence growth or shrink heuristics, hierarchical
  motifs, motif families, corpus clustering, or cross-recording accumulation beyond one pair of
  recordings. Whether fixed boundaries are a material failure mode is *observed and recorded* here;
  it is not addressed.
- Any similarity facet task:18 deferred: paths, extensions, working sets, payload sizes, edit
  magnitude, reported intent, semantic tool categories, agent hierarchy, duration features, learned
  representations. Manual inspection may record a counterfactual observation about one, clearly
  labelled as interpretation.
- Any change to the metric's weights, costs, normalization, or timing policy.
- A public motif or corpus schema, a generic detector framework, or a refactor of task:18 into one.
- A dependency, a product CLI surface, a viewer change, a Spectroscope change, or UI of any kind for
  the inspection protocol.
- Committing a real recording, or putting an absolute path, prompt text, response text, command
  content, or file content into archaeology.
