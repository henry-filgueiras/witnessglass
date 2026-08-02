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
