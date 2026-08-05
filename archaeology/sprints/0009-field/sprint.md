---
id: spr_01KZ9TDVC2YRR1SK15MTGT5DTG
sequence: 9
kind: sprint
status: closed
created: 2026-08-05
closed: 2026-08-05
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

## Outcome

One task, closed. **Supported**, weakly, and the round's most useful output is not the verdict.

sprint:8 recovered a planted figure from two fixtures built to contain one. This sprint took that
metric, changed nothing in it, and pointed it at two real recordings two days apart with no shared
prompt. Something survived: two candidates clear every preregistered condition, the strongest of them
anchored by the *rarest* mark in the larger recording rather than the commonest. Above `k = 3` the top
matches are not one- or two-mark vocabulary, which is the specific failure this round most expected.

**The positive control is what keeps that honest.** Two executions of one runbook score five to twenty
times better and separate from the null four to thirteen times more strongly. That is what "these two
recordings really do share a figure" looks like through this metric, and the primary result is nowhere
near it. The detector is not the limiting factor; the recordings are.

### Success criteria, against evidence

- **Two real recordings, independence argued.** Four existed; one holds a single record, and two are
  executions of the same runbook agreeing in 27 of 32 observed marks. The rejected sibling became the
  positive control rather than being discarded.
- **The metric used unchanged**, verifiable by diff: the alignment, the timing policy, the five
  constants, and the normalization are byte-identical to task:18. The additions decide which pairs get
  compared and what gets printed.
- **Cross-recording only.** `cross_pairs` ranks nothing but (A-window, B-window) pairs, refuses two
  sequences carrying the same session id, and carries provenance on the value.
- **A ladder chosen and justified in advance** — `k = 3, 4, 6, 8, 12` — with every rung's reason
  recorded, and nothing outside it scanned.
- **A deterministic order null on both sides**, with separation reported per rung rather than a low
  absolute distance treated as a discovery. The timing null went along for one line and turned out to
  say something: timing helps when two sequences really are the same figure and hurts when they are
  not.
- **Marginal mark frequencies for both recordings**, which is how the frequency hypothesis got tested
  instead of waved at. A is 75.8% one tool name.
- **A four-category rubric recorded before any candidate was seen**, classifications written to disk
  from a distance-withheld packet, and the self-blinding disclosed as weak rather than presented as a
  protocol.
- **A criterion-feasibility check on cardinalities alone**, which caught one defect in advance and
  missed one. Both are recorded.
- **`scripts/check.sh` passes unweakened**, no existing test changed, no dependency added.

### What the sprint found that it was not looking for

**Fixed window boundaries are the dominant failure mode, and now it is measured.** One four-event core
recurred as the anchor of a top candidate at every rung of the ladder. At `k = 4` it is the whole
window. At 6, 8, and 12 the same core is present with divergent context attached on both sides, and
the distance degrades monotonically as the window forces more rubbish in. The metric found the figure
five times and was made to carry the surroundings each time. That is the evidence the next round was
supposed to be looking for, and it arrived as a by-product.

**The strongest persistent match is half an adapter artefact.** `tool_requested/Agent` is always
followed by `subagent_started` because that is how the Claude adapter emits a subagent launch. Two of
the four events in the round's best-anchored figure are therefore a property of the integration, not
of anything an agent chose. This was written down in the blind classification before any distance was
revealed, which is the only reason it is a finding rather than an embarrassment.

**A preregistered threshold can be reachable and still be weak.** The feasibility check this sprint was
asked to perform caught the rank-cutoff problem that spoiled two previous rounds and produced a
de-duplication policy before any output existed. It then missed a threshold imported from a round whose
distances were an order of magnitude smaller. Third criterion defect in four rounds, third distinct
shape.

### What this sprint deliberately leaves open

Variable-length boundary discovery, which is now the one recommended next experiment and has evidence
behind it rather than a hunch. The excluded similarity facets, three of which looked genuinely useful
during inspection and are logged with the claim each would actually be making. And the sample-size
problem: two recordings, one of them 32 events long, is not a rate, a distribution, or a base
expectation — only an existence proof above a null.

Nothing here changed the raw format, the schema, the recorder, `inspection`, the viewer, the workbench,
the Spectroscope, or the product CLI's verbs, and no dependency was added.
