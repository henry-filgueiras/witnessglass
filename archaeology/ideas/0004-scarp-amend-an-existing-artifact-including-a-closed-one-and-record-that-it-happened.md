---
id: ide_01KZ73KDBARNGPN589DQJ5XVMR
sequence: 4
kind: idea
status: parked
created: 2026-08-04
---

# Scarp: amend an existing artifact, including a closed one, and record that it happened

## Problem

Archaeology accumulates after the fact, and Scarp has no command for that. A closed task turns
out to have recorded a claim that a later measurement supersedes; an open dragon collects a
fourth round of findings; a sprint gains a follow-up. Every one of those is an *append to an
existing artifact*, and every one of them is done with `cat >> file.md`.

idea:1 covers the narrower case — writing a task's `## Result` in the same write that closes it
— and explicitly leaves this one alone: "No opinion here on whether closed tasks stay editable
afterwards." This is that question, and the answer in practice is that they are edited, by hand,
with the tool watching and unable to help.

Two costs, and the second is the one that matters.

**The heading level and section vocabulary are hand-matched against a template nothing
validates.** `scarp doctor` passes either way, because it is checking front matter and identity
rather than body structure. That is the same gap idea:1 names.

**Nothing records that a closed artifact changed after it closed.** A task closed 2026-08-03
can carry a section dated 2026-08-04 without its front matter, its status, or `scarp show`
acknowledging that anything happened. A reader has to notice the date inside the prose. For a
tool whose value proposition is a reviewable record of what was known and when, an amendment
that leaves no trace outside its own text is a strange thing to be silent about — the git
history has it, and the artifact does not.

The pressure this creates runs the wrong way: the cheapest correct-looking option is to edit the
original conclusion in place, which is exactly what a project archaeology must never do.

## Sketch

An append that knows what it is appending to:

```sh
scarp amend task:12 --body-file addendum.md
scarp amend dragon:3 --body-file findings.md
```

Scarp owns the placement, keeps the existing body untouched above it, and stamps the artifact
with something a reader sees without reading the prose — an `amended:` date in front matter, or
a count. Whether an amendment to a *closed* artifact should be permitted freely, permitted with
a flag, or refused is the interesting design question, and the answer that fits observed use is
"permitted, and recorded".

## Boundaries

- Not general editing. Appending a new section is a different act from rewriting an existing
  one, and the second is the act this workflow exists to prevent.
- Must not reopen or otherwise change lifecycle state. An addendum to a closed task is not a
  reopened task, and an addendum to a dragon is not a resolution.
- Must not rewrite what is already there, including its own earlier amendments.

## Evidence

Observed across four rounds of WitnessGlass work on Scarp 0.2.0, all with the same `cat >>`
workaround:

- three dragons extended with first-contact findings (sprint:1);
- dragon:1, dragon:2, and dragon:3 extended again after the pass-2 hostile session;
- dragon:1 and dragon:3 extended a third time after pass 3;
- task:13 appending a dated pass-3 addendum to **task:12, which was already closed**, and a
  further section to the open dragon:3.

The last is the first time a *closed* artifact was amended, and it is the case with no support
at all: the artifact says `closed: 2026-08-03` and now contains material written on 2026-08-04,
with nothing but the prose to say so.
