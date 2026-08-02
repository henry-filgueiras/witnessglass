---
id: tsk_01KZ2CFM599MNFZ7HACRBPW2ZY
sequence: 11
kind: task
status: pending
sprint: spr_01KZ2CCWX3JTRFXRG957Y8A2DR
created: 2026-08-02
---

# Exercise the viewer against first-contact evidence

## Objective

Point the finished viewer at the original first-contact recording, privately, and find out
whether it materially shortens that investigation while preserving the raw receipts and their
uncertainty.

**Depends on task:8, task:9, and task:10.**

task:4 characterized a 234-record, 580 KB recording with `jq`, in a session that had to
reconstruct the file's shape before it could read it. This task asks whether the viewer answers
the same questions without that, and whether it stays honest while doing so. The measure of
success is not that the interface looks finished; it is that a reader reaches task:4's findings
faster **and** cannot reach a conclusion task:4 refused to draw.

The recording is real and unredacted. It stays local, untracked, unquoted, unexcerpted, and
unchanged, and nothing derived from it enters this repository except aggregate measurements and
interface findings (dragon:2, `CLAUDE.md` §5).

## Acceptance criteria

- Committed fixtures are synthetic and obviously so, and include representative anomalies from
  first contact — an unmatched subagent stop, absent `duration_ms`, absent parentage, a
  duplicated reported description, a truncated tail — reproduced in shape without copying any
  real payload, path, identifier, or excerpt.
- The viewer is run privately against the original first-contact recording.
- The interface can answer, without `jq`:
  - whether the recording is complete;
  - how many tool requests and how many outcomes were recorded;
  - which outcomes are unresolved or anomalous;
  - what activity is attributable to each supplied agent identifier;
  - which subagent boundaries do not pair;
  - whether `duration_ms` and parentage were actually supplied;
  - which raw records support a selected reported/observed correlation.
- The interface is verified **not** to claim a turn count, a causal hierarchy, an execution
  duration, a complete set of file mutations, or the absence of failures and denials. Two silences
  agreeing is not corroboration, and the viewer must not present it as one.
- Startup and interaction behaviour are measured on the real 580 KB specimen and written down as
  numbers. This does not become premature optimization for recordings nobody has.
- Usability and epistemic defects found during the pass are fixed, and the epistemic ones take
  precedence.
- README and user documentation state the final invocation, the security boundary, the privacy
  warning, and the known limitations — what the viewer shows, what it derives, and what it cannot
  know.
- Only aggregate measurements and interface findings are recorded. No line, path, command,
  identifier, or fragment of the recording appears anywhere in the repository.
- The recording is left local, untracked, and byte-for-byte unchanged, and the viewer is confirmed
  to have written nothing.
- `scripts/check.sh` passes, the slice is committed, and dragons 1–3 stay open. If the pass
  produces new evidence about coverage, sensitivity, or identifier meaning, it is appended to the
  dragon it belongs to rather than left here.
