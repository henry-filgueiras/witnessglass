---
id: drg_01KZ1SQTZ4867HDQFRFPT4ZRNY
sequence: 2
kind: dragon
status: open
created: 2026-08-02
---

# Recordings may capture secrets or sensitive source material

## Context

A recording that is faithful to an agent session is, by construction, a recording of
whatever that session touched. The surfaces that are likely to appear in a raw stream
include:

- prompts and agent narration, which routinely quote source, configuration, and error text
- file contents and diffs, including files never intended to leave the machine
- shell commands and their arguments, where tokens are frequently passed inline
- command output and stack traces, which leak connection strings, headers, and keys
- environment variable reads and dumps
- absolute filesystem paths, which carry usernames, client names, and directory structure
- URLs with embedded credentials or signed query parameters
- test output referencing fixture credentials that are real more often than anyone admits

The tension is structural rather than incidental. Fidelity is the product. Every mechanism
that removes sensitive material removes evidence, and every mechanism that preserves
evidence preserves whatever was sensitive in it. There is no configuration of this system
in which the two goals do not trade against each other.

What makes this a dragon rather than a task is the claim surface, not the filtering. Regex
scrubbing of common key shapes is easy to write and easy to demonstrate on a good day. It
is also unable to recognize a credential that does not look like one, and the moment a tool
advertises redaction, users reasonably start sharing recordings — attaching them to bug
reports, pasting them into issues, sending them to colleagues. An unreliable filter that is
described as reliable is more dangerous than no filter at all, because it converts a
recording users treated as sensitive into one they treat as safe.

The honest posture before that contract exists is that recordings are sensitive artifacts
that stay on the machine that produced them.

## Question

What capture and redaction contract can WitnessGlass actually honor, and what exactly may
be claimed about the safety of sharing a recording?

## Constraints

- Recordings are potentially sensitive by default, with no exceptions assumed.
- No silent or magical redaction. Anything removed, truncated, or transformed must be
  visible as such in the record, so a reader can tell that something was there.
- Real recordings must not be committed to this repository during bootstrap. Any committed
  fixture is synthetic and obviously so.
- No claim of redaction, sanitization, or safe sharing may be made until the contract is
  implemented and tested. This applies to the README, the CLI, and any future export path.
- Redaction is a transformation on the way out, not a rewrite of raw evidence in place —
  the canonical stream stays canonical.
- Whatever is decided must degrade honestly: an unrecognized secret that survives filtering
  is a known limit to state, not a bug to discover later.

## Candidate direction

Treat the safe artifact as an explicit derived export rather than a property of the raw
recording. Raw stays raw, local, and sensitive by default. Sharing goes through a separate
step that is allowed to be lossy, that marks every elision in place, and whose limits are
documented in the same breath as its capabilities. Default posture in the meantime: the
recording does not leave the machine, and the documentation says so plainly.

## Resolution criteria

This dragon is resolved when:

- The sensitive surfaces that capture can actually encounter are enumerated against real
  recorded sessions rather than imagined ones.
- A capture and redaction contract is written down, stating what is captured, what is
  transformed on export, what is not attempted, and what is known to slip through.
- That contract is implemented and covered by tests, including tests that assert redaction
  is visible in the output rather than silent.
- User-facing documentation makes a claim no stronger than what the tests demonstrate, with
  the residual risk stated in the same place.
- The default behavior for an unconfigured user is the conservative one, and the
  destructive or exporting path is the one requiring an explicit act.

## Findings from first contact (task:4)

This dragon's context enumerated sensitive surfaces that capture was *likely* to encounter.
One real recording now says which of them actually appeared, and in what proportion. The
recording was read but nothing from it — no line, path, command, or fragment of output — is
reproduced here or anywhere else in this repository.

**Volume, from one 17-minute session of ordinary repository work: 580 KB across 234 records.**
Of that, 58% is tool response bodies, 24% is tool input, and 0.3% is reported-intent text. The
median record is 652 bytes; the 90th percentile is 6.2 KB; the largest single record is 34 KB.
A day of agent work at this rate is tens of megabytes of unredacted material per session
directory.

### Which of the predicted surfaces actually appeared

Confirmed present, by direct measurement:

- **Absolute filesystem paths carrying the host's username** — in 56 of 234 records, roughly a
  quarter of the file. A session-scoped scratch directory path appears in 16 more.
- **Full file contents**, in both directions: every `Read` response and every `Write` input is
  the complete text.
- **Complete shell commands with all arguments**, 64 of them.
- **Complete command output**, including the full text of files printed with `cat`.
- **Agent narration**, as 65 `reported_intent` records — and, because decision:4 duplicates
  rather than moves the description, each of those strings is in the file **twice**.

Predicted and **not** present in this particular session: environment dumps, URLs with
embedded credentials, and connection strings. That is a property of what this session
happened to do, not of the adapter, and says nothing about the next one.

### The finding that most sharpens the dragon

**The recording contains at least one string deliberately shaped like a credential which is in
fact a synthetic test marker.** The subject session was writing tests for payload silence and
used a marker string built to look like a secret, precisely so a leak would be conspicuous.
That marker is now sitting in a real recording.

This is a concrete instance of the dragon's core argument, arrived at by accident rather than
by construction. A regex scrubber run over this recording would flag that string and would be
wrong; the same scrubber would have no way to recognize a real credential written in a shape
it does not know. **The recording contains both false positives and, potentially,
unrecognizable true negatives, and no filter can tell them apart from the bytes alone.** An
unreliable filter described as reliable is the failure mode this dragon exists to prevent, and
first contact produced the evidence for it in the first session recorded.

A second, quieter point in the same direction: **agent narration is in the file twice by
design.** Any future redaction contract has to handle the same sensitive string appearing in
two records with different channels and different provenance, and removing one occurrence
while leaving the other would be worse than removing neither, because it would look redacted.

### What did not go wrong

The privacy posture held mechanically. `.witnessglass/` was gitignored and untracked
throughout; the recording was never committed, excerpted, or moved; the archivist session ran
disarmed so it did not record itself; and `scripts/check-recording.sh` answered "did the
recorder survive" without putting any of the 580 KB on the terminal. The one documented
exception to payload silence — a corrupt record's parser diagnostic quoting the bytes it
rejected — did not fire, because the recording was not corrupt. It remains the case where the
payload-silent check is least useful, and it is still correctly documented as such.

### What this changes about the resolution criteria

The first criterion — enumerate sensitive surfaces against real recordings rather than
imagined ones — is now met for **one** session shape. It should not be treated as met
generally: a session that runs a deploy, reads a `.env`, or handles an API token would extend
this list, and none of that has been recorded.

Nothing else changes. No redaction is implemented, no export path exists, and no claim about
safe sharing may be made. The measured volume above makes the case that the export path, when
it exists, will need to be selective rather than exhaustive — 580 KB of unredacted material to
convey what a 17-minute session did is not an artifact anyone will read, and "attach the
recording" will be the tempting wrong answer.
