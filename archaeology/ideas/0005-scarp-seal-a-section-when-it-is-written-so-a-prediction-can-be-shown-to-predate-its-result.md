---
id: ide_01KZ7B9ZJHYTZC8WC93G0F1DNE
sequence: 5
kind: idea
status: parked
created: 2026-08-04
---

# Scarp: seal a section when it is written, so a prediction can be shown to predate its result

## Problem

An experiment's credibility often rests on something being written down *before* a result was seen —
a hypothesis, a prediction, a parameter chosen on prior evidence, a threshold that decides
falsification. Scarp has nowhere to record that, and no way to show that it happened.

WitnessGlass hit this twice in consecutive rounds. sprint:5 wrote six predictions about a wavelet
transform before running it. sprint:6 wrote a six-entry parameter ladder, four predictions, a
comparison metric, and a falsification threshold before running a detector. In both cases the
material went into a `###` subsection of `## Acceptance criteria`, because a task has exactly two
sections and neither of them is for this. Predictions are not acceptance criteria; a reader
skimming the artifact cannot tell which parts were written first, and nothing in the front matter,
the status, or `scarp show` distinguishes them.

The workaround is a commit made for no other purpose than to timestamp the file — `363ac20` in
sprint:6 — plus a paragraph of prose in the Result asserting that the parameters did not move. That
works, and it means the evidence for the strongest claim in the round lives in the git log rather
than in the artifact, which is backwards for a tool whose value is a reviewable record of what was
known and when.

It got sharper in sprint:6, where the experiment found a defect in its own input handling *during*
the run and had to fix it. Demonstrating that the preregistered ladder did not quietly change in
response took an extra commit and a section of explanation. If the artifact could say "this section
was sealed on this date and has not been modified since", none of that prose would be needed.

The pressure runs the wrong way, as with idea:1 and idea:4: the cheapest option is to write
predictions after seeing results and present them as though they came first, and nothing detects it.

## Sketch

A section that is stamped when written and refuses silent modification afterwards. Roughly:

- `scarp seal <ref> --section <name> --body-file <path>` — write a named section once, record the
  date it was sealed in front matter, and refuse a second write to the same section.
- `scarp doctor` verifies the sealed section still matches what was sealed, so an edit is a
  detectable corruption rather than an invisible one.
- `scarp show` marks it, so a reader sees "sealed 2026-08-04" without reading the prose.

Deliberately not proposed: cryptographic notarization, external timestamping, or anything that
outlives the repository. The git history already provides the hard evidence; the gap is that the
artifact does not carry the claim, so a reader has to know to go looking.

This overlaps idea:4's `scarp amend` — both are about what happens to an artifact after it is
written — but they want opposite guarantees. `amend` makes append-after-close visible; `seal` makes
modification-after-write impossible. A tool that had `amend` and not `seal` would still leave
"predicted, not fitted" unverifiable.

## Boundaries

## Evidence

- **sprint:5, task:15** — six predictions about Haar behaviour, including an isolated-impulse null
  and a ±25% falsification threshold. Five held, one was falsified and recorded as falsified. The
  value of that honesty depends entirely on the predictions predating the run.
- **sprint:6, task:16** — a preregistered window ladder, expected synthetic matches, a comparison
  metric defined before any number was seen, and explicit supported/mixed/falsified criteria.
  Committed separately at `363ac20` for the sole purpose of timestamping. The round then found and
  fixed an input defect mid-run, which is exactly the situation where a reader is entitled to ask
  whether the parameters moved.
- Both rounds also hit the older friction that a body cannot introduce a section the collection does
  not define, which is idea:2's territory and is what forced this material into a subsection of
  something else.
